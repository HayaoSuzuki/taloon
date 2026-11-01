//! Opt-in metadata-heavy core evaluation; no elapsed-time CI threshold.
mod common;
use rstest::rstest;
use std::{fmt::Write as _, hint::black_box, time::Instant};
use taloon::{
    description::Description,
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{ContentHash, EvaluationDate},
};
#[derive(Clone, Copy, Debug)]
enum Consumer {
    Check,
    Describe,
}
#[rstest]
#[case::check(Consumer::Check)]
#[case::describe(Consumer::Describe)]
#[ignore = "opt-in release measurement with synthetic 256KiB headers"]
fn metadata_heavy_observation_evaluation(#[case] consumer: Consumer) {
    // Arrange: setup and input cloning are outside each measured evaluation.
    let mut meta = common::metadata("License-Expression: MIT");
    let blank = meta.windows(2).position(|w| w == b"\n\n").unwrap();
    meta.splice(
        blank..blank,
        format!("\nX-Padding: {}", "a".repeat(256 * 1024)).bytes(),
    );
    let bytes = common::wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let mut lock="version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={registry='https://example.test/simple'}\nwheels=[".to_owned();
    for n in 0..64 {
        write!(
            lock,
            "{{url='https://example.test/{n}.whl',hash='{hash}'}},"
        )
        .unwrap();
    }
    lock.push_str("]\n");
    let inspection = Inspection::from_documents(
        &lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        EvaluationDate::parse("2026-11-01").unwrap(),
    )
    .unwrap();
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let observations: Vec<_> = inspection
        .requests()
        .iter()
        .map(|r| Evidence::from_archive(r, &bytes).unwrap())
        .collect();
    for sample in 1..=5 {
        let results = inspection
            .requests()
            .iter()
            .zip(&observations)
            .map(|(r, e)| AcquisitionResult::observed(r.id(), e.clone()))
            .collect();
        // Act
        let started = Instant::now();
        let exit = match consumer {
            Consumer::Check => black_box(inspection.evaluate(results)).exit_code(false),
            Consumer::Describe => black_box(description.evaluate(results)).exit_code(),
        };
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_eq!(exit, 0);
        println!(
            "consumer={consumer:?} artifacts=64 padding=262144 sample={sample} elapsed_us={elapsed_us}"
        );
    }
}
