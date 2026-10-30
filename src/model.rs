//! Validated identities shared by the pure inspection modules.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::{fmt, str::FromStr};
use url::Url;

/// Stable machine-readable diagnostic categories, independent of wording.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    Configuration,
    Expired,
    LockFormat,
    UnsupportedSource,
    MissingArtifact,
    Acquisition,
    HashMismatch,
    IdentityMismatch,
    InvalidMetadata,
    UnknownLicense,
    InsufficientEvidence,
    MissingResult,
    InvalidResult,
    Cache,
    Archive,
    UnusedPermission,
    CacheWrite,
    MetadataVersion,
}

/// A safe diagnostic message. Callers must not put credential-bearing locators in messages.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct Error {
    pub code: Code,
    pub message: String,
}
impl Error {
    #[must_use]
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
pub type InputError = Error;
pub type LicenseError = Error;
pub type EvidenceError = Error;
pub type AcquisitionFailure = Error;

/// A UTC calendar day supplied by the caller, inclusive for expiry comparisons.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EvaluationDate(time::Date);
impl EvaluationDate {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let format = time::format_description::parse_borrowed::<2>("[year]-[month]-[day]")
            .map_err(|_| Error::new(Code::Configuration, "invalid date format"))?;
        time::Date::parse(value, &format)
            .map(Self)
            .map_err(|_| Error::new(Code::Configuration, "expected a valid YYYY-MM-DD date"))
    }
}

/// SHA-256 content identity; digest algorithms cannot be confused with raw strings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ContentHash([u8; 32]);
impl ContentHash {
    #[must_use]
    pub fn digest(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn parse(value: &str) -> Result<Self, Error> {
        let hex = value
            .strip_prefix("sha256:")
            .ok_or_else(|| Error::new(Code::Configuration, "expected sha256 digest"))?;
        if hex.len() != 64 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(Error::new(Code::Configuration, "invalid SHA-256 digest"));
        }
        let mut bytes = [0; 32];
        for (i, out) in bytes.iter_mut().enumerate() {
            *out = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| Error::new(Code::Configuration, "invalid digest"))?;
        }
        Ok(Self(bytes))
    }
}
impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("sha256:")?;
        for b in self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}
impl Serialize for ContentHash {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(d)?).map_err(serde::de::Error::custom)
    }
}

/// Source kind stays distinct even if two locators spell the same location.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Registry,
    Url,
    Path,
    Git,
    Directory,
    Editable,
    Virtual,
}
impl SourceKind {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn parse(s: &str) -> Result<Self, Error> {
        match s {
            "registry" => Ok(Self::Registry),
            "url" => Ok(Self::Url),
            "path" => Ok(Self::Path),
            "git" => Ok(Self::Git),
            "directory" => Ok(Self::Directory),
            "editable" => Ok(Self::Editable),
            "virtual" => Ok(Self::Virtual),
            _ => Err(Error::new(
                Code::UnsupportedSource,
                "unrecognized source kind",
            )),
        }
    }
}

/// Canonical source identity. Authentication userinfo does not change package identity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceId {
    kind: SourceKind,
    location: String,
}
impl SourceId {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn parse(kind: &str, location: &str) -> Result<Self, Error> {
        let kind = SourceKind::parse(kind)?;
        let location = if matches!(kind, SourceKind::Registry | SourceKind::Url) {
            let mut url = parse_url(location)?;
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_fragment(None);
            if kind == SourceKind::Registry {
                let path = url.path().trim_end_matches('/').to_owned();
                url.set_path(&path);
            }
            url.to_string()
        } else {
            location.to_owned()
        };
        Ok(Self { kind, location })
    }
    #[must_use]
    pub fn kind(&self) -> SourceKind {
        self.kind
    }
    #[must_use]
    pub fn location(&self) -> &str {
        &self.location
    }
    #[must_use]
    pub fn fingerprint(&self) -> ContentHash {
        ContentHash::digest(format!("{:?}:{}", self.kind, self.location).as_bytes())
    }
}
impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}", self.kind, safe_locator(&self.location))
    }
}

/// Python package identity uses normalized name and parsed version equality, including local versions.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PackageId {
    name: String,
    version: pep440_rs::Version,
    source: SourceId,
}
impl PackageId {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn parse(name: &str, version: &str, source: SourceId) -> Result<Self, Error> {
        let name = normalize_name(name)?;
        let version = pep440_rs::Version::from_str(version)
            .map_err(|_| Error::new(Code::Configuration, "invalid Python version"))?;
        Ok(Self {
            name,
            version,
            source,
        })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn version(&self) -> String {
        self.version.to_string()
    }
    #[must_use]
    pub fn source(&self) -> &SourceId {
        &self.source
    }
    #[must_use]
    pub fn key(&self) -> String {
        format!(
            "{}@{}:{}",
            self.name,
            self.version,
            self.source.fingerprint()
        )
    }
}
impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}=={} ({})", self.name, self.version, self.source)
    }
}

/// # Errors
/// Returns a typed diagnostic if input validation fails.
pub fn normalize_name(name: &str) -> Result<String, Error> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name.as_bytes()[name.len() - 1].is_ascii_alphanumeric()
    {
        return Err(Error::new(
            Code::Configuration,
            "invalid Python package name",
        ));
    }
    let mut result = String::new();
    let mut separator = false;
    for b in name.bytes() {
        if matches!(b, b'-' | b'_' | b'.') {
            if !separator {
                result.push('-');
            }
            separator = true;
        } else {
            result.push(char::from(b.to_ascii_lowercase()));
            separator = false;
        }
    }
    Ok(result)
}

pub(crate) fn parse_url(value: &str) -> Result<Url, Error> {
    let url = Url::parse(value)
        .map_err(|_| Error::new(Code::Configuration, "invalid artifact/source URL"))?;
    if !matches!(url.scheme(), "https" | "http") {
        return Err(Error::new(
            Code::UnsupportedSource,
            "unsupported URL scheme",
        ));
    }
    Ok(url)
}
/// Strip credentials and query values while keeping enough locator context to diagnose failures.
#[must_use]
pub fn safe_locator(value: &str) -> String {
    if let Ok(mut u) = Url::parse(value) {
        let _ = u.set_username("");
        let _ = u.set_password(None);
        u.set_fragment(None);
        if u.query().is_some() {
            u.set_query(Some("redacted"));
        }
        u.to_string()
    } else if value.contains("://") || value.contains('@') || value.contains('?') {
        "[invalid/redacted locator]".to_owned()
    } else {
        value.to_owned()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormat {
    Wheel,
    Sdist,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirement {
    IndexAssociated,
    ArchiveVerified,
}

/// Fetch locations are private data. They are never serialized into reports or cache records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactLocator {
    Remote { index: Option<String>, url: String },
    Local(String),
}
impl ArtifactLocator {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn registry(index: &str, url: &str) -> Result<Self, Error> {
        Ok(Self::Remote {
            index: Some(parse_url(index)?.to_string()),
            url: parse_url(url)?.to_string(),
        })
    }
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn url(url: &str) -> Result<Self, Error> {
        Ok(Self::Remote {
            index: None,
            url: parse_url(url)?.to_string(),
        })
    }
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn local(path: &str) -> Result<Self, Error> {
        if path.is_empty() {
            Err(Error::new(Code::Configuration, "empty local artifact path"))
        } else {
            Ok(Self::Local(path.into()))
        }
    }
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Remote { url, .. } => safe_locator(url),
            Self::Local(path) => path.clone(),
        }
    }
    pub(crate) fn key(&self) -> String {
        match self {
            Self::Remote { url, .. } => url.clone(),
            Self::Local(path) => path.clone(),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct RequestId(ContentHash);
impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A request binds a package, location and expected content; remote content requires a lock digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactRequest {
    id: RequestId,
    package: PackageId,
    locator: ArtifactLocator,
    format: ArtifactFormat,
    hash: Option<ContentHash>,
    requirement: EvidenceRequirement,
}
impl ArtifactRequest {
    /// # Errors
    /// Returns a typed diagnostic if input validation fails.
    pub fn new(
        package: PackageId,
        locator: ArtifactLocator,
        format: ArtifactFormat,
        hash: Option<ContentHash>,
    ) -> Result<Self, Error> {
        match &locator {
            ArtifactLocator::Remote { index, url } => {
                parse_url(url)?;
                if let Some(index) = index {
                    parse_url(index)?;
                }
            }
            ArtifactLocator::Local(path) if path.is_empty() => {
                return Err(Error::new(
                    Code::MissingArtifact,
                    "empty local artifact path",
                ));
            }
            ArtifactLocator::Local(_) => {}
        }
        if matches!(locator, ArtifactLocator::Remote { .. }) && hash.is_none() {
            return Err(Error::new(
                Code::MissingArtifact,
                "remote artifact lacks a lock digest",
            ));
        }
        let id = RequestId(ContentHash::digest(
            format!("{}|{}|{hash:?}|{format:?}", package.key(), locator.key()).as_bytes(),
        ));
        Ok(Self {
            id,
            package,
            locator,
            format,
            hash,
            requirement: EvidenceRequirement::IndexAssociated,
        })
    }
    #[must_use]
    pub fn requiring(mut self, requirement: EvidenceRequirement) -> Self {
        self.requirement = requirement;
        self
    }
    #[must_use]
    pub fn id(&self) -> RequestId {
        self.id
    }
    #[must_use]
    pub fn package(&self) -> &PackageId {
        &self.package
    }
    #[must_use]
    pub fn locator(&self) -> &ArtifactLocator {
        &self.locator
    }
    #[must_use]
    pub fn format(&self) -> ArtifactFormat {
        self.format
    }
    #[must_use]
    pub fn hash(&self) -> Option<ContentHash> {
        self.hash
    }
    #[must_use]
    pub fn requirement(&self) -> EvidenceRequirement {
        self.requirement
    }
}
