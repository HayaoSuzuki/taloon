use proptest::prelude::*;
use rstest::rstest;
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

#[rstest]
#[case::or_accepts_allowed_alternative("MIT OR Apache-2.0", true)]
#[case::and_requires_both_units("MIT AND Apache-2.0", false)]
#[case::with_is_a_distinct_unit("MIT WITH LLVM-exception", false)]
#[case::nested_alternative("MIT AND (Apache-2.0 OR MIT)", true)]
fn nested_expression_respects_explicit_units(#[case] expression: &str, #[case] expected: bool) {
    // Arrange
    let policy = policy("'MIT'");
    let request = request("1.2.3", "https://pypi.org/simple");
    let expression = LicenseExpression::parse(expression).unwrap();

    // Act
    let observed = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);

    // Assert
    assert_eq!(observed.is_permitted(), expected);
}

#[test]
fn gpl_only_permission_does_not_grant_or_later() {
    // Arrange
    let policy = policy("'GPL-2.0-only'");
    let request = request("1.2.3", "https://pypi.org/simple");
    let expression = LicenseExpression::parse("GPL-2.0-or-later").unwrap();

    // Act
    let observed = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);

    // Assert
    assert!(!observed.is_permitted());
}

#[rstest]
#[case::exact("1.2.3", "https://pypi.org/simple", true)]
#[case::other_version("1.2.4", "https://pypi.org/simple", false)]
#[case::local_version("1.2.3+local", "https://pypi.org/simple", false)]
#[case::other_source("1.2.3", "https://other.test/simple", false)]
fn policy_requires_exact_scoped_overrides(
    #[case] version: &str,
    #[case] source: &str,
    #[case] expected: bool,
) {
    // Arrange
    let policy = Policy::from_pyproject("[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='example-pkg'\nversion='1.2.3'\nsource={kind='registry',location='https://pypi.org/simple'}\nallow=['MIT']\nreason='reviewed'",date()).unwrap();
    let expression = LicenseExpression::parse("MIT").unwrap();
    let request = request(version, source);

    // Act
    let observed = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);

    // Assert
    assert_eq!(observed.is_permitted(), expected);
}

#[rstest]
#[case::missing_policy("")]
#[case::missing_schema("[tool.taloon]\nallow=[]")]
#[case::missing_allow("[tool.taloon]\nschema-version=1")]
#[case::compound_permission("[tool.taloon]\nschema-version=1\nallow=['MIT OR Apache-2.0']")]
#[case::unscoped_reference("[tool.taloon]\nschema-version=1\nallow=['LicenseRef-Vendor']")]
#[case::unknown_field("[tool.taloon]\nschema-version=1\nallow=[]\nalllow=['MIT']")]
fn policy_rejects_unknown_or_unscoped_permissions(#[case] text: &str) {
    // Arrange
    let today = date();

    // Act
    let observed = Policy::from_pyproject(text, today);

    // Assert
    assert!(observed.is_err(), "{text}");
}

#[rstest]
#[case::on_expiry("2026-10-30", true)]
#[case::after_expiry("2026-10-29", false)]
fn expiry_is_inclusive_and_applies_to_unmatched_entries(#[case] expiry: &str, #[case] valid: bool) {
    // Arrange
    let text = format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='not-used'\nversion='1.0'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nallow=['MIT']\nreason='reviewed'\nexpires='{expiry}'"
    );

    // Act
    let observed = Policy::from_pyproject(&text, date());

    // Assert
    assert_eq!(observed.is_ok(), valid);
}

#[test]
fn source_display_redacts_userinfo_and_all_query_values() {
    // Arrange
    let s = SourceId::parse(
        "url",
        "https://user:password@example.test/a%20b.whl?token=secret&download=1",
    )
    .unwrap();
    // Act
    let visible = s.to_string();

    // Assert
    assert!(
        !visible.contains("password") && !visible.contains("secret") && !visible.contains("user:")
    );
    assert!(visible.contains("a%20b.whl"));
}

#[test]
fn malformed_locator_cannot_leak_credentials_in_diagnostics() {
    // Arrange
    let locator = "https://alice:password@/bad?token=secret";

    // Act
    let visible = taloon::model::safe_locator(locator);

    // Assert
    assert!(!visible.contains("password") && !visible.contains("secret"));
}

#[test]
fn request_constructor_rejects_unvalidated_locator_variants() {
    // Arrange
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
    // Act
    let observed = ArtifactRequest::new(
        package,
        locator,
        ArtifactFormat::Wheel,
        Some(ContentHash::digest(b"test")),
    );

    // Assert
    assert!(observed.is_err());
}

fn custom_permission_config() -> String {
    let hash = ContentHash::digest(b"example");
    format!(
        "[tool.taloon]\nschema-version=1\nallow=[]\n[[tool.taloon.exceptions]]\nname='example-pkg'\nversion='1.2.3'\nsource={{kind='registry',location='https://pypi.org/simple'}}\nartifact-hash='{hash}'\nallow=['LicenseRef-Vendor']\nreason='reviewed'"
    )
}

#[test]
fn custom_permission_requires_archive_verified_evidence() {
    // Arrange
    let policy = Policy::from_pyproject(&custom_permission_config(), date()).unwrap();
    let request = request("1.2.3", "https://pypi.org/simple");

    // Act
    let observed = policy.minimum_evidence(&request);

    // Assert
    assert_eq!(
        observed,
        taloon::model::EvidenceRequirement::ArchiveVerified
    );
}

#[rstest]
#[case::matching_content(b"example", true)]
#[case::changed_content(b"changed", false)]
fn custom_permission_is_bound_to_content(#[case] content: &[u8], #[case] expected: bool) {
    // Arrange
    let policy = Policy::from_pyproject(&custom_permission_config(), date()).unwrap();
    let request = request("1.2.3", "https://pypi.org/simple");
    let expression = LicenseExpression::parse("LicenseRef-Vendor").unwrap();

    // Act
    let observed = policy.evaluate(&request, ContentHash::digest(content), &expression);

    // Assert
    assert_eq!(observed.is_permitted(), expected);
}

#[test]
fn custom_permission_requires_a_content_hash() {
    // Arrange
    let hash = ContentHash::digest(b"example");
    let text = custom_permission_config().replace(&format!("artifact-hash='{hash}'\n"), "");

    // Act
    let observed = Policy::from_pyproject(&text, date());

    // Assert
    assert!(observed.is_err());
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
    fn generated_binary_permissions_follow_boolean_algebra(
        left in 0_usize..3,
        right in 0_usize..3,
        conjunction in any::<bool>(),
        mask in 0_u8..8,
    ) {
        // Arrange: numeric Boolean expectations are independent of SPDX parsing.
        let units = ["MIT", "Apache-2.0", "BSD-3-Clause"];
        let left_allowed = mask & (1_u8 << left) != 0;
        let right_allowed = mask & (1_u8 << right) != 0;
        let expected = if conjunction { left_allowed && right_allowed } else { left_allowed || right_allowed };
        let allowed = units.iter().enumerate().filter(|(i,_)| mask & (1 << i) != 0).map(|(_,unit)| format!("'{unit}'")).collect::<Vec<_>>().join(",");
        let policy = policy(&allowed);
        let request = request("1.2.3", "https://pypi.org/simple");
        let operator = if conjunction { "AND" } else { "OR" };
        let expression = LicenseExpression::parse(&format!("{} {operator} {}", units[left], units[right])).unwrap();

        // Act
        let observed = policy.evaluate(&request, ContentHash::digest(b"example"), &expression).is_permitted();

        // Assert
        prop_assert_eq!(observed, expected);
    }
    #[test]
    fn adding_an_explicit_permission_is_monotone(expression in expressions(), mask in 0_u8..8) {
        // Arrange
        let req=request("1.2.3","https://pypi.org/simple");
        let expr=LicenseExpression::parse(&expression).unwrap();
        let units=["MIT","Apache-2.0","BSD-3-Clause"];
        let allowed=units.iter().enumerate().filter(|(i,_)|mask&(1<<i)!=0).map(|(_,s)|format!("'{s}'")).collect::<Vec<_>>().join(",");
        let small=policy(&allowed);
        let large=policy("'MIT','Apache-2.0','BSD-3-Clause'");
        let hash=ContentHash::digest(b"example");
        // Act
        let small_permitted = small.evaluate(&req,hash,&expr).is_permitted();
        let large_permitted = large.evaluate(&req,hash,&expr).is_permitted();
        // Assert
        proptest::prop_assert!(!small_permitted || large_permitted);
    }
    #[test]
    fn package_name_normalization_is_idempotent(name in "[A-Za-z0-9][A-Za-z0-9_.-]{0,32}[A-Za-z0-9]") {
        // Arrange
        let source=SourceId::parse("registry","https://pypi.org/simple").unwrap();
        let once=PackageId::parse(&name,"1.0",source.clone()).unwrap();
        // Act
        let twice=PackageId::parse(once.name(),"1.0",source).unwrap();
        // Assert
        prop_assert_eq!(once,twice);
    }
}

#[rstest]
#[case::empty_allow("allow=[]", true)]
#[case::empty_deny("deny=[]", true)]
#[case::missing_mode("", false)]
#[case::both_empty("allow=[]\ndeny=[]", false)]
#[case::both_populated("allow=['MIT']\ndeny=['Apache-2.0']", false)]
#[case::empty_allow_and_deny("allow=[]\ndeny=['MIT']", false)]
#[case::allow_and_empty_deny("allow=['MIT']\ndeny=[]", false)]
#[case::compound_deny("deny=['MIT OR Apache-2.0']", false)]
#[case::unknown_deny("deny=['Not-A-License']", false)]
#[case::custom_denial("deny=['LicenseRef-Vendor']", true)]
fn configuration_requires_one_valid_policy_mode(#[case] settings: &str, #[case] valid: bool) {
    // Arrange
    let text = format!("[tool.taloon]\nschema-version=1\n{settings}");
    // Act
    let result = Policy::from_pyproject(&text, date());
    // Assert
    assert_eq!(result.is_ok(), valid, "{settings}");
}

#[rstest]
#[case::listed("'MIT'", "MIT", false)]
#[case::unlisted("'MIT'", "Apache-2.0", true)]
#[case::or_alternative("'MIT'", "MIT OR Apache-2.0", true)]
#[case::and_requires_every_atom("'MIT'", "MIT AND Apache-2.0", false)]
#[case::nested("'MIT'", "BSD-3-Clause AND (MIT OR Apache-2.0)", true)]
#[case::with_is_distinct("'MIT'", "MIT WITH LLVM-exception", true)]
#[case::with_denied("'MIT WITH LLVM-exception'", "MIT WITH LLVM-exception", false)]
#[case::bare_is_distinct("'MIT WITH LLVM-exception'", "MIT", true)]
#[case::gnu_distinction("'GPL-2.0-only'", "GPL-2.0-or-later", true)]
#[case::empty_standard("", "MIT", true)]
#[case::empty_custom("", "LicenseRef-Vendor", false)]
#[case::empty_custom_with("", "LicenseRef-Vendor WITH LLVM-exception", false)]
fn deny_evaluates_exact_atomic_units(
    #[case] denied: &str,
    #[case] expression: &str,
    #[case] expected: bool,
) {
    // Arrange
    let policy = Policy::from_pyproject(
        &format!("[tool.taloon]\nschema-version=1\ndeny=[{denied}]"),
        date(),
    )
    .unwrap();
    let request = request("1.2.3", "https://pypi.org/simple");
    let expression = LicenseExpression::parse(expression).unwrap();
    // Act
    let decision = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);
    // Assert
    assert_eq!(decision.is_permitted(), expected);
}

#[rstest]
#[case::custom_explicit_permission("", "LicenseRef-Vendor", b"example", true)]
#[case::custom_with_explicit_permission(
    "",
    "LicenseRef-Vendor WITH LLVM-exception",
    b"example",
    true
)]
#[case::custom_with_denied(
    "'LicenseRef-Vendor WITH LLVM-exception'",
    "LicenseRef-Vendor WITH LLVM-exception",
    b"example",
    false
)]
#[case::custom_denied_despite_exception(
    "'LicenseRef-Vendor'",
    "LicenseRef-Vendor",
    b"example",
    false
)]
#[case::custom_wrong_content("", "LicenseRef-Vendor", b"changed", false)]
#[case::standard_denied_despite_exception("'MIT'", "MIT", b"example", false)]
fn deny_precedes_matching_exceptions(
    #[case] denied: &str,
    #[case] unit: &str,
    #[case] content: &[u8],
    #[case] expected: bool,
) {
    // Arrange
    let text = custom_permission_config()
        .replacen("allow=[]", &format!("deny=[{denied}]"), 1)
        .replace("allow=['LicenseRef-Vendor']", &format!("allow=['{unit}']"));
    let policy = Policy::from_pyproject(&text, date()).unwrap();
    let request = request("1.2.3", "https://pypi.org/simple");
    let expression = LicenseExpression::parse(unit).unwrap();
    // Act
    let decision = policy.evaluate(&request, ContentHash::digest(content), &expression);
    // Assert
    assert_eq!(decision.is_permitted(), expected);
}

#[test]
fn deny_custom_exception_still_requires_archive_proof() {
    // Arrange
    let text = custom_permission_config().replacen("allow=[]", "deny=[]", 1);
    let policy = Policy::from_pyproject(&text, date()).unwrap();
    let request = request("1.2.3", "https://pypi.org/simple");
    // Act
    let minimum = policy.minimum_evidence(&request);
    // Assert
    assert_eq!(minimum, taloon::model::EvidenceRequirement::ArchiveVerified);
}

proptest! {
    #[test]
    fn deny_boolean_decisions_match_independent_truth_values(left in 0_usize..3, right in 0_usize..3, conjunction in any::<bool>(), mask in 0_u8..8) {
        // Arrange: the integer mask marks prohibited atoms, not whole expressions.
        let units = ["MIT", "Apache-2.0", "BSD-3-Clause"];
        let left_permitted = mask & (1 << left) == 0;
        let right_permitted = mask & (1 << right) == 0;
        let expected = if conjunction { left_permitted && right_permitted } else { left_permitted || right_permitted };
        let denied = units.iter().enumerate().filter(|(i,_)| mask & (1 << i) != 0).map(|(_,u)| format!("'{u}'")).collect::<Vec<_>>().join(",");
        let text = format!("[tool.taloon]\nschema-version=1\ndeny=[{denied}]");
        let policy = Policy::from_pyproject(&text, date()).unwrap();
        let request = request("1.2.3", "https://pypi.org/simple");
        let operator = if conjunction { "AND" } else { "OR" };
        let expression = LicenseExpression::parse(&format!("{} {operator} {}", units[left], units[right])).unwrap();
        // Act
        let decision = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);
        // Assert
        prop_assert_eq!(decision.is_permitted(), expected);
    }
}

proptest! {
    #[test]
    fn deny_nested_expression_matches_independent_boolean_oracle(
        left in 0_usize..3, middle in 0_usize..3, right in 0_usize..3,
        inner_and in any::<bool>(), outer_and in any::<bool>(), mask in 0_u8..8,
    ) {
        // Arrange: numeric leaf permissions and native Boolean operators supply the oracle.
        let units = ["MIT", "Apache-2.0", "BSD-3-Clause"];
        let a = mask & (1 << left) == 0;
        let b = mask & (1 << middle) == 0;
        let c = mask & (1 << right) == 0;
        let inner = if inner_and { a && b } else { a || b };
        let expected = if outer_and { inner && c } else { inner || c };
        let denied = units.iter().enumerate().filter(|(i,_)| mask & (1 << i) != 0).map(|(_,u)| format!("'{u}'")).collect::<Vec<_>>().join(",");
        let text = format!("[tool.taloon]\nschema-version=1\ndeny=[{denied}]");
        let policy = Policy::from_pyproject(&text, date()).unwrap();
        let request = request("1.2.3", "https://pypi.org/simple");
        let inner_operator = if inner_and { "AND" } else { "OR" };
        let outer_operator = if outer_and { "AND" } else { "OR" };
        let expression = LicenseExpression::parse(&format!("({} {inner_operator} {}) {outer_operator} {}", units[left], units[middle], units[right])).unwrap();
        // Act
        let decision = policy.evaluate(&request, ContentHash::digest(b"example"), &expression);
        // Assert
        prop_assert_eq!(decision.is_permitted(), expected);
    }
}
