use super::identify;
use proptest::prelude::*;
use rstest::rstest;

fn canonical(id: &str) -> &'static str {
    spdx::license_id(id).unwrap().text()
}

fn apache_terms() -> &'static str {
    canonical("Apache-2.0")
        .split_once("APPENDIX:")
        .unwrap()
        .0
        .trim_end()
}

#[test]
fn apache_accepts_complete_terms_without_optional_appendix() {
    // Arrange
    let text = apache_terms();
    // Act
    let actual = identify(text);
    // Assert
    assert_eq!(actual, Some("Apache-2.0"));
}

#[rstest]
#[case::brackets("[2020] [Example Author <author@example.org>]")]
#[case::plain("2026 Example Author")]
#[case::copyright_symbol("(c) 2026 Example Author")]
fn apache_accepts_structured_appendix_copyright(#[case] notice: &str) {
    // Arrange
    let text = canonical("Apache-2.0").replace("[yyyy] [name of copyright owner]", notice);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some("Apache-2.0"));
}

#[rstest]
#[case::added_condition("2026 Example; noncommercial use only")]
#[case::new_line("2026 Example Author\nRedistribution Requires Consent.")]
#[case::same_line_sentence("2026 Example Author. Redistribution Requires Consent.")]
#[case::bracket_condition("[2026] [Example; noncommercial use only]")]
#[case::broken_brackets("[2026] [Example Author")]
fn apache_rejects_conditions_in_appendix_copyright(#[case] notice: &str) {
    // Arrange
    let text = canonical("Apache-2.0").replace("[yyyy] [name of copyright owner]", notice);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::partial_appendix("APPENDIX: How to apply the Apache License to your work.")]
#[case::restriction("Noncommercial use only.")]
#[case::copyright("Copyright 2026 Example Author")]
fn apache_rejects_material_after_complete_terms(#[case] suffix: &str) {
    // Arrange
    let text = format!("{}\n{suffix}", apache_terms());
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::terms_only(false)]
#[case::full_appendix(true)]
fn apache_rejects_changed_mandatory_terms(#[case] appendix: bool) {
    // Arrange
    let source = if appendix {
        canonical("Apache-2.0")
    } else {
        apache_terms()
    };
    let text = source.replace("perpetual,", "temporary,");
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

proptest! {
    #[test]
    fn apache_appendix_identifying_information_is_variable(year in 1900_u16..2100, name in "Author[a-z]{2,15}") {
        // Arrange
        let text = canonical("Apache-2.0").replace("[yyyy] [name of copyright owner]", &format!("[{year}] [{name} Writer]"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, Some("Apache-2.0"));
    }

    #[test]
    fn apache_terms_do_not_hide_appended_conditions(word in "[a-z]{1,30}") {
        // Arrange
        let text = format!("{}\nAdditional condition: {word}", apache_terms());
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }
}
