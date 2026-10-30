use serde::Deserialize;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvaluationDate, PackageId,
    SourceId,
};
use taloon::policy::{LicenseExpression, Policy};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Case {
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

#[test]
fn rust_policy_matches_lean_generated_cases() {
    let corpus = include_str!("fixtures/oracle/policy.jsonl");
    let mut count = 0;
    for line in corpus.lines().filter(|s| !s.is_empty()) {
        let case: Case = serde_json::from_str(line)
            .expect("oracle infrastructure error: malformed generated case");
        match case {
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
