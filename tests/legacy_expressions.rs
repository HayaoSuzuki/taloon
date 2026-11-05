mod common;

use proptest::prelude::*;
use rstest::rstest;
use taloon::{evidence::Evidence, model::ArtifactFormat};

fn observed(value: &str) -> Evidence {
    let bytes = common::wheel(&common::metadata(&format!("License: {value}")));
    Evidence::from_archive(&common::request(&bytes, ArtifactFormat::Wheel), &bytes).unwrap()
}

#[rstest]
#[case::or("The MIT License OR Apache 2.0", "MIT OR Apache-2.0")]
#[case::mpl_or("MPL 2.0 OR MIT", "MPL-2.0 OR MIT")]
#[case::and("The MIT License AND Apache 2.0", "MIT AND Apache-2.0")]
#[case::precedence("The MIT License OR ISC AND Apache 2.0", "MIT OR ISC AND Apache-2.0")]
#[case::group(
    "(The MIT License OR ISC) AND Apache 2.0",
    "(MIT OR ISC) AND Apache-2.0"
)]
#[case::alias_parentheses("The MIT License (MIT) OR Apache License (v2.0)", "MIT OR Apache-2.0")]
#[case::name_case("the mit license OR apache 2.0", "MIT OR Apache-2.0")]
#[case::operator_word_in_name(
    "GNU General Public License (GPL) version 2, or any later version OR The MIT License",
    "GPL-2.0-or-later OR MIT"
)]
#[case::versioned("ZPL 2.1 OR Mozilla Public License v2.0", "ZPL-2.1 OR MPL-2.0")]
#[case::single_group("(The MIT License)", "(MIT)")]
#[case::with(
    "Apache 2.0 WITH LLVM-exception OR The MIT License",
    "Apache-2.0 WITH LLVM-exception OR MIT"
)]
#[case::strict_unit(
    "MIT WITH LLVM-exception OR Apache 2.0",
    "MIT WITH LLVM-exception OR Apache-2.0"
)]
fn resolves_complete_explicit_legacy_expressions(#[case] value: &str, #[case] expected: &str) {
    // Arrange
    let evidence = observed(value);
    // Act
    let actual = evidence.license().unwrap();
    // Assert
    assert_eq!(actual.text(), expected);
    assert_eq!(evidence.summary().original_license, vec![value.to_owned()]);
}

#[rstest]
#[case::unknown_leaf("The MIT License OR Unreviewed Terms")]
#[case::extra_terms("The MIT License OR Apache 2.0, noncommercial use only")]
#[case::missing_operator("The MIT License Apache 2.0")]
#[case::missing_operand("The MIT License OR")]
#[case::adjacent_operators("The MIT License AND OR Apache 2.0")]
#[case::lowercase_operator("The MIT License or Apache 2.0")]
#[case::operator_inside_word("The MIT License ORApache 2.0")]
#[case::empty_group("The MIT License OR ()")]
#[case::unclosed_group("(The MIT License OR Apache 2.0")]
#[case::trailing_group("The MIT License (Apache 2.0)")]
#[case::group_with("(The MIT License OR Apache 2.0) WITH LLVM-exception")]
#[case::exception_alias("The MIT License WITH LLVM exception")]
#[case::missing_exception("The MIT License WITH")]
#[case::external_ref("The MIT License OR DocumentRef-Other:LicenseRef-Test")]
#[case::style_license("The MIT License OR BSD-style")]
fn rejects_unconsumed_or_ambiguous_expression_material(#[case] value: &str) {
    // Arrange
    let evidence = observed(value);
    // Act
    let result = evidence.license();
    // Assert
    assert!(result.is_err());
}

#[rstest]
#[case::strict_expression_field(
    "License-Expression: The MIT License OR Apache 2.0\nLicense: MIT",
    None
)]
#[case::authoritative_expression(
    "License-Expression: ISC\nLicense: The MIT License OR Apache 2.0",
    Some("ISC")
)]
#[case::repeated_legacy("License: The MIT License OR Apache 2.0\nLicense: MIT", None)]
#[case::invalid_authoritative(
    "License-Expression: INVALID\nLicense: The MIT License OR Apache 2.0",
    None
)]
fn compound_fallback_preserves_field_rules(#[case] fields: &str, #[case] expected: Option<&str>) {
    // Arrange
    let bytes = common::wheel(&common::metadata(fields));
    let evidence =
        Evidence::from_archive(&common::request(&bytes, ArtifactFormat::Wheel), &bytes).unwrap();
    // Act
    let actual = evidence.license();
    // Assert
    assert_eq!(
        actual
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected
    );
}

#[rstest]
#[case::maximum(32, true)]
#[case::excess(33, false)]
fn bounds_legacy_structural_group_depth(#[case] count: usize, #[case] accepted: bool) {
    // Arrange
    let value = format!("{}The MIT License{}", "(".repeat(count), ")".repeat(count));
    let evidence = observed(&value);
    // Act
    let result = evidence.license();
    // Assert
    assert_eq!(result.is_ok(), accepted);
}

#[rstest]
#[case::maximum(64, true)]
#[case::excess(65, false)]
fn bounds_legacy_atomic_leaf_count(#[case] count: usize, #[case] accepted: bool) {
    // Arrange
    let value = vec!["The MIT License"; count].join(" OR ");
    let evidence = observed(&value);
    // Act
    let result = evidence.license();
    // Assert
    assert_eq!(result.is_ok(), accepted);
}

#[rstest]
#[case::maximum(16384, true)]
#[case::excess(16385, false)]
fn bounds_additional_legacy_expression_bytes(#[case] bytes: usize, #[case] accepted: bool) {
    // Arrange
    let prefix = "The MIT License OR LicenseRef-";
    let evidence = observed(&format!("{prefix}{}", "a".repeat(bytes - prefix.len())));
    // Act
    let result = evidence.license();
    // Assert
    assert_eq!(result.is_ok(), accepted);
}

#[derive(Clone, Debug)]
enum Formula {
    Leaf(usize),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}
impl Formula {
    fn legacy(&self) -> String {
        match self {
            Self::Leaf(i) => ["The MIT License", "ISC License", "Apache 2.0"][*i].to_owned(),
            Self::And(a, b) => format!("({} AND {})", a.legacy(), b.legacy()),
            Self::Or(a, b) => format!("({} OR {})", a.legacy(), b.legacy()),
        }
    }
    fn evaluate(&self, allowed: [bool; 3]) -> bool {
        match self {
            Self::Leaf(i) => allowed[*i],
            Self::And(a, b) => a.evaluate(allowed) && b.evaluate(allowed),
            Self::Or(a, b) => a.evaluate(allowed) || b.evaluate(allowed),
        }
    }
}
fn formulas() -> impl Strategy<Value = Formula> {
    (0_usize..3)
        .prop_map(Formula::Leaf)
        .prop_recursive(5, 32, 2, |inner| {
            prop_oneof![
                (inner.clone(), inner.clone())
                    .prop_map(|(a, b)| Formula::And(Box::new(a), Box::new(b))),
                (inner.clone(), inner).prop_map(|(a, b)| Formula::Or(Box::new(a), Box::new(b))),
            ]
        })
}
fn policy_result(value: &str, setting: &str, configured: [bool; 3]) -> u8 {
    let bytes = common::wheel(&common::metadata(&format!("License: {value}")));
    let hash = taloon::model::ContentHash::digest(&bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://pypi.org/simple'}}\nwheels=[{{url='https://files.example.test/example.whl',hash='{hash}'}}]"
    );
    let ids = ["MIT", "ISC", "Apache-2.0"]
        .into_iter()
        .zip(configured)
        .filter(|(_, selected)| *selected)
        .map(|(id, _)| format!("'{id}'"))
        .collect::<Vec<_>>()
        .join(",");
    let inspection = taloon::inspection::Inspection::from_documents(
        &lock,
        &format!("[tool.taloon]\nschema-version=1\n{setting}=[{ids}]"),
        taloon::model::EvaluationDate::parse("2026-11-04").unwrap(),
    )
    .unwrap();
    let r = &inspection.requests()[0];
    inspection
        .evaluate(vec![taloon::inspection::AcquisitionResult::observed(
            r.id(),
            Evidence::from_archive(r, &bytes).unwrap(),
        )])
        .exit_code(false)
}

#[rstest]
#[case::or_precedes_whole_conjunction("The MIT License OR ISC AND Apache 2.0", [true, false, false], 0)]
#[case::and_requires_both("The MIT License OR ISC AND Apache 2.0", [false, true, false], 1)]
#[case::explicit_group_changes_order("(The MIT License OR ISC) AND Apache 2.0", [true, false, false], 1)]
fn explicit_precedence_controls_permission(
    #[case] value: &str,
    #[case] allowed: [bool; 3],
    #[case] expected: u8,
) {
    // Arrange
    let setting = "allow";
    // Act
    let actual = policy_result(value, setting, allowed);
    // Assert
    assert_eq!(actual, expected);
}

proptest! {
    #[test]
    fn alias_expressions_preserve_allow_semantics(formula in formulas(), allowed in any::<[bool;3]>()) {
        // Arrange
        let value = formula.legacy();
        let expected = u8::from(!formula.evaluate(allowed));
        // Act
        let actual = policy_result(&value, "allow", allowed);
        // Assert
        prop_assert_eq!(actual, expected);
    }
    #[test]
    fn alias_expressions_preserve_deny_semantics(formula in formulas(), denied in any::<[bool;3]>()) {
        // Arrange
        let value = formula.legacy();
        let expected = u8::from(!formula.evaluate(denied.map(|b| !b)));
        // Act
        let actual = policy_result(&value, "deny", denied);
        // Assert
        prop_assert_eq!(actual, expected);
    }
}
