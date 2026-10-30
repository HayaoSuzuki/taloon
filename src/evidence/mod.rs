//! Per-artifact declarations and validated proof strength; all readers use memory only.
use crate::{
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, Error,
        EvidenceRequirement, PackageId, RequestId, SourceId, safe_locator,
    },
    policy::LicenseExpression,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

const MAX_METADATA: u64 = 4 * 1024 * 1024;
const MAX_TAR_EXPANSION: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Proof {
    ArchiveVerified,
    IndexAssociated {
        source: ContentHash,
        request: RequestId,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordData {
    schema: u32,
    content: ContentHash,
    format: ArtifactFormat,
    metadata_hash: ContentHash,
    headers_hash: ContentHash,
    proof: Proof,
    metadata: String,
}
/// A versioned, integrity-checked observation from the trusted local cache.
/// This is not a signature proving that an arbitrary third party inspected bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    data: RecordData,
    checksum: ContentHash,
}
#[derive(Debug, Clone)]
struct Metadata {
    version: String,
    expression: Vec<String>,
    legacy: Vec<String>,
    warnings: Vec<Error>,
}
/// Observed declarations remain available even when their license is unknown.
#[derive(Debug, Clone)]
pub struct Evidence {
    record: EvidenceRecord,
    metadata: Metadata,
}
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceSummary {
    pub content_hash: ContentHash,
    pub metadata_hash: ContentHash,
    pub strength: EvidenceRequirement,
    pub metadata_version: String,
    pub original_expression: Vec<String>,
    pub original_license: Vec<String>,
    pub warnings: Vec<Error>,
}
/// Association advertised by the locked registry; it does not verify archive bytes.
#[derive(Debug, Clone, Copy)]
pub struct SidecarAssociation {
    archive: ContentHash,
    metadata: ContentHash,
}
impl SidecarAssociation {
    /// # Errors
    /// Rejects an index artifact whose digest differs from the lock.
    pub fn new(
        archive: ContentHash,
        index: ContentHash,
        metadata: ContentHash,
    ) -> Result<Self, Error> {
        if archive != index {
            return Err(Error::new(
                Code::HashMismatch,
                "index artifact digest differs from lock",
            ));
        }
        Ok(Self { archive, metadata })
    }
}
fn failure(message: &str) -> Error {
    Error::new(Code::Archive, message)
}
fn read_metadata(reader: impl Read) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_METADATA + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure("cannot read distribution metadata"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_METADATA {
        return Err(failure("metadata exceeds the 4 MiB reading limit"));
    }
    Ok(bytes)
}
fn member_path(name: &str, format: ArtifactFormat) -> bool {
    let mut parts = name.split('/');
    let Some(root) = parts.next() else {
        return false;
    };
    let Some(file) = parts.next() else {
        return false;
    };
    if parts.next().is_some() || root.is_empty() || root == ".." || root == "." {
        return false;
    }
    match format {
        ArtifactFormat::Wheel => {
            root.strip_suffix(".dist-info")
                .is_some_and(|base| !base.is_empty())
                && file == "METADATA"
        }
        ArtifactFormat::Sdist => file == "PKG-INFO",
    }
}
fn wheel_metadata(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| failure("invalid wheel ZIP archive"))?;
    if archive.has_duplicate_names() {
        return Err(failure("duplicate ZIP members"));
    }
    let mut metadata = None;
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|_| failure("invalid ZIP member"))?;
        let is_metadata = {
            let name = file
                .name()
                .map_err(|_| failure("invalid ZIP member name"))?;
            member_path(name.as_ref(), ArtifactFormat::Wheel)
        };
        if is_metadata {
            if metadata.is_some() {
                return Err(failure("multiple wheel metadata members"));
            }
            if file
                .unix_mode()
                .is_some_and(|mode| mode & 0o170_000 == 0o120_000)
            {
                return Err(failure("metadata member is a symlink"));
            }
            metadata = Some(read_metadata(file)?);
        }
    }
    metadata.ok_or_else(|| failure("wheel has no unique dist-info/METADATA"))
}
fn sdist_metadata(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let decoder = flate2::read::MultiGzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(decoder.take(MAX_TAR_EXPANSION + 1));
    let mut metadata = None;
    for entry in archive
        .entries()
        .map_err(|_| failure("invalid source tar archive"))?
    {
        let entry = entry.map_err(|_| failure("invalid tar member"))?;
        let path = entry.path_bytes();
        let name = std::str::from_utf8(&path).map_err(|_| failure("invalid tar member name"))?;
        if member_path(name.trim_start_matches("./"), ArtifactFormat::Sdist) {
            if metadata.is_some() {
                return Err(failure("multiple source metadata members"));
            }
            if !entry.header().entry_type().is_file() {
                return Err(failure("source metadata is not a regular file"));
            }
            metadata = Some(read_metadata(entry)?);
        }
    }
    let mut limited = archive.into_inner();
    std::io::copy(&mut limited, &mut std::io::sink())
        .map_err(|_| failure("corrupt gzip stream"))?;
    if limited.limit() == 0 {
        return Err(failure("source archive exceeds the 1 GiB expansion limit"));
    }
    metadata.ok_or_else(|| failure("source archive lacks top-level PKG-INFO"))
}
fn singleton<'a>(headers: &'a BTreeMap<String, Vec<String>>, key: &str) -> Result<&'a str, Error> {
    match headers.get(key).map(Vec::as_slice) {
        Some([value]) => Ok(value),
        _ => Err(Error::new(
            Code::InvalidMetadata,
            "missing or repeated metadata identity/version field",
        )),
    }
}
fn parse_metadata(request: &ArtifactRequest, text: &str) -> Result<Metadata, Error> {
    let mut headers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if line.is_empty() {
            break;
        }
        if line.starts_with([' ', '\t']) {
            let last = current
                .as_ref()
                .and_then(|key| headers.get_mut(key))
                .and_then(|v| v.last_mut())
                .ok_or_else(|| Error::new(Code::InvalidMetadata, "orphan metadata continuation"))?;
            last.push(' ');
            last.push_str(line.trim());
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| Error::new(Code::InvalidMetadata, "malformed metadata header"))?;
        if key.is_empty()
            || !key.bytes().all(|b| b.is_ascii_graphic() && b != b':')
            || line.contains('\r')
        {
            return Err(Error::new(
                Code::InvalidMetadata,
                "invalid metadata header characters",
            ));
        }
        let key = key.to_ascii_lowercase();
        headers
            .entry(key.clone())
            .or_default()
            .push(value.trim().into());
        current = Some(key);
    }
    let version = singleton(&headers, "metadata-version")?;
    let (major, minor) = version
        .split_once('.')
        .ok_or_else(|| Error::new(Code::InvalidMetadata, "invalid Metadata-Version"))?;
    let major: u16 = major
        .parse()
        .map_err(|_| Error::new(Code::InvalidMetadata, "invalid metadata major version"))?;
    let minor: u16 = minor
        .parse()
        .map_err(|_| Error::new(Code::InvalidMetadata, "invalid metadata minor version"))?;
    if !(1..=2).contains(&major) {
        return Err(Error::new(
            Code::InvalidMetadata,
            "unsupported metadata major version",
        ));
    }
    let mut warnings = Vec::new();
    if (major == 1 && minor > 2) || (major == 2 && minor == 0) {
        return Err(Error::new(
            Code::InvalidMetadata,
            "unrecognized historical metadata version",
        ));
    }
    if major == 2 && minor > 6 {
        warnings.push(Error::new(
            Code::MetadataVersion,
            "future metadata minor version: interpreting known fields",
        ));
    }
    let name = singleton(&headers, "name")?;
    let package_version = singleton(&headers, "version")?;
    let actual = PackageId::parse(name, package_version, request.package().source().clone())
        .map_err(|_| Error::new(Code::InvalidMetadata, "invalid metadata package identity"))?;
    if actual != *request.package() {
        return Err(Error::new(
            Code::IdentityMismatch,
            "metadata identity differs from locked package",
        ));
    }
    Ok(Metadata {
        version: version.into(),
        expression: headers
            .get("license-expression")
            .cloned()
            .unwrap_or_default(),
        legacy: headers.get("license").cloned().unwrap_or_default(),
        warnings,
    })
}
fn source_binding(request: &ArtifactRequest) -> Result<ContentHash, Error> {
    if let ArtifactLocator::Remote {
        index: Some(index), ..
    } = request.locator()
    {
        let source = SourceId::parse("registry", index)?;
        if source == *request.package().source() {
            return Ok(source.fingerprint());
        }
    }
    Err(Error::new(
        Code::InsufficientEvidence,
        "sidecar lacks matching locked registry association",
    ))
}
fn checksum(data: &RecordData) -> Result<ContentHash, Error> {
    serde_json::to_vec(data)
        .map(|bytes| ContentHash::digest(&bytes))
        .map_err(|_| Error::new(Code::Cache, "cannot encode evidence record"))
}
fn metadata_headers(text: &str) -> &str {
    let mut end = 0;
    for line in text.split_inclusive('\n') {
        end += line.len();
        if line.trim_end_matches(['\r', '\n']).is_empty() {
            break;
        }
    }
    &text[..end]
}
impl Evidence {
    fn new(
        request: &ArtifactRequest,
        bytes: Vec<u8>,
        content: ContentHash,
        proof: Proof,
    ) -> Result<Self, Error> {
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_METADATA {
            return Err(Error::new(
                Code::InvalidMetadata,
                "metadata exceeds the 4 MiB reading limit",
            ));
        }
        let metadata_hash = ContentHash::digest(&bytes);
        let text = String::from_utf8(bytes)
            .map_err(|_| Error::new(Code::InvalidMetadata, "metadata must be UTF-8"))?;
        let metadata = parse_metadata(request, &text)?;
        // Retain declarations and full-observation digests, not per-artifact descriptions.
        let headers = metadata_headers(&text);
        let data = RecordData {
            schema: 1,
            format: request.format(),
            content,
            metadata_hash,
            headers_hash: ContentHash::digest(headers.as_bytes()),
            proof,
            metadata: headers.to_owned(),
        };
        let record = EvidenceRecord {
            checksum: checksum(&data)?,
            data,
        };
        Ok(Self { record, metadata })
    }
    /// Verify the full archive before interpreting its unique metadata member.
    /// # Errors
    /// Rejects digest/identity mismatches, corrupt or ambiguous archives, invalid metadata,
    /// metadata above 4 MiB, and tar expansion above 1 GiB.
    pub fn from_archive(request: &ArtifactRequest, bytes: &[u8]) -> Result<Self, Error> {
        let content = ContentHash::digest(bytes);
        if request.hash().is_some_and(|h| h != content) {
            return Err(Error::new(
                Code::HashMismatch,
                "archive digest differs from locked content",
            ));
        }
        let metadata = match request.format() {
            ArtifactFormat::Wheel => wheel_metadata(bytes)?,
            ArtifactFormat::Sdist => sdist_metadata(bytes)?,
        };
        Self::new(request, metadata, content, Proof::ArchiveVerified)
    }
    /// Validate metadata bytes and the registry association without claiming archive verification.
    /// # Errors
    /// Rejects mismatching archive/metadata digests, wrong registries, and invalid metadata.
    pub fn from_sidecar(
        request: &ArtifactRequest,
        binding: SidecarAssociation,
        bytes: &[u8],
    ) -> Result<Self, Error> {
        if request.hash() != Some(binding.archive) || ContentHash::digest(bytes) != binding.metadata
        {
            return Err(Error::new(
                Code::HashMismatch,
                "sidecar binding or digest mismatch",
            ));
        }
        Self::new(
            request,
            bytes.to_vec(),
            binding.archive,
            Proof::IndexAssociated {
                source: source_binding(request)?,
                request: request.id(),
            },
        )
    }
    /// # Errors
    /// Returns unknown-license for missing, duplicate, or invalid declarations; never infers from classifiers.
    pub fn license(&self) -> Result<LicenseExpression, Error> {
        let values = if self.metadata.expression.is_empty() {
            &self.metadata.legacy
        } else {
            &self.metadata.expression
        };
        match values.as_slice() {
            [value] => LicenseExpression::parse(value),
            _ => Err(Error::new(
                Code::UnknownLicense,
                "missing or repeated license declaration",
            )),
        }
    }
    #[must_use]
    pub fn requirement_satisfied(&self, need: EvidenceRequirement) -> bool {
        matches!(self.record.data.proof, Proof::ArchiveVerified)
            || need == EvidenceRequirement::IndexAssociated
    }
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        self.record.data.content
    }
    #[must_use]
    pub fn summary(&self) -> EvidenceSummary {
        EvidenceSummary {
            content_hash: self.content_hash(),
            metadata_hash: self.record.data.metadata_hash,
            strength: if matches!(self.record.data.proof, Proof::ArchiveVerified) {
                EvidenceRequirement::ArchiveVerified
            } else {
                EvidenceRequirement::IndexAssociated
            },
            metadata_version: self.metadata.version.clone(),
            original_expression: self
                .metadata
                .expression
                .iter()
                .map(|s| safe_locator(s))
                .collect(),
            original_license: self
                .metadata
                .legacy
                .iter()
                .map(|s| safe_locator(s))
                .collect(),
            warnings: self.metadata.warnings.clone(),
        }
    }
    #[must_use]
    pub fn to_record(&self) -> EvidenceRecord {
        self.record.clone()
    }
    /// Restore trusted local-cache provenance and reparse declarations under the current parser.
    /// # Errors
    /// Rejects incompatible schemas, changed bytes/provenance, and wrong package/content/source bindings.
    pub fn from_record(request: &ArtifactRequest, record: EvidenceRecord) -> Result<Self, Error> {
        let data = &record.data;
        if data.metadata.len() > usize::try_from(MAX_METADATA).unwrap_or(usize::MAX)
            || data.schema != 1
            || data.format != request.format()
            || checksum(data)? != record.checksum
            || ContentHash::digest(data.metadata.as_bytes()) != data.headers_hash
        {
            return Err(Error::new(
                Code::Cache,
                "corrupt or incompatible evidence record",
            ));
        }
        if request.hash().is_some_and(|hash| hash != data.content) {
            return Err(Error::new(
                Code::HashMismatch,
                "cached content differs from locked artifact",
            ));
        }
        if let Proof::IndexAssociated {
            source,
            request: association,
        } = data.proof
            && (source_binding(request)? != source || association != request.id())
        {
            return Err(Error::new(
                Code::IdentityMismatch,
                "cached sidecar belongs to another source",
            ));
        }
        let metadata = parse_metadata(request, &data.metadata)?;
        Ok(Self { record, metadata })
    }
}
