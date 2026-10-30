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
#[test]
fn missing_or_weak_results_never_succeed() {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let i = Inspection::from_documents(&lock(&bytes), &policy("'MIT'"), date()).unwrap();
    assert_eq!(i.requests().len(), 2);
    let mut partial = observations(&i, &bytes);
    partial.pop();
    assert_eq!(i.evaluate(partial).exit_code(false), 2);
    assert_eq!(i.evaluate(observations(&i, &bytes)).exit_code(false), 0);
    let denied = Inspection::from_documents(&lock(&bytes), &policy(""), date()).unwrap();
    assert_eq!(
        denied
            .evaluate(observations(&denied, &bytes))
            .exit_code(false),
        1
    );
    let mut partial = observations(&denied, &bytes);
    partial.pop();
    assert_eq!(denied.evaluate(partial).exit_code(false), 2);
    let empty = Inspection::from_documents("version=1\nrevision=5", &policy(""), date()).unwrap();
    assert_eq!(empty.evaluate(vec![]).exit_code(false), 0);
}
#[test]
fn clarification_is_not_permission() {
    let bytes = wheel(&metadata("License-Expression: WRONG"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='Apache-2.0'\nreason='reviewed declaration'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    assert_eq!(i.evaluate(observations(&i, &bytes)).exit_code(false), 1);
}

#[test]
fn duplicate_unknown_wrong_identity_and_weak_results_are_incomplete() {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let config = format!(
        "{}\n[[tool.taloon.clarifications]]\nartifact-hash='{}'\nexpression='MIT'\nreason='verified'",
        policy("'MIT'"),
        ContentHash::digest(&bytes)
    );
    let i = Inspection::from_documents(&lock(&bytes), &config, date()).unwrap();
    let mut duplicate = observations(&i, &bytes);
    duplicate.extend(observations(&i, &bytes));
    assert_eq!(i.evaluate(duplicate).exit_code(false), 2);
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
    assert_eq!(i.evaluate(weak).exit_code(false), 2);
    let foreign = common::request(&bytes, taloon::model::ArtifactFormat::Wheel);
    let mut unknown = observations(&i, &bytes);
    unknown.push(AcquisitionResult::observed(
        foreign.id(),
        Evidence::from_archive(&foreign, &bytes).unwrap(),
    ));
    assert_eq!(i.evaluate(unknown).exit_code(false), 2);
    let wrong = wheel(&metadata("Version: 9.0\nLicense-Expression: MIT"));
    assert!(Evidence::from_archive(&i.requests()[0], &wrong).is_err());
}

#[test]
fn warnings_are_conservative_and_escalation_preserves_incompleteness() {
    let bytes = wheel(&metadata("License-Expression: MIT OR Apache-2.0"));
    let i =
        Inspection::from_documents(&lock(&bytes), &policy("'MIT','Apache-2.0'"), date()).unwrap();
    assert_eq!(i.evaluate(observations(&i, &bytes)).exit_code(true), 0);
    let unused = Inspection::from_documents(&lock(&bytes), &policy("'MIT','ISC'"), date()).unwrap();
    let report = unused.evaluate(observations(&unused, &bytes));
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.exit_code(true), 1);
    let incomplete = unused.evaluate(vec![]);
    assert!(incomplete.diagnostics().is_empty());
    assert_eq!(incomplete.exit_code(true), 2);
}

#[test]
fn observed_proof_can_be_rebound_but_each_association_is_required() {
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
    assert_eq!(i.evaluate(results).exit_code(false), 0);
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(256))]
    #[test]
    fn result_order_cannot_change_semantic_reports(reverse in proptest::bool::ANY,missing in proptest::bool::ANY,permitted in proptest::bool::ANY){
      let bytes=wheel(&metadata("License-Expression: MIT"));let i=Inspection::from_documents(&lock(&bytes),&policy(if permitted{"'MIT'"}else{""}),date()).unwrap();
      let mut a=observations(&i,&bytes);if missing{a.pop();}let mut b=observations(&i,&bytes);if missing{b.pop();}
      if reverse{b.reverse();}
      let a=i.evaluate(a);let b=i.evaluate(b);proptest::prop_assert_eq!(serde_json::to_value(&a).unwrap(),serde_json::to_value(&b).unwrap());
      proptest::prop_assert_eq!(a.exit_code(false),if missing{2}else{u8::from(!permitted)});
    }
}

#[test]
fn failures_without_a_proven_origin_explain_the_missing_path() {
    let bytes = wheel(&metadata("License-Expression: MIT"));
    let i = Inspection::from_documents(&lock(&bytes), &policy(""), date()).unwrap();
    let json = serde_json::to_value(i.evaluate(observations(&i, &bytes))).unwrap();
    assert_eq!(
        json["packages"][0]["origin_note"],
        "No unambiguous dependency origin is recorded in the lock."
    );
}
