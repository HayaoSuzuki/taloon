//! Worker grouping contract and opt-in measurements.
use super::partition_requests;
use rstest::rstest;
use std::{hint::black_box, time::Instant};
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, PackageId, SourceId,
};

fn request(n: usize, digest: Option<usize>, sdist: bool) -> ArtifactRequest {
    ArtifactRequest::new(
        PackageId::parse(
            "example",
            "1.0",
            SourceId::parse("path", "artifacts").unwrap(),
        )
        .unwrap(),
        ArtifactLocator::local(&format!("{n}.archive")).unwrap(),
        if sdist {
            ArtifactFormat::Sdist
        } else {
            ArtifactFormat::Wheel
        },
        digest.map(|n| ContentHash::digest(&n.to_le_bytes())),
    )
    .unwrap()
}
#[rstest]
#[case::one_thousand(1000)]
#[case::ten_thousand(10000)]
#[ignore = "opt-in release worker-key measurement, no time threshold"]
fn worker_grouping_measurement(#[case] count: usize) {
    // Arrange
    let requests: Vec<_> = (0..count).map(|n| request(n, Some(n / 2), false)).collect();
    for sample in 1..=5 {
        // Act
        let started = Instant::now();
        let partitions = black_box(partition_requests(black_box(&requests)));
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_eq!(partitions.iter().map(Vec::len).sum::<usize>(), count);
        println!("requests={count} sample={sample} elapsed_us={elapsed_us}");
    }
}

#[rstest]
#[case::empty(0, 0)]
#[case::one(1, 1)]
#[case::seven(7, 7)]
#[case::eight(8, 8)]
#[case::nine(9, 8)]
#[case::seventeen(17, 8)]
fn worker_count_matches_nonempty_groups(#[case] count: usize, #[case] expected: usize) {
    // Arrange
    let requests: Vec<_> = (0..count).map(|n| request(n, Some(n), false)).collect();
    // Act
    let partitions = partition_requests(&requests);
    // Assert
    assert_eq!(partitions.len(), expected);
    assert_eq!(partitions.iter().map(Vec::len).sum::<usize>(), count);
}

#[rstest]
#[case::same_content(Some(7), false, Some(7), false, 1)]
#[case::different_format(Some(7), false, Some(7), true, 2)]
#[case::different_content(Some(7), false, Some(8), false, 2)]
#[case::hashless(None, false, None, false, 2)]
#[case::mixed(None, false, Some(7), false, 2)]
fn grouping_respects_content_format_and_request_identity(
    #[case] first: Option<usize>,
    #[case] first_sdist: bool,
    #[case] second: Option<usize>,
    #[case] second_sdist: bool,
    #[case] expected: usize,
) {
    // Arrange
    let requests = [
        request(0, first, first_sdist),
        request(1, second, second_sdist),
    ];
    // Act
    let partitions = partition_requests(&requests);
    // Assert
    assert_eq!(partitions.len(), expected);
    assert_eq!(partitions.iter().map(Vec::len).sum::<usize>(), 2);
}

use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn partitioning_matches_integer_group_oracle(
        inputs in prop::collection::vec((prop::option::of(0_usize..16), any::<bool>()), 0..96),
    ) {
        // Arrange: integer labels, not formatted hashes, define expected groups.
        let requests: Vec<_> = inputs.iter().enumerate()
            .map(|(n, &(hash, sdist))| request(n, hash, sdist)).collect();
        let labels: Vec<_> = inputs.iter().enumerate().map(|(n, &(hash, sdist))|
            (hash.is_none(), hash.unwrap_or(n), sdist)).collect();
        let groups: BTreeSet<_> = labels.iter().copied().collect();

        // Act
        let partitions = partition_requests(&requests);
        let assignments: BTreeMap<_, _> = partitions.iter().enumerate()
            .flat_map(|(worker, requests)| requests.iter().map(move |r| (r.id(), worker))).collect();

        // Assert
        prop_assert_eq!(partitions.len(), groups.len().min(8));
        prop_assert_eq!(assignments.len(), requests.len());
        prop_assert_eq!(partitions.iter().map(Vec::len).sum::<usize>(), requests.len());
        for (n, label) in labels.iter().enumerate() {
            for (m, other) in labels.iter().enumerate() {
                prop_assert!(label != other || assignments[&requests[n].id()] == assignments[&requests[m].id()]);
            }
        }
    }
}
