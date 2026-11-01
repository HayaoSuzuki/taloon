use proptest::prelude::*;
use taloon::report::AcquisitionSummary;

#[test]
fn acquisition_metrics_merge_saturates_without_summing_wall_time() {
    let mut first = AcquisitionSummary {
        network_requests: u64::MAX,
        elapsed_us: 100,
        index_wait_us: 3,
        ..AcquisitionSummary::default()
    };
    first.http.index.requests = u64::MAX;
    let mut second = AcquisitionSummary {
        network_requests: 2,
        downloaded_bytes: 17,
        elapsed_us: 900,
        index_cache_hits: 1,
        index_wait_us: 7,
        index_parse_count: 1,
        index_parse_us: 12,
        ..AcquisitionSummary::default()
    };
    second.http.index.requests = 2;
    second.http.index.downloaded_bytes = 17;
    second.http.index.request_time_us = 25;
    second.http.index.retry_delay_us = 50;
    first.merge(&second);
    assert_eq!(first.network_requests, u64::MAX);
    assert_eq!(first.http.index.requests, u64::MAX);
    assert_eq!(first.downloaded_bytes, 17);
    assert_eq!(first.http.index.downloaded_bytes, 17);
    assert_eq!(first.http.index.request_time_us, 25);
    assert_eq!(first.http.index.retry_delay_us, 50);
    assert_eq!(first.elapsed_us, 100);
    assert_eq!(first.index_wait_us, 10);
    assert_eq!(first.index_cache_hits, 1);
    assert_eq!((first.index_parse_count, first.index_parse_us), (1, 12));
}

proptest! {
    #[test]
    fn metric_merge_is_order_independent(values in prop::collection::vec(any::<u64>(), 0..16)) {
        let expected = values.iter().map(|v| u128::from(*v)).sum::<u128>().min(u128::from(u64::MAX));
        let mut forward = AcquisitionSummary::default();
        let mut reverse = AcquisitionSummary::default();
        for (output, entries) in [(&mut forward, values.clone()), (&mut reverse, values.iter().rev().copied().collect())] {
            for value in entries {
                let mut part = AcquisitionSummary {network_requests:value, elapsed_us:999, ..AcquisitionSummary::default()};
                part.http.index.requests = value;
                output.merge(&part);
            }
        }
        prop_assert_eq!(u128::from(forward.network_requests), expected);
        prop_assert_eq!(forward.http.index.requests, forward.network_requests);
        prop_assert_eq!(serde_json::to_value(&forward).unwrap(), serde_json::to_value(&reverse).unwrap());
        prop_assert_eq!(forward.elapsed_us, 0);
    }
}
