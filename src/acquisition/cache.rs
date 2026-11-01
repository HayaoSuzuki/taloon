//! Trusted persisted observations; cache damage is a miss, never a verdict.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufWriter, Read, Write},
    path::PathBuf,
};
use taloon::{
    evidence::{Evidence, EvidenceRecord, MAX_METADATA_BYTES},
    model::{ArtifactFormat, ArtifactRequest, Code, ContentHash, Error, EvidenceRequirement},
};
// A JSON string may use six bytes per input byte; preserve schema headroom.
const MAX_RECORD: u64 = 6 * MAX_METADATA_BYTES + 64 * 1024;
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
            let entry = entry.map_err(|_| failure())?;
            if !entry.file_name().to_str().is_some_and(is_record_name) {
                continue;
            }
            let kind = match entry.file_type() {
                Ok(kind) => kind,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return Err(failure()),
            };
            if !kind.is_file() || kind.is_symlink() {
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
                .map_err(|_| failure())?;
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
        let e = Evidence::from_record(r, e.to_record())?;
        fs::create_dir_all(&self.root).map_err(|_| failure())?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".evidence-")
            .tempfile_in(&self.root)
            .map_err(|_| failure())?;
        let envelope = Envelope {
            schema: 1,
            observation: e.to_record(),
        };
        {
            let mut buffered = BufWriter::new(&mut temporary);
            serde_json::to_writer(&mut buffered, &envelope).map_err(|_| failure())?;
            buffered.flush().map_err(|_| failure())?;
        }
        temporary.as_file().sync_all().map_err(|_| failure())?;
        temporary
            .persist(self.path(
                r,
                e.content_hash(),
                e.requirement_satisfied(EvidenceRequirement::ArchiveVerified),
            ))
            .map_err(|_| failure())?;
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

    proptest! {
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
            prop_assert_eq!(cache.clear().unwrap(), 1);
            prop_assert!(!path.exists());
            for name in &preserved {
                prop_assert_eq!(std::fs::read(temp.path().join(name)).unwrap(), b"keep");
            }
            prop_assert_eq!(cache.clear().unwrap(), 0);
        }
    }
}
