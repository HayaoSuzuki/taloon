//! Binary-owned I/O. The library sees only observations and failures.
mod cache;
#[cfg(test)]
mod tests;
use cache::EvidenceCache;
use std::{io::Read, path::Path};
use taloon::{
    evidence::Evidence,
    model::{ArtifactLocator, ArtifactRequest, Code, ContentHash, Error},
    report::{AcquisitionSummary, Diagnostic},
};
const MAX_ARCHIVE: u64 = 256 * 1024 * 1024;
fn cache_warning(warnings: &mut Vec<Diagnostic>, message: &str) {
    warnings.push(Diagnostic::warning(Error::new(Code::CacheWrite, message)));
}
fn acquire_local(
    req: &ArtifactRequest,
    root: &Path,
    cache: &EvidenceCache,
    stats: &mut AcquisitionSummary,
    warnings: &mut Vec<Diagnostic>,
) -> Result<Evidence, Error> {
    let ArtifactLocator::Local(path) = req.locator() else {
        return Err(Error::new(
            Code::Acquisition,
            "request is not a local archive",
        ));
    };
    let file = std::fs::File::open(root.join(path))
        .map_err(|_| Error::new(Code::Acquisition, "cannot open local artifact"))?;
    let mut bytes = Vec::new();
    file.take(MAX_ARCHIVE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::new(Code::Acquisition, "cannot read local artifact"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_ARCHIVE {
        return Err(Error::new(
            Code::Acquisition,
            "archive exceeds the 256 MiB input limit",
        ));
    }
    // Always observe local content afresh, even when a previous digest was cached.
    let observed = ContentHash::digest(&bytes);
    if req.hash().is_some_and(|h| h != observed) {
        return Err(Error::new(
            Code::HashMismatch,
            "local content differs from locked artifact",
        ));
    }
    match cache.get(req, observed) {
        Ok(Some(e)) => {
            stats.cache_hits += 1;
            return Ok(e);
        }
        Ok(None) => {}
        Err(_) => cache_warning(warnings, "cache read failed; acquiring evidence directly"),
    }
    stats.archive_extractions += 1;
    let evidence = Evidence::from_archive(req, &bytes)?;
    if cache.put(req, &evidence).is_err() {
        cache_warning(warnings, "cache write failed; using in-memory evidence");
    }
    Ok(evidence)
}
