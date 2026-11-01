//! Indexed matching must preserve locked ambiguity and annotated path semantics.
use proptest::prelude::*;
use rstest::rstest;
use std::fmt::Write as _;
use taloon::{model::Code, resolution::Resolution};

fn lock(selector: &str, unrelated: usize) -> String {
    let mut text = format!(
        "version=1\nrevision=5\n[[package]]\nname='root'\nsource={{virtual='.'}}\ndependencies=[{{name='DEP_pkg'{selector}}}]\n"
    );
    for (version, host) in [("1.0", "one"), ("2.0", "one"), ("1.0", "two")] {
        writeln!(text, "[[package]]\nname='dep-pkg'\nversion='{version}'\nsource={{registry='https://{host}.test/simple'}}").unwrap();
    }
    for n in 0..unrelated {
        writeln!(text, "[[package]]\nname='unrelated-{n}'\nversion='1.0'\nsource={{registry='https://one.test/simple'}}").unwrap();
    }
    text
}
#[rstest]
#[case::version(",version='2.0'", "2.0", "one")]
#[case::source(",source={registry='https://two.test/simple'}", "1.0", "two")]
#[case::exact(
    ",version='1.0',source={registry='https://one.test/simple'}",
    "1.0",
    "one"
)]
#[case::specifier(",specifier='>=2'", "2.0", "one")]
fn selectors_preserve_the_unique_locked_identity(
    #[case] selector: &str,
    #[case] version: &str,
    #[case] host: &str,
) {
    // Arrange
    let text = lock(selector, 32);
    let source = format!("https://{host}.test/simple");
    // Act
    let resolution = Resolution::from_uv_lock(&text).unwrap();
    // Assert: origin exists only for the selected identity; all other targets remain.
    assert_eq!(resolution.targets().len(), 35);
    let reached: Vec<_> = resolution
        .targets()
        .iter()
        .filter(|t| !resolution.paths(t.package()).is_empty())
        .collect();
    assert_eq!(reached.len(), 1);
    assert_eq!(reached[0].package().version(), version);
    assert_eq!(
        reached[0]
            .package()
            .source()
            .location()
            .trim_end_matches('/'),
        source
    );
}
#[rstest]
#[case::ambiguous("", Code::LockFormat)]
#[case::no_match(",version='3.0'", Code::LockFormat)]
#[case::invalid_version(",version='invalid'", Code::Configuration)]
#[case::invalid_constraint(",specifier='invalid'", Code::LockFormat)]
#[case::unmatched_invalid_constraint(",version='3.0',specifier='invalid'", Code::LockFormat)]
fn invalid_selectors_preserve_failure_codes(#[case] selector: &str, #[case] expected: Code) {
    // Arrange
    let text = lock(selector, 16);
    // Act
    let result = Resolution::from_uv_lock(&text);
    // Assert
    assert_eq!(result.err().unwrap().code, expected);
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn unrelated_records_cannot_change_exact_origin_selection(unrelated in 0_usize..128, choose_second in any::<bool>()) {
        // Arrange: source choice has one known version independently of resolution.
        let (version, host) = if choose_second { ("1.0", "two") } else { ("2.0", "one") };
        let selector = format!(",version='{version}',source={{registry='https://{host}.test/simple'}}");
        let text = lock(&selector, unrelated);
        // Act
        let resolution = Resolution::from_uv_lock(&text).unwrap();
        // Assert
        let reached: Vec<_> = resolution.targets().iter().filter(|t| !resolution.paths(t.package()).is_empty()).collect();
        prop_assert_eq!(resolution.targets().len(), unrelated + 3);
        prop_assert_eq!(reached.len(), 1);
        prop_assert_eq!(reached[0].package().version(), version);
        let expected_host = format!("{host}.test");
        prop_assert!(reached[0].package().source().location().contains(&expected_host));
    }
}
