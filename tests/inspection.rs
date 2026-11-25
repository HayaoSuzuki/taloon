mod common;
use common::{metadata, wheel};
use std::fmt::Write as _;
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

/// Each observation list is a canonical representative of one model input.
#[rstest::rstest]
fn inspection_exit_matches_model(
    #[values(&[], &[true], &[false], &[true, true])] left: &[bool],
    #[values(&[], &[true], &[false], &[true, true])] right: &[bool],
    #[values(false, true)] permitted: bool,
    #[values(false, true)] unused_permission: bool,
    #[values(false, true)] deny_warnings: bool,
) {
    // Arrange
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let allow = match (permitted, unused_permission) {
        (false, false) => "",
        (true, false) => "'MIT'",
        (false, true) => "'ISC'",
        (true, true) => "'MIT','ISC'",
    };
    let inspection = Inspection::from_documents(&lock(&bytes), &policy(allow), date()).unwrap();
    let outcomes = [left, right]
        .into_iter()
        .enumerate()
        .flat_map(|(index, results)| results.iter().map(move |&success| (index, success)));
    let results = inspection_results(&inspection, &bytes, outcomes);
    let expected = match (left, right) {
        ([true], [true]) => u8::from(!permitted || (unused_permission && deny_warnings)),
        _ => 2,
    };

    // Act
    let report = inspection.evaluate(results);

    // Assert
    assert_eq!(report.exit_code(deny_warnings), expected);
}

fn inspection_results(
    inspection: &Inspection,
    bytes: &[u8],
    outcomes: impl IntoIterator<Item = (usize, bool)>,
) -> Vec<AcquisitionResult> {
    outcomes
        .into_iter()
        .map(|(index, success)| {
            let request = &inspection.requests()[index];
            if success {
                let evidence = Evidence::from_archive(request, bytes).unwrap();
                AcquisitionResult::observed(request.id(), evidence)
            } else {
                AcquisitionResult::failed(
                    request.id(),
                    taloon::model::Error::new(taloon::model::Code::Acquisition, "fixture failure"),
                )
            }
        })
        .collect()
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(128))]
    #[test]
    fn report_exit_requires_exact_successful_coverage_before_policy(
        outcomes in proptest::collection::vec((0_usize..2, proptest::bool::ANY), 0..7),
        permitted in proptest::bool::ANY,
    ) {
        // Arrange
        let bytes = wheel(&metadata("License-Expression: MIT"));
        let allow = if permitted { "'MIT'" } else { "" };
        let inspection = Inspection::from_documents(&lock(&bytes), &policy(allow), date()).unwrap();
        let complete = matches!(
            outcomes.as_slice(),
            [(0, true), (1, true)] | [(1, true), (0, true)]
        );
        let expected = if complete { u8::from(!permitted) } else { 2 };
        let results = inspection_results(&inspection, &bytes, outcomes);

        // Act
        let report = inspection.evaluate(results);

        // Assert
        proptest::prop_assert_eq!(report.exit_code(false), expected);
    }

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

#[derive(serde::Deserialize)]
struct ScopedPolicyCase {
    scenario: ScopedPolicyScenario,
    decision: ScopedPolicyDecision,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct ScopedPolicyScenario {
    scope: usize,
    clarification: usize,
    deny: bool,
    listed: bool,
    archive: bool,
}

#[derive(serde::Deserialize)]
struct ScopedPolicyDecision {
    strong: bool,
    exit: u8,
    expression: String,
    overrides: Vec<String>,
}

fn scoped_configuration(scenario: &ScopedPolicyScenario, hash: ContentHash) -> String {
    let mode = if scenario.deny { "deny" } else { "allow" };
    let listing = if scenario.listed { "'MIT'" } else { "" };
    let wrong = ContentHash::digest(b"a different archive");
    let mut config = format!("[tool.taloon]\nschema-version=1\n{mode}=[{listing}]");
    if scenario.scope != 0 {
        let exception_hash = if scenario.scope == 2 { wrong } else { hash };
        let host = if scenario.scope == 3 {
            "elsewhere.test"
        } else {
            "pypi.org"
        };
        write!(
            config,
            "\n[[tool.taloon.exceptions]]\nname='example'\nversion='1.0'\n\
             source={{kind='registry',location='https://{host}/simple'}}\n\
             artifact-hash='{exception_hash}'\nallow=['MIT','LicenseRef-Reviewed']\nreason='reviewed scope'"
        ).unwrap();
    }
    if scenario.clarification != 0 {
        let clarified_hash = if scenario.clarification == 1 {
            hash
        } else {
            wrong
        };
        write!(
            config,
            "\n[[tool.taloon.clarifications]]\nartifact-hash='{clarified_hash}'\n\
             expression='MIT'\nreason='reviewed declaration'"
        )
        .unwrap();
    }
    config
}

#[rstest::rstest]
fn quint_replays_scoped_policy(
    #[values(0, 1, 2, 3)] scope: usize,
    #[values(0, 1, 2)] clarification: usize,
    #[values(false, true)] deny: bool,
    #[values(false, true)] listed: bool,
    #[values(false, true)] archive: bool,
) {
    // Arrange
    use taloon::{
        evidence::SidecarAssociation,
        model::{Code, EvidenceRequirement},
        policy::OverrideKind,
    };
    let index = (((scope * 3 + clarification) * 2 + usize::from(deny)) * 2 + usize::from(listed))
        * 2
        + usize::from(archive);
    let row = include_str!("fixtures/oracle/scoped-policy.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: ScopedPolicyCase = serde_json::from_str(row).unwrap();
    assert_eq!(
        case.scenario,
        ScopedPolicyScenario {
            scope,
            clarification,
            deny,
            listed,
            archive
        }
    );
    let meta = metadata("License-Expression: LicenseRef-Reviewed");
    let bytes = wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let config = scoped_configuration(&case.scenario, hash);
    let inspection = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let proof = SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap();
    let requirements: Vec<_> = inspection
        .requests()
        .iter()
        .map(taloon::model::ArtifactRequest::requirement)
        .collect();
    let results = inspection
        .requests()
        .iter()
        .map(|request| {
            let evidence = if archive {
                Evidence::from_archive(request, &bytes)
            } else {
                Evidence::from_sidecar(request, proof, &meta)
            }
            .unwrap();
            AcquisitionResult::observed(request.id(), evidence)
        })
        .collect();

    // Act
    let report = inspection.evaluate(results);

    // Assert
    let expected = case.decision;
    let need = if expected.strong {
        EvidenceRequirement::ArchiveVerified
    } else {
        EvidenceRequirement::IndexAssociated
    };
    assert_eq!(requirements, [need; 2]);
    assert_eq!(report.exit_code(false), expected.exit);
    let expression = Some(expected.expression.as_str()).filter(|value| !value.is_empty());
    let artifacts = &report.packages()[0].artifacts;
    assert_eq!(artifacts.len(), 2);
    for artifact in artifacts {
        assert_eq!(artifact.status.exit_code(false, false), expected.exit);
        assert_eq!(artifact.interpreted_expression.as_deref(), expression);
        assert_eq!(artifact.decision.is_some(), expression.is_some());
        if let Some(decision) = &artifact.decision {
            let overrides: Vec<_> = decision
                .overrides
                .iter()
                .map(|entry| match entry.kind {
                    OverrideKind::Exception => {
                        assert_eq!(entry.id, "exception[1]");
                        "exception"
                    }
                    OverrideKind::Clarification => {
                        assert_eq!(entry.id, format!("clarification:{hash}"));
                        "clarification"
                    }
                })
                .collect();
            assert_eq!(overrides, expected.overrides);
            assert_eq!(decision.is_permitted(), expected.exit == 0);
        } else {
            assert_eq!(expected.exit, 2);
            assert!(
                artifact
                    .diagnostics
                    .iter()
                    .any(|diagnostic| { diagnostic.error.code == Code::InsufficientEvidence })
            );
        }
    }
}
