//! Synthetic policy costs, measured independently of I/O and parser setup.
use rstest::rstest;
use std::{fmt::Write as _, hint::black_box, time::Instant};
use taloon::{
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvaluationDate, PackageId,
        SourceId,
    },
    policy::{LicenseExpression, Policy},
};
#[derive(Clone, Copy, Debug)]
enum Workload {
    GlobalPermissions,
    ScopedExceptions,
}
fn fixture(workload: Workload, count: usize) -> (Policy, Vec<ArtifactRequest>) {
    let mut config = "[tool.taloon]\nschema-version=1\nallow=[".to_owned();
    if matches!(workload, Workload::GlobalPermissions) {
        for id in spdx::identifiers::LICENSES.iter().take(256) {
            write!(config, "'{}',", id.name).unwrap();
        }
        config.push_str("'MIT'");
    }
    config.push_str("]\n");
    let source = SourceId::parse("registry", "https://example.test/simple").unwrap();
    let hash = ContentHash::digest(b"benchmark content");
    let mut requests = Vec::new();
    for n in 0..count {
        let name = format!("p-{n:04}");
        if matches!(workload, Workload::ScopedExceptions) {
            writeln!(config, "[[tool.taloon.exceptions]]\nname='{name}'\nversion='1.0'\nsource={{kind='registry',location='https://example.test/simple'}}\nallow=['MIT']\nreason='benchmark'").unwrap();
        }
        requests.push(
            ArtifactRequest::new(
                PackageId::parse(&name, "1.0", source.clone()).unwrap(),
                ArtifactLocator::registry(
                    source.location(),
                    &format!("https://example.test/{name}.whl"),
                )
                .unwrap(),
                ArtifactFormat::Wheel,
                Some(hash),
            )
            .unwrap(),
        );
    }
    (
        Policy::from_pyproject(&config, EvaluationDate::parse("2026-11-01").unwrap()).unwrap(),
        requests,
    )
}
#[rstest]
#[case::global(Workload::GlobalPermissions)]
#[case::scoped(Workload::ScopedExceptions)]
#[ignore = "opt-in release measurement; no timing assertion"]
fn policy_lookup_and_evaluation_cost(#[case] workload: Workload) {
    // Arrange
    let (policy, requests) = fixture(workload, 2000);
    let expression = LicenseExpression::parse("MIT").unwrap();
    for sample in 1..=5 {
        let mut permitted = 0;
        // Act
        let started = Instant::now();
        for request in &requests {
            black_box(policy.minimum_evidence(black_box(request)));
            permitted += usize::from(
                black_box(policy.evaluate(request, request.hash().unwrap(), &expression))
                    .is_permitted(),
            );
        }
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_eq!(permitted, requests.len());
        println!(
            "workload={workload:?} requests={} sample={sample} elapsed_us={elapsed_us}",
            requests.len()
        );
    }
}
