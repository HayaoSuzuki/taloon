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
pub(super) struct EvidenceCache {
    root: PathBuf,
}
fn failure() -> Error {
    Error::new(Code::Cache, "evidence cache is unavailable")
}
impl EvidenceCache {
    pub(super) fn new(root: PathBuf) -> Self {
        Self { root }
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
