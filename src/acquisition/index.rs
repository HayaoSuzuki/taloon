//! Per-run package index lookup. Exact locked URL/hash checks remain per artifact.
use super::{
    http::{self, Http, Response},
    metrics::{RequestPurpose, micros, timed},
};
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        BufferQueue, Tag, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    },
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
    time::Instant,
};
use taloon::{
    evidence::SidecarAssociation,
    model::{ArtifactLocator, ArtifactRequest, Code, ContentHash, Error, EvidenceRequirement},
    report::AcquisitionSummary,
};
use url::Url;
mod json;
// Interpret links as tokens arrive; historical tags never become a second
// document representation. A terminal error invalidates even earlier matches.
struct HtmlSink<'a> {
    state: RefCell<Result<HtmlIndex<'a>, Error>>,
}
struct HtmlIndex<'a> {
    response_url: &'a Url,
    targets: &'a BTreeSet<Url>,
    // None means no base with href has been encountered yet.
    base: Option<Url>,
    found: BTreeMap<Url, Binding>,
}
impl TokenSink for HtmlSink<'_> {
    type Handle = ();
    fn process_token(&self, token: Token, _line: u64) -> TokenSinkResult<()> {
        if let Token::TagToken(tag) = token
            && tag.kind == TagKind::StartTag
        {
            if tag.name.as_ref() == "script" {
                return TokenSinkResult::RawData(html5ever::tokenizer::states::RawKind::ScriptData);
            }
            if matches!(
                tag.name.as_ref(),
                "style" | "xmp" | "iframe" | "noembed" | "noframes"
            ) {
                return TokenSinkResult::RawData(html5ever::tokenizer::states::RawKind::Rawtext);
            }
            if matches!(tag.name.as_ref(), "textarea" | "title") {
                return TokenSinkResult::RawData(html5ever::tokenizer::states::RawKind::Rcdata);
            }
            if matches!(tag.name.as_ref(), "a" | "base" | "meta") {
                let mut state = self.state.borrow_mut();
                if let Ok(index) = state.as_mut()
                    && let Err(error) = index.record_tag(&tag)
                {
                    *state = Err(error);
                }
            }
        }
        TokenSinkResult::Continue
    }
}
impl HtmlIndex<'_> {
    fn record_tag(&mut self, tag: &Tag) -> Result<(), Error> {
        let name = tag.name.as_ref();
        // Borrow tokenizer-owned values. Reverse lookup preserves the old map's
        // last-local-name rule without allocating another attribute map.
        let attr = |name: &str| {
            tag.attrs
                .iter()
                .rev()
                .find(|a| a.name.local.as_ref() == name)
                .map(|a| a.value.as_ref())
        };
        if name == "base" && self.base.is_none() {
            if let Some(href) = attr("href") {
                self.base = Some(self.response_url.join(href).map_err(|_| invalid())?);
            }
            return Ok(());
        }
        if name == "meta"
            && attr("name").is_some_and(|s| s == "pypi:repository-version")
            && attr("content").is_some_and(|s| s.split('.').next() != Some("1"))
        {
            return Err(invalid());
        }
        if name != "a" {
            return Ok(());
        }
        let base = self.base.as_ref().unwrap_or(self.response_url);
        let Some(url) = attr("href").and_then(|s| base.join(s).ok()) else {
            return Ok(());
        };
        if !self.targets.contains(&normalized(url.clone())) {
            return Ok(());
        }
        let archive = url
            .fragment()
            .and_then(|s| s.strip_prefix("sha256="))
            .and_then(digest);
        let metadata = attr("data-core-metadata")
            .or_else(|| attr("data-dist-info-metadata"))
            .and_then(|s| s.strip_prefix("sha256="))
            .and_then(digest);
        record(
            &mut self.found,
            normalized(url),
            Binding::from_hashes(archive, metadata),
        );
        Ok(())
    }
}

fn digest(hex: &str) -> Option<ContentHash> {
    ContentHash::parse(&format!("sha256:{hex}")).ok()
}
fn normalized(mut url: Url) -> Url {
    url.set_fragment(None);
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url
}
fn invalid() -> Error {
    Error::new(
        Code::InvalidMetadata,
        "invalid or ambiguous Simple API metadata association",
    )
}
/// A sealed request scope: all endpoint keys and locked URLs are collected before
/// workers start. No map lock or historical-release cache is needed. Snapshots
/// (including terminal failures) live only for this acquisition call.
pub(super) struct IndexCatalog<'a> {
    http: &'a Http,
    entries: BTreeMap<Url, Entry>,
}
#[derive(Default)]
struct Entry {
    targets: BTreeSet<Url>,
    snapshot: OnceLock<Result<PackageIndex, Error>>,
}
struct Target {
    endpoint: Url,
    artifact: Url,
}
// ArtifactRequest validates both URLs before storing its immutable locator.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn validated_url_failure(_: url::ParseError) -> Error {
    http::failure()
}
impl Target {
    fn for_request(request: &ArtifactRequest) -> Result<Option<Self>, Error> {
        if request.requirement() != EvidenceRequirement::IndexAssociated {
            return Ok(None);
        }
        let ArtifactLocator::Remote {
            index: Some(index),
            url,
        } = request.locator()
        else {
            return Ok(None);
        };
        let mut endpoint = Url::parse(index).map_err(validated_url_failure)?;
        let path = format!(
            "{}/{}/",
            endpoint.path().trim_end_matches('/'),
            request.package().name()
        );
        endpoint.set_path(&path);
        Ok(Some(Self {
            endpoint: normalized(endpoint),
            artifact: normalized(Url::parse(url).map_err(validated_url_failure)?),
        }))
    }
}
impl<'a> IndexCatalog<'a> {
    pub(super) fn new(requests: &[ArtifactRequest], http: &'a Http) -> Self {
        let mut catalog = Self {
            http,
            entries: BTreeMap::new(),
        };
        for request in requests {
            if let Ok(Some(target)) = Target::for_request(request) {
                catalog
                    .entries
                    .entry(target.endpoint)
                    .or_default()
                    .targets
                    .insert(target.artifact);
            }
        }
        catalog
    }
    /// Fetch/parse at most once per requested package endpoint, except that an
    /// initializer panic permits retry. Waiters share the result, never proof:
    /// each request's exact URL and locked archive hash are still checked here.
    /// Only locked URLs are retained; one ambiguous slot bounds duplicate links.
    pub(super) fn sidecar(
        &self,
        request: &ArtifactRequest,
        stats: &mut AcquisitionSummary,
    ) -> Result<Option<(Url, SidecarAssociation)>, Error> {
        let Some(target) = Target::for_request(request)? else {
            return Ok(None);
        };
        let entry = self
            .entries
            .get(&target.endpoint)
            .ok_or_else(http::failure)?;
        let snapshot = if let Some(snapshot) = entry.snapshot.get() {
            stats.index_cache_hits = stats.index_cache_hits.saturating_add(1);
            snapshot
        } else {
            let waiting = Instant::now();
            let mut initialized_here = false;
            let snapshot = entry.snapshot.get_or_init(|| {
                initialized_here = true;
                // Coordination excludes this owner's transport and parse work.
                stats.index_wait_us = stats
                    .index_wait_us
                    .saturating_add(micros(waiting.elapsed()));
                let response = self.http.get(
                    RequestPurpose::Index,
                    target.endpoint.clone(),
                    "application/vnd.pypi.simple.v1+json, text/html;q=0.1",
                    16 * 1024 * 1024,
                    stats,
                )?;
                stats.index_parse_count = stats.index_parse_count.saturating_add(1);
                timed(&mut stats.index_parse_us, || {
                    PackageIndex::parse(&response, &entry.targets)
                })
            });
            if !initialized_here {
                stats.index_cache_hits = stats.index_cache_hits.saturating_add(1);
                stats.index_wait_us = stats
                    .index_wait_us
                    .saturating_add(micros(waiting.elapsed()));
            }
            snapshot
        };
        snapshot
            .as_ref()
            .map_err(Clone::clone)?
            .sidecar(&target.artifact, request.hash().ok_or_else(http::failure)?)
    }
}

struct PackageIndex {
    links: BTreeMap<Url, Binding>,
}
#[derive(Clone, Copy)]
enum Binding {
    Unavailable,
    MissingArchive,
    Hashed {
        archive: ContentHash,
        metadata: ContentHash,
    },
    Ambiguous,
}
impl Binding {
    fn from_hashes(archive: Option<ContentHash>, metadata: Option<ContentHash>) -> Self {
        match (archive, metadata) {
            (_, None) => Self::Unavailable,
            (None, Some(_)) => Self::MissingArchive,
            (Some(archive), Some(metadata)) => Self::Hashed { archive, metadata },
        }
    }
}
fn record(found: &mut BTreeMap<Url, Binding>, url: Url, binding: Binding) {
    found
        .entry(url)
        .and_modify(|value| *value = Binding::Ambiguous)
        .or_insert(binding);
}
impl PackageIndex {
    fn parse(response: &Response, targets: &BTreeSet<Url>) -> Result<Self, Error> {
        let links =
            if response.content_type.contains("json") || response.body.first() == Some(&b'{') {
                json::links(response, targets)?
            } else {
                html_links(response, targets)?
            };
        Ok(Self { links })
    }
    fn sidecar(
        &self,
        artifact: &Url,
        locked: ContentHash,
    ) -> Result<Option<(Url, SidecarAssociation)>, Error> {
        let Some(value) = self.links.get(&normalized(artifact.clone())) else {
            return Ok(None);
        };
        let (archive, metadata) = match value {
            Binding::Unavailable => return Ok(None),
            Binding::MissingArchive | Binding::Ambiguous => return Err(invalid()),
            Binding::Hashed { archive, metadata } => (*archive, *metadata),
        };
        let binding = SidecarAssociation::new(locked, archive, metadata)?;
        let mut url = normalized(artifact.clone());
        let path = format!("{}.metadata", url.path());
        url.set_path(&path);
        Ok(Some((url, binding)))
    }
}
#[cfg(test)]
pub(super) fn sidecar(
    response: &Response,
    artifact: &Url,
    locked: ContentHash,
) -> Result<Option<(Url, SidecarAssociation)>, Error> {
    PackageIndex::parse(response, &BTreeSet::from([normalized(artifact.clone())]))?
        .sidecar(artifact, locked)
}

fn html_links(
    response: &Response,
    targets: &BTreeSet<Url>,
) -> Result<BTreeMap<Url, Binding>, Error> {
    let text = std::str::from_utf8(&response.body).map_err(|_| invalid())?;
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(text));
    let sink = HtmlSink {
        state: RefCell::new(Ok(HtmlIndex {
            response_url: &response.url,
            targets,
            base: None,
            found: BTreeMap::new(),
        })),
    };
    let tokenizer = Tokenizer::new(sink, TokenizerOpts::default());
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    tokenizer.sink.state.into_inner().map(|index| index.found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::fmt::Write as _;

    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CaseEntry {
        url: String,
        archive: Option<u8>,
        metadata: Option<u8>,
    }
    #[derive(serde::Deserialize)]
    #[serde(tag = "kind", deny_unknown_fields)]
    enum Expected {
        #[serde(rename = "none")]
        None,
        InvalidMetadata,
        HashMismatch,
        #[serde(rename = "associated")]
        Associated {
            metadata: u8,
        },
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Case {
        schema: u32,
        id: String,
        entries: Vec<CaseEntry>,
        scope: Vec<String>,
        url: String,
        locked: u8,
        expected: Expected,
    }
    fn artifact_url(key: &str) -> Url {
        Url::parse(&format!("https://files.test/{key}.whl")).unwrap()
    }
    fn metadata_bytes(id: u8) -> Vec<u8> {
        crate::testfixtures::metadata(&format!("License-Expression: MIT\nX-Witness: {id}"))
    }

    #[test]
    fn lean_index_corpus_matches_both_representation_lookups() {
        use taloon::{
            evidence::Evidence,
            model::{ArtifactFormat, PackageId, SourceId},
        };
        let corpus = include_str!("../../tests/fixtures/oracle/index.jsonl");
        let mut count = 0;
        for line in corpus.lines() {
            let case: Case = serde_json::from_str(line).unwrap();
            assert_eq!(case.schema, 1);
            let targets = case.scope.iter().map(|key| artifact_url(key)).collect();
            let mut files = Vec::new();
            let mut html = String::new();
            for entry in &case.entries {
                let url = artifact_url(&entry.url);
                let archive = entry.archive.map(|id| {
                    ContentHash::digest(&[id])
                        .to_string()
                        .replace("sha256:", "")
                });
                let metadata = entry.metadata.map(|id| {
                    ContentHash::digest(&metadata_bytes(id))
                        .to_string()
                        .replace("sha256:", "")
                });
                files.push(serde_json::json!({"url":url.to_string(),"hashes":{"sha256":archive},"core-metadata":{"sha256":metadata}}));
                write!(
                    html,
                    "<a href='{url}{}' {}>x</a>",
                    archive.map_or_else(String::new, |h| format!("#sha256={h}")),
                    metadata
                        .map_or_else(String::new, |h| format!("data-core-metadata='sha256={h}'"))
                )
                .unwrap();
            }
            let locked = ContentHash::digest(&[case.locked]);
            let artifact = artifact_url(&case.url);
            for (body, content_type) in [
                (
                    serde_json::to_vec(&serde_json::json!({"files":files})).unwrap(),
                    "application/json",
                ),
                (html.into_bytes(), "text/html"),
            ] {
                let response = Response {
                    status: 200,
                    location: None,
                    body,
                    content_type: content_type.into(),
                    url: Url::parse("https://index.test/simple/example/").unwrap(),
                };
                let parsed = PackageIndex::parse(&response, &targets).unwrap();
                let result = parsed.sidecar(&artifact, locked);
                match case.expected {
                    Expected::None => assert!(result.unwrap().is_none(), "{}", case.id),
                    Expected::InvalidMetadata => assert_eq!(
                        result.err().unwrap().code,
                        Code::InvalidMetadata,
                        "{}",
                        case.id
                    ),
                    Expected::HashMismatch => assert_eq!(
                        result.err().unwrap().code,
                        Code::HashMismatch,
                        "{}",
                        case.id
                    ),
                    Expected::Associated { metadata } => {
                        let (url, binding) = result.unwrap().unwrap();
                        assert_eq!(url, Url::parse(&format!("{artifact}.metadata")).unwrap());
                        let request = ArtifactRequest::new(
                            PackageId::parse(
                                "example",
                                "1.0",
                                SourceId::parse("registry", "https://index.test/simple").unwrap(),
                            )
                            .unwrap(),
                            ArtifactLocator::Remote {
                                index: Some("https://index.test/simple".into()),
                                url: artifact.to_string(),
                            },
                            ArtifactFormat::Wheel,
                            Some(locked),
                        )
                        .unwrap();
                        let bytes = metadata_bytes(metadata);
                        let evidence = Evidence::from_sidecar(&request, binding, &bytes).unwrap();
                        assert_eq!(
                            evidence.summary().metadata_hash,
                            ContentHash::digest(&bytes)
                        );
                        assert_eq!(
                            evidence.summary().strength,
                            EvidenceRequirement::IndexAssociated
                        );
                    }
                }
            }
            count += 1;
        }
        assert_eq!(count, 216);
    }

    proptest! {
        #[test]
        fn malformed_index_inputs_cannot_escape_the_locked_url_scope(
            bytes in prop::collection::vec(any::<u8>(), 0..4096), text in ".{0,256}"
        ) {
            let artifact = Url::parse("https://files.test/example.whl?locked=1").unwrap();
            let targets = BTreeSet::from([artifact.clone()]);
            let hash = ContentHash::digest(b"archive");
            let hex = hash.to_string().replace("sha256:", "");
            let html = format!("<base href='{text}'><script>{text}</script><a href='{text}#sha256={hex}' data-core-metadata='sha256={hex}'>{text}</a>");
            let json = serde_json::to_vec(&serde_json::json!({"files":[{"url":text,"hashes":{"sha256":hex},"core-metadata":{"sha256":hex}}]})).unwrap();
            for (body, content_type) in [(bytes,"application/json"),(html.into_bytes(),"text/html"),(json,"application/json")] {
                let response = Response {status:200,location:None,body,content_type:content_type.into(),
                    url:Url::parse("https://index.test/simple/example/").unwrap()};
                if let Ok(parsed) = PackageIndex::parse(&response,&targets) {
                    prop_assert!(parsed.links.len() <= 1);
                    if let Ok(Some((url,_))) = parsed.sidecar(&artifact,hash) {
                        prop_assert_eq!(url.as_str(),"https://files.test/example.whl.metadata?locked=1");
                    }
                }
            }
        }

        #[test]
        fn filtered_associations_preserve_exact_queries_and_ambiguity(
            entries in prop::collection::vec(0_u8..16, 0..64), selected in any::<u8>()
        ) {
            let archive = ContentHash::digest(b"archive");
            let metadata = ContentHash::digest(b"metadata");
            let archive_hex = archive.to_string().replace("sha256:", "");
            let metadata_hex = metadata.to_string().replace("sha256:", "");
            let url = |n| Url::parse(&format!("https://files.test/artifact.whl?build={n}")).unwrap();
            let targets: BTreeSet<_> = (0..8).filter(|n| selected & (1 << n) != 0).map(url).collect();
            let mut files: Vec<_> = entries.iter().map(|n| serde_json::json!({"url":url(*n).to_string(),
                "hashes":{"sha256":archive_hex},"core-metadata":{"sha256":metadata_hex}})).collect();
            files.push(serde_json::json!({"url":"https://files.test/unrelated.whl",
                "hashes":{"sha256":"invalid"},"core-metadata":{"sha256":"invalid"}}));
            let mut html = String::new();
            for n in &entries {
                write!(html,"<a href='{}#sha256={archive_hex}' data-core-metadata='sha256={metadata_hex}'>x</a>",url(*n)).unwrap();
            }
            html.push_str("<a href='https://files.test/unrelated.whl#sha256=invalid' data-core-metadata='sha256=invalid'>x</a>");
            for (body, content_type) in [
                (serde_json::to_vec(&serde_json::json!({"files":files})).unwrap(), "application/json"),
                (html.into_bytes(), "text/html"),
            ] {
                let response = Response {status:200,location:None,body,content_type:content_type.into(),
                    url:Url::parse("https://redirected.test/simple/example/").unwrap()};
                let parsed = PackageIndex::parse(&response, &targets).unwrap();
                prop_assert!(parsed.links.len() <= targets.len());
                for target in &targets {
                    let count = entries.iter().filter(|n| url(**n) == *target).count();
                    let result = parsed.sidecar(target, archive);
                    if count == 0 { prop_assert!(result.unwrap().is_none()); }
                    else if count == 1 {
                        let metadata_url = result.unwrap().unwrap().0;
                        prop_assert_eq!(metadata_url.query(),target.query());
                        prop_assert!(parsed.sidecar(target, ContentHash::digest(b"wrong")).is_err());
                    } else { prop_assert_eq!(result.err().unwrap().code, Code::InvalidMetadata); }
                }
            }
        }
    }
}

#[cfg(test)]
mod html_tests;

#[cfg(test)]
mod json_tests;
