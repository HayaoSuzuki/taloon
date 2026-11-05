//! Finite substitutions from the released SPDX templates, not template wildcards.
use super::{identify, tests::with_attribution};
use proptest::prelude::*;
use rstest::rstest;

#[rstest]
#[case::mit("MIT")]
#[case::isc("ISC")]
#[case::bsd_two("BSD-2-Clause")]
#[case::bsd_three("BSD-3-Clause")]
fn recognizes_equivalent_copyright_markers(
    #[case] id: &str,
    #[values("Copyright (C)", "(c)", "(C)", "©")] marker: &str,
) {
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
#[case::author("AUTHOR")]
#[case::authors("AUTHORS")]
#[case::owner("COPYRIGHT OWNER")]
#[case::owners("COPYRIGHT OWNERS")]
#[case::holders("COPYRIGHT HOLDERS")]
#[case::holder("COPYRIGHT HOLDER")]
fn recognizes_generic_disclaimer_roles(
    #[case] role: &str,
    #[values("BSD-2-Clause", "BSD-3-Clause")] id: &str,
) {
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
#[case::author("author")]
#[case::authors("authors")]
#[case::owner("copyright owner")]
#[case::owners("copyright owners")]
#[case::holders("copyright holders")]
fn recognizes_third_clause_roles(
    #[case] role: &str,
    #[values("its", "their", "any")] reference: &str,
) {
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
#[case::star("*")]
#[case::dash("-")]
#[case::absent("")]
fn recognizes_bsd_clause_markers(
    #[case] marker: &str,
    #[values("BSD-2-Clause", "BSD-3-Clause")] id: &str,
) {
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
