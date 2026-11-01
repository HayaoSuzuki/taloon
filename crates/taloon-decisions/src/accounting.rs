//! Saturating observed-work accounting, independent of transport and clocks.
#[cfg(creusot)]
use creusot_std::prelude::*;

#[cfg_attr(not(creusot), derive(Clone, Debug, Default))]
#[cfg_attr(all(feature = "serde", not(creusot)), derive(serde::Serialize))]
pub struct TransferSummary {
    pub requests: u64,
    pub downloaded_bytes: u64,
    pub request_time_us: u64,
    pub retry_delay_us: u64,
}
#[cfg_attr(not(creusot), derive(Clone, Debug, Default))]
#[cfg_attr(all(feature = "serde", not(creusot)), derive(serde::Serialize))]
pub struct HttpSummary {
    pub index: TransferSummary,
    pub metadata: TransferSummary,
    pub archive: TransferSummary,
}
#[cfg_attr(not(creusot), derive(Clone, Debug, Default))]
#[cfg_attr(all(feature = "serde", not(creusot)), derive(serde::Serialize))]
pub struct AcquisitionSummary {
    pub network_requests: u64,
    pub downloaded_bytes: u64,
    pub archive_extractions: u64,
    pub cache_hits: u64,
    /// Wall time of acquisition; not a sum of overlapping worker durations.
    pub elapsed_us: u64,
    pub index_cache_hits: u64,
    pub index_wait_us: u64,
    pub index_parse_count: u64,
    pub index_parse_us: u64,
    pub http: HttpSummary,
}
impl TransferSummary {
    /// Merge observed counters with saturation at `u64::MAX`.
    #[cfg_attr(creusot, ensures((^self).requests@ == (self.requests@ + other.requests@).min(u64::MAX@)))]
    #[cfg_attr(creusot, ensures((^self).requests >= self.requests))]
    #[cfg_attr(creusot, ensures((^self).downloaded_bytes@ == (self.downloaded_bytes@ + other.downloaded_bytes@).min(u64::MAX@)))]
    #[cfg_attr(creusot, ensures((^self).downloaded_bytes >= self.downloaded_bytes))]
    #[cfg_attr(creusot, ensures((^self).request_time_us@ == (self.request_time_us@ + other.request_time_us@).min(u64::MAX@)))]
    #[cfg_attr(creusot, ensures((^self).request_time_us >= self.request_time_us))]
    #[cfg_attr(creusot, ensures((^self).retry_delay_us@ == (self.retry_delay_us@ + other.retry_delay_us@).min(u64::MAX@)))]
    #[cfg_attr(creusot, ensures((^self).retry_delay_us >= self.retry_delay_us))]
    fn merge(&mut self, other: &Self) {
        self.requests = self.requests.saturating_add(other.requests);
        self.downloaded_bytes = self.downloaded_bytes.saturating_add(other.downloaded_bytes);
        self.request_time_us = self.request_time_us.saturating_add(other.request_time_us);
        self.retry_delay_us = self.retry_delay_us.saturating_add(other.retry_delay_us);
    }
}

impl AcquisitionSummary {
    /// Merge observed counters with saturation at `u64::MAX`.
    /// The receiver retains its wall elapsed time.
    #[cfg_attr(creusot, ensures(
        (^self).network_requests@ == (self.network_requests@ + other.network_requests@).min(u64::MAX@)
        && (^self).downloaded_bytes@ == (self.downloaded_bytes@ + other.downloaded_bytes@).min(u64::MAX@)
        && (^self).archive_extractions@ == (self.archive_extractions@ + other.archive_extractions@).min(u64::MAX@)
        && (^self).cache_hits@ == (self.cache_hits@ + other.cache_hits@).min(u64::MAX@)
        && (^self).index_cache_hits@ == (self.index_cache_hits@ + other.index_cache_hits@).min(u64::MAX@)
        && (^self).index_wait_us@ == (self.index_wait_us@ + other.index_wait_us@).min(u64::MAX@)
        && (^self).index_parse_count@ == (self.index_parse_count@ + other.index_parse_count@).min(u64::MAX@)
        && (^self).index_parse_us@ == (self.index_parse_us@ + other.index_parse_us@).min(u64::MAX@)
        && (^self).http.index.requests@ == (self.http.index.requests@ + other.http.index.requests@).min(u64::MAX@)
        && (^self).http.index.downloaded_bytes@ == (self.http.index.downloaded_bytes@ + other.http.index.downloaded_bytes@).min(u64::MAX@)
        && (^self).http.index.request_time_us@ == (self.http.index.request_time_us@ + other.http.index.request_time_us@).min(u64::MAX@)
        && (^self).http.index.retry_delay_us@ == (self.http.index.retry_delay_us@ + other.http.index.retry_delay_us@).min(u64::MAX@)
        && (^self).http.metadata.requests@ == (self.http.metadata.requests@ + other.http.metadata.requests@).min(u64::MAX@)
        && (^self).http.metadata.downloaded_bytes@ == (self.http.metadata.downloaded_bytes@ + other.http.metadata.downloaded_bytes@).min(u64::MAX@)
        && (^self).http.metadata.request_time_us@ == (self.http.metadata.request_time_us@ + other.http.metadata.request_time_us@).min(u64::MAX@)
        && (^self).http.metadata.retry_delay_us@ == (self.http.metadata.retry_delay_us@ + other.http.metadata.retry_delay_us@).min(u64::MAX@)
        && (^self).http.archive.requests@ == (self.http.archive.requests@ + other.http.archive.requests@).min(u64::MAX@)
        && (^self).http.archive.downloaded_bytes@ == (self.http.archive.downloaded_bytes@ + other.http.archive.downloaded_bytes@).min(u64::MAX@)
        && (^self).http.archive.request_time_us@ == (self.http.archive.request_time_us@ + other.http.archive.request_time_us@).min(u64::MAX@)
        && (^self).http.archive.retry_delay_us@ == (self.http.archive.retry_delay_us@ + other.http.archive.retry_delay_us@).min(u64::MAX@)
    ))]
    #[cfg_attr(creusot, ensures(
        (^self).network_requests >= self.network_requests
        && (^self).downloaded_bytes >= self.downloaded_bytes
        && (^self).archive_extractions >= self.archive_extractions
        && (^self).cache_hits >= self.cache_hits
        && (^self).index_cache_hits >= self.index_cache_hits
        && (^self).index_wait_us >= self.index_wait_us
        && (^self).index_parse_count >= self.index_parse_count
        && (^self).index_parse_us >= self.index_parse_us
        && (^self).http.index.requests >= self.http.index.requests
        && (^self).http.index.downloaded_bytes >= self.http.index.downloaded_bytes
        && (^self).http.index.request_time_us >= self.http.index.request_time_us
        && (^self).http.index.retry_delay_us >= self.http.index.retry_delay_us
        && (^self).http.metadata.requests >= self.http.metadata.requests
        && (^self).http.metadata.downloaded_bytes >= self.http.metadata.downloaded_bytes
        && (^self).http.metadata.request_time_us >= self.http.metadata.request_time_us
        && (^self).http.metadata.retry_delay_us >= self.http.metadata.retry_delay_us
        && (^self).http.archive.requests >= self.http.archive.requests
        && (^self).http.archive.downloaded_bytes >= self.http.archive.downloaded_bytes
        && (^self).http.archive.request_time_us >= self.http.archive.request_time_us
        && (^self).http.archive.retry_delay_us >= self.http.archive.retry_delay_us
    ))]
    #[cfg_attr(creusot, ensures(
        (^self).elapsed_us == self.elapsed_us
    ))]
    pub fn merge(&mut self, other: &Self) {
        self.network_requests = self.network_requests.saturating_add(other.network_requests);
        self.downloaded_bytes = self.downloaded_bytes.saturating_add(other.downloaded_bytes);
        self.archive_extractions = self
            .archive_extractions
            .saturating_add(other.archive_extractions);
        self.cache_hits = self.cache_hits.saturating_add(other.cache_hits);
        self.index_cache_hits = self.index_cache_hits.saturating_add(other.index_cache_hits);
        self.index_wait_us = self.index_wait_us.saturating_add(other.index_wait_us);
        self.index_parse_count = self
            .index_parse_count
            .saturating_add(other.index_parse_count);
        self.index_parse_us = self.index_parse_us.saturating_add(other.index_parse_us);
        self.http.index.merge(&other.http.index);
        self.http.metadata.merge(&other.http.metadata);
        self.http.archive.merge(&other.http.archive);
    }
}
