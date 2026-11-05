//! Per-artifact declarations and validated proof strength; all readers use memory only.
mod archive_files;
mod file_observation;
mod legacy;
use file_observation::FileObservation;
pub use file_observation::{LicenseFileKind, LicenseFileSummary};
mod text_match;
use crate::{
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, Error,
        EvidenceRequirement, LicenseProblem, PackageId, RequestId, SourceId,
    },
    policy::LicenseExpression,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    sync::Arc,
};
use taloon_decisions::evidence::{ProofBinding, content_matches, request_matches, sidecar_matches};

/// Maximum observed metadata bytes, shared with bounded acquisition and persistence.
pub const MAX_METADATA_BYTES: u64 = 4 * 1024 * 1024;
/// Maximum total bundled license text retained per observation.
pub const MAX_LICENSE_OBSERVATION_BYTES: usize = archive_files::MAX_LICENSE_TOTAL;
const MAX_TAR_EXPANSION: u64 = 1024 * 1024 * 1024;
const RECORD_SCHEMA: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Proof {
    ArchiveVerified,
    IndexAssociated {
        source: ContentHash,
        request: RequestId,
    },
}
impl Proof {
    fn binding(&self) -> ProofBinding<'_> {
        match self {
            Self::ArchiveVerified => ProofBinding::ArchiveVerified,
            Self::IndexAssociated { source, request } => ProofBinding::IndexAssociated {
                source: source.binding_bytes(),
                request: request.binding_bytes(),
            },
        }
    }
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
    license_files: FileObservation,
}
/// A versioned, integrity-checked observation from the trusted local cache.
/// This is not a signature proving that an arbitrary third party inspected bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    // Serialized payload is immutable; per-request interpretation stays in Evidence.
    data: Arc<RecordData>,
    checksum: ContentHash,
}
#[derive(Debug, Clone)]
struct Metadata {
    version: String,
    expression: Vec<String>,
    legacy: Vec<String>,
    // Byte span in the retained raw headers; no duplicate owned license text.
    legacy_span: Option<std::ops::Range<usize>>,
    files: Vec<String>,
    warnings: Vec<Error>,
}
/// Observed declarations remain available even when their license is unknown.
#[derive(Debug, Clone)]
pub struct Evidence {
    // Acquisition context is transient: persisted observations are content-keyed
    // and restored only after the runtime freshly observes local content.
    observed_request: RequestId,
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
    /// Interpretation source before any policy clarification; absent when unknown.
    pub license_origin: Option<LicenseOrigin>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub license_files: Vec<LicenseFileSummary>,
    pub warnings: Vec<Error>,
}
/// How observed evidence supplies an expression, independently of policy overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseOrigin {
    LicenseExpression,
    LegacyDeclaration,
    BundledFiles,
    LegacySupplement,
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
        if !content_matches(Some(archive.binding_bytes()), index.binding_bytes()) {
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
        .take(MAX_METADATA_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure("cannot read distribution metadata"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_METADATA_BYTES {
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
fn wheel_metadata(bytes: &[u8]) -> Result<(Vec<u8>, String), Error> {
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
        let path = file
            .name()
            .map_err(|_| failure("invalid ZIP member name"))?
            .into_owned();
        let is_metadata = member_path(&path, ArtifactFormat::Wheel);
        if is_metadata {
            if metadata.is_some() {
                return Err(failure("multiple wheel metadata members"));
            }
            if file
                .unix_mode()
                .is_some_and(|mode| !matches!(mode & 0o170_000, 0 | 0o100_000))
            {
                return Err(failure("metadata member is not a regular file"));
            }
            let root = format!(
                "{}/",
                path.split_once('/').expect("validated metadata path").0
            );
            metadata = Some((read_metadata(file)?, root));
        }
    }
    metadata.ok_or_else(|| failure("wheel has no unique dist-info/METADATA"))
}
fn sdist_metadata(bytes: &[u8]) -> Result<(Vec<u8>, String), Error> {
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
            let root = format!(
                "{}/",
                name.trim_start_matches("./")
                    .split_once('/')
                    .expect("validated metadata path")
                    .0
            );
            metadata = Some((read_metadata(entry)?, root));
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
    let mut legacy_span: Option<std::ops::Range<usize>> = None;
    let mut offset = 0;
    for physical in text.split_inclusive('\n') {
        let line = physical
            .strip_suffix('\n')
            .map_or(physical, |line| line.strip_suffix('\r').unwrap_or(line));
        let line_start = offset;
        let line_end = line_start + line.len();
        offset += physical.len();
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
            if current.as_deref() == Some("license")
                && let Some(span) = &mut legacy_span
            {
                span.end = line_end;
            }
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
        let values = headers.entry(key.clone()).or_default();
        values.push(value.trim().into());
        if key == "license" {
            legacy_span = (values.len() == 1).then_some(line_start + key.len() + 1..line_end);
        }
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
        legacy_span,
        files: headers.get("license-file").cloned().unwrap_or_default(),
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
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_METADATA_BYTES {
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
            schema: RECORD_SCHEMA,
            format: request.format(),
            content,
            metadata_hash,
            headers_hash: ContentHash::digest(headers.as_bytes()),
            proof,
            metadata: headers.to_owned(),
            license_files: FileObservation::NotRead,
        };
        let record = EvidenceRecord {
            checksum: checksum(&data)?,
            data: Arc::new(data),
        };
        Ok(Self {
            observed_request: request.id(),
            record,
            metadata,
        })
    }
    /// Verify the full archive before interpreting its unique metadata member.
    /// # Errors
    /// Rejects digest/identity mismatches, corrupt or ambiguous archives, invalid metadata,
    /// metadata above 4 MiB, and tar expansion above 1 GiB.
    pub fn from_archive(request: &ArtifactRequest, bytes: &[u8]) -> Result<Self, Error> {
        let content = ContentHash::digest(bytes);
        if !content_matches(
            request.hash().as_ref().map(ContentHash::binding_bytes),
            content.binding_bytes(),
        ) {
            return Err(Error::new(
                Code::HashMismatch,
                "archive digest differs from locked content",
            ));
        }
        let (metadata, root) = match request.format() {
            ArtifactFormat::Wheel => wheel_metadata(bytes)?,
            ArtifactFormat::Sdist => sdist_metadata(bytes)?,
        };
        let mut evidence = Self::new(request, metadata, content, Proof::ArchiveVerified)?;
        if evidence.needs_license_files() {
            let files =
                FileObservation::read(bytes, request.format(), &evidence.metadata.files, &root);
            let data = Arc::make_mut(&mut evidence.record.data);
            data.license_files = files;
            evidence.record.checksum = checksum(data)?;
        }
        Ok(evidence)
    }
    /// Validate metadata bytes and the registry association without claiming archive verification.
    /// # Errors
    /// Rejects mismatching archive/metadata digests, wrong registries, and invalid metadata.
    pub fn from_sidecar(
        request: &ArtifactRequest,
        binding: SidecarAssociation,
        bytes: &[u8],
    ) -> Result<Self, Error> {
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_METADATA_BYTES {
            return Err(Error::new(
                Code::InvalidMetadata,
                "metadata exceeds the 4 MiB reading limit",
            ));
        }
        let locked = request.hash();
        if !sidecar_matches(
            locked.as_ref().map(ContentHash::binding_bytes),
            binding.archive.binding_bytes(),
        ) || !content_matches(
            Some(binding.metadata.binding_bytes()),
            ContentHash::digest(bytes).binding_bytes(),
        ) {
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
    /// Returns unknown-license for duplicate/invalid declarations or unrecognized bundled text; never infers from classifiers.
    pub fn license(&self) -> Result<LicenseExpression, Error> {
        self.interpret_license().map(|(expression, _)| expression)
    }

    // Both acquisition and interpretation use this decision. A sidecar can
    // request an archive scan, but only the fresh archive's metadata authorizes it.
    fn file_constraint(&self) -> Option<legacy::FileConstraint> {
        if !self.metadata.expression.is_empty() {
            return None;
        }
        match self.metadata.legacy.as_slice() {
            [] => Some(legacy::FileConstraint::AnyRecognized),
            [value] => legacy::FileConstraint::for_legacy(value),
            _ => None,
        }
    }

    fn interpret_license(&self) -> Result<(LicenseExpression, LicenseOrigin), Error> {
        let values = if self.metadata.expression.is_empty() {
            &self.metadata.legacy
        } else {
            &self.metadata.expression
        };
        let declaration = match values.as_slice() {
            [value] if self.metadata.expression.is_empty() => legacy::interpret(
                value,
                self.metadata
                    .legacy_span
                    .as_ref()
                    .and_then(|span| self.record.data.metadata.get(span.clone())),
            )
            .map_err(|error| Error::license(LicenseProblem::UnrecognizedLegacy, error.message))
            .map(|expression| (expression, LicenseOrigin::LegacyDeclaration)),
            [value] => LicenseExpression::parse(value)
                .map_err(|error| Error::license(LicenseProblem::InvalidExpression, error.message))
                .map(|expression| (expression, LicenseOrigin::LicenseExpression)),
            [] => Err(Error::license(
                LicenseProblem::MissingDeclaration,
                "missing license declaration",
            )),
            _ => Err(Error::license(
                LicenseProblem::RepeatedDeclaration,
                "repeated license declaration",
            )),
        };
        if declaration.is_ok() {
            return declaration;
        }
        if let Some(constraint) = self.file_constraint()
            && !matches!(self.record.data.license_files, FileObservation::NotRead)
        {
            let expression = self.record.data.license_files.license()?;
            if !constraint.accepts(&expression) {
                return Err(Error::license(
                    LicenseProblem::LegacyFamilyMismatch,
                    "bundled license text conflicts with legacy license family",
                ));
            }
            let origin = if self.metadata.legacy.is_empty() {
                LicenseOrigin::BundledFiles
            } else {
                LicenseOrigin::LegacySupplement
            };
            return Ok((expression, origin));
        }
        declaration
    }
    /// Whether absent or narrowly incomplete declarations need archive-file evidence.
    /// Completed unknown scans return false, avoiding repeated archive fetches.
    #[must_use]
    pub fn needs_license_files(&self) -> bool {
        matches!(self.record.data.license_files, FileObservation::NotRead)
            && self.file_constraint().is_some()
    }
    #[must_use]
    pub fn requirement_satisfied(&self, need: EvidenceRequirement) -> bool {
        self.record.data.proof.binding().satisfies(need)
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
            strength: self.record.data.proof.binding().strength(),
            metadata_version: self.metadata.version.clone(),
            original_expression: self
                .metadata
                .expression
                .iter()
                .map(|s| crate::declaration::redact_urls(s))
                .collect(),
            original_license: self
                .metadata
                .legacy
                .iter()
                .map(|s| crate::declaration::redact_urls(s))
                .collect(),
            license_origin: self.interpret_license().ok().map(|(_, origin)| origin),
            license_files: self.record.data.license_files.summary(),
            warnings: self.metadata.warnings.clone(),
        }
    }
    #[must_use]
    pub fn to_record(&self) -> EvidenceRecord {
        self.record.clone()
    }
    /// Revalidate an acquired observation for an expected request. Hashless
    /// archives stay bound to their acquisition request; digest-pinned archives
    /// can be reused by verified content identity. Sidecars retain exact bindings.
    /// # Errors
    /// Rejects cross-request hashless observations and invalid record bindings.
    pub fn rebind(&self, request: &ArtifactRequest) -> Result<Self, Error> {
        let locked = request.hash();
        if !request_matches(
            locked.as_ref().map(ContentHash::binding_bytes),
            self.observed_request.binding_bytes(),
            request.id().binding_bytes(),
        ) {
            return Err(Error::new(
                Code::InvalidResult,
                "hashless observation belongs to another artifact request",
            ));
        }
        Self::from_record(request, self.to_record())
    }
    /// Restore trusted local-cache provenance and reparse declarations under the current parser.
    /// For a hashless local request, the runtime must first freshly hash its
    /// bytes and match that observed digest to the content-keyed cache record.
    /// # Errors
    /// Rejects incompatible schemas, changed bytes/provenance, and wrong package/content/source bindings.
    pub fn from_record(request: &ArtifactRequest, record: EvidenceRecord) -> Result<Self, Error> {
        let data = &record.data;
        if data.metadata.len() > usize::try_from(MAX_METADATA_BYTES).unwrap_or(usize::MAX)
            || !data.license_files.valid()
            || (!matches!(data.license_files, FileObservation::NotRead)
                && !matches!(data.proof, Proof::ArchiveVerified))
            || data.schema != RECORD_SCHEMA
            || data.format != request.format()
            || checksum(data)? != record.checksum
            || ContentHash::digest(data.metadata.as_bytes()) != data.headers_hash
        {
            return Err(Error::new(
                Code::Cache,
                "corrupt or incompatible evidence record",
            ));
        }
        let locked = request.hash();
        if !content_matches(
            locked.as_ref().map(ContentHash::binding_bytes),
            data.content.binding_bytes(),
        ) {
            return Err(Error::new(
                Code::HashMismatch,
                "cached content differs from locked artifact",
            ));
        }
        if let Proof::IndexAssociated { .. } = data.proof {
            let source = source_binding(request)?;
            if !data
                .proof
                .binding()
                .matches(source.binding_bytes(), request.id().binding_bytes())
            {
                return Err(Error::new(
                    Code::IdentityMismatch,
                    "cached sidecar belongs to another source",
                ));
            }
        }
        let metadata = parse_metadata(request, &data.metadata)?;
        Ok(Self {
            observed_request: request.id(),
            record,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests;
