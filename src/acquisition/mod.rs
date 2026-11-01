//! Binary-owned I/O. The library sees only observations and failures.
mod cache;
#[cfg(test)]
mod tests;
pub(crate) use cache::EvidenceCache;
use std::{io::Read, path::Path};
use taloon::{
    evidence::{Evidence, MAX_METADATA_BYTES},
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, Error, RequestId,
    },
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

mod http;
#[cfg(test)]
mod http_tests;
mod index;
mod metrics;
use metrics::{RequestPurpose, micros};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};
use taloon::{inspection::AcquisitionResult, model::EvidenceRequirement};
pub(crate) struct AcquisitionOptions {
    pub project_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub offline: bool,
    pub netrc: Option<PathBuf>,
}
impl AcquisitionOptions {
    #[cfg(test)]
    fn testing(root: &Path, offline: bool) -> Self {
        Self {
            project_dir: root.to_owned(),
            cache_dir: root.join("cache"),
            offline,
            netrc: None,
        }
    }
}
#[derive(Default)]
pub(crate) struct AcquisitionBatch {
    pub results: Vec<AcquisitionResult>,
    pub warnings: Vec<Diagnostic>,
    pub summary: AcquisitionSummary,
}
/// Terminal outcomes accounted for, including failures. No package details leave
/// acquisition through this channel; the caller owns display and timing policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AcquisitionProgress {
    pub completed: usize,
    pub total: usize,
}
pub(crate) struct Acquirer {
    options: AcquisitionOptions,
    cache: EvidenceCache,
    http: http::Http,
}
impl Acquirer {
    pub(crate) fn new(options: AcquisitionOptions) -> Result<Self, Error> {
        Self::with_transport(options, http::HttpsTransport::new()?)
    }
}
impl Acquirer {
    fn with_transport<T: http::Transport + 'static>(
        options: AcquisitionOptions,
        transport: T,
    ) -> Result<Self, Error> {
        let auth = if options.offline {
            http::Authentication::default()
        } else {
            http::Authentication::from_file(options.netrc.as_deref())?
        };
        let cache = EvidenceCache::new(options.cache_dir.clone());
        Ok(Self {
            options,
            cache,
            http: http::Http::new(transport, auth),
        })
    }
    #[cfg(test)]
    fn acquire(&self, requests: &[ArtifactRequest]) -> AcquisitionBatch {
        self.acquire_with_progress(requests, |_| {})
    }
    pub(crate) fn acquire_with_progress(
        &self,
        requests: &[ArtifactRequest],
        mut observe: impl FnMut(AcquisitionProgress),
    ) -> AcquisitionBatch {
        let started = Instant::now();
        let mut progress = AcquisitionProgress {
            completed: 0,
            total: requests.len(),
        };
        observe(progress);
        let indexes = index::IndexCatalog::new(requests, &self.http);
        let partitions = partition_requests(requests);
        let workers = partitions.len();
        // Summaries outlive worker return values, so an unwind cannot discard
        // already observed traffic. Each worker owns its lock until it finishes.
        let summaries: Vec<_> = (0..workers)
            .map(|_| Mutex::new(AcquisitionSummary::default()))
            .collect();
        let mut batch = std::thread::scope(|scope| {
            let (completed_tx, completed_rx) = mpsc::channel();
            let jobs: Vec<_> = partitions
                .iter()
                .zip(&summaries)
                .map(|(partition, summary)| {
                    let completed_tx = completed_tx.clone();
                    let indexes = &indexes;
                    (
                        partition,
                        summary,
                        scope.spawn(move || {
                            let mut batch = AcquisitionBatch::default();
                            let mut stats = summary
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            for request in partition {
                                let result = match request.locator() {
                                    ArtifactLocator::Local(_) => acquire_local(
                                        request,
                                        &self.options.project_dir,
                                        &self.cache,
                                        &mut stats,
                                        &mut batch.warnings,
                                    ),
                                    ArtifactLocator::Remote { .. } => self.acquire_remote(
                                        request,
                                        indexes,
                                        &mut stats,
                                        &mut batch.warnings,
                                    ),
                                };
                                batch.results.push(match result {
                                    Ok(e) => AcquisitionResult::observed(request.id(), e),
                                    Err(e) => AcquisitionResult::failed(request.id(), e),
                                });
                                let _ = completed_tx.send(());
                            }
                            batch
                        }),
                    )
                })
                .collect();
            drop(completed_tx);
            // Keep reporting while a worker is awaiting network I/O. Joining
            // the workers first would hide both completions and stalled requests.
            loop {
                match completed_rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(()) => progress.completed += 1,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                observe(progress);
            }
            let mut batch = AcquisitionBatch::default();
            for (partition, summary, job) in jobs {
                match job.join() {
                    Ok(mut part) => {
                        batch.results.append(&mut part.results);
                        batch.warnings.append(&mut part.warnings);
                    }
                    Err(_) => {
                        batch.results.extend(partition.iter().map(|r| {
                            AcquisitionResult::failed(
                                r.id(),
                                Error::new(Code::Acquisition, "acquisition worker failed"),
                            )
                        }));
                    }
                }
                let stats = summary
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                batch.summary.merge(&stats);
            }
            if progress.completed != batch.results.len() {
                // Panicked workers become terminal failures when joined.
                progress.completed = batch.results.len();
                observe(progress);
            }
            batch
        });
        batch.summary.elapsed_us = micros(started.elapsed());
        batch
    }
    fn acquire_remote(
        &self,
        request: &ArtifactRequest,
        indexes: &index::IndexCatalog<'_>,
        stats: &mut AcquisitionSummary,
        warnings: &mut Vec<Diagnostic>,
    ) -> Result<Evidence, Error> {
        let hash = request.hash().ok_or_else(http::failure)?;
        match self.cache.get(request, hash) {
            Ok(Some(e)) => {
                stats.cache_hits += 1;
                return Ok(e);
            }
            Ok(None) => {}
            Err(_) => cache_warning(warnings, "cache read failed; acquiring evidence directly"),
        }
        if self.options.offline {
            return Err(Error::new(
                Code::Acquisition,
                "offline cache lacks sufficient evidence for this artifact",
            ));
        }
        let ArtifactLocator::Remote { url, .. } = request.locator() else {
            return Err(http::failure());
        };
        let url = url::Url::parse(url).map_err(|_| http::failure())?;
        if request.requirement() == EvidenceRequirement::IndexAssociated {
            match self.try_sidecar(request, indexes, stats) {
                Ok(Some(evidence)) => {
                    self.persist(request, &evidence, warnings);
                    return Ok(evidence);
                }
                Ok(None) => {}
                Err(e) => {
                    if matches!(
                        e.code,
                        Code::HashMismatch | Code::IdentityMismatch | Code::InvalidMetadata
                    ) {
                        warnings.push(Diagnostic::warning(e));
                    }
                }
            }
        }
        let response = self.http.get(
            RequestPurpose::Archive,
            url,
            "application/octet-stream",
            MAX_ARCHIVE,
            stats,
        )?;
        stats.archive_extractions += 1;
        let evidence = Evidence::from_archive(request, &response.body)?;
        self.persist(request, &evidence, warnings);
        Ok(evidence)
    }
    fn try_sidecar(
        &self,
        request: &ArtifactRequest,
        indexes: &index::IndexCatalog<'_>,
        stats: &mut AcquisitionSummary,
    ) -> Result<Option<Evidence>, Error> {
        let Some((url, binding)) = indexes.sidecar(request, stats)? else {
            return Ok(None);
        };
        let response = self.http.get(
            RequestPurpose::Metadata,
            url,
            "application/octet-stream",
            MAX_METADATA_BYTES,
            stats,
        )?;
        Evidence::from_sidecar(request, binding, &response.body).map(Some)
    }
    fn persist(
        &self,
        request: &ArtifactRequest,
        evidence: &Evidence,
        warnings: &mut Vec<Diagnostic>,
    ) {
        if self.cache.put(request, evidence).is_err() {
            cache_warning(warnings, "cache write failed; using in-memory evidence");
        }
    }
}

// Different identity domains cannot alias, and formatting is never part of
// scheduling. Enum/value order is internal; it does not rank evidence strength.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
enum WorkKey {
    Pinned(ContentHash, ArtifactFormat),
    Unpinned(RequestId),
}
fn partition_requests(requests: &[ArtifactRequest]) -> Vec<Vec<&ArtifactRequest>> {
    if requests.is_empty() {
        return Vec::new();
    }
    // Keep shared-content jobs in one worker. Requests still receive distinct results.
    let mut grouped: BTreeMap<WorkKey, Vec<&ArtifactRequest>> = BTreeMap::new();
    for request in requests {
        let key = match request.hash() {
            Some(hash) => WorkKey::Pinned(hash, request.format()),
            None => WorkKey::Unpinned(request.id()),
        };
        grouped.entry(key).or_default().push(request);
    }
    let workers = 8.min(grouped.len());
    let mut partitions = vec![Vec::new(); workers];
    for (n, group) in grouped.into_values().enumerate() {
        partitions[n % workers].extend(group);
    }
    partitions
}

#[cfg(test)]
mod partition_tests;
