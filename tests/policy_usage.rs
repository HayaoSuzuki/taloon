//! Replay model-generated policy usage through the public report boundary.
mod common;
use rstest::rstest;
use std::fmt::Write as _;
use taloon::{
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{ArtifactRequest, Code, ContentHash, Error, EvaluationDate},
    report::Severity,
};

#[derive(serde::Deserialize)]
struct Case {
    scenario: Scenario,
    decision: Decision,
}

// This record mirrors independent switches in the generated model input.
#[allow(clippy::struct_excessive_bools)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    outcome: u8,
    deny: bool,
    listed: bool,
    beta_scope: bool,
    alpha_scope: bool,
    clarify: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Decision {
    exit: u8,
    strict_exit: u8,
    warnings: Vec<String>,
}

fn archive(package: &str, expression: &str) -> Vec<u8> {
    common::wheel(
        format!("Metadata-Version: 2.4\nName: {package}\nVersion: 1.0\nLicense-Expression: {expression}\n\n")
            .as_bytes(),
    )
}

fn lock_document(alpha: ContentHash, beta: ContentHash) -> String {
    let mut lock = "version=1\nrevision=5\n".to_owned();
    for (name, hash) in [("alpha", alpha), ("beta", beta)] {
        writeln!(
            lock,
            "[[package]]\nname='{name}'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}\n\
             wheels=[{{url='https://files.example.test/{name}.whl',hash='{hash}'}}]"
        ).unwrap();
    }
    lock
}

fn configuration(scenario: &Scenario, alpha: ContentHash, beta: ContentHash) -> String {
    let mode = if scenario.deny { "deny" } else { "allow" };
    let listing = if scenario.listed { ",'ISC'" } else { "" };
    let mut policy =
        format!("[tool.taloon]\nschema-version=1\n{mode}=['MIT','Apache-2.0'{listing}]");
    // File order differs from the internal package order.
    let beta_scope = if scenario.beta_scope { beta } else { alpha };
    let alpha_scope = if scenario.alpha_scope { alpha } else { beta };
    for (name, hash) in [("beta", beta_scope), ("alpha", alpha_scope)] {
        write!(
            policy,
            "\n[[tool.taloon.exceptions]]\nname='{name}'\nversion='1.0'\n\
             source={{kind='registry',location='https://one.test/simple'}}\n\
             artifact-hash='{hash}'\nallow=['BSD-3-Clause']\nreason='reviewed exception'"
        )
        .unwrap();
    }
    let clarified = if scenario.clarify {
        beta
    } else {
        ContentHash::digest(b"absent archive")
    };
    write!(
        policy,
        "\n[[tool.taloon.clarifications]]\nartifact-hash='{clarified}'\n\
         expression='BSD-3-Clause'\nreason='reviewed declaration'"
    )
    .unwrap();
    policy
}

fn request<'a>(inspection: &'a Inspection, name: &str) -> &'a ArtifactRequest {
    inspection
        .requests()
        .iter()
        .find(|request| request.package().name() == name)
        .unwrap()
}

fn observed(request: &ArtifactRequest, bytes: &[u8]) -> AcquisitionResult {
    AcquisitionResult::observed(
        request.id(),
        Evidence::from_archive(request, bytes).unwrap(),
    )
}

fn expected_message(label: &str, deny: bool) -> String {
    match label {
        "ISC" => {
            let kind = if deny { "denial" } else { "permission" };
            format!("unused global {kind}: ISC")
        }
        "beta exception" => "unused exception entry: 1".to_owned(),
        "alpha exception" => "unused exception entry: 2".to_owned(),
        "clarification" => format!(
            "unused clarification: {}",
            ContentHash::digest(b"absent archive")
        ),
        unexpected => panic!("unexpected warning in model trace: {unexpected}"),
    }
}

#[rstest]
fn quint_replays_unused_policy(
    #[values(0, 1, 2, 3)] outcome: u8,
    #[values(0, 1, 2, 3)] globals: usize,
    #[values(0, 1, 2, 3)] scopes: usize,
    #[values(false, true)] clarify: bool,
) {
    // Arrange
    let index = usize::from(outcome) * 32 + globals * 8 + scopes * 2 + usize::from(clarify);
    let row = include_str!("fixtures/oracle/unused-policy.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: Case = serde_json::from_str(row).unwrap();
    assert_eq!(case.scenario.outcome, outcome);
    assert_eq!(case.scenario.clarify, clarify);
    assert_eq!(
        usize::from(case.scenario.deny) * 2 + usize::from(case.scenario.listed),
        globals
    );
    assert_eq!(
        usize::from(case.scenario.beta_scope) * 2 + usize::from(case.scenario.alpha_scope),
        scopes
    );
    let alpha = archive("alpha", "Apache-2.0 OR MIT OR BSD-3-Clause");
    let beta = archive("beta", "ISC");
    let alpha_hash = ContentHash::digest(&alpha);
    let beta_hash = ContentHash::digest(&beta);
    let inspection = Inspection::from_documents(
        &lock_document(alpha_hash, beta_hash),
        &configuration(&case.scenario, alpha_hash, beta_hash),
        EvaluationDate::parse("2026-11-01").unwrap(),
    )
    .unwrap();
    let beta_request = request(&inspection, "beta");
    let mut observations = vec![observed(request(&inspection, "alpha"), &alpha)];
    match outcome {
        0 => observations.push(observed(beta_request, &beta)),
        1 => {}
        2 => observations.extend([observed(beta_request, &beta), observed(beta_request, &beta)]),
        3 => observations.push(AcquisitionResult::failed(
            beta_request.id(),
            Error::new(Code::Acquisition, "fixture failure"),
        )),
        _ => unreachable!(),
    }

    // Act
    let report = inspection.evaluate(observations);

    // Assert
    let expected = case.decision;
    let warnings: Vec<_> = report
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.severity, Severity::Warning);
            assert_eq!(diagnostic.error.code, Code::UnusedPermission);
            diagnostic.error.message.as_str()
        })
        .collect();
    let expected_warnings: Vec<_> = expected
        .warnings
        .iter()
        .map(|label| expected_message(label, case.scenario.deny))
        .collect();
    assert_eq!(warnings, expected_warnings);
    assert_eq!(report.exit_code(false), expected.exit);
    assert_eq!(report.exit_code(true), expected.strict_exit);
}
