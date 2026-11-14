//! Deny-mode evidence accounting through the public inspection boundary.
mod common;
use rstest::rstest;
use taloon::{
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{ContentHash, EvaluationDate},
};
fn date() -> EvaluationDate {
    EvaluationDate::parse("2026-11-02").unwrap()
}
fn lock(hash: ContentHash) -> String {
    format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}\nwheels=[{{url='https://files.example.test/a.whl',hash='{hash}'}}]"
    )
}
#[rstest]
#[case::standard("License-Expression: MIT", 0)]
#[case::custom("License-Expression: LicenseRef-Unknown", 1)]
#[case::unknown("License-Expression: Not-A-License", 2)]
#[case::invalid("License-Expression: MIT OR", 2)]
#[case::missing("", 2)]
fn empty_deny_preserves_declaration_outcomes(#[case] fields: &str, #[case] expected: u8) {
    // Arrange
    let bytes = common::wheel(&common::metadata(fields));
    let inspection = Inspection::from_documents(
        &lock(ContentHash::digest(&bytes)),
        "[tool.taloon]\nschema-version=1\ndeny=[]",
        date(),
    )
    .unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, &bytes).unwrap()))
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), expected);
}
#[test]
fn complete_deny_run_warns_only_for_unobserved_units() {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: MIT OR Apache-2.0"));
    let inspection = Inspection::from_documents(
        &lock(ContentHash::digest(&bytes)),
        "[tool.taloon]\nschema-version=1\ndeny=['MIT','ISC']",
        date(),
    )
    .unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, &bytes).unwrap()))
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].error.message,
        "unused global denial: ISC"
    );
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 1);
}

#[test]
fn incomplete_deny_run_suppresses_unused_unit_warnings() {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: MIT OR Apache-2.0"));
    let inspection = Inspection::from_documents(
        &lock(ContentHash::digest(&bytes)),
        "[tool.taloon]\nschema-version=1\ndeny=['MIT','ISC']",
        date(),
    )
    .unwrap();
    // Act
    let report = inspection.evaluate(vec![]);
    // Assert
    assert!(report.diagnostics().is_empty());
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn clarification_does_not_override_a_denied_unit() {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: Apache-2.0"));
    let hash = ContentHash::digest(&bytes);
    let text = format!(
        "[tool.taloon]\nschema-version=1\ndeny=['MIT']\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='reviewed synthetic artifact'"
    );
    let inspection = Inspection::from_documents(&lock(hash), &text, date()).unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, &bytes).unwrap()))
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), 1);
    assert_eq!(
        report.packages()[0].artifacts[0]
            .decision
            .as_ref()
            .unwrap()
            .overrides
            .len(),
        1
    );
}

#[test]
fn deny_custom_permission_rejects_sidecar_only_proof() {
    // Arrange
    let metadata = common::metadata("License-Expression: LicenseRef-Reviewed");
    let bytes = common::wheel(&metadata);
    let hash = ContentHash::digest(&bytes);
    let text = format!(
        "[tool.taloon]\nschema-version=1\ndeny=[]\n[[tool.taloon.exceptions]]\nname='example'\nversion='1.0'\nsource={{kind='registry',location='https://one.test/simple'}}\nartifact-hash='{hash}'\nallow=['LicenseRef-Reviewed']\nreason='reviewed synthetic archive'"
    );
    let inspection = Inspection::from_documents(&lock(hash), &text, date()).unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| {
            AcquisitionResult::observed(
                r.id(),
                Evidence::from_sidecar(
                    r,
                    taloon::evidence::SidecarAssociation::new(
                        hash,
                        hash,
                        ContentHash::digest(&metadata),
                    )
                    .unwrap(),
                    &metadata,
                )
                .unwrap(),
            )
        })
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), 2);
    assert_eq!(
        report.packages()[0].artifacts[0].diagnostics[0].error.code,
        taloon::model::Code::InsufficientEvidence
    );
}
