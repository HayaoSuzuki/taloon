//! Replay generated index binding, sidecar admission and archive fallback cases.
use super::*;
use serde::Deserialize;

const INDEX: &str = "https://example.test/simple/example/";
const ARCHIVE: &str = "https://files.test/a.whl?variant=a";
const METADATA: &str = "https://files.test/a.whl.metadata?variant=a";
const OTHER_URL: &str = "https://files.test/a.whl?variant=b";

#[derive(Deserialize)]
struct Case {
    scenario: Scenario,
    decision: Decision,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Scenario {
    advertisement: usize,
    metadata: usize,
    strong: bool,
    archive: usize,
    html: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Decision {
    index_requests: u64,
    metadata_requests: u64,
    archive_requests: u64,
    proof: String,
    expression: String,
    warning: String,
    issue: String,
    exit: u8,
    offline_hits: u64,
}

fn inspection(hash: ContentHash, strong: bool) -> Inspection {
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\n\
         source={{registry='https://example.test/simple'}}\n\
         wheels=[{{url='{ARCHIVE}',hash='{hash}'}}]"
    );
    let mut config = "[tool.taloon]\nschema-version=1\nallow=['MIT','Apache-2.0']".to_owned();
    if strong {
        use std::fmt::Write as _;
        write!(
            config,
            "\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\n\
             expression='Apache-2.0'\nreason='fixture reviewed archive'"
        )
        .unwrap();
    }
    Inspection::from_documents(&lock, &config, EvaluationDate::parse("2026-11-01").unwrap())
        .unwrap()
}

fn advertise(fixture: &Fixture, scenario: &Scenario, hash: ContentHash, meta: ContentHash) {
    let url = if scenario.advertisement == 1 {
        OTHER_URL
    } else {
        ARCHIVE
    };
    let archive = if scenario.advertisement == 2 {
        ContentHash::digest(b"wrong archive")
    } else {
        hash
    };
    let archive = archive.to_string().replace("sha256:", "");
    let metadata = meta.to_string().replace("sha256:", "");
    let copies = if scenario.advertisement == 3 { 2 } else { 1 };
    if scenario.html {
        let anchor = format!(
            "<a href='{url}#sha256={archive}' data-core-metadata='sha256={metadata}'>wheel</a>"
        );
        fixture.add(INDEX, 200, anchor.repeat(copies).as_bytes());
        fixture
            .routes
            .lock()
            .unwrap()
            .get_mut(INDEX)
            .unwrap()
            .content_type = "text/html".to_owned();
    } else {
        let file = serde_json::json!({
            "url": url,
            "hashes": {"sha256": archive},
            "core-metadata": {"sha256": metadata}
        });
        let index = serde_json::json!({
            "meta": {"api-version": "1.0"},
            "files": vec![file; copies]
        });
        fixture.add(INDEX, 200, &serde_json::to_vec(&index).unwrap());
    }
}

fn transport(scenario: &Scenario, bytes: &[u8]) -> Fixture {
    let fixture = Fixture::default();
    let correct = crate::testfixtures::metadata("License-Expression: MIT");
    let unexpected = crate::testfixtures::metadata("License-Expression: MIT\nX-Variant: changed");
    let wrong_name =
        b"Metadata-Version: 2.4\nName: elsewhere\nVersion: 1.0\nLicense-Expression: MIT\n\n";
    let malformed = b"not a metadata header\n";
    let (metadata_status, advertised, returned): (_, &[u8], &[u8]) = match scenario.metadata {
        0 => (200, &correct, &correct),
        1 => (200, &correct, &unexpected),
        2 => (200, wrong_name, wrong_name),
        3 => (200, malformed, malformed),
        4 => (404, &correct, b""),
        _ => unreachable!(),
    };
    advertise(
        &fixture,
        scenario,
        ContentHash::digest(bytes),
        ContentHash::digest(advertised),
    );
    fixture.add(METADATA, metadata_status, returned);
    match scenario.archive {
        0 => fixture.add(ARCHIVE, 200, bytes),
        1 => fixture.add(ARCHIVE, 200, b"different archive"),
        2 => fixture.add(ARCHIVE, 404, b""),
        _ => unreachable!(),
    }
    fixture
}

fn expected_code(label: &str) -> Option<Code> {
    match label {
        "" => None,
        "acquisition" => Some(Code::Acquisition),
        "hash_mismatch" => Some(Code::HashMismatch),
        "identity_mismatch" => Some(Code::IdentityMismatch),
        "invalid_metadata" => Some(Code::InvalidMetadata),
        _ => panic!("unexpected error in sidecar trace: {label}"),
    }
}

fn check_report(
    inspection: &Inspection,
    results: Vec<AcquisitionResult>,
    expected: &Decision,
    offline: bool,
) {
    let report = inspection.evaluate(results);
    assert_eq!(report.exit_code(false), expected.exit);
    let artifact = &report.packages()[0].artifacts[0];
    let proof = artifact
        .evidence
        .as_ref()
        .map(|evidence| match evidence.strength {
            EvidenceRequirement::ArchiveVerified => "archive",
            EvidenceRequirement::IndexAssociated => "index",
        });
    assert_eq!(proof.unwrap_or_default(), expected.proof);
    assert_eq!(
        artifact
            .interpreted_expression
            .as_deref()
            .unwrap_or_default(),
        expected.expression
    );
    let issue = if offline && expected.exit == 2 {
        Some(Code::Acquisition)
    } else {
        expected_code(&expected.issue)
    };
    let errors: Vec<_> = artifact
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == taloon::report::Severity::Error)
        .map(|diagnostic| diagnostic.error.code)
        .collect();
    assert_eq!(errors, issue.into_iter().collect::<Vec<_>>());
}

#[rstest::rstest]
fn quint_replays_sidecar_fallback(
    #[values(0, 1, 2, 3)] advertisement: usize,
    #[values(0, 1, 2, 3, 4)] metadata: usize,
    #[values(false, true)] strong: bool,
    #[values(0, 1, 2)] archive: usize,
    #[values(false, true)] html: bool,
) {
    // Arrange
    let index = (((advertisement * 5 + metadata) * 2 + usize::from(strong)) * 3 + archive) * 2
        + usize::from(html);
    let row = include_str!("../../../tests/fixtures/oracle/sidecar-fallback.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: Case = serde_json::from_str(row).unwrap();
    assert_eq!(
        case.scenario,
        Scenario {
            advertisement,
            metadata,
            strong,
            archive,
            html
        }
    );
    let temp = tempfile::tempdir().unwrap();
    let bytes = crate::testfixtures::wheel(&crate::testfixtures::metadata(
        "License-Expression: Apache-2.0",
    ));
    let inspection = inspection(ContentHash::digest(&bytes), strong);
    let fixture = transport(&case.scenario, &bytes);
    let online = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let offline =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), true), fixture).unwrap();

    // Act
    let cold = online.acquire(inspection.requests());
    let warm = offline.acquire(inspection.requests());

    // Assert
    let expected = case.decision;
    assert_eq!(cold.summary.http.index.requests, expected.index_requests);
    assert_eq!(
        cold.summary.http.metadata.requests,
        expected.metadata_requests
    );
    assert_eq!(
        cold.summary.http.archive.requests,
        expected.archive_requests
    );
    let warnings: Vec<_> = cold
        .warnings
        .iter()
        .map(|diagnostic| diagnostic.error.code)
        .collect();
    assert_eq!(
        warnings,
        expected_code(&expected.warning)
            .into_iter()
            .collect::<Vec<_>>()
    );
    check_report(&inspection, cold.results, &expected, false);
    assert_eq!(warm.summary.network_requests, 0);
    assert_eq!(warm.summary.cache_hits, expected.offline_hits);
    assert!(warm.warnings.is_empty());
    check_report(&inspection, warm.results, &expected, true);
}
