//! Only consumed Simple API fields are retained; malformed known shapes fail closed.
use super::{Binding, digest, invalid, normalized, record};
use crate::acquisition::http::Response;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};
use taloon::model::{ContentHash, Error};
use url::Url;

// Serde structs also accept positional sequences. Every consumed record must
// instead be a JSON object; keep that wire invariant in one private wrapper.
#[derive(Default)]
struct Object<T>(T);
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for ObjectVisitor<T> {
            type Value = Object<T>;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                T::deserialize(serde::de::value::MapAccessDeserializer::new(map)).map(Object)
            }
        }
        deserializer.deserialize_map(ObjectVisitor(PhantomData))
    }
}

#[derive(Deserialize)]
struct Document {
    #[serde(default)]
    meta: Object<Meta>,
    files: Vec<Object<File>>,
}
#[derive(Default, Deserialize)]
struct Meta {
    #[serde(rename = "api-version")]
    api_version: Option<String>,
}
#[derive(Default, Deserialize)]
struct Hashes {
    sha256: Option<String>,
}
#[derive(Deserialize)]
struct File {
    url: String,
    #[serde(default)]
    hashes: Object<Hashes>,
    #[serde(
        default,
        rename = "core-metadata",
        deserialize_with = "present_metadata"
    )]
    metadata: Option<Advertisement>,
    #[serde(
        default,
        rename = "dist-info-metadata",
        deserialize_with = "present_metadata"
    )]
    legacy_metadata: Option<Advertisement>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Advertisement {
    Hashes(Object<Hashes>),
    Availability(bool),
}
impl Advertisement {
    fn hash(&self) -> Option<ContentHash> {
        match self {
            Self::Hashes(hashes) => hashes.0.sha256.as_deref().and_then(digest),
            // Either availability hint lacks the digest required as evidence.
            Self::Availability(true | false) => None,
        }
    }
}
// Option alone treats explicit null as missing, which could incorrectly select
// the legacy field. Present values must decode as a real advertisement.
fn present_metadata<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Advertisement>, D::Error> {
    Advertisement::deserialize(deserializer).map(Some)
}

pub(super) fn links(
    response: &Response,
    targets: &BTreeSet<Url>,
) -> Result<BTreeMap<Url, Binding>, Error> {
    let Object(document): Object<Document> =
        serde_json::from_slice(&response.body).map_err(|_| invalid())?;
    if document
        .meta
        .0
        .api_version
        .as_deref()
        .unwrap_or("1.0")
        .split('.')
        .next()
        != Some("1")
    {
        return Err(invalid());
    }
    let mut found = BTreeMap::new();
    for Object(file) in document.files {
        let Ok(url) = response.url.join(&file.url) else {
            continue;
        };
        let url = normalized(url);
        if !targets.contains(&url) {
            continue;
        }
        let archive = file.hashes.0.sha256.as_deref().and_then(digest);
        let metadata = file
            .metadata
            .as_ref()
            .or(file.legacy_metadata.as_ref())
            .and_then(Advertisement::hash);
        record(&mut found, url, Binding::from_hashes(archive, metadata));
    }
    Ok(found)
}
