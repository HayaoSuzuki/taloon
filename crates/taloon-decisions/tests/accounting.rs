use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::accounting::{AcquisitionSummary, HttpSummary, TransferSummary};

fn transfer(values: &[u64]) -> TransferSummary {
    TransferSummary {
        requests: values[0],
        downloaded_bytes: values[1],
        request_time_us: values[2],
        retry_delay_us: values[3],
    }
}

fn summary(values: [u64; 21]) -> AcquisitionSummary {
    AcquisitionSummary {
        network_requests: values[0],
        downloaded_bytes: values[1],
        archive_extractions: values[2],
        cache_hits: values[3],
        index_cache_hits: values[4],
        index_wait_us: values[5],
        index_parse_count: values[6],
        index_parse_us: values[7],
        elapsed_us: values[8],
        http: HttpSummary {
            index: transfer(&values[9..13]),
            metadata: transfer(&values[13..17]),
            archive: transfer(&values[17..21]),
        },
    }
}

fn counters(value: &AcquisitionSummary) -> [u64; 20] {
    [
        value.network_requests,
        value.downloaded_bytes,
        value.archive_extractions,
        value.cache_hits,
        value.index_cache_hits,
        value.index_wait_us,
        value.index_parse_count,
        value.index_parse_us,
        value.http.index.requests,
        value.http.index.downloaded_bytes,
        value.http.index.request_time_us,
        value.http.index.retry_delay_us,
        value.http.metadata.requests,
        value.http.metadata.downloaded_bytes,
        value.http.metadata.request_time_us,
        value.http.metadata.retry_delay_us,
        value.http.archive.requests,
        value.http.archive.downloaded_bytes,
        value.http.archive.request_time_us,
        value.http.archive.retry_delay_us,
    ]
}

#[rstest]
#[case::zero(0, 0, 0)]
#[case::identity(u64::MAX, 0, u64::MAX)]
#[case::exact_maximum(u64::MAX - 1, 1, u64::MAX)]
#[case::overflow(u64::MAX - 1, 2, u64::MAX)]
#[case::both_maximum(u64::MAX, u64::MAX, u64::MAX)]
#[case::ordinary(17, 23, 40)]
fn merge_counter_boundaries(#[case] left: u64, #[case] right: u64, #[case] expected: u64) {
    // Arrange
    let mut actual = summary([left; 21]);
    let other = summary([right; 21]);
    // Act
    actual.merge(&other);
    // Assert
    assert_eq!(counters(&actual), [expected; 20]);
}

#[rstest]
#[case::zero(0, 7)]
#[case::finite(13, 29)]
#[case::maximum(u64::MAX, 1)]
fn merge_preserves_wall_elapsed(#[case] receiver: u64, #[case] worker: u64) {
    // Arrange
    let mut actual = summary([receiver; 21]);
    let other = summary([worker; 21]);
    // Act
    actual.merge(&other);
    // Assert
    assert_eq!(actual.elapsed_us, receiver);
}

proptest! {
    #[test]
    fn merge_matches_independent_wide_integer_oracle(left in any::<[u64; 21]>(), right in any::<[u64; 21]>()) {
        // Arrange
        let mut actual = summary(left);
        let other = summary(right);
        let expected = counters(&actual).into_iter().zip(counters(&other)).map(|(a,b)| {
            u64::try_from((u128::from(a) + u128::from(b)).min(u128::from(u64::MAX))).unwrap()
        }).collect::<Vec<_>>();
        // Act
        actual.merge(&other);
        // Assert
        prop_assert_eq!(counters(&actual).to_vec(), expected);
    }

    #[test]
    fn merge_counters_never_decrease(left in any::<[u64; 21]>(), right in any::<[u64; 21]>()) {
        // Arrange
        let mut actual = summary(left);
        let other = summary(right);
        let before = counters(&actual);
        // Act
        actual.merge(&other);
        // Assert
        prop_assert!(counters(&actual).into_iter().zip(before).all(|(after, prior)| after >= prior));
    }

    #[test]
    fn merge_retains_receiver_wall_elapsed(left in any::<[u64; 21]>(), right in any::<[u64; 21]>()) {
        // Arrange
        let mut actual = summary(left);
        let other = summary(right);
        // Act
        actual.merge(&other);
        // Assert
        prop_assert_eq!(actual.elapsed_us, left[8]);
    }
}
