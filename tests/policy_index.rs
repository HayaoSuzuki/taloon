//! Finite integer selectors give independent expectations for typed policy lookup.
mod common;
use proptest::prelude::*;
use rstest::rstest;
use std::fmt::Write as _;
use taloon::{
    evidence::Evidence,
    inspection::{AcquisitionResult, Inspection},
    model::{
        ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvaluationDate,
        EvidenceRequirement, PackageId, SourceId,
    },
    policy::{LicenseExpression, Policy},
};
const IDENTITIES: [(&str, &str, &str); 4] = [
    ("example", "1.0", "one"),
    ("example", "2.0", "one"),
    ("example", "1.0", "two"),
    ("unrelated", "1.0", "one"),
];
type Rule = (usize, u8, bool);
fn date() -> EvaluationDate {
    EvaluationDate::parse("2026-11-01").unwrap()
}
fn config(rules: &[Rule], globals: &str, first: ContentHash, other: ContentHash) -> String {
    let mut text = format!("[tool.taloon]\nschema-version=1\nallow=[{globals}]\n");
    for (position, (target, scope, apache)) in rules.iter().enumerate() {
        let (name, version, host) = IDENTITIES[*target];
        let unit = if *apache { "Apache-2.0" } else { "MIT" };
        writeln!(text,"[[tool.taloon.exceptions]]\nname='{name}'\nversion='{version}'\nsource={{kind='registry',location='https://{host}.test/simple'}}\nallow=['{unit}']\nreason='entry-{position}'").unwrap();
        if *scope != 0 {
            writeln!(
                text,
                "artifact-hash='{}'",
                if *scope == 1 { first } else { other }
            )
            .unwrap();
        }
    }
    text
}
fn request(target: usize, hash: ContentHash) -> ArtifactRequest {
    let (name, version, host) = IDENTITIES[target];
    let source = SourceId::parse("registry", &format!("https://{host}.test/simple")).unwrap();
    ArtifactRequest::new(
        PackageId::parse(name, version, source.clone()).unwrap(),
        ArtifactLocator::registry(source.location(), "https://files.example.test/a.whl").unwrap(),
        ArtifactFormat::Wheel,
        Some(hash),
    )
    .unwrap()
}
#[rstest]
#[case::first_identity(0,vec!["exception[2]","exception[4]"])]
#[case::other_version(1,vec!["exception[3]"])]
#[case::other_source(2,vec!["exception[1]"])]
fn interleaved_exceptions_keep_original_ordinals_and_order(
    #[case] selected: usize,
    #[case] expected: Vec<&str>,
) {
    // Arrange
    let first = ContentHash::digest(b"first");
    let other = ContentHash::digest(b"other");
    let rules = [(2, 0, false), (0, 1, false), (1, 0, false), (0, 0, true)];
    let policy = Policy::from_pyproject(&config(&rules, "", first, other), date()).unwrap();
    let request = request(selected, first);
    let expression = LicenseExpression::parse("MIT").unwrap();
    // Act
    let decision = policy.evaluate(&request, first, &expression);
    // Assert
    assert_eq!(
        decision
            .overrides
            .iter()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}
#[rstest]
#[case::exact(0, true, EvidenceRequirement::ArchiveVerified)]
#[case::wrong_hash(0, false, EvidenceRequirement::IndexAssociated)]
#[case::wrong_version(1, true, EvidenceRequirement::IndexAssociated)]
#[case::wrong_source(2, true, EvidenceRequirement::IndexAssociated)]
fn custom_proof_requirement_uses_exact_package_and_content(
    #[case] selected: usize,
    #[case] exact_hash: bool,
    #[case] expected: EvidenceRequirement,
    #[values("LicenseRef-Reviewed", "LicenseRef-Reviewed WITH LLVM-exception")] unit: &str,
) {
    // Arrange
    let first = ContentHash::digest(b"first");
    let other = ContentHash::digest(b"other");
    let text = config(&[(0, 1, false)], "", first, other)
        .replace("allow=['MIT']", &format!("allow=['{unit}']"));
    let policy = Policy::from_pyproject(&text, date()).unwrap();
    let request = request(selected, if exact_hash { first } else { other });
    // Act
    let requirement = policy.minimum_evidence(&request);
    // Assert
    assert_eq!(requirement, expected);
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn additive_decisions_and_entry_ids_match_integer_selectors(
        rules in prop::collection::vec((0_usize..4,0_u8..3,any::<bool>()),0..24),
        selected in 0_usize..4, alternate_hash in any::<bool>(), globals in 0_u8..4, and in any::<bool>(),
    ) {
        // Arrange: integer identities/scopes are independent of PackageId and lookup representation.
        let first=ContentHash::digest(b"first");let other=ContentHash::digest(b"other");
        let hash=if alternate_hash {other}else{first};
        let global_text=match globals {0=>"",1=>"'MIT'",2=>"'Apache-2.0'",_=>"'MIT','Apache-2.0'"};
        let matching:Vec<_>=rules.iter().enumerate().filter(|(_, (target,scope,_))| *target==selected && (*scope==0 || *scope==if alternate_hash {2}else{1})).collect();
        let mit=globals&1!=0 || matching.iter().any(|(_,(_,_,apache))| !apache);
        let apache=globals&2!=0 || matching.iter().any(|(_,(_,_,apache))| *apache);
        let expected=if and {mit&&apache}else{mit||apache};
        let ids:Vec<_>=matching.iter().map(|(position,_)|format!("exception[{}]",position+1)).collect();
        let policy=Policy::from_pyproject(&config(&rules,global_text,first,other),date()).unwrap();
        let request=request(selected,hash);
        let expression=LicenseExpression::parse(if and {"MIT AND Apache-2.0"}else{"MIT OR Apache-2.0"}).unwrap();
        // Act
        let decision=policy.evaluate(&request,hash,&expression);
        // Assert
        prop_assert_eq!(decision.is_permitted(),expected);
        prop_assert_eq!(decision.overrides.iter().map(|e|e.id.clone()).collect::<Vec<_>>(),ids);
        prop_assert_eq!(decision.units.iter().map(|u| (u.unit.as_str(),u.permitted)).collect::<Vec<_>>(),vec![("Apache-2.0",apache),("MIT",mit)]);
    }
    #[test]
    fn unused_warnings_match_observed_identity_and_all_or_terms(
        rules in prop::collection::vec((0_usize..4,0_u8..3,any::<bool>()),0..24),
    ) {
        // Arrange: the one observed identity is 0 and its content is the first hash.
        let bytes=common::wheel(&common::metadata("License-Expression: MIT OR Apache-2.0"));
        let first=ContentHash::digest(&bytes);let other=ContentHash::digest(b"other");
        let text=config(&rules,"'MIT','Apache-2.0','ISC'",first,other);
        let lock=format!("version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}\nwheels=[{{url='https://files.example.test/a.whl',hash='{first}'}}]");
        let inspection=Inspection::from_documents(&lock,&text,date()).unwrap();
        let results=inspection.requests().iter().map(|r|AcquisitionResult::observed(r.id(),Evidence::from_archive(r,&bytes).unwrap())).collect();
        let mut expected=vec!["unused global permission: ISC".to_owned()];
        expected.extend(rules.iter().enumerate().filter(|(_, (target,scope,_))| *target!=0 || *scope==2).map(|(n,_)|format!("unused exception entry: {}",n+1)));
        expected.sort();
        // Act
        let report=inspection.evaluate(results);
        // Assert
        prop_assert_eq!(report.exit_code(false),0);
        prop_assert_eq!(report.diagnostics().iter().map(|d|d.error.message.clone()).collect::<Vec<_>>(),expected);
    }
}

#[test]
fn normalized_selectors_keep_the_original_entry_identity() {
    // Arrange
    let first = ContentHash::digest(b"first");
    let text = config(&[(0, 0, false)], "", first, first)
        .replace("name='example'", "name='EXAMPLE'")
        .replace("version='1.0'", "version='1.0.0'")
        .replace(
            "https://one.test/simple",
            "https://user:password@one.test/simple/#fragment",
        );
    let policy = Policy::from_pyproject(&text, date()).unwrap();
    let request = request(0, first);
    let expression = LicenseExpression::parse("MIT").unwrap();
    // Act
    let decision = policy.evaluate(&request, first, &expression);
    // Assert
    assert!(decision.is_permitted());
    assert_eq!(decision.overrides[0].id, "exception[1]");
}
#[rstest]
#[case::observed(true)]
#[case::unobserved(false)]
fn clarification_usage_is_content_bound(#[case] observed: bool) {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let content = ContentHash::digest(&bytes);
    let other = ContentHash::digest(b"other");
    let configured = if observed { content } else { other };
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=['MIT']\n[[tool.taloon.clarifications]]\nartifact-hash='{configured}'\nexpression='MIT'\nreason='reviewed'"
    );
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}\nwheels=[{{url='https://files.example.test/a.whl',hash='{content}'}}]"
    );
    let inspection = Inspection::from_documents(&lock, &text, date()).unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, &bytes).unwrap()))
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    let expected = if observed {
        vec![]
    } else {
        vec![format!("unused clarification: {configured}")]
    };
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|d| d.error.message.clone())
            .collect::<Vec<_>>(),
        expected
    );
}

#[rstest]
#[case::matching_content(true)]
#[case::other_content(false)]
fn hashless_local_usage_is_matched_against_observed_content(#[case] matching: bool) {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: LicenseRef-Reviewed"));
    let content = ContentHash::digest(&bytes);
    let configured = if matching {
        content
    } else {
        ContentHash::digest(b"other")
    };
    let lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='example.whl'}";
    let config = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='example'\nversion='1.0'\nsource={{kind='path',location='example.whl'}}\nartifact-hash='{configured}'\nallow=['LicenseRef-Reviewed']\nreason='reviewed'"
    );
    let inspection = Inspection::from_documents(lock, &config, date()).unwrap();
    let results = inspection
        .requests()
        .iter()
        .map(|r| AcquisitionResult::observed(r.id(), Evidence::from_archive(r, &bytes).unwrap()))
        .collect();
    // Act
    let report = inspection.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(false), u8::from(!matching));
    assert_eq!(report.diagnostics().len(), usize::from(!matching));
}
