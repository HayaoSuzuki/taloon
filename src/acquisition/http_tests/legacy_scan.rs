//! Replay pending and completed license-file observations across acquisition runs.
use super::*;
use serde::Deserialize;
use taloon::evidence::LicenseOrigin;

const ARCHIVE: &str = "https://files.test/a.whl";
const METADATA: &str = "https://files.test/a.whl.metadata";

#[derive(Deserialize)]
struct Case {
    scenario: Scenario,
    decision: Decision,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    bsd: bool,
    outcome: usize,
    initially_available: bool,
    phase: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Decision {
    cache: String,
    hits: u64,
    index_requests: u64,
    metadata_requests: u64,
    archive_requests: u64,
    exit: u8,
    proof: String,
    expression: String,
    origin: String,
    issue: String,
}

fn legacy_fixture(bsd: bool, outcome: usize) -> (Fixture, Vec<u8>, Inspection) {
    let legacy = if bsd { "BSD" } else { "UNKNOWN" };
    let pending = format!("License: {legacy}\nLicense-File: LICENSE");
    let matching_license = if bsd { "BSD-3-Clause" } else { "MIT" };
    let matching_text = spdx::license_id(matching_license).unwrap().text();
    let (fields, text) = match outcome {
        0 => (pending.as_str(), matching_text),
        1 if bsd => (
            pending.as_str(),
            spdx::license_id("Apache-2.0").unwrap().text(),
        ),
        1 => (pending.as_str(), "Custom redistribution conditions apply."),
        2 => (
            "License-Expression: ISC\nLicense-File: LICENSE",
            matching_text,
        ),
        3 => (
            "License: BSD License; redistribution requires separate permission\nLicense-File: LICENSE",
            matching_text,
        ),
        _ => unreachable!(),
    };
    let (fixture, request, _) = bundled_fixture(fields, text);
    let hint = crate::testfixtures::metadata(&pending);
    mutate_fixture_index(&fixture, |index| {
        index["files"][0]["core-metadata"]["sha256"] = ContentHash::digest(&hint)
            .to_string()
            .replace("sha256:", "")
            .into();
    });
    fixture.add(METADATA, 200, &hint);
    let bytes = fixture.routes.lock().unwrap()[ARCHIVE].body.clone();
    let hash = request.hash().unwrap();
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\n\
         source={{registry='https://example.test/simple'}}\n\
         wheels=[{{url='{ARCHIVE}',hash='{hash}'}}]"
    );
    let inspection = Inspection::from_documents(
        &lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT','BSD-3-Clause','ISC']",
        EvaluationDate::parse("2026-11-01").unwrap(),
    )
    .unwrap();
    (fixture, bytes, inspection)
}

fn check_step(
    acquirer: &Acquirer,
    inspection: &Inspection,
    batch: AcquisitionBatch,
    expected: &Decision,
) {
    let summary = batch.summary;
    assert_eq!(summary.cache_hits, expected.hits);
    assert_eq!(summary.http.index.requests, expected.index_requests);
    assert_eq!(summary.http.metadata.requests, expected.metadata_requests);
    assert_eq!(summary.http.archive.requests, expected.archive_requests);
    assert_eq!(
        summary.network_requests,
        expected.index_requests + expected.metadata_requests + expected.archive_requests
    );
    assert!(batch.warnings.is_empty());
    let report = inspection.evaluate(batch.results);
    assert_eq!(report.exit_code(false), expected.exit);
    let artifact = &report.packages()[0].artifacts[0];
    assert_eq!(
        artifact
            .interpreted_expression
            .as_deref()
            .unwrap_or_default(),
        expected.expression
    );
    let evidence = artifact.evidence.as_ref();
    let strength = evidence.map(|summary| summary.strength);
    let origin = evidence.and_then(|summary| summary.license_origin);
    let expected_strength = match expected.proof.as_str() {
        "index" => Some(EvidenceRequirement::IndexAssociated),
        "archive" => Some(EvidenceRequirement::ArchiveVerified),
        "" => None,
        label => panic!("unexpected proof in model trace: {label}"),
    };
    let expected_origin = match expected.origin.as_str() {
        "license_expression" => Some(LicenseOrigin::LicenseExpression),
        "legacy_supplement" => Some(LicenseOrigin::LegacySupplement),
        "" => None,
        label => panic!("unexpected origin in model trace: {label}"),
    };
    assert_eq!(strength, expected_strength);
    assert_eq!(origin, expected_origin);
    let issue = match expected.issue.as_str() {
        "" => None,
        "acquisition" => Some(Code::Acquisition),
        "unknown_license" => Some(Code::UnknownLicense),
        label => panic!("unexpected issue in model trace: {label}"),
    };
    let actual: Vec<_> = artifact
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == taloon::report::Severity::Error)
        .map(|diagnostic| diagnostic.error.code)
        .collect();
    assert_eq!(actual, issue.into_iter().collect::<Vec<_>>());

    let request = &inspection.requests()[0];
    let cached = acquirer
        .cache
        .get(request, request.hash().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(cached.needs_license_files(), expected.cache == "pending");
    assert_eq!(
        cached.requirement_satisfied(EvidenceRequirement::ArchiveVerified),
        expected.cache == "archive"
    );
}

#[rstest::rstest]
fn quint_replays_legacy_scan(
    #[values(false, true)] bsd: bool,
    #[values(0, 1, 2, 3)] outcome: usize,
    #[values(false, true)] initially_available: bool,
) {
    // Arrange
    let index = ((usize::from(bsd) * 4 + outcome) * 2 + usize::from(initially_available)) * 5;
    let cases: Vec<Case> = include_str!("../../../tests/fixtures/oracle/legacy-scan.jsonl")
        .lines()
        .skip(index)
        .take(5)
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(cases.len(), 5);
    let root = tempfile::tempdir().unwrap();
    let (fixture, bytes, inspection) = legacy_fixture(bsd, outcome);

    for (phase, case) in cases.iter().enumerate() {
        assert_eq!(
            case.scenario,
            Scenario {
                bsd,
                outcome,
                initially_available,
                phase
            }
        );
        let available = phase == 2 || (phase == 0 && initially_available);
        if available {
            fixture.add(ARCHIVE, 200, &bytes);
        } else {
            fixture.add(ARCHIVE, 404, b"");
        }
        let offline = matches!(phase, 1 | 3);
        let acquirer = Acquirer::with_transport(
            AcquisitionOptions::testing(root.path(), offline),
            fixture.clone(),
        )
        .unwrap();

        // Act
        let batch = acquirer.acquire(inspection.requests());

        // Assert
        check_step(&acquirer, &inspection, batch, &case.decision);
    }
}
