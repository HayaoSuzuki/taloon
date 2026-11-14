//! Finite substitutions from the released SPDX templates, not template wildcards.
use super::{identify, tests::with_attribution};
use proptest::prelude::*;
use rstest::rstest;

#[rstest]
#[case::mit_copyright_c("MIT", "Copyright (C)")]
#[case::mit_c("MIT", "(c)")]
#[case::mit_uppercase_c("MIT", "(C)")]
#[case::mit_symbol("MIT", "©")]
#[case::isc_copyright_c("ISC", "Copyright (C)")]
#[case::isc_c("ISC", "(c)")]
#[case::isc_uppercase_c("ISC", "(C)")]
#[case::isc_symbol("ISC", "©")]
#[case::bsd_two_copyright_c("BSD-2-Clause", "Copyright (C)")]
#[case::bsd_two_c("BSD-2-Clause", "(c)")]
#[case::bsd_two_uppercase_c("BSD-2-Clause", "(C)")]
#[case::bsd_two_symbol("BSD-2-Clause", "©")]
#[case::bsd_three_copyright_c("BSD-3-Clause", "Copyright (C)")]
#[case::bsd_three_c("BSD-3-Clause", "(c)")]
#[case::bsd_three_uppercase_c("BSD-3-Clause", "(C)")]
#[case::bsd_three_symbol("BSD-3-Clause", "©")]
fn recognizes_equivalent_copyright_markers(#[case] id: &str, #[case] marker: &str) {
    // Arrange
    let text = with_attribution(id, &format!("{marker} 2026 Example Author"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some(id));
}

fn bsd(id: &str) -> String {
    with_attribution(id, "Copyright (c) 2026 Example Author")
}

#[rstest]
#[case::author_bsd_2_clause("AUTHOR", "BSD-2-Clause")]
#[case::author_bsd_3_clause("AUTHOR", "BSD-3-Clause")]
#[case::authors_bsd_2_clause("AUTHORS", "BSD-2-Clause")]
#[case::authors_bsd_3_clause("AUTHORS", "BSD-3-Clause")]
#[case::owner_bsd_2_clause("COPYRIGHT OWNER", "BSD-2-Clause")]
#[case::owner_bsd_3_clause("COPYRIGHT OWNER", "BSD-3-Clause")]
#[case::owners_bsd_2_clause("COPYRIGHT OWNERS", "BSD-2-Clause")]
#[case::owners_bsd_3_clause("COPYRIGHT OWNERS", "BSD-3-Clause")]
#[case::holders_bsd_2_clause("COPYRIGHT HOLDERS", "BSD-2-Clause")]
#[case::holders_bsd_3_clause("COPYRIGHT HOLDERS", "BSD-3-Clause")]
#[case::holder_bsd_2_clause("COPYRIGHT HOLDER", "BSD-2-Clause")]
#[case::holder_bsd_3_clause("COPYRIGHT HOLDER", "BSD-3-Clause")]
fn recognizes_generic_disclaimer_roles(#[case] role: &str, #[case] id: &str) {
    // Arrange
    let text = bsd(id)
        .replace(
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
            &format!("BY THE {role} AND CONTRIBUTORS"),
        )
        .replace(
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
            &format!("THE {role} OR CONTRIBUTORS"),
        );
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some(id));
}

#[rstest]
#[case::author_its("author", "its")]
#[case::author_their("author", "their")]
#[case::author_any("author", "any")]
#[case::authors_its("authors", "its")]
#[case::authors_their("authors", "their")]
#[case::authors_any("authors", "any")]
#[case::owner_its("copyright owner", "its")]
#[case::owner_their("copyright owner", "their")]
#[case::owner_any("copyright owner", "any")]
#[case::owners_its("copyright owners", "its")]
#[case::owners_their("copyright owners", "their")]
#[case::owners_any("copyright owners", "any")]
#[case::holders_its("copyright holders", "its")]
#[case::holders_their("copyright holders", "their")]
#[case::holders_any("copyright holders", "any")]
fn recognizes_third_clause_roles(#[case] role: &str, #[case] reference: &str) {
    // Arrange
    let text = bsd("BSD-3-Clause").replace(
        "name of the copyright holder nor the names of its contributors",
        &format!("name of the {role} nor the names of {reference} contributors"),
    );
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some("BSD-3-Clause"));
}

#[rstest]
#[case::star_bsd_2_clause("*", "BSD-2-Clause")]
#[case::star_bsd_3_clause("*", "BSD-3-Clause")]
#[case::dash_bsd_2_clause("-", "BSD-2-Clause")]
#[case::dash_bsd_3_clause("-", "BSD-3-Clause")]
#[case::absent_bsd_2_clause("", "BSD-2-Clause")]
#[case::absent_bsd_3_clause("", "BSD-3-Clause")]
fn recognizes_bsd_clause_markers(#[case] marker: &str, #[case] id: &str) {
    // Arrange
    let text = bsd(id)
        .replace("1. Redistributions", &format!("{marker} Redistributions"))
        .replace("2. Redistributions", &format!("{marker} Redistributions"))
        .replace("3. Neither", &format!("{marker} Neither"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, Some(id));
}

#[rstest]
#[case::negation("are permitted", "are not permitted")]
#[case::deleted_condition("this list of conditions and ", "")]
#[case::changed_warranty("IMPLIED WARRANTIES", "IMPLIED GUARANTEES")]
#[case::unknown_marker("1. Redistributions", "NOTE Redistributions")]
#[case::unknown_role(
    "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
    "THE EXAMPLE ORGANIZATION OR CONTRIBUTORS"
)]
#[case::changed_role_condition(
    "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
    "BY THE AUTHORS AND CONTRIBUTORS ONLY FOR NONCOMMERCIAL USE"
)]
fn rejects_unreviewed_bsd_changes(#[case] from: &str, #[case] to: &str) {
    // Arrange
    let text = bsd("BSD-3-Clause").replace(from, to);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[rstest]
#[case::restriction("© 2026 Example Author. Use Requires Consent.")]
#[case::new_line("(c) 2026 Example Author\nCommercial use is prohibited.")]
#[case::missing_date("© Example Author")]
#[case::unknown_marker("COPY 2026 Example Author")]
#[case::doubled_marker("(c) (C) 2026 Example Author")]
fn rejects_invalid_symbol_notices(#[case] header: &str) {
    // Arrange
    let text = with_attribution("BSD-3-Clause", header);
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[test]
fn rejects_reordered_bsd_conditions() {
    // Arrange
    let text = bsd("BSD-3-Clause");
    let (prefix, rest) = text.split_once("1. Redistributions").unwrap();
    let (first, rest) = rest.split_once("2. Redistributions").unwrap();
    let (second, rest) = rest.split_once("3. Neither").unwrap();
    let text =
        format!("{prefix}2. Redistributions{second}1. Redistributions{first}3. Neither{rest}");
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

#[test]
fn rejects_two_bsd_bodies() {
    // Arrange
    let text = format!("{}\n{}", bsd("BSD-2-Clause"), bsd("BSD-3-Clause"));
    // Act
    let actual = identify(&text);
    // Assert
    assert_eq!(actual, None);
}

proptest! {
    #[test]
    fn mixed_markers_preserve_complete_bsd_terms(
        first in prop::sample::select(vec!["1.", "*", "-", ""]),
        second in prop::sample::select(vec!["2.", "*", "-", ""]),
        third in prop::sample::select(vec!["3.", "*", "-", ""]),
        role in prop::sample::select(vec!["AUTHOR", "AUTHORS", "COPYRIGHT OWNER", "COPYRIGHT OWNERS"]),
    ) {
        // Arrange
        let text = bsd("BSD-3-Clause").replace("1. Redistributions", &format!("{first} Redistributions"))
            .replace("2. Redistributions", &format!("{second} Redistributions"))
            .replace("3. Neither", &format!("{third} Neither"))
            .replace("THE COPYRIGHT HOLDER OR CONTRIBUTORS", &format!("THE {role} OR CONTRIBUTORS"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, Some("BSD-3-Clause"));
    }

    #[test]
    fn added_terms_after_symbol_notice_are_rejected(word in "[a-z]{1,30}") {
        // Arrange
        let text = format!("{}\nAdditional condition: {word}", with_attribution("BSD-2-Clause", "© 2026 Example Author"));
        // Act
        let actual = identify(&text);
        // Assert
        prop_assert_eq!(actual, None);
    }
}
