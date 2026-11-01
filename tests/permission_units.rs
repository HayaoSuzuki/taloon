//! Atomic permission construction and canonical identity are application contracts.
use proptest::prelude::*;
use rstest::rstest;
use std::collections::BTreeSet;
use taloon::{
    model::{Code, EvaluationDate},
    policy::{LicenseExpression, PermissionUnit, Policy},
};

#[rstest]
#[case::license("MIT", "MIT", false)]
#[case::with("MIT WITH LLVM-exception", "MIT WITH LLVM-exception", false)]
#[case::deprecated("GPL-2.0", "GPL-2.0", false)]
#[case::custom("LicenseRef-Reviewed", "LicenseRef-Reviewed", true)]
#[case::custom_with(
    "LicenseRef-Reviewed WITH LLVM-exception",
    "LicenseRef-Reviewed WITH LLVM-exception",
    true
)]
fn atomic_units_have_canonical_spelling_and_classification(
    #[case] input: &str,
    #[case] expected: &str,
    #[case] custom: bool,
) {
    // Arrange: one atomic spelling, including supported deprecated identifiers.
    // Act
    let unit = PermissionUnit::parse(input).unwrap();
    // Assert
    assert_eq!(unit.as_str(), expected);
    assert_eq!(unit.to_string(), expected);
    assert_eq!(unit.is_custom(), custom);
}
#[rstest]
#[case::empty("")]
#[case::gnu_postfix_plus("GPL-2.0+")]
#[case::lgpl_postfix_plus("LGPL-3.0+")]
#[case::unknown("Definitely-Unknown-License")]
#[case::or("MIT OR Apache-2.0")]
#[case::and("MIT AND MIT")]
#[case::external("DocumentRef-remote:LicenseRef-Local")]
#[case::addition("MIT WITH AdditionRef-Local")]
fn non_atomic_or_unsupported_units_are_rejected(#[case] input: &str) {
    // Arrange: the input is outside the atomic Python permission contract.
    // Act
    let result = PermissionUnit::parse(input);
    // Assert
    assert!(result.is_err());
}
#[rstest]
#[case::same("MIT", "MIT", true)]
#[case::non_gnu_plus("MIT", "MIT+", false)]
#[case::gnu_only("GPL-2.0-only", "GPL-2.0-or-later", false)]
#[case::whitespace_alias("MIT WITH LLVM-exception", "MIT  WITH  LLVM-exception", true)]
#[case::deprecated_distinction("GPL-2.0", "GPL-2.0-only", false)]
#[case::different_exception("MIT", "MIT WITH LLVM-exception", false)]
#[case::different_custom("LicenseRef-One", "LicenseRef-Two", false)]
fn equality_and_order_follow_canonical_permission_identity(
    #[case] left: &str,
    #[case] right: &str,
    #[case] equal: bool,
) {
    // Arrange
    let left = PermissionUnit::parse(left).unwrap();
    let right = PermissionUnit::parse(right).unwrap();
    // Act
    let observed_equal = left == right;
    let observed_order = left.cmp(&right);
    // Assert
    assert_eq!(observed_equal, equal);
    assert_eq!(observed_order.is_eq(), equal);
    assert_eq!(observed_order, left.as_str().cmp(right.as_str()));
}
fn inputs() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(
            spdx::identifiers::LICENSES
                .iter()
                // The catalogue includes deprecated spellings rejected by our strict mode.
                .filter(|license| LicenseExpression::parse(license.name).is_ok())
                .map(|license| license.name.to_owned())
                .collect::<Vec<_>>()
        ),
        "LicenseRef-[A-Za-z0-9]{1,16}",
        prop::sample::select(vec![
            "MIT+".to_owned(),
            "GPL-2.0".to_owned(),
            "MIT WITH LLVM-exception".to_owned()
        ]),
    ]
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn typed_sets_and_borrowed_lookups_agree_with_canonical_string_sets(
        spellings in prop::collection::vec(inputs(),0..24),
    ) {
        // Arrange: expected membership/order uses ordinary canonical string values.
        let parsed: Vec<_> = spellings.iter().map(|value| PermissionUnit::parse(value).unwrap()).collect();
        let expected:BTreeSet<_> = parsed.iter().map(|unit| unit.as_str().to_owned()).collect();
        // Act
        let actual:BTreeSet<_> = parsed.into_iter().collect();
        // Assert
        prop_assert_eq!(actual.iter().map(|unit| unit.as_str().to_owned()).collect::<BTreeSet<_>>(),expected.clone());
        prop_assert_eq!(actual.iter().map(PermissionUnit::as_str).collect::<Vec<_>>(),expected.iter().map(String::as_str).collect::<Vec<_>>());
        for canonical in &expected { prop_assert!(actual.contains(canonical.as_str())); }
    }
}

#[rstest]
#[case::bare("LicenseRef-Reviewed")]
#[case::with("LicenseRef-Reviewed WITH LLVM-exception")]
fn custom_units_require_scoped_policy_permission(#[case] unit: &str) {
    // Arrange
    let config = format!("[tool.taloon]\nschema-version=1\nallow=['{unit}']");
    let date = EvaluationDate::parse("2026-11-01").unwrap();
    // Act
    let result = Policy::from_pyproject(&config, date);
    // Assert
    assert_eq!(result.err().unwrap().code, Code::Configuration);
}
