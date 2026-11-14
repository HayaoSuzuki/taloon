//! Trusted persisted observations; cache damage is a miss, never a verdict.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};
use taloon::{
    evidence::{Evidence, EvidenceRecord, MAX_LICENSE_OBSERVATION_BYTES, MAX_METADATA_BYTES},
    model::{ArtifactFormat, ArtifactRequest, Code, ContentHash, Error, EvidenceRequirement},
};
// A JSON string may use six bytes per input byte; preserve schema headroom.
const MAX_RECORD: u64 = 6 * (MAX_METADATA_BYTES + MAX_LICENSE_OBSERVATION_BYTES as u64) + 64 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: u32,
    observation: EvidenceRecord,
}
pub(crate) struct EvidenceCache {
    root: PathBuf,
}
fn failure() -> Error {
    Error::new(Code::Cache, "evidence cache is unavailable")
}
// Filesystem diagnostics must not expose configured paths or record contents.
fn io_failure(_: std::io::Error) -> Error {
    failure()
}
fn json_failure(_: serde_json::Error) -> Error {
    failure()
}
// Directory entries can disappear between enumeration and metadata inspection.
// Keep that classification independent of the filesystem so both OS outcomes
// are deterministic to verify on platforms which cache DirEntry file types.
fn regular_record(result: std::io::Result<fs::FileType>) -> Result<bool, Error> {
    match result {
        Ok(kind) => Ok(kind.is_file() && !kind.is_symlink()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(failure()),
    }
}
impl EvidenceCache {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }
    /// Remove completed v1 records without following links or descending into
    /// directories. The configured root may contain unrelated files. Concurrent
    /// publishers can recreate records; clearing is intentionally not atomic.
    pub(crate) fn clear(&self) -> Result<u64, Error> {
        // Trailing '/' or '/.' would make lstat follow a root directory link.
        // Components removes redundant spellings without resolving symlinks.
        let root_path: PathBuf = self.root.components().collect();
        let root = match fs::symlink_metadata(&root_path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => return Err(failure()),
        };
        if !root.is_dir() || root.file_type().is_symlink() {
            return Err(failure());
        }
        let entries = match fs::read_dir(&root_path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => return Err(failure()),
        };
        let mut removed = 0_u64;
        for entry in entries {
            let entry = entry.map_err(io_failure)?;
            if !entry.file_name().to_str().is_some_and(is_record_name) {
                continue;
            }
            if !regular_record(entry.file_type())? {
                continue;
            }
            match fs::remove_file(entry.path()) {
                Ok(()) => removed = removed.saturating_add(1),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(failure()),
            }
        }
        Ok(removed)
    }
    fn path(&self, r: &ArtifactRequest, hash: ContentHash, strong: bool) -> PathBuf {
        let format = match r.format() {
            ArtifactFormat::Wheel => "wheel",
            ArtifactFormat::Sdist => "sdist",
        };
        let digest = hash.to_string().replace(':', "-");
        let association = if strong {
            "archive".into()
        } else {
            format!("index-{}", r.id()).replace(':', "-")
        };
        self.root
            .join(format!("v1-{association}-{format}-{digest}.json"))
    }
    pub(super) fn get(
        &self,
        r: &ArtifactRequest,
        observed: ContentHash,
    ) -> Result<Option<Evidence>, Error> {
        if r.hash().is_some_and(|locked| locked != observed) {
            return Ok(None);
        }
        for strong in [true, false] {
            if !strong && r.requirement() == EvidenceRequirement::ArchiveVerified {
                continue;
            }
            let file = match fs::File::open(self.path(r, observed, strong)) {
                Ok(file) => file,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(failure()),
            };
            let mut bytes = Vec::new();
            file.take(MAX_RECORD + 1)
                .read_to_end(&mut bytes)
                .map_err(io_failure)?;
            if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_RECORD {
                continue;
            }
            let Ok(envelope) = serde_json::from_slice::<Envelope>(&bytes) else {
                continue;
            };
            if envelope.schema != 1 {
                continue;
            }
            if let Ok(evidence) = Evidence::from_record(r, envelope.observation)
                && evidence.content_hash() == observed
                && evidence.requirement_satisfied(r.requirement())
            {
                return Ok(Some(evidence));
            }
        }
        Ok(None)
    }
    pub(super) fn put(&self, r: &ArtifactRequest, e: &Evidence) -> Result<(), Error> {
        // Validate the association before publishing caller-supplied observations.
        let e = e.rebind(r)?;
        fs::create_dir_all(&self.root).map_err(io_failure)?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".evidence-")
            .tempfile_in(&self.root)
            .map_err(io_failure)?;
        let envelope = Envelope {
            schema: 1,
            observation: e.to_record(),
        };
        {
            let mut buffered = BufWriter::new(&mut temporary);
            serde_json::to_writer(&mut buffered, &envelope).map_err(json_failure)?;
            buffered.flush().map_err(io_failure)?;
        }
        temporary.as_file().sync_all().map_err(io_failure)?;
        temporary
            .persist(self.path(
                r,
                e.content_hash(),
                e.requirement_satisfied(EvidenceRequirement::ArchiveVerified),
            ))
            .map_err(|error| io_failure(error.error))?;
        Ok(())
    }
}

// Match the complete canonical filename produced by path(), never a prefix or
// record contents: corrupt evidence must remain removable, unrelated data intact.
fn is_record_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".json") else {
        return false;
    };
    let parts: Vec<_> = stem.split('-').collect();
    let (format, hash) = match parts.as_slice() {
        ["v1", "archive", format, "sha256", hash] => (*format, *hash),
        ["v1", "index", "sha256", id, format, "sha256", hash] if canonical_digest(id) => {
            (*format, *hash)
        }
        _ => return false,
    };
    matches!(format, "wheel" | "sdist") && canonical_digest(hash)
}
fn canonical_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixtures::{metadata, request, wheel};
    use proptest::prelude::*;
    use taloon::evidence::SidecarAssociation;

    #[test]
    fn filesystem_failure_does_not_disclose_private_path() {
        // Arrange
        let source = std::io::Error::other("/private/token-secret/cache.json");
        // Act
        let error = io_failure(source);
        // Assert
        assert_eq!(error.code, Code::Cache);
        assert_eq!(error.message, "evidence cache is unavailable");
    }
    #[test]
    fn serialization_failure_does_not_disclose_record_data() {
        // Arrange
        let source = serde_json::Error::io(std::io::Error::other("secret record data"));
        // Act
        let error = json_failure(source);
        // Assert
        assert_eq!(error.code, Code::Cache);
        assert_eq!(error.message, "evidence cache is unavailable");
    }
    #[test]
    fn clear_recognizes_records_written_by_both_evidence_producers() {
        let temp = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(temp.path().to_owned());
        let meta = metadata("License-Expression: MIT");
        let bytes = wheel(&meta);
        let request = request(&bytes, ArtifactFormat::Wheel);
        let archive = ContentHash::digest(&bytes);
        let evidence = Evidence::from_archive(&request, &bytes).unwrap();
        cache.put(&request, &evidence).unwrap();
        let sidecar = Evidence::from_sidecar(
            &request,
            SidecarAssociation::new(archive, archive, ContentHash::digest(&meta)).unwrap(),
            &meta,
        )
        .unwrap();
        cache.put(&request, &sidecar).unwrap();
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
        assert_eq!(cache.clear().unwrap(), 2);
        assert!(cache.get(&request, archive).unwrap().is_none());
    }

    #[test]
    fn oversized_persisted_record_is_a_cache_miss() {
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(directory.path().to_owned());
        let request = request(b"archive", ArtifactFormat::Wheel);
        let hash = ContentHash::digest(b"archive");
        fs::File::create(cache.path(&request, hash, true))
            .unwrap()
            .set_len(MAX_RECORD + 1)
            .unwrap();
        // Act
        let observation = cache.get(&request, hash).unwrap();
        // Assert
        assert!(observation.is_none());
    }

    #[test]
    fn unreadable_record_reports_cache_error() {
        // Arrange: a directory opens as a File but cannot be read as JSON bytes.
        let directory = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(directory.path().to_owned());
        let request = request(b"archive", ArtifactFormat::Wheel);
        let hash = ContentHash::digest(b"archive");
        fs::create_dir(cache.path(&request, hash, true)).unwrap();
        // Act
        let error = cache.get(&request, hash).err().unwrap();
        // Assert
        assert_eq!(error.code, Code::Cache);
    }

    #[test]
    fn failed_publication_does_not_replace_a_directory() {
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(directory.path().to_owned());
        let bytes = wheel(&metadata("License-Expression: MIT"));
        let request = request(&bytes, ArtifactFormat::Wheel);
        let evidence = Evidence::from_archive(&request, &bytes).unwrap();
        let destination = cache.path(&request, evidence.content_hash(), true);
        fs::create_dir(&destination).unwrap();
        // Act
        let error = cache.put(&request, &evidence).unwrap_err();
        // Assert
        assert_eq!(error.code, Code::Cache);
        assert!(destination.is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn nonwritable_cache_cannot_create_a_temporary_record() {
        use std::os::unix::fs::PermissionsExt;
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(directory.path().to_owned());
        let bytes = wheel(&metadata("License-Expression: MIT"));
        let request = request(&bytes, ArtifactFormat::Wheel);
        let evidence = Evidence::from_archive(&request, &bytes).unwrap();
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o500)).unwrap();
        // Act
        let result = cache.put(&request, &evidence);
        fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
        // Assert
        assert_eq!(result.unwrap_err().code, Code::Cache);
    }

    #[test]
    fn vanished_directory_entry_is_skipped() {
        // Arrange
        let error = std::io::Error::from(std::io::ErrorKind::NotFound);
        // Act
        let regular = regular_record(Err(error)).unwrap();
        // Assert
        assert!(!regular);
    }

    #[test]
    fn directory_entry_inspection_failure_is_not_a_successful_clear() {
        // Arrange
        let error = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        // Act
        let error = regular_record(Err(error)).unwrap_err();
        // Assert
        assert_eq!(error.code, Code::Cache);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn clear_preserves_non_utf8_names() {
        use std::{ffi::OsString, os::unix::ffi::OsStringExt};
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join(OsString::from_vec(vec![b'v', b'1', 255]));
        std::fs::write(&path, b"keep").unwrap();
        assert_eq!(
            EvidenceCache::new(temp.path().to_owned()).clear().unwrap(),
            0
        );
        assert_eq!(std::fs::read(path).unwrap(), b"keep");
    }

    #[test]
    fn maximum_escaped_file_observation_survives_disk_cache() {
        use std::fmt::Write as _;
        // Arrange: legal maximum text/path bytes with worst-case JSON expansion.
        let temp = tempfile::tempdir().unwrap();
        let cache = EvidenceCache::new(temp.path().to_owned());
        let paths: Vec<_> = (0..32)
            .map(|n| format!("{n:02}{}", "\"".repeat(4094)))
            .collect();
        let mut metadata = String::from("Metadata-Version: 2.4\nName: example\nVersion: 1.0\n");
        for path in &paths {
            writeln!(metadata, "License-File: {path}").unwrap();
        }
        metadata.push_str("X-Padding: ");
        metadata.push_str(&"\u{1}".repeat(4 * 1024 * 1024 - metadata.len() - 2));
        metadata.push_str("\n\n");
        let text = "\u{1}".repeat(32 * 1024);
        let names: Vec<_> = paths
            .iter()
            .map(|p| format!("example-1.0.dist-info/licenses/{p}"))
            .collect();
        let files: Vec<_> = names
            .iter()
            .map(|p| (p.as_str(), text.as_bytes()))
            .collect();
        let bytes = crate::testfixtures::wheel_with_files(metadata.as_bytes(), &files);
        let request = crate::testfixtures::request(&bytes, ArtifactFormat::Wheel);
        let evidence = Evidence::from_archive(&request, &bytes).unwrap();
        cache.put(&request, &evidence).unwrap();
        // Act
        let restored = cache.get(&request, ContentHash::digest(&bytes)).unwrap();
        // Assert
        assert!(restored.is_some());
    }

    proptest! {
        #[test]
        fn clearing_an_already_cleared_record_is_idempotent(identity in any::<[u8;32]>()) {
            // Arrange
            let temp = tempfile::tempdir().unwrap();
            let cache = EvidenceCache::new(temp.path().to_owned());
            let request = request(&identity, ArtifactFormat::Wheel);
            let path = cache.path(&request, ContentHash::digest(&identity), true);
            std::fs::write(path, b"damaged record").unwrap();
            cache.clear().unwrap();
            // Act
            let removed = cache.clear().unwrap();
            // Assert
            prop_assert_eq!(removed, 0);
        }

        #[test]
        fn clear_generated_records_preserves_other_names(
            archive in any::<[u8;32]>(), identity in any::<[u8;32]>(),
            sidecar in any::<bool>(), sdist in any::<bool>(), neighbor in "[a-z0-9_]{1,24}"
        ) {
            let temp = tempfile::tempdir().unwrap();
            let cache = EvidenceCache::new(temp.path().to_owned());
            let request = request(&archive, if sdist { ArtifactFormat::Sdist } else { ArtifactFormat::Wheel });
            // Exercise the actual producer's key builder, with independent content
            // identities and both association/format variants.
            let path = cache.path(&request, ContentHash::digest(&identity), !sidecar);
            std::fs::write(&path, b"damaged record").unwrap();
            let name = path.file_name().unwrap().to_str().unwrap();
            let preserved = [
                format!("keep-{neighbor}"), format!("x{name}"), format!("{name}.bak"),
                name.replacen("v1-", "v2-", 1), name.replace("sha256", "sha512"),
            ];
            for name in &preserved {
                std::fs::write(temp.path().join(name), b"keep").unwrap();
            }
            // Act
            let removed = cache.clear().unwrap();
            let remaining: Vec<_> = preserved.iter().map(|name| std::fs::read(temp.path().join(name)).unwrap()).collect();
            // Assert: every neighbor belongs to this single clear operation.
            prop_assert_eq!(removed, 1);
            prop_assert!(!path.exists());
            prop_assert_eq!(remaining, vec![b"keep".to_vec(); preserved.len()]);
        }
    }
}

#[cfg(all(test, unix))]
mod permission_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn unreadable_cache_directory_cannot_be_cleared() {
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("cache");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o000)).unwrap();
        // Act
        let result = EvidenceCache::new(root.clone()).clear();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        // Assert
        assert_eq!(result.unwrap_err().code, Code::Cache);
    }

    #[test]
    fn inaccessible_cache_parent_cannot_be_cleared() {
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("parent");
        let root = parent.join("cache");
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap();
        // Act
        let result = EvidenceCache::new(root).clear();
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
        // Assert
        assert_eq!(result.unwrap_err().code, Code::Cache);
    }

    #[test]
    fn nonwritable_cache_keeps_records_and_reports_clear_failure() {
        // Arrange
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("cache");
        fs::create_dir(&root).unwrap();
        let record = root.join(format!("v1-archive-wheel-sha256-{}.json", "11".repeat(32)));
        fs::write(&record, b"record").unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o500)).unwrap();
        // Act
        let result = EvidenceCache::new(root.clone()).clear();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        // Assert
        assert_eq!(result.unwrap_err().code, Code::Cache);
        assert_eq!(fs::read(record).unwrap(), b"record");
    }
}
