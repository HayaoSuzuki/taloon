use super::identify;
use proptest::prelude::*;
use rstest::rstest;

fn with_preamble(preamble: &str) -> String {
    let text = spdx::license_id("MIT").unwrap().text();
    let (_, body) = text.split_once("Permission is hereby granted").unwrap();
    format!("{preamble}\n\nPermission is hereby granted{body}")
}

#[rstest]
#[case::absent("")]
#[case::title_only("MIT License")]
#[case::markdown("# MIT License\nCopyright (c) 2026 Example Author")]
#[case::yearless("Copyright (c) Example Foundation")]
#[case::yearless_corporations("Copyright (c) Example, Inc. and Other Foundation.")]
#[case::yearless_affiliates("Copyright (c) Example Company, Inc. and affiliates.")]
#[case::dated_groups("Copyright (c) 2020 Example Author, 2021 Other Writer, 2022 Third Person")]
#[case::alias("Copyright (c) 2020, 2021, 2022 Example Author (Nickname) and contributors")]
#[case::contacts(
    "Copyright (c) 2020, 2021 Example Author (Nickname) and contributors\nauthor.example\ncontact.example"
)]
#[case::spaced_range(
    "Copyright (c) 2020 - 2026 Example Author (Nickname) and contributors\nauthor.example\ncontact.example"
)]
fn recognizes_reviewed_mit_preambles(#[case] preamble: &str) {
    // Arrange
    let text = with_preamble(preamble);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some("MIT"));
}

#[rstest]
#[case::title_condition("MIT License, noncommercial use only")]
#[case::markdown_condition("# MIT License, noncommercial use only")]
#[case::yearless_restriction("Copyright (c) Noncommercial Use Only")]
#[case::corporate_sentence("Copyright (c) Example, Inc. Redistribution Requires Consent.")]
#[case::alias_restriction("Copyright (c) 2026 Example Author (Noncommercial) and contributors")]
#[case::extra_line("Copyright (c) Example Foundation\nRedistribution Requires Consent.")]
#[case::bare_contact_without_notice("author.example")]
#[case::contact_path("Copyright (c) 2026 Example Author\nauthor.example/extra-terms")]
#[case::contact_query("Copyright (c) 2026 Example Author\nauthor.example?terms=restricted")]
#[case::notice_after_contact(
    "Copyright (c) 2026 Example Author\nauthor.example\nCopyright (c) 2025 Other Writer"
)]
#[case::excess_contacts(
    "Copyright (c) 2026 Example Author\none.example\ntwo.example\nthree.example"
)]
#[case::condition_after_contact(
    "Copyright (c) 2026 Example Author\nauthor.example\nNoncommercial use only."
)]
#[case::invalid_dated_group(
    "Copyright (c) 2020 Example Author, 2021 Other Writer, Consent Required"
)]
#[case::affiliates_condition(
    "Copyright (c) Example Company, Inc. and affiliates. Noncommercial use only."
)]
#[case::broken_range("Copyright (c) 2020 - Example Author (Nickname) and contributors")]
#[case::split_year_digits("Copyright (c) 20 26 Example Author (Nickname) and contributors")]
fn rejects_substantive_or_unstructured_preambles(#[case] preamble: &str) {
    // Arrange
    let text = with_preamble(preamble);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[test]
fn missing_preamble_does_not_allow_changed_terms() {
    // Arrange
    let text = with_preamble("").replace(
        "Permission is hereby granted",
        "Permission is not hereby granted",
    );
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[test]
fn missing_preamble_does_not_allow_appended_material() {
    // Arrange
    let text = format!("{}\nNoncommercial use only.", with_preamble(""));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[test]
fn other_licenses_keep_their_existing_notice_requirements() {
    // Arrange
    let text = spdx::license_id("ISC")
        .unwrap()
        .text()
        .split_once("Permission to use")
        .unwrap()
        .1;
    let text = format!("Permission to use{text}");
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::maximum(32, Some("MIT"))]
#[case::excess(33, None)]
fn bounds_yearless_notice_count(#[case] count: usize, #[case] expected: Option<&str>) {
    // Arrange
    let preamble = vec!["Copyright (c) Example Author"; count].join("\n");
    let text = with_preamble(&preamble);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, expected);
}

#[rstest]
#[case::maximum(32, Some("MIT"))]
#[case::excess(33, None)]
fn bounds_dated_holder_groups(#[case] count: usize, #[case] expected: Option<&str>) {
    // Arrange
    let groups = vec!["2026 Example Author"; count].join(", ");
    let text = with_preamble(&format!("Copyright (c) {groups}"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, expected);
}

#[rstest]
#[case::maximum(5000, Some("MIT"))]
#[case::excess(5001, None)]
fn bounds_additional_preamble_bytes(#[case] bytes: usize, #[case] expected: Option<&str>) {
    // Arrange
    let prefix = "Copyright (c) Name";
    let text = with_preamble(&format!("{prefix}{}", "a".repeat(bytes - prefix.len() - 2)));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, expected);
}

#[rstest]
#[case::empty_label("author..example")]
#[case::leading_hyphen("-author.example")]
#[case::trailing_hyphen("author-.example")]
#[case::credentials("person@author.example")]
#[case::fragment("author.example#terms")]
#[case::port("author.example:443")]
#[case::numeric_tail("author.123")]
fn rejects_invalid_bare_contacts(#[case] contact: &str) {
    // Arrange
    let text = with_preamble(&format!("Copyright (c) Example Author\n{contact}"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

proptest! {
    #[test]
    fn yearless_corporate_notices_preserve_recognition(name in "Company[a-z]{2,12}", other in "Foundation[a-z]{2,12}") {
        // Arrange
        let text = with_preamble(&format!("Copyright (c) {name}, Inc. and {other}."));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, Some("MIT"));
    }

    #[test]
    fn contact_lines_do_not_hide_conditions(word in "[A-Z][a-z]{2,15}") {
        // Arrange
        let text = with_preamble(&format!("Copyright (c) 2026 Example Author\nauthor.example\n{word} Requires Consent."));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }

    #[test]
    fn dated_holder_groups_preserve_recognition(first in 1900_u16..2100, second in 1900_u16..2100) {
        // Arrange
        let text = with_preamble(&format!("Copyright (c) {first} Example Author, {second} Other Writer"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, Some("MIT"));
    }
}
