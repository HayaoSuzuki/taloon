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

fn mit_with_header(header: &str) -> String {
    let (_, body) = canonical("MIT")
        .split_once("Permission is hereby granted")
        .unwrap();
    format!("{header}\n\nPermission is hereby granted{body}")
}

#[rstest]
#[case::alternate_title("The MIT License (MIT)\nCopyright (c) 2026 Example Author")]
#[case::two_persons("Copyright (c) 2026 Example Author, Other Writer.")]
#[case::https_contact("Copyright (c) 2026 Example Author (https://author.example/)")]
#[case::http_contact("Copyright (c) 2026 Example Author (http://author.example/)")]
#[case::email_and_contact(
    "Copyright (c) 2026 Example Author <author@example.org> (https://author.example/)"
)]
fn mit_accepts_reviewed_attribution_formats(#[case] header: &str) {
    // Arrange
    let text = mit_with_header(header);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some("MIT"));
}

#[rstest]
#[case::title_condition(
    "The MIT License (MIT) for noncommercial use\nCopyright (c) 2026 Example Author"
)]
#[case::comma_condition("Copyright (c) 2026 Example Author, Redistribution Requires Consent")]
#[case::short_condition("Copyright (c) 2026 Example Author, Consent Required")]
#[case::third_clause("Copyright (c) 2026 Example Author, Other Writer, Consent Required")]
#[case::url_condition(
    "Copyright (c) 2026 Example Author (https://author.example/) noncommercial use only"
)]
#[case::url_path("Copyright (c) 2026 Example Author (https://author.example/extra-terms)")]
#[case::url_dot_path("Copyright (c) 2026 Example Author (https://author.example/extra-terms/..)")]
#[case::url_encoded_dot_path("Copyright (c) 2026 Example Author (https://author.example/%2e/)")]
#[case::url_backslashes(
    "Copyright (c) 2026 Example Author (https://author.example\\extra-terms\\..)"
)]
#[case::url_query("Copyright (c) 2026 Example Author (https://author.example/?terms=restricted)")]
#[case::url_fragment("Copyright (c) 2026 Example Author (https://author.example/#conditions)")]
#[case::url_credentials("Copyright (c) 2026 Example Author (https://user@host.example/)")]
#[case::url_wrong_scheme("Copyright (c) 2026 Example Author (file:///author)")]
#[case::url_without_holder("Copyright (c) 2026 (https://author.example/)")]
#[case::url_on_extra_line("Copyright (c) 2026 Example Author\n(https://author.example/)")]
fn mit_rejects_material_disguised_as_attribution(#[case] header: &str) {
    // Arrange
    let text = mit_with_header(header);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::canonical(false)]
#[case::historical_url(true)]
fn mpl_accepts_complete_text_with_reviewed_locator(#[case] historical_url: bool) {
    // Arrange
    let text = if historical_url {
        canonical("MPL-2.0").replace(
            "https://mozilla.org/MPL/2.0/.",
            "http://mozilla.org/MPL/2.0/.",
        )
    } else {
        canonical("MPL-2.0").to_owned()
    };
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some("MPL-2.0"));
}

#[rstest]
#[case::missing_both("Exhibit A - Source Code Form License Notice")]
#[case::missing_exhibit_b("Exhibit B - \"Incompatible With Secondary Licenses\" Notice")]
fn mpl_rejects_missing_exhibits(#[case] boundary: &str) {
    // Arrange
    let text = canonical("MPL-2.0").split_once(boundary).unwrap().0;
    // Act
    let actual = identify(text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::changed_grant("royalty-free", "royalty-bearing")]
#[case::changed_secondary_licenses(
    "Incompatible With Secondary Licenses",
    "Compatible With Secondary Licenses"
)]
#[case::unreviewed_locator("https://mozilla.org/MPL/2.0/.", "https://example.test/MPL/2.0/.")]
fn mpl_rejects_changed_complete_text(#[case] from: &str, #[case] to: &str) {
    // Arrange
    let text = canonical("MPL-2.0").replace(from, to);
    assert_ne!(text, canonical("MPL-2.0"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

proptest! {
    #[test]
    fn mpl_additional_conditions_remain_unknown(word in "[a-z]{1,30}", historical_url in any::<bool>()) {
        // Arrange
        let source = if historical_url {
            canonical("MPL-2.0").replace("https://mozilla.org/MPL/2.0/.", "http://mozilla.org/MPL/2.0/.")
        } else { canonical("MPL-2.0").to_owned() };
        let text = format!("{source}\nAdditional condition: {word}");
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }

    #[test]
    fn contact_paths_cannot_be_normalized_into_origins(path in "[a-z]{1,24}") {
        // Arrange
        let text = mit_with_header(&format!("Copyright (c) 2026 Example Author (https://author.example/{path}/..)"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }

    #[test]
    fn mit_two_person_notices_preserve_complete_body(first in "Given[a-z]{2,10}", second in "Other[a-z]{2,10}", year in 1900_u16..2100) {
        // Arrange
        let text = mit_with_header(&format!("Copyright (c) {year} {first} Family, {second} Writer."));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, Some("MIT"));
    }

    #[test]
    fn mit_contact_origins_do_not_hide_following_terms(word in "[a-z]{1,30}") {
        // Arrange
        let text = mit_with_header(&format!("Copyright (c) 2026 Example Author (https://author.example/) {word}"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }

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
