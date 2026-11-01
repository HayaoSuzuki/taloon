//! Runtime-only clocks and panic-safe attempt accounting. Report values stay pure.
use std::time::Instant;
use taloon::report::{AcquisitionSummary, TransferSummary};

#[derive(Clone, Copy)]
pub(super) enum RequestPurpose {
    Index,
    Metadata,
    Archive,
}
impl RequestPurpose {
    pub(super) fn summary(self, stats: &mut AcquisitionSummary) -> &mut TransferSummary {
        match self {
            Self::Index => &mut stats.http.index,
            Self::Metadata => &mut stats.http.metadata,
            Self::Archive => &mut stats.http.archive,
        }
    }
}
pub(super) fn micros(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}
struct Elapsed<'a> {
    total: &'a mut u64,
    started: Instant,
}
impl Drop for Elapsed<'_> {
    fn drop(&mut self) {
        *self.total = self.total.saturating_add(micros(self.started.elapsed()));
    }
}
pub(super) fn timed<T>(total: &mut u64, action: impl FnOnce() -> T) -> T {
    let _elapsed = Elapsed {
        total,
        started: Instant::now(),
    };
    action()
}
/// One physical transport attempt, including received bytes before errors/unwind.
pub(super) struct Attempt<'a> {
    stats: &'a mut AcquisitionSummary,
    purpose: RequestPurpose,
    started: Instant,
    bytes_before: u64,
}
impl<'a> Attempt<'a> {
    pub(super) fn begin(stats: &'a mut AcquisitionSummary, purpose: RequestPurpose) -> Self {
        stats.network_requests = stats.network_requests.saturating_add(1);
        let phase = purpose.summary(stats);
        phase.requests = phase.requests.saturating_add(1);
        let bytes_before = stats.downloaded_bytes;
        Self {
            stats,
            purpose,
            started: Instant::now(),
            bytes_before,
        }
    }
    pub(super) fn summary(&mut self) -> &mut AcquisitionSummary {
        self.stats
    }
}
impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        let bytes = self
            .stats
            .downloaded_bytes
            .saturating_sub(self.bytes_before);
        let phase = self.purpose.summary(self.stats);
        phase.downloaded_bytes = phase.downloaded_bytes.saturating_add(bytes);
        phase.request_time_us = phase
            .request_time_us
            .saturating_add(micros(self.started.elapsed()));
    }
}
