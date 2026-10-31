use proptest::prelude::*;
use taloon::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, ContentHash, EvaluationDate, PackageId,
    SourceId,
};
use taloon::policy::{LicenseExpression, Policy};

fn date() -> EvaluationDate {
    EvaluationDate::parse("2026-10-30").unwrap()
}
fn request(version: &str, source: &str) -> ArtifactRequest {
    ArtifactRequest::new(
        PackageId::parse(
            "Example_Pkg",
            version,
            SourceId::parse("registry", source).unwrap(),
        )
        .unwrap(),
        ArtifactLocator::registry(source, "https://files.example.test/pkg.whl").unwrap(),
        ArtifactFormat::Wheel,
        Some(ContentHash::digest(b"example")),
    )
    .unwrap()
}
fn policy(allow: &str) -> Policy {
    Policy::from_pyproject(
        &format!("[tool.taloon]\nschema-version=1\nallow=[{allow}]"),
        date(),
    )
    .unwrap()
}

#[test]
fn nested_expression_respects_explicit_units() {
    let p = policy("'MIT'");
    let req = request("1.2.3", "https://pypi.org/simple");
    for (expression, expected) in [
        ("MIT OR Apache-2.0", true),
        ("MIT AND Apache-2.0", false),
        ("MIT WITH LLVM-exception", false),
        ("MIT AND (Apache-2.0 OR MIT)", true),
    ] {
        assert_eq!(
            p.evaluate(
                &req,
                ContentHash::digest(b"example"),
                &LicenseExpression::parse(expression).unwrap()
            )
            .is_permitted(),
            expected,
            "{expression}"
        );
    }
    let gpl = policy("'GPL-2.0-only'");
    assert!(
        !gpl.evaluate(
            &req,
            ContentHash::digest(b"example"),
            &LicenseExpression::parse("GPL-2.0-or-later").unwrap()
        )
        .is_permitted()
    );
}

#[test]
fn policy_requires_exact_scoped_overrides() {
    let p = Policy::from_pyproject("[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='example-pkg'\nversion='1.2.3'\nsource={kind='registry',location='https://pypi.org/simple'}\nallow=['MIT']\nreason='reviewed'",date()).unwrap();
    let expr = LicenseExpression::parse("MIT").unwrap();
    for (version, source, expected) in [
        ("1.2.3", "https://pypi.org/simple", true),
        ("1.2.4", "https://pypi.org/simple", false),
        ("1.2.3+local", "https://pypi.org/simple", false),
        ("1.2.3", "https://other.test/simple", false),
    ] {
        assert_eq!(
            p.evaluate(
                &request(version, source),
                ContentHash::digest(b"example"),
                &expr
            )
            .is_permitted(),
            expected
        );
    }
}

#[test]
fn policy_rejects_unknown_or_unscoped_permissions() {
    for text in [
        "",
        "[tool.taloon]\nallow=[]",
        "[tool.taloon]\nschema-version=1",
        "[tool.taloon]\nschema-version=1\nallow=['MIT OR Apache-2.0']",
        "[tool.taloon]\nschema-version=1\nallow=['LicenseRef-Vendor']",
        "[tool.taloon]\nschema-version=1\nallow=[]\nalllow=['MIT']",
    ] {
        assert!(Policy::from_pyproject(text, date()).is_err(), "{text}");
    }
}

#[test]
fn expiry_is_inclusive_and_applies_to_unmatched_entries() {
    for (expiry, valid) in [("2026-10-30", true), ("2026-10-29", false)] {
        let text = format!(
            "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='not-used'\nversion='1.0'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nallow=['MIT']\nreason='reviewed'\nexpires='{expiry}'"
        );
        assert_eq!(Policy::from_pyproject(&text, date()).is_ok(), valid);
    }
}

#[test]
fn source_display_redacts_userinfo_and_all_query_values() {
    let s = SourceId::parse(
        "url",
        "https://user:password@example.test/a%20b.whl?token=secret&download=1",
    )
    .unwrap();
    let visible = s.to_string();
    assert!(
        !visible.contains("password") && !visible.contains("secret") && !visible.contains("user:")
    );
    assert!(visible.contains("a%20b.whl"));
}

#[test]
fn malformed_locator_cannot_leak_credentials_in_diagnostics() {
    let visible = taloon::model::safe_locator("https://alice:password@/bad?token=secret");
    assert!(!visible.contains("password") && !visible.contains("secret"));
}

#[test]
fn request_constructor_rejects_unvalidated_locator_variants() {
    let package = PackageId::parse(
        "example",
        "1.0",
        SourceId::parse("registry", "https://pypi.org/simple").unwrap(),
    )
    .unwrap();
    let locator = ArtifactLocator::Remote {
        index: None,
        url: "not-a-url".into(),
    };
    assert!(
        ArtifactRequest::new(
            package,
            locator,
            ArtifactFormat::Wheel,
            Some(ContentHash::digest(b"test"))
        )
        .is_err()
    );
}

#[test]
fn custom_permissions_require_exact_content_and_strong_evidence() {
    let hash = ContentHash::digest(b"example");
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='example-pkg'\nversion='1.2.3'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nartifact-hash='{hash}'\nallow=['LicenseRef-Vendor']\nreason='reviewed'"
    );
    let p = Policy::from_pyproject(&text, date()).unwrap();
    let req = request("1.2.3", "https://pypi.org/simple");
    assert_eq!(
        p.minimum_evidence(&req),
        taloon::model::EvidenceRequirement::ArchiveVerified
    );
    let expr = LicenseExpression::parse("LicenseRef-Vendor").unwrap();
    assert!(p.evaluate(&req, hash, &expr).is_permitted());
    assert!(
        !p.evaluate(&req, ContentHash::digest(b"changed"), &expr)
            .is_permitted()
    );
    assert!(
        Policy::from_pyproject(
            &text.replace(&format!("artifact-hash='{hash}'\n"), ""),
            date()
        )
        .is_err()
    );
}

fn expressions() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "MIT".to_owned(),
        "Apache-2.0".to_owned(),
        "BSD-3-Clause".to_owned(),
    ])
    .prop_recursive(3, 24, 2, |inner| {
        (inner.clone(), inner, any::<bool>()).prop_map(|(left, right, and)| {
            format!("({left} {} {right})", if and { "AND" } else { "OR" })
        })
    })
}

proptest::proptest! {
    #![proptest_config(ProptestConfig {cases:256,failure_persistence:Some(Box::new(proptest::test_runner::FileFailurePersistence::Direct("tests/policy.proptest-regressions"))),..ProptestConfig::default()})]
    #[test]
    fn adding_an_explicit_permission_is_monotone(expression in expressions(), mask in 0_u8..8) {
        let req=request("1.2.3","https://pypi.org/simple");
        let expr=LicenseExpression::parse(&expression).unwrap();
        let units=["MIT","Apache-2.0","BSD-3-Clause"];
        let allowed=units.iter().enumerate().filter(|(i,_)|mask&(1<<i)!=0).map(|(_,s)|format!("'{s}'")).collect::<Vec<_>>().join(",");
        let small=policy(&allowed);
        let large=policy("'MIT','Apache-2.0','BSD-3-Clause'");
        let hash=ContentHash::digest(b"example");
        proptest::prop_assert!(!small.evaluate(&req,hash,&expr).is_permitted() || large.evaluate(&req,hash,&expr).is_permitted());
    }
    #[test]
    fn package_name_normalization_is_idempotent(name in "[A-Za-z0-9][A-Za-z0-9_.-]{0,32}[A-Za-z0-9]") {
        let source=SourceId::parse("registry","https://pypi.org/simple").unwrap();
        let once=PackageId::parse(&name,"1.0",source.clone()).unwrap();
        let twice=PackageId::parse(once.name(),"1.0",source).unwrap();
        prop_assert_eq!(once,twice);
    }
}
