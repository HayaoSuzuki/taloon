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
    fn lean_index_corpus_retains_reviewed_case_count() {
        // Arrange: the named row cases below cover the reviewed 216-row corpus.
        let corpus = include_str!("../../tests/fixtures/oracle/index.jsonl");
        // Act
        let count = corpus.lines().count();
        // Assert
        assert_eq!(count, 216);
    }

    #[rstest::rstest]
    #[case::index_0_json(0, "application/json")]
    #[case::index_0_html(0, "text/html")]
    #[case::index_1_json(1, "application/json")]
    #[case::index_1_html(1, "text/html")]
    #[case::index_2_json(2, "application/json")]
    #[case::index_2_html(2, "text/html")]
    #[case::index_3_json(3, "application/json")]
    #[case::index_3_html(3, "text/html")]
    #[case::index_4_json(4, "application/json")]
    #[case::index_4_html(4, "text/html")]
    #[case::index_5_json(5, "application/json")]
    #[case::index_5_html(5, "text/html")]
    #[case::index_6_json(6, "application/json")]
    #[case::index_6_html(6, "text/html")]
    #[case::index_7_json(7, "application/json")]
    #[case::index_7_html(7, "text/html")]
    #[case::index_8_json(8, "application/json")]
    #[case::index_8_html(8, "text/html")]
    #[case::index_9_json(9, "application/json")]
    #[case::index_9_html(9, "text/html")]
    #[case::index_10_json(10, "application/json")]
    #[case::index_10_html(10, "text/html")]
    #[case::index_11_json(11, "application/json")]
    #[case::index_11_html(11, "text/html")]
    #[case::index_12_json(12, "application/json")]
    #[case::index_12_html(12, "text/html")]
    #[case::index_13_json(13, "application/json")]
    #[case::index_13_html(13, "text/html")]
    #[case::index_14_json(14, "application/json")]
    #[case::index_14_html(14, "text/html")]
    #[case::index_15_json(15, "application/json")]
    #[case::index_15_html(15, "text/html")]
    #[case::index_16_json(16, "application/json")]
    #[case::index_16_html(16, "text/html")]
    #[case::index_17_json(17, "application/json")]
    #[case::index_17_html(17, "text/html")]
    #[case::index_18_json(18, "application/json")]
    #[case::index_18_html(18, "text/html")]
    #[case::index_19_json(19, "application/json")]
    #[case::index_19_html(19, "text/html")]
    #[case::index_20_json(20, "application/json")]
    #[case::index_20_html(20, "text/html")]
    #[case::index_21_json(21, "application/json")]
    #[case::index_21_html(21, "text/html")]
    #[case::index_22_json(22, "application/json")]
    #[case::index_22_html(22, "text/html")]
    #[case::index_23_json(23, "application/json")]
    #[case::index_23_html(23, "text/html")]
    #[case::index_24_json(24, "application/json")]
    #[case::index_24_html(24, "text/html")]
    #[case::index_25_json(25, "application/json")]
    #[case::index_25_html(25, "text/html")]
    #[case::index_26_json(26, "application/json")]
    #[case::index_26_html(26, "text/html")]
    #[case::index_27_json(27, "application/json")]
    #[case::index_27_html(27, "text/html")]
    #[case::index_28_json(28, "application/json")]
    #[case::index_28_html(28, "text/html")]
    #[case::index_29_json(29, "application/json")]
    #[case::index_29_html(29, "text/html")]
    #[case::index_30_json(30, "application/json")]
    #[case::index_30_html(30, "text/html")]
    #[case::index_31_json(31, "application/json")]
    #[case::index_31_html(31, "text/html")]
    #[case::index_32_json(32, "application/json")]
    #[case::index_32_html(32, "text/html")]
    #[case::index_33_json(33, "application/json")]
    #[case::index_33_html(33, "text/html")]
    #[case::index_34_json(34, "application/json")]
    #[case::index_34_html(34, "text/html")]
    #[case::index_35_json(35, "application/json")]
    #[case::index_35_html(35, "text/html")]
    #[case::index_36_json(36, "application/json")]
    #[case::index_36_html(36, "text/html")]
    #[case::index_37_json(37, "application/json")]
    #[case::index_37_html(37, "text/html")]
    #[case::index_38_json(38, "application/json")]
    #[case::index_38_html(38, "text/html")]
    #[case::index_39_json(39, "application/json")]
    #[case::index_39_html(39, "text/html")]
    #[case::index_40_json(40, "application/json")]
    #[case::index_40_html(40, "text/html")]
    #[case::index_41_json(41, "application/json")]
    #[case::index_41_html(41, "text/html")]
    #[case::index_42_json(42, "application/json")]
    #[case::index_42_html(42, "text/html")]
    #[case::index_43_json(43, "application/json")]
    #[case::index_43_html(43, "text/html")]
    #[case::index_44_json(44, "application/json")]
    #[case::index_44_html(44, "text/html")]
    #[case::index_45_json(45, "application/json")]
    #[case::index_45_html(45, "text/html")]
    #[case::index_46_json(46, "application/json")]
    #[case::index_46_html(46, "text/html")]
    #[case::index_47_json(47, "application/json")]
    #[case::index_47_html(47, "text/html")]
    #[case::index_48_json(48, "application/json")]
    #[case::index_48_html(48, "text/html")]
    #[case::index_49_json(49, "application/json")]
    #[case::index_49_html(49, "text/html")]
    #[case::index_50_json(50, "application/json")]
    #[case::index_50_html(50, "text/html")]
    #[case::index_51_json(51, "application/json")]
    #[case::index_51_html(51, "text/html")]
    #[case::index_52_json(52, "application/json")]
    #[case::index_52_html(52, "text/html")]
    #[case::index_53_json(53, "application/json")]
    #[case::index_53_html(53, "text/html")]
    #[case::index_54_json(54, "application/json")]
    #[case::index_54_html(54, "text/html")]
    #[case::index_55_json(55, "application/json")]
    #[case::index_55_html(55, "text/html")]
    #[case::index_56_json(56, "application/json")]
    #[case::index_56_html(56, "text/html")]
    #[case::index_57_json(57, "application/json")]
    #[case::index_57_html(57, "text/html")]
    #[case::index_58_json(58, "application/json")]
    #[case::index_58_html(58, "text/html")]
    #[case::index_59_json(59, "application/json")]
    #[case::index_59_html(59, "text/html")]
    #[case::index_60_json(60, "application/json")]
    #[case::index_60_html(60, "text/html")]
    #[case::index_61_json(61, "application/json")]
    #[case::index_61_html(61, "text/html")]
    #[case::index_62_json(62, "application/json")]
    #[case::index_62_html(62, "text/html")]
    #[case::index_63_json(63, "application/json")]
    #[case::index_63_html(63, "text/html")]
    #[case::index_64_json(64, "application/json")]
    #[case::index_64_html(64, "text/html")]
    #[case::index_65_json(65, "application/json")]
    #[case::index_65_html(65, "text/html")]
    #[case::index_66_json(66, "application/json")]
    #[case::index_66_html(66, "text/html")]
    #[case::index_67_json(67, "application/json")]
    #[case::index_67_html(67, "text/html")]
    #[case::index_68_json(68, "application/json")]
    #[case::index_68_html(68, "text/html")]
    #[case::index_69_json(69, "application/json")]
    #[case::index_69_html(69, "text/html")]
    #[case::index_70_json(70, "application/json")]
    #[case::index_70_html(70, "text/html")]
    #[case::index_71_json(71, "application/json")]
    #[case::index_71_html(71, "text/html")]
    #[case::index_72_json(72, "application/json")]
    #[case::index_72_html(72, "text/html")]
    #[case::index_73_json(73, "application/json")]
    #[case::index_73_html(73, "text/html")]
    #[case::index_74_json(74, "application/json")]
    #[case::index_74_html(74, "text/html")]
    #[case::index_75_json(75, "application/json")]
    #[case::index_75_html(75, "text/html")]
    #[case::index_76_json(76, "application/json")]
    #[case::index_76_html(76, "text/html")]
    #[case::index_77_json(77, "application/json")]
    #[case::index_77_html(77, "text/html")]
    #[case::index_78_json(78, "application/json")]
    #[case::index_78_html(78, "text/html")]
    #[case::index_79_json(79, "application/json")]
    #[case::index_79_html(79, "text/html")]
    #[case::index_80_json(80, "application/json")]
    #[case::index_80_html(80, "text/html")]
    #[case::index_81_json(81, "application/json")]
    #[case::index_81_html(81, "text/html")]
    #[case::index_82_json(82, "application/json")]
    #[case::index_82_html(82, "text/html")]
    #[case::index_83_json(83, "application/json")]
    #[case::index_83_html(83, "text/html")]
    #[case::index_84_json(84, "application/json")]
    #[case::index_84_html(84, "text/html")]
    #[case::index_85_json(85, "application/json")]
    #[case::index_85_html(85, "text/html")]
    #[case::index_86_json(86, "application/json")]
    #[case::index_86_html(86, "text/html")]
    #[case::index_87_json(87, "application/json")]
    #[case::index_87_html(87, "text/html")]
    #[case::index_88_json(88, "application/json")]
    #[case::index_88_html(88, "text/html")]
    #[case::index_89_json(89, "application/json")]
    #[case::index_89_html(89, "text/html")]
    #[case::index_90_json(90, "application/json")]
    #[case::index_90_html(90, "text/html")]
    #[case::index_91_json(91, "application/json")]
    #[case::index_91_html(91, "text/html")]
    #[case::index_92_json(92, "application/json")]
    #[case::index_92_html(92, "text/html")]
    #[case::index_93_json(93, "application/json")]
    #[case::index_93_html(93, "text/html")]
    #[case::index_94_json(94, "application/json")]
    #[case::index_94_html(94, "text/html")]
    #[case::index_95_json(95, "application/json")]
    #[case::index_95_html(95, "text/html")]
    #[case::index_96_json(96, "application/json")]
    #[case::index_96_html(96, "text/html")]
    #[case::index_97_json(97, "application/json")]
    #[case::index_97_html(97, "text/html")]
    #[case::index_98_json(98, "application/json")]
    #[case::index_98_html(98, "text/html")]
    #[case::index_99_json(99, "application/json")]
    #[case::index_99_html(99, "text/html")]
    #[case::index_100_json(100, "application/json")]
    #[case::index_100_html(100, "text/html")]
    #[case::index_101_json(101, "application/json")]
    #[case::index_101_html(101, "text/html")]
    #[case::index_102_json(102, "application/json")]
    #[case::index_102_html(102, "text/html")]
    #[case::index_103_json(103, "application/json")]
    #[case::index_103_html(103, "text/html")]
    #[case::index_104_json(104, "application/json")]
    #[case::index_104_html(104, "text/html")]
    #[case::index_105_json(105, "application/json")]
    #[case::index_105_html(105, "text/html")]
    #[case::index_106_json(106, "application/json")]
    #[case::index_106_html(106, "text/html")]
    #[case::index_107_json(107, "application/json")]
    #[case::index_107_html(107, "text/html")]
    #[case::index_108_json(108, "application/json")]
    #[case::index_108_html(108, "text/html")]
    #[case::index_109_json(109, "application/json")]
    #[case::index_109_html(109, "text/html")]
    #[case::index_110_json(110, "application/json")]
    #[case::index_110_html(110, "text/html")]
    #[case::index_111_json(111, "application/json")]
    #[case::index_111_html(111, "text/html")]
    #[case::index_112_json(112, "application/json")]
    #[case::index_112_html(112, "text/html")]
    #[case::index_113_json(113, "application/json")]
    #[case::index_113_html(113, "text/html")]
    #[case::index_114_json(114, "application/json")]
    #[case::index_114_html(114, "text/html")]
    #[case::index_115_json(115, "application/json")]
    #[case::index_115_html(115, "text/html")]
    #[case::index_116_json(116, "application/json")]
    #[case::index_116_html(116, "text/html")]
    #[case::index_117_json(117, "application/json")]
    #[case::index_117_html(117, "text/html")]
    #[case::index_118_json(118, "application/json")]
    #[case::index_118_html(118, "text/html")]
    #[case::index_119_json(119, "application/json")]
    #[case::index_119_html(119, "text/html")]
    #[case::index_120_json(120, "application/json")]
    #[case::index_120_html(120, "text/html")]
    #[case::index_121_json(121, "application/json")]
    #[case::index_121_html(121, "text/html")]
    #[case::index_122_json(122, "application/json")]
    #[case::index_122_html(122, "text/html")]
    #[case::index_123_json(123, "application/json")]
    #[case::index_123_html(123, "text/html")]
    #[case::index_124_json(124, "application/json")]
    #[case::index_124_html(124, "text/html")]
    #[case::index_125_json(125, "application/json")]
    #[case::index_125_html(125, "text/html")]
    #[case::index_126_json(126, "application/json")]
    #[case::index_126_html(126, "text/html")]
    #[case::index_127_json(127, "application/json")]
    #[case::index_127_html(127, "text/html")]
    #[case::index_128_json(128, "application/json")]
    #[case::index_128_html(128, "text/html")]
    #[case::index_129_json(129, "application/json")]
    #[case::index_129_html(129, "text/html")]
    #[case::index_130_json(130, "application/json")]
    #[case::index_130_html(130, "text/html")]
    #[case::index_131_json(131, "application/json")]
    #[case::index_131_html(131, "text/html")]
    #[case::index_132_json(132, "application/json")]
    #[case::index_132_html(132, "text/html")]
    #[case::index_133_json(133, "application/json")]
    #[case::index_133_html(133, "text/html")]
    #[case::index_134_json(134, "application/json")]
    #[case::index_134_html(134, "text/html")]
    #[case::index_135_json(135, "application/json")]
    #[case::index_135_html(135, "text/html")]
    #[case::index_136_json(136, "application/json")]
    #[case::index_136_html(136, "text/html")]
    #[case::index_137_json(137, "application/json")]
    #[case::index_137_html(137, "text/html")]
    #[case::index_138_json(138, "application/json")]
    #[case::index_138_html(138, "text/html")]
    #[case::index_139_json(139, "application/json")]
    #[case::index_139_html(139, "text/html")]
    #[case::index_140_json(140, "application/json")]
    #[case::index_140_html(140, "text/html")]
    #[case::index_141_json(141, "application/json")]
    #[case::index_141_html(141, "text/html")]
    #[case::index_142_json(142, "application/json")]
    #[case::index_142_html(142, "text/html")]
    #[case::index_143_json(143, "application/json")]
    #[case::index_143_html(143, "text/html")]
    #[case::index_144_json(144, "application/json")]
    #[case::index_144_html(144, "text/html")]
    #[case::index_145_json(145, "application/json")]
    #[case::index_145_html(145, "text/html")]
    #[case::index_146_json(146, "application/json")]
    #[case::index_146_html(146, "text/html")]
    #[case::index_147_json(147, "application/json")]
    #[case::index_147_html(147, "text/html")]
    #[case::index_148_json(148, "application/json")]
    #[case::index_148_html(148, "text/html")]
    #[case::index_149_json(149, "application/json")]
    #[case::index_149_html(149, "text/html")]
    #[case::index_150_json(150, "application/json")]
    #[case::index_150_html(150, "text/html")]
    #[case::index_151_json(151, "application/json")]
    #[case::index_151_html(151, "text/html")]
    #[case::index_152_json(152, "application/json")]
    #[case::index_152_html(152, "text/html")]
    #[case::index_153_json(153, "application/json")]
    #[case::index_153_html(153, "text/html")]
    #[case::index_154_json(154, "application/json")]
    #[case::index_154_html(154, "text/html")]
    #[case::index_155_json(155, "application/json")]
    #[case::index_155_html(155, "text/html")]
    #[case::index_156_json(156, "application/json")]
    #[case::index_156_html(156, "text/html")]
    #[case::index_157_json(157, "application/json")]
    #[case::index_157_html(157, "text/html")]
    #[case::index_158_json(158, "application/json")]
    #[case::index_158_html(158, "text/html")]
    #[case::index_159_json(159, "application/json")]
    #[case::index_159_html(159, "text/html")]
    #[case::index_160_json(160, "application/json")]
    #[case::index_160_html(160, "text/html")]
    #[case::index_161_json(161, "application/json")]
    #[case::index_161_html(161, "text/html")]
    #[case::index_162_json(162, "application/json")]
    #[case::index_162_html(162, "text/html")]
    #[case::index_163_json(163, "application/json")]
    #[case::index_163_html(163, "text/html")]
    #[case::index_164_json(164, "application/json")]
    #[case::index_164_html(164, "text/html")]
    #[case::index_165_json(165, "application/json")]
    #[case::index_165_html(165, "text/html")]
    #[case::index_166_json(166, "application/json")]
    #[case::index_166_html(166, "text/html")]
    #[case::index_167_json(167, "application/json")]
    #[case::index_167_html(167, "text/html")]
    #[case::index_168_json(168, "application/json")]
    #[case::index_168_html(168, "text/html")]
    #[case::index_169_json(169, "application/json")]
    #[case::index_169_html(169, "text/html")]
    #[case::index_170_json(170, "application/json")]
    #[case::index_170_html(170, "text/html")]
    #[case::index_171_json(171, "application/json")]
    #[case::index_171_html(171, "text/html")]
    #[case::index_172_json(172, "application/json")]
    #[case::index_172_html(172, "text/html")]
    #[case::index_173_json(173, "application/json")]
    #[case::index_173_html(173, "text/html")]
    #[case::index_174_json(174, "application/json")]
    #[case::index_174_html(174, "text/html")]
    #[case::index_175_json(175, "application/json")]
    #[case::index_175_html(175, "text/html")]
    #[case::index_176_json(176, "application/json")]
    #[case::index_176_html(176, "text/html")]
    #[case::index_177_json(177, "application/json")]
    #[case::index_177_html(177, "text/html")]
    #[case::index_178_json(178, "application/json")]
    #[case::index_178_html(178, "text/html")]
    #[case::index_179_json(179, "application/json")]
    #[case::index_179_html(179, "text/html")]
    #[case::index_180_json(180, "application/json")]
    #[case::index_180_html(180, "text/html")]
    #[case::index_181_json(181, "application/json")]
    #[case::index_181_html(181, "text/html")]
    #[case::index_182_json(182, "application/json")]
    #[case::index_182_html(182, "text/html")]
    #[case::index_183_json(183, "application/json")]
    #[case::index_183_html(183, "text/html")]
    #[case::index_184_json(184, "application/json")]
    #[case::index_184_html(184, "text/html")]
    #[case::index_185_json(185, "application/json")]
    #[case::index_185_html(185, "text/html")]
    #[case::index_186_json(186, "application/json")]
    #[case::index_186_html(186, "text/html")]
    #[case::index_187_json(187, "application/json")]
    #[case::index_187_html(187, "text/html")]
    #[case::index_188_json(188, "application/json")]
    #[case::index_188_html(188, "text/html")]
    #[case::index_189_json(189, "application/json")]
    #[case::index_189_html(189, "text/html")]
    #[case::index_190_json(190, "application/json")]
    #[case::index_190_html(190, "text/html")]
    #[case::index_191_json(191, "application/json")]
    #[case::index_191_html(191, "text/html")]
    #[case::index_192_json(192, "application/json")]
    #[case::index_192_html(192, "text/html")]
    #[case::index_193_json(193, "application/json")]
    #[case::index_193_html(193, "text/html")]
    #[case::index_194_json(194, "application/json")]
    #[case::index_194_html(194, "text/html")]
    #[case::index_195_json(195, "application/json")]
    #[case::index_195_html(195, "text/html")]
    #[case::index_196_json(196, "application/json")]
    #[case::index_196_html(196, "text/html")]
    #[case::index_197_json(197, "application/json")]
    #[case::index_197_html(197, "text/html")]
    #[case::index_198_json(198, "application/json")]
    #[case::index_198_html(198, "text/html")]
    #[case::index_199_json(199, "application/json")]
    #[case::index_199_html(199, "text/html")]
    #[case::index_200_json(200, "application/json")]
    #[case::index_200_html(200, "text/html")]
    #[case::index_201_json(201, "application/json")]
    #[case::index_201_html(201, "text/html")]
    #[case::index_202_json(202, "application/json")]
    #[case::index_202_html(202, "text/html")]
    #[case::index_203_json(203, "application/json")]
    #[case::index_203_html(203, "text/html")]
    #[case::index_204_json(204, "application/json")]
    #[case::index_204_html(204, "text/html")]
    #[case::index_205_json(205, "application/json")]
    #[case::index_205_html(205, "text/html")]
    #[case::index_206_json(206, "application/json")]
    #[case::index_206_html(206, "text/html")]
    #[case::index_207_json(207, "application/json")]
    #[case::index_207_html(207, "text/html")]
    #[case::index_208_json(208, "application/json")]
    #[case::index_208_html(208, "text/html")]
    #[case::index_209_json(209, "application/json")]
    #[case::index_209_html(209, "text/html")]
    #[case::index_210_json(210, "application/json")]
    #[case::index_210_html(210, "text/html")]
    #[case::index_211_json(211, "application/json")]
    #[case::index_211_html(211, "text/html")]
    #[case::index_212_json(212, "application/json")]
    #[case::index_212_html(212, "text/html")]
    #[case::index_213_json(213, "application/json")]
    #[case::index_213_html(213, "text/html")]
    #[case::index_214_json(214, "application/json")]
    #[case::index_214_html(214, "text/html")]
    #[case::index_215_json(215, "application/json")]
    #[case::index_215_html(215, "text/html")]
    fn lean_index_corpus_matches_representation_lookup(
        #[case] position: usize,
        #[case] content_type: &str,
    ) {
        use taloon::{
            evidence::Evidence,
            model::{ArtifactFormat, PackageId, SourceId},
        };
        let corpus = include_str!("../../tests/fixtures/oracle/index.jsonl");
        // Arrange: each generated corpus row and representation is an independent case.
        let line = corpus.lines().nth(position).unwrap();
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
                metadata.map_or_else(String::new, |h| format!("data-core-metadata='sha256={h}'"))
            )
            .unwrap();
        }
        let locked = ContentHash::digest(&[case.locked]);
        let artifact = artifact_url(&case.url);
        let body = if content_type == "application/json" {
            serde_json::to_vec(&serde_json::json!({"files":files})).unwrap()
        } else {
            html.into_bytes()
        };
        let response = Response {
            status: 200,
            location: None,
            body,
            content_type: content_type.into(),
            url: Url::parse("https://index.test/simple/example/").unwrap(),
        };
        // Act
        let parsed = PackageIndex::parse(&response, &targets).unwrap();
        let result = parsed.sidecar(&artifact, locked);
        // Assert: expected values come from Lean, not a Rust decision implementation.
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

    fn scoped_response(body: Vec<u8>, content_type: &str) -> Response {
        Response {
            status: 200,
            location: None,
            body,
            content_type: content_type.into(),
            url: Url::parse("https://index.test/simple/example/").unwrap(),
        }
    }

    fn scope_invariant(response: &Response) -> Result<(), proptest::test_runner::TestCaseError> {
        let artifact = Url::parse("https://files.test/example.whl?locked=1").unwrap();
        let targets = BTreeSet::from([artifact.clone()]);
        // Parsing rejection is permitted for arbitrary bytes; every successful
        // parse must retain only exact-scope links and exact metadata URLs.
        if let Ok(parsed) = PackageIndex::parse(response, &targets) {
            prop_assert!(parsed.links.len() <= 1);
            if let Ok(Some((url, _))) = parsed.sidecar(&artifact, ContentHash::digest(b"archive")) {
                prop_assert_eq!(
                    url.as_str(),
                    "https://files.test/example.whl.metadata?locked=1"
                );
            }
        }
        Ok(())
    }

    fn generated_page(entries: &[u8], html: bool) -> Response {
        let archive = ContentHash::digest(b"archive")
            .to_string()
            .replace("sha256:", "");
        let metadata = ContentHash::digest(b"metadata")
            .to_string()
            .replace("sha256:", "");
        if html {
            let mut body = String::new();
            for n in entries {
                write!(body, "<a href='https://files.test/artifact.whl?build={n}#sha256={archive}' data-core-metadata='sha256={metadata}'>x</a>").unwrap();
            }
            body.push_str("<a href='https://files.test/unrelated.whl#sha256=invalid' data-core-metadata='sha256=invalid'>x</a>");
            scoped_response(body.into_bytes(), "text/html")
        } else {
            let mut files: Vec<_> = entries
                .iter()
                .map(|n| {
                    serde_json::json!({"url":format!("https://files.test/artifact.whl?build={n}"),
                "hashes":{"sha256":archive},"core-metadata":{"sha256":metadata}})
                })
                .collect();
            files.push(serde_json::json!({"url":"https://files.test/unrelated.whl", "hashes":{"sha256":"invalid"},"core-metadata":{"sha256":"invalid"}}));
            scoped_response(
                serde_json::to_vec(&serde_json::json!({"files":files})).unwrap(),
                "application/json",
            )
        }
    }

    proptest! {
        #[test]
        fn arbitrary_json_bytes_cannot_escape_locked_scope(bytes in prop::collection::vec(any::<u8>(), 0..4096)) {
            // Arrange
            let response = scoped_response(bytes, "application/json");
            // Act / Assert: a successful parse must obey the exact-scope invariant.
            scope_invariant(&response)?;
        }

        #[test]
        fn html_links_cannot_escape_locked_scope(text in prop_oneof![
            Just("https://files.test/example.whl?locked=1".to_owned()), ".{0,256}",
        ]) {
            // Arrange
            let hex = ContentHash::digest(b"archive").to_string().replace("sha256:", "");
            let body = format!("<base href='{text}'><script>{text}</script><a href='{text}#sha256={hex}' data-core-metadata='sha256={hex}'>{text}</a>");
            let response = scoped_response(body.into_bytes(), "text/html");
            // Act / Assert
            scope_invariant(&response)?;
        }

        #[test]
        fn json_links_cannot_escape_locked_scope(text in prop_oneof![
            Just("https://files.test/example.whl?locked=1".to_owned()), ".{0,256}",
        ]) {
            // Arrange
            let hex = ContentHash::digest(b"archive").to_string().replace("sha256:", "");
            let body = serde_json::to_vec(&serde_json::json!({"files":[{"url":text,"hashes":{"sha256":hex},"core-metadata":{"sha256":hex}}]})).unwrap();
            let response = scoped_response(body, "application/json");
            // Act / Assert
            scope_invariant(&response)?;
        }

        #[test]
        fn filtered_associations_match_independent_occurrence_counts(
            entries in prop::collection::vec(0_u8..16, 0..64), selected in any::<u8>(), html in any::<bool>()
        ) {
            // Arrange: count occurrences in integer labels before constructing a page.
            let labels: Vec<_> = (0_u8..8).filter(|n| selected & (1 << n) != 0).collect();
            let mut counts = [0_usize; 16];
            for label in &entries { counts[usize::from(*label)] += 1; }
            let expected: Vec<_> = labels.iter().map(|label| {
                match counts[usize::from(*label)] {
                    0 => Ok(None),
                    1 => Ok(Some(format!("https://files.test/artifact.whl.metadata?build={label}"))),
                    _ => Err(Code::InvalidMetadata),
                }
            }).collect();
            let targets: BTreeSet<_> = labels.iter().map(|n| Url::parse(&format!("https://files.test/artifact.whl?build={n}")).unwrap()).collect();
            let response = generated_page(&entries, html);
            // Act
            let parsed = PackageIndex::parse(&response, &targets).unwrap();
            let actual: Vec<_> = targets.iter().map(|target| parsed.sidecar(target, ContentHash::digest(b"archive"))
                .map(|association| association.map(|(url, _)| url.to_string())).map_err(|error| error.code)).collect();
            // Assert
            prop_assert_eq!(actual, expected);
            prop_assert!(parsed.links.len() <= targets.len());
        }

        #[test]
        fn generated_associations_reject_wrong_archive_hash(label in 0_u8..16, html in any::<bool>()) {
            // Arrange
            let target = Url::parse(&format!("https://files.test/artifact.whl?build={label}")).unwrap();
            let response = generated_page(&[label], html);
            let targets = BTreeSet::from([target.clone()]);
            let parsed = PackageIndex::parse(&response, &targets).unwrap();
            // Act
            let result = parsed.sidecar(&target, ContentHash::digest(b"wrong"));
            // Assert
            prop_assert_eq!(result.err().unwrap().code, Code::HashMismatch);
        }
    }
}

#[cfg(test)]
mod html_tests;

#[cfg(test)]
mod json_tests;
