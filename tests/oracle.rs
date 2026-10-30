use taloon::{
    evidence::{Evidence, SidecarAssociation},
    inspection::{AcquisitionResult, Inspection},
    model::{Code, Error},
    report::Diagnostic,
};

mod common;
use serde::Deserialize;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvaluationDate, PackageId,
    SourceId,
};
use taloon::policy::{LicenseExpression, Policy};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Case {
    Inspection(InspectionCase),
    Permission {
        schema: u32,
        id: String,
        mode: String,
        expression: String,
        allow: Vec<String>,
        expected: bool,
    },
    Expiry {
        schema: u32,
        id: String,
        mode: String,
        date: String,
        expires: String,
        expected: bool,
    },
}

#[derive(Deserialize)]
struct InspectionCase {
    expression: String,
    schema: u32,
    id: String,
    mode: String,
    slots: Vec<u8>,
    required: bool,
    permitted: bool,
    anomaly: u8,
    #[serde(flatten)]
    escalation: Escalation,
    expected: u8,
}

#[derive(Deserialize)]
struct Escalation {
    warnings: bool,
    deny: bool,
}

#[test]
fn rust_policy_matches_lean_generated_cases() {
    let corpus = include_str!("fixtures/oracle/policy.jsonl");
    let mut count = 0;
    for line in corpus.lines().filter(|s| !s.is_empty()) {
        let case: Case = serde_json::from_str(line)
            .expect("oracle infrastructure error: malformed generated case");
        match case {
            Case::Inspection(case) => check_inspection(case),
            Case::Permission {
                schema,
                id,
                mode,
                expression,
                allow,
                expected,
            } => {
                assert_eq!(schema, 1);
                assert_eq!(mode, "strict");
                let date = EvaluationDate::parse("2026-10-30").unwrap();
                let text = format!(
                    "[tool.taloon]\nschema-version=1\nallow={}",
                    serde_json::to_string(&allow).unwrap()
                );
                let policy = Policy::from_pyproject(&text, date).expect("oracle setup failed");
                let hash = ContentHash::digest(b"oracle");
                let request = ArtifactRequest::new(
                    PackageId::parse(
                        "oracle",
                        "1.0",
                        SourceId::parse("registry", "https://pypi.org/simple").unwrap(),
                    )
                    .unwrap(),
                    ArtifactLocator::url("https://example.test/oracle.whl").unwrap(),
                    ArtifactFormat::Wheel,
                    Some(hash),
                )
                .unwrap();
                let expr = LicenseExpression::parse(&expression)
                    .expect("oracle expression adapter failed");
                assert_eq!(
                    policy.evaluate(&request, hash, &expr).is_permitted(),
                    expected,
                    "semantic mismatch: {id}"
                );
            }
            Case::Expiry {
                schema,
                id,
                mode,
                date,
                expires,
                expected,
            } => {
                assert_eq!(schema, 1);
                assert_eq!(mode, "strict");
                let text = format!(
                    "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='unused'\nversion='1.0'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nallow=['MIT']\nreason='oracle'\nexpires='{expires}'"
                );
                assert_eq!(
                    Policy::from_pyproject(&text, EvaluationDate::parse(&date).unwrap()).is_ok(),
                    expected,
                    "semantic mismatch: {id}"
                );
            }
        }
        count += 1;
    }
    assert!(count > 0, "oracle infrastructure error: empty corpus");
}

fn check_inspection(case: InspectionCase) {
    use std::fmt::Write;
    let InspectionCase {
        expression,
        schema,
        id,
        mode,
        slots,
        required,
        permitted,
        anomaly,
        escalation: Escalation { warnings, deny },
        expected,
    } = case;

    assert_eq!(schema, 1);
    assert_eq!(mode, "strict");
    assert_eq!(slots.len(), 2);
    let meta = common::metadata(&format!("License-Expression: {expression}"));
    let bytes = common::wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{{url='https://example.test/a.whl',hash='{hash}'}},{{url='https://example.test/b.whl',hash='{hash}'}}]"
    );
    let mut config = format!(
        "[tool.taloon]\nschema-version=1\nallow={}",
        if permitted { "['MIT']" } else { "[]" }
    );
    if required {
        write!(config,"\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='{expression}'\nreason='oracle'").unwrap();
    }
    let inspection =
        Inspection::from_documents(&lock, &config, EvaluationDate::parse("2026-10-30").unwrap())
            .expect("oracle inspection setup failed");
    let mut results = Vec::new();
    for (request, slot) in inspection.requests().iter().zip(slots) {
        assert!(slot <= 2, "oracle invalid slot");
        if slot == 0 {
            continue;
        }
        let evidence = if slot == 2 {
            Evidence::from_archive(request, &bytes)
        } else {
            Evidence::from_sidecar(
                request,
                SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
                &meta,
            )
        }
        .expect("oracle evidence setup failed");
        results.push(AcquisitionResult::observed(request.id(), evidence.clone()));
        if anomaly == 1 {
            results.push(AcquisitionResult::observed(request.id(), evidence));
        }
    }
    if anomaly == 2 {
        let foreign = common::request(&bytes, ArtifactFormat::Wheel);
        results.push(AcquisitionResult::observed(
            foreign.id(),
            Evidence::from_archive(&foreign, &bytes).unwrap(),
        ));
    }
    let report = inspection.evaluate(results);
    let report = if warnings {
        report.with_warnings(vec![Diagnostic::warning(Error::new(
            Code::CacheWrite,
            "oracle warning",
        ))])
    } else {
        report
    };
    assert_eq!(report.exit_code(deny), expected, "semantic mismatch: {id}");
}
