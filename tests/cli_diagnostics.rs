mod common;
use rstest::rstest;
use std::fmt::Write as _;
use std::{
    path::Path,
    process::{Command, Output},
};

fn fixture(root: &Path, declarations: &[&str]) {
    let mut lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='.'}\nwheels=[".to_owned();
    for (i, fields) in declarations.iter().enumerate() {
        let name = format!("example-{i}.whl");
        std::fs::write(root.join(&name), common::wheel(&common::metadata(fields))).unwrap();
        write!(lock, "{{path='{name}'}},").unwrap();
    }
    lock.push(']');
    std::fs::write(root.join("uv.lock"), lock).unwrap();
    std::fs::write(
        root.join("pyproject.toml"),
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
    )
    .unwrap();
}
fn invoke(root: &Path, command: &str, extra: &[&str]) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_taloon"));
    process
        .current_dir(root)
        .env("TALOON_CACHE_DIR", root.join("cache"))
        .env_remove("NETRC")
        .arg(command);
    if command == "describe" {
        process.arg("example");
    }
    process.arg("--offline").args(extra).output().unwrap()
}

#[rstest]
#[case::check("check")]
#[case::describe("describe")]
fn identical_problem_artifacts_have_one_actionable_summary(#[case] command: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &["License: custom terms", "License: custom terms"],
    );
    // Act
    let result = invoke(temp.path(), command, &[]);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2), "{text}");
    assert!(text.contains("2/2 artifacts"), "{text}");
    assert_eq!(text.matches("UnrecognizedLegacy").count(), 1, "{text}");
    assert_eq!(text.matches("License: custom terms").count(), 1, "{text}");
    assert!(text.contains("hash-bound clarification"), "{text}");
    assert!(text.contains("--verbose"), "{text}");
}

#[rstest]
#[case::declaration("License: custom terms", "License: other terms")]
#[case::cause("License: custom terms", "License-Expression: bad")]
fn distinct_problems_are_not_merged(
    #[case] first: &str,
    #[case] second: &str,
    #[values("check", "describe")] command: &str,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path(), &[first, second]);
    // Act
    let result = invoke(temp.path(), command, &[]);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2), "{text}");
    assert_eq!(text.matches("1/2 artifacts").count(), 2, "{text}");
    assert!(text.contains(first), "{text}");
    assert!(text.contains(second), "{text}");
}

#[rstest]
#[case::check("check")]
#[case::describe("describe")]
fn verbose_retains_each_artifact_and_observed_hash(#[case] command: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &["License: custom terms", "License: custom terms"],
    );
    // Act
    let result = invoke(temp.path(), command, &["--verbose"]);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2), "{text}");
    assert!(text.contains("example-0.whl"), "{text}");
    assert!(text.contains("example-1.whl"), "{text}");
    assert_eq!(text.matches("observed sha256:").count(), 2, "{text}");
    assert_eq!(text.matches("UnrecognizedLegacy").count(), 2, "{text}");
}

#[rstest]
#[case::check("check", &["--format=json"])]
#[case::check_verbose("check", &["--verbose", "--format=json"])]
#[case::describe("describe", &["--format=json"])]
#[case::describe_verbose("describe", &["--verbose", "--format=json"])]
fn json_retains_individual_artifacts_and_structured_causes(
    #[case] command: &str,
    #[case] flags: &[&str],
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &["License: custom terms", "License-Expression: invalid"],
    );
    // Act
    let result = invoke(temp.path(), command, flags);
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(report["schema_version"], 1);
    let artifacts = report["packages"][0]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 2);
    let mut problems: Vec<_> = artifacts
        .iter()
        .map(|a| a["diagnostics"][0]["license_problem"].as_str().unwrap())
        .collect();
    problems.sort_unstable();
    assert_eq!(problems, vec!["invalid_expression", "unrecognized_legacy"]);
}

#[rstest]
#[case::check("check")]
#[case::describe("describe")]
fn acquisition_failure_is_distinct_from_declaration_failure(#[case] command: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &["License: custom terms", "License: custom terms"],
    );
    std::fs::remove_file(temp.path().join("example-1.whl")).unwrap();
    // Act
    let result = invoke(temp.path(), command, &[]);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2), "{text}");
    assert_eq!(text.matches("1/2 artifacts").count(), 2, "{text}");
    assert_eq!(text.matches("UnrecognizedLegacy").count(), 1, "{text}");
    assert!(text.contains("Acquisition"), "{text}");
    assert!(text.contains("--offline"), "{text}");
}

proptest::proptest! {
    #[test]
    fn different_declarations_after_preview_boundary_stay_separate(suffix in "[a-z]{1,30}") {
        // Arrange
        let temp=tempfile::tempdir().unwrap();
        let common="a".repeat(250);
        let first=format!("License: {common}X{suffix}");
        let second=format!("License: {common}Y{suffix}");
        fixture(temp.path(),&[&first,&second]);
        // Act
        let result=invoke(temp.path(),"check",&[]);
        let text=String::from_utf8(result.stdout).unwrap();
        // Assert
        proptest::prop_assert_eq!(result.status.code(),Some(2));
        proptest::prop_assert_eq!(text.matches("1/2 artifacts").count(),2);
    }
}

#[test]
fn different_hash_scoped_clarifications_remain_separate() {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &[
            "License: custom terms\nSummary: first",
            "License: custom terms\nSummary: second",
        ],
    );
    let mut policy = "[tool.taloon]\nschema-version=1\nallow=['MIT']\n".to_owned();
    for (i, reason) in [(0, "First review"), (1, "Second review")] {
        let bytes = std::fs::read(temp.path().join(format!("example-{i}.whl"))).unwrap();
        let hash = taloon::model::ContentHash::digest(&bytes);
        writeln!(policy,"[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='{reason}'").unwrap();
    }
    std::fs::write(temp.path().join("pyproject.toml"), policy).unwrap();
    // Act
    let result = invoke(temp.path(), "check", &[]);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(0), "{text}");
    assert_eq!(text.matches("1/2 artifacts").count(), 2, "{text}");
    assert!(text.contains("First review"), "{text}");
    assert!(text.contains("Second review"), "{text}");
}

#[rstest]
#[case::check("check", &[])]
#[case::check_verbose("check", &["--verbose"])]
#[case::describe("describe", &[])]
#[case::describe_verbose("describe", &["--verbose"])]
fn preview_escapes_controls_and_redacts_credential_urls(
    #[case] command: &str,
    #[case] flags: &[&str],
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    fixture(
        temp.path(),
        &[
            "License: custom\u{1b}\u{202e} terms https://user:password@example.test/license?token=secret",
        ],
    );
    // Act
    let result = invoke(temp.path(), command, flags);
    let text = String::from_utf8(result.stdout).unwrap();
    // Assert
    assert_eq!(result.status.code(), Some(2), "{text}");
    assert!(!text.contains('\u{1b}'), "{text}");
    assert!(!text.contains('\u{202e}'), "{text}");
    assert!(!text.contains("password"), "{text}");
    assert!(!text.contains("token=secret"), "{text}");
    assert!(text.contains("example.test/license?redacted"), "{text}");
}
