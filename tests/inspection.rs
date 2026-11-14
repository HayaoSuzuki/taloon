mod common;
use common::{metadata, wheel};
use taloon::{
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{ContentHash, EvaluationDate},
};
fn date() -> EvaluationDate {
    EvaluationDate::parse("2026-10-31").unwrap()
}
fn lock(bytes: &[u8]) -> String {
    format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{{url='https://files.example.test/a.whl',hash='{hash}'}},{{url='https://files.example.test/b.whl',hash='{hash}'}}]",
        hash = ContentHash::digest(bytes)
    )
}
fn policy(allow: &str) -> String {
    format!("[tool.taloon]\nschema-version=1\nallow=[{allow}]")
}
fn observations(i: &Inspection, b: &[u8]) -> Vec<AcquisitionResult> {
    i.requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, b).unwrap()))
        .collect()
}

#[rstest::rstest]
#[case::success_success(vec![true, true])]
#[case::success_failure(vec![true, false])]
#[case::failure_success(vec![false, true])]
#[case::failure_failure(vec![false, false])]
#[case::third_success(vec![true, false, true])]
fn duplicate_outcomes_remain_invalid(#[case] successful: Vec<bool>) {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let inspection = Inspection::from_documents(&lock(&bytes), &policy("'MIT'"), date()).unwrap();
    let request = &inspection.requests()[0];
    let mut results = observations(&inspection, &bytes);
    results.remove(0);
    results.extend(successful.into_iter().map(|success| {
        if success {
            AcquisitionResult::observed(
                request.id(),
                Evidence::from_archive(request, &bytes).unwrap(),
            )
        } else {
            AcquisitionResult::failed(
                request.id(),
                taloon::model::Error::new(taloon::model::Code::Acquisition, "fixture failure"),
            )
        }
    }));

    // Act
    let report = inspection.evaluate(results);

    // Assert
    assert_eq!(report.status(), taloon::report::Status::Incomplete);
    assert!(report.packages()[0].artifacts.iter().any(|artifact| {
        artifact
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.error.code == taloon::model::Code::InvalidResult)
    }));
}
#[rstest::rstest]
#[case::allowed("'MIT'", 0)]
#[case::denied("", 1)]
fn complete_results_obey_permission(#[case] allow: &str, #[case] expected: u8) {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let inspection = Inspection::from_documents(&lock(&bytes), &policy(allow), date()).unwrap();
    let results = observations(&inspection, &bytes);
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(inspection.requests().len(), 2);
    assert_eq!(report.exit_code(false), expected);
}

#[rstest::rstest]
#[case::allowed("'MIT'")]
#[case::denied("")]
fn partial_results_are_incomplete(#[case] allow: &str) {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let inspection = Inspection::from_documents(&lock(&bytes), &policy(allow), date()).unwrap();
    let mut results = observations(&inspection, &bytes);
    results.pop();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn empty_inventory_succeeds() {
    // Arrange
    let inspection =
        Inspection::from_documents("version=1\nrevision=5", &policy(""), date()).unwrap();
    // Act
    let report = inspection.evaluate(vec![]);
    // Assert
    assert_eq!(report.exit_code(false), 0);
}

#[test]
fn clarification_is_not_permission() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: WRONG"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='Apache-2.0'\nreason='reviewed declaration'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    // Act
    let report = i.evaluate(observations(&i, &bytes));
    // Assert
    assert_eq!(report.exit_code(false), 1);
}

#[test]
fn duplicate_results_with_clarification_are_incomplete() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='MIT'\nreason='verified'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let mut duplicate = observations(&i, &bytes);
    duplicate.extend(observations(&i, &bytes));
    // Act
    let report = i.evaluate(duplicate);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn weak_results_with_clarification_are_incomplete() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='MIT'\nreason='verified'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let meta = metadata("License-Expression: MIT");
    let hash = ContentHash::digest(&bytes);
    let weak = i
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
                        ContentHash::digest(&meta),
                    )
                    .unwrap(),
                    &meta,
                )
                .unwrap(),
            )
        })
        .collect();
    // Act
    let report = i.evaluate(weak);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn foreign_results_with_clarification_are_incomplete() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='MIT'\nreason='verified'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let foreign = common::request(&bytes, taloon::model::ArtifactFormat::Wheel);
    let mut unknown = observations(&i, &bytes);
    unknown.push(AcquisitionResult::observed(
        foreign.id(),
        Evidence::from_archive(&foreign, &bytes).unwrap(),
    ));
    // Act
    let report = i.evaluate(unknown);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn wrong_archive_identity_is_rejected() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='MIT'\nreason='verified'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let wrong = wheel(&metadata("Version: 9.0\nLicense-Expression: MIT"));
    // Act
    let result = Evidence::from_archive(&i.requests()[0], &wrong);
    // Assert
    assert!(result.is_err());
}

#[test]
fn fully_used_policy_has_no_escalated_warning() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT OR Apache-2.0"));
    let i =
        Inspection::from_documents(&lock(&bytes), &policy("'MIT','Apache-2.0'"), date()).unwrap();
    // Act
    let report = i.evaluate(observations(&i, &bytes));
    // Assert
    assert_eq!(report.exit_code(true), 0);
}

#[test]
fn unused_permission_warning_escalates_only_when_requested() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT OR Apache-2.0"));
    let unused = Inspection::from_documents(&lock(&bytes), &policy("'MIT','ISC'"), date()).unwrap();
    // Act
    let report = unused.evaluate(observations(&unused, &bytes));
    // Assert
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 1);
}

#[test]
fn warning_escalation_preserves_incompleteness() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT OR Apache-2.0"));
    let unused = Inspection::from_documents(&lock(&bytes), &policy("'MIT','ISC'"), date()).unwrap();
    // Act
    let incomplete = unused.evaluate(vec![]);
    // Assert
    assert!(incomplete.diagnostics().is_empty());
    assert_eq!(incomplete.exit_code(true), 2);
}

#[test]
fn observed_proof_can_be_rebound_but_each_association_is_required() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let doc = lock(&bytes);
    let doc = format!(
        "{doc}\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://other.test/simple'}}\nwheels=[{{url='https://files.example.test/a.whl',hash='{}'}}]",
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&doc, &policy("'MIT'"), date()).unwrap();
    let evidence = Evidence::from_archive(&i.requests()[0], &bytes).unwrap();
    let results = i
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), evidence.clone()))
        .collect();
    // Act
    let report = i.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), 0);
}

fn ordered_results_fixture(permitted: bool, missing: bool) -> (Inspection, Vec<AcquisitionResult>) {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let allow = if permitted { "'MIT'" } else { "" };
    let inspection = Inspection::from_documents(&lock(&bytes), &policy(allow), date()).unwrap();
    let mut results = observations(&inspection, &bytes);
    if missing {
        results.pop();
    }
    (inspection, results)
}

#[rstest::rstest]
#[case::permitted_complete(true, false)]
#[case::permitted_missing(true, true)]
#[case::denied_complete(false, false)]
#[case::denied_missing(false, true)]
fn result_order_cannot_change_semantic_reports(#[case] permitted: bool, #[case] missing: bool) {
    // Arrange
    let (inspection, forward) = ordered_results_fixture(permitted, missing);
    let (_, mut backward) = ordered_results_fixture(permitted, missing);
    backward.reverse();
    // Act
    let forward = inspection.evaluate(forward);
    let backward = inspection.evaluate(backward);
    // Assert
    assert_eq!(
        serde_json::to_value(forward).unwrap(),
        serde_json::to_value(backward).unwrap()
    );
}

#[test]
fn failures_without_a_proven_origin_explain_the_missing_path() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let i = Inspection::from_documents(&lock(&bytes), &policy(""), date()).unwrap();
    // Act
    let json = serde_json::to_value(i.evaluate(observations(&i, &bytes))).unwrap();
    // Assert
    assert_eq!(
        json["packages"][0]["origin_note"],
        "No unambiguous dependency origin is recorded in the lock."
    );
}

#[rstest::rstest]
#[case::permitted_alternative("MIT OR LicenseRef-Other", 0)]
#[case::custom_only("LicenseRef-Other", 1)]
fn an_unpermitted_custom_or_branch_does_not_require_archive_permission(
    #[case] declaration: &str,
    #[case] expected: u8,
) {
    // Arrange
    let meta = metadata(&format!("License-Expression: {declaration}"));
    let bytes = wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let inspection = Inspection::from_documents(&lock(&bytes), &policy("'MIT'"), date()).unwrap();
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
                        ContentHash::digest(&meta),
                    )
                    .unwrap(),
                    &meta,
                )
                .unwrap(),
            )
        })
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), expected);
}

#[test]
fn unsupported_unreachable_sources_remain_identifiable_without_secrets() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={git='https://user:password@git.test/example?signature=secret'}";
    let inspection = Inspection::from_documents(lock, &policy("'MIT'"), date()).unwrap();
    // Act
    let report = serde_json::to_value(inspection.evaluate(vec![])).unwrap();
    // Assert
    assert_eq!(report["packages"][0]["source"]["kind"], "git");
    assert_eq!(
        report["packages"][0]["source"]["location"],
        "https://git.test/example?redacted"
    );
    let text = report.to_string();
    assert!(!text.contains("password"));
    assert!(!text.contains("secret"));
}

#[rstest::rstest]
#[case::editable("editable")]
#[case::virtual_source("virtual")]
fn empty_editable_or_virtual_locations_do_not_disappear_as_workspace_roots(#[case] source: &str) {
    // Arrange
    let doc = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{{source}=''}}"
    );
    // Act
    let error = Inspection::from_documents(&doc, &policy(""), date())
        .err()
        .expect("empty local source must fail input validation");
    let report = taloon::report::Report::from_input_error(error);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}

#[test]
fn applied_overrides_have_unambiguous_policy_entry_identifiers() {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: Apache-2.0"));
    let hash = ContentHash::digest(&bytes);
    let config = format!(
        "{}\n[[tool.taloon.exceptions]]\nname='example'\nversion='1.0'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nallow=['Apache-2.0']\nreason='same reason'\n[[tool.taloon.exceptions]]\nname='example'\nversion='1.0'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nartifact-hash='{hash}'\nallow=['ISC']\nreason='same reason'\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='Apache-2.0'\nreason='same reason'",
        policy("'MIT'")
    );
    let inspection = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    // Act
    let report =
        serde_json::to_value(inspection.evaluate(observations(&inspection, &bytes))).unwrap();
    let entries = report["packages"][0]["artifacts"][0]["decision"]["overrides"]
        .as_array()
        .unwrap();
    // Assert
    assert_eq!(entries[0]["id"], "exception[1]");
    assert_eq!(entries[1]["id"], "exception[2]");
    assert_eq!(entries[2]["id"], format!("clarification:{hash}"));
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(128))]
    #[test]
    fn repeated_successful_results_never_replace_a_unique_observation(repetitions in 2_usize..32) {
        // Arrange
        let bytes = wheel(&metadata("License-Expression: MIT"));
        let inspection = Inspection::from_documents(&lock(&bytes), &policy("'MIT'"), date()).unwrap();
        let request = &inspection.requests()[0];
        let evidence = Evidence::from_archive(request, &bytes).unwrap();
        let mut results = observations(&inspection, &bytes);
        results.remove(0);
        results.extend((0..repetitions).map(|_| AcquisitionResult::observed(request.id(), evidence.clone())));
        // Act
        let report = inspection.evaluate(results);
        // Assert
        proptest::prop_assert_eq!(report.status(), taloon::report::Status::Incomplete);
        proptest::prop_assert!(report.packages()[0].artifacts.iter().any(|artifact| artifact.diagnostics.iter().any(|diagnostic| diagnostic.error.code == taloon::model::Code::InvalidResult)));
    }
}
