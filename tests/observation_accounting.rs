//! Shared behavioral contract for checking and policy-free description.
mod common;
use proptest::prelude::*;
use rstest::rstest;
use taloon::{
    description::Description,
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{ArtifactRequest, Code, ContentHash, Error, EvaluationDate},
};

#[derive(Clone, Copy, Debug)]
enum Consumer {
    Check,
    Describe,
}
enum Subject {
    Check(Inspection),
    Describe(Description),
}
impl Subject {
    fn new(consumer: Consumer, bytes: &[u8]) -> Self {
        let hash = ContentHash::digest(bytes);
        let lock = format!(
            "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{{url='https://example.test/a.whl',hash='{hash}'}},{{url='https://example.test/b.whl',hash='{hash}'}},{{url='https://example.test/c.whl',hash='{hash}'}}]"
        );
        match consumer {
            Consumer::Check => Self::Check(
                Inspection::from_documents(
                    &lock,
                    "[tool.taloon]\nschema-version=1\nallow=['MIT']",
                    EvaluationDate::parse("2026-11-01").unwrap(),
                )
                .unwrap(),
            ),
            Consumer::Describe => {
                Self::Describe(Description::from_uv_lock(&lock, "example").unwrap())
            }
        }
    }
    fn requests(&self) -> &[ArtifactRequest] {
        match self {
            Self::Check(s) => s.requests(),
            Self::Describe(s) => s.requests(),
        }
    }
    fn evaluate(&self, results: Vec<AcquisitionResult>) -> (u8, serde_json::Value) {
        match self {
            Self::Check(s) => {
                let report = s.evaluate(results);
                (
                    report.exit_code(false),
                    serde_json::to_value(report).unwrap(),
                )
            }
            Self::Describe(s) => {
                let report = s.evaluate(results);
                (report.exit_code(), serde_json::to_value(report).unwrap())
            }
        }
    }
}
fn observed(request: &ArtifactRequest, bytes: &[u8]) -> AcquisitionResult {
    AcquisitionResult::observed(
        request.id(),
        Evidence::from_archive(request, bytes).unwrap(),
    )
}
#[rstest]
#[case::check(Consumer::Check)]
#[case::describe(Consumer::Describe)]
fn duplicate_after_failure_is_an_invalid_result(#[case] consumer: Consumer) {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let subject = Subject::new(consumer, &bytes);
    let request = &subject.requests()[0];
    let mut results = vec![AcquisitionResult::failed(
        request.id(),
        Error::new(Code::Acquisition, "failed attempt"),
    )];
    results.extend(subject.requests().iter().map(|r| observed(r, &bytes)));
    // Act
    let (exit, report) = subject.evaluate(results);
    // Assert
    assert_eq!(exit, 2);
    let failed = report["packages"][0]["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["request_id"] == request.id().to_string())
        .unwrap();
    assert_eq!(failed["diagnostics"][0]["code"], "invalid_result");
    assert!(failed["evidence"].is_null());
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn cardinality_and_permutation_have_the_same_meaning_for_both_consumers(
        counts in prop::array::uniform3(0_usize..5),
        priorities in prop::collection::vec(any::<u64>(), 12),
        consumer in prop::sample::select(vec![Consumer::Check, Consumer::Describe]),
    ) {
        // Arrange: integer counts supply the expected completeness, independently of the ledger.
        let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
        let subject = Subject::new(consumer, &bytes);
        let mut indexed = Vec::new();
        for (request, count) in subject.requests().iter().zip(counts) {
            indexed.extend((0..count).map(|_| observed(request, &bytes)));
        }
        let mut ordered: Vec<_> = indexed.into_iter().zip(priorities).collect();
        ordered.sort_by_key(|(_, priority)| *priority);
        let results = ordered.into_iter().map(|(result, _)| result).collect();
        let expected = if counts == [1;3] { 0 } else { 2 };
        // Act
        let (exit, report) = subject.evaluate(results);
        // Assert
        prop_assert_eq!(exit, expected);
        let artifacts = report["packages"][0]["artifacts"].as_array().unwrap();
        for (request, count) in subject.requests().iter().zip(counts) {
            let artifact = artifacts.iter().find(|a| a["request_id"] == request.id().to_string()).unwrap();
            let diagnostics = artifact["diagnostics"].as_array().unwrap();
            match count {
                0 => prop_assert_eq!(&diagnostics[0]["code"], "missing_result"),
                1 => prop_assert!(diagnostics.is_empty()),
                _ => prop_assert_eq!(&diagnostics[0]["code"], "invalid_result"),
            }
        }
    }
}
