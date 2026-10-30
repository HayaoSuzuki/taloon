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

mod http;
#[cfg(test)]
mod http_tests;
mod index;
use std::{collections::BTreeMap, path::PathBuf};
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
    pub(crate) fn acquire(&self, requests: &[ArtifactRequest]) -> AcquisitionBatch {
        // Keep shared-content jobs in one worker. Requests still receive distinct results.
        let mut grouped: BTreeMap<String, Vec<&ArtifactRequest>> = BTreeMap::new();
        for request in requests {
            let key = request.hash().map_or_else(
                || request.id().to_string(),
                |h| format!("{h}:{:?}", request.format()),
            );
            grouped.entry(key).or_default().push(request);
        }
        let workers = 8.min(grouped.len().max(1));
        let mut partitions = vec![Vec::new(); workers];
        for (n, group) in grouped.into_values().enumerate() {
            partitions[n % workers].extend(group);
        }
        std::thread::scope(|scope| {
            let jobs: Vec<_> = partitions
                .iter()
                .map(|partition| {
                    (
                        partition,
                        scope.spawn(move || {
                            let mut batch = AcquisitionBatch::default();
                            for request in partition {
                                let result = match request.locator() {
                                    ArtifactLocator::Local(_) => acquire_local(
                                        request,
                                        &self.options.project_dir,
                                        &self.cache,
                                        &mut batch.summary,
                                        &mut batch.warnings,
                                    ),
                                    ArtifactLocator::Remote { .. } => self.acquire_remote(
                                        request,
                                        &mut batch.summary,
                                        &mut batch.warnings,
                                    ),
                                };
                                batch.results.push(match result {
                                    Ok(e) => AcquisitionResult::observed(request.id(), e),
                                    Err(e) => AcquisitionResult::failed(request.id(), e),
                                });
                            }
                            batch
                        }),
                    )
                })
                .collect();
            let mut batch = AcquisitionBatch::default();
            for (partition, job) in jobs {
                match job.join() {
                    Ok(mut part) => {
                        batch.results.append(&mut part.results);
                        batch.warnings.append(&mut part.warnings);
                        batch.summary.network_requests += part.summary.network_requests;
                        batch.summary.downloaded_bytes += part.summary.downloaded_bytes;
                        batch.summary.archive_extractions += part.summary.archive_extractions;
                        batch.summary.cache_hits += part.summary.cache_hits;
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
            }
            batch
        })
    }
    fn acquire_remote(
        &self,
        request: &ArtifactRequest,
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
        let ArtifactLocator::Remote { index, url } = request.locator() else {
            return Err(http::failure());
        };
        let url = url::Url::parse(url).map_err(|_| http::failure())?;
        if request.requirement() == EvidenceRequirement::IndexAssociated
            && let Some(index) = index
        {
            match self.try_sidecar(request, index, &url, stats) {
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
        let response = self
            .http
            .get(url, "application/octet-stream", MAX_ARCHIVE, stats)?;
        stats.archive_extractions += 1;
        let evidence = Evidence::from_archive(request, &response.body)?;
        self.persist(request, &evidence, warnings);
        Ok(evidence)
    }
    fn try_sidecar(
        &self,
        request: &ArtifactRequest,
        index: &str,
        url: &url::Url,
        stats: &mut AcquisitionSummary,
    ) -> Result<Option<Evidence>, Error> {
        let mut base = url::Url::parse(index).map_err(|_| http::failure())?;
        let path = format!(
            "{}/{}/",
            base.path().trim_end_matches('/'),
            request.package().name()
        );
        base.set_path(&path);
        let response = self.http.get(
            base,
            "application/vnd.pypi.simple.v1+json, text/html;q=0.1",
            16 * 1024 * 1024,
            stats,
        )?;
        let Some((url, binding)) =
            index::sidecar(&response, url, request.hash().ok_or_else(http::failure)?)?
        else {
            return Ok(None);
        };
        let response = self
            .http
            .get(url, "application/octet-stream", 4 * 1024 * 1024, stats)?;
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
