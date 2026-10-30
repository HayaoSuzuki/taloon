//! Optional index evidence lookup. It never chooses a substitute artifact.
use super::http::Response;
use html5ever::{
    tendril::StrTendril,
    tokenizer::{
        BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
    },
};
use std::{cell::RefCell, collections::BTreeMap};
use taloon::{
    evidence::SidecarAssociation,
    model::{Code, ContentHash, Error},
};
use url::Url;
#[derive(Default)]
struct Links {
    tags: RefCell<Vec<(String, BTreeMap<String, String>)>>,
}
impl TokenSink for Links {
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
            if !matches!(tag.name.as_ref(), "a" | "base" | "meta") {
                return TokenSinkResult::Continue;
            }
            self.tags.borrow_mut().push((
                tag.name.to_string(),
                tag.attrs
                    .into_iter()
                    .map(|a| (a.name.local.to_string(), a.value.to_string()))
                    .collect(),
            ));
        }
        TokenSinkResult::Continue
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
pub(super) fn sidecar(
    response: &Response,
    artifact: &Url,
    locked: ContentHash,
) -> Result<Option<(Url, SidecarAssociation)>, Error> {
    let matches = if response.content_type.contains("json") || response.body.first() == Some(&b'{')
    {
        json_links(response, artifact)?
    } else {
        html_links(response, artifact)?
    };
    if matches.len() > 1 {
        return Err(invalid());
    }
    let Some((archive, metadata)) = matches.into_iter().next() else {
        return Ok(None);
    };
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let binding = SidecarAssociation::new(locked, archive.ok_or_else(invalid)?, metadata)?;
    let mut url = normalized(artifact.clone());
    let path = format!("{}.metadata", url.path());
    url.set_path(&path);
    Ok(Some((url, binding)))
}
type Association = (Option<ContentHash>, Option<ContentHash>);
fn json_links(response: &Response, artifact: &Url) -> Result<Vec<Association>, Error> {
    let doc: serde_json::Value = serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    let version = doc
        .pointer("/meta/api-version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("1.0");
    if version.split('.').next() != Some("1") {
        return Err(invalid());
    }
    let files = doc
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(invalid)?;
    let mut found = vec![];
    for file in files {
        let Some(url) = file
            .get("url")
            .and_then(serde_json::Value::as_str)
            .and_then(|s| response.url.join(s).ok())
        else {
            continue;
        };
        if normalized(url) != normalized(artifact.clone()) {
            continue;
        }
        let archive = file
            .pointer("/hashes/sha256")
            .and_then(serde_json::Value::as_str)
            .and_then(digest);
        let metadata = file
            .get("core-metadata")
            .or_else(|| file.get("dist-info-metadata"))
            .and_then(|v| v.get("sha256"))
            .and_then(serde_json::Value::as_str)
            .and_then(digest);
        found.push((archive, metadata));
    }
    Ok(found)
}
fn html_links(response: &Response, artifact: &Url) -> Result<Vec<Association>, Error> {
    let text = std::str::from_utf8(&response.body).map_err(|_| invalid())?;
    let input = BufferQueue::default();
    input.push_back(StrTendril::from(text));
    let tokenizer = Tokenizer::new(Links::default(), TokenizerOpts::default());
    let _ = tokenizer.feed(&input);
    tokenizer.end();
    let tags = tokenizer.sink.tags.into_inner();
    let mut base = response.url.clone();
    let mut base_seen = false;
    let mut found = Vec::new();
    for (name, attrs) in tags {
        if name == "base" && !base_seen {
            if let Some(href) = attrs.get("href") {
                base = response.url.join(href).map_err(|_| invalid())?;
                base_seen = true;
            }
            continue;
        }
        if name == "meta"
            && attrs
                .get("name")
                .is_some_and(|s| s == "pypi:repository-version")
            && attrs
                .get("content")
                .is_some_and(|s| s.split('.').next() != Some("1"))
        {
            return Err(invalid());
        }
        if name != "a" {
            continue;
        }
        let Some(url) = attrs.get("href").and_then(|s| base.join(s).ok()) else {
            continue;
        };
        if normalized(url.clone()) != normalized(artifact.clone()) {
            continue;
        }
        let archive = url
            .fragment()
            .and_then(|s| s.strip_prefix("sha256="))
            .and_then(digest);
        let metadata = attrs
            .get("data-core-metadata")
            .or_else(|| attrs.get("data-dist-info-metadata"))
            .and_then(|s| s.strip_prefix("sha256="))
            .and_then(digest);
        found.push((archive, metadata));
    }
    Ok(found)
}
