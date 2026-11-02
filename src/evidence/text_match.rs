//! Conservative, complete-text matching of selected SPDX license texts.

use std::sync::LazyLock;

// SPDX canonical texts are supplied by the `spdx` crate's `text` feature
// (https://github.com/EmbarkStudios/spdx); no copied text catalog is maintained here.
const SUPPORTED: [&str; 5] = ["MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Apache-2.0"];

static CANONICAL: LazyLock<[(&str, String); 5]> = LazyLock::new(|| {
    SUPPORTED.map(|id| {
        let text = spdx::text::LICENSE_TEXTS
            .iter()
            .find(|(name, _)| *name == id)
            .expect("supported SPDX license has canonical text")
            .1;
        (id, words(text))
    })
});

/// Recognizes complete texts, allowing whitespace and narrowly bounded attribution
/// substitutions. Unrecognized variants remain unknown; this is not similarity detection.
pub(super) fn identify(text: &str) -> Option<&'static str> {
    let original = text;
    let text = words(text);
    CANONICAL
        .iter()
        .find(|(id, canonical)| {
            if text == *canonical {
                return true;
            }
            if *id == "Apache-2.0" {
                return false;
            }
            let start = if id.starts_with("BSD-") {
                "Redistribution and use"
            } else if *id == "MIT" {
                "Permission is hereby granted"
            } else {
                "Permission to use"
            };
            let Some((header, body)) = text.split_once(start) else {
                return false;
            };
            let (canonical_header, expected) =
                canonical.split_once(start).expect("canonical body marker");
            (header == canonical_header || attribution(original, header, id))
                && body_matches(body, expected, id)
        })
        .map(|(id, _)| *id)
}

fn words(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn attribution(original: &str, normalized_header: &str, id: &str) -> bool {
    // Locate the body by token count, preserving the original header's line
    // boundaries even when whitespace inside the body has been reformatted.
    let mut in_word = false;
    let body_offset = original
        .char_indices()
        .filter_map(|(offset, character)| {
            let starts_word = !character.is_whitespace() && !in_word;
            in_word = !character.is_whitespace();
            starts_word.then_some(offset)
        })
        .nth(normalized_header.split_whitespace().count());
    let Some(body_offset) = body_offset else {
        return false;
    };
    let mut lines = original[..body_offset]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let Some(mut header) = lines.next() else {
        return false;
    };
    if matches!(
        (id, header),
        ("MIT", "MIT License") | ("ISC", "ISC License")
    ) {
        let Some(copyright) = lines.next() else {
            return false;
        };
        header = copyright;
    }
    // Substituted attribution is exactly one copyright line, with an optional
    // standalone BSD reservation. Additional lines cannot become holder names.
    if let Some(line) = lines.next()
        && (!id.starts_with("BSD-") || line != "All rights reserved." || lines.next().is_some())
    {
        return false;
    }
    let Some(header) = header.strip_prefix("Copyright ") else {
        return false;
    };
    let header = header.strip_prefix("(c) ").unwrap_or(header);
    let Some((year, holder)) = header.split_once(' ') else {
        return false;
    };
    // A single year or year range and proper-name tokens, optionally followed by
    // an email address. Restricting the grammar avoids treating arbitrary lines
    // beginning with Copyright as disposable metadata.
    let year = year.trim_end_matches(',');
    let valid_year = |part: &str| part.len() == 4 && part.bytes().all(|b| b.is_ascii_digit());
    let year_valid = valid_year(year)
        || year
            .split_once('-')
            .is_some_and(|(a, b)| valid_year(a) && valid_year(b));
    let holder = holder.strip_suffix('.').unwrap_or(holder);
    year_valid && !holder.is_empty() && holder.split_whitespace().all(holder_token)
}

fn holder_token(token: &str) -> bool {
    if let Some(email) = token.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
        return email.contains('@')
            && email
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"@._+-".contains(&b));
    }
    // Periods within an attribution are reserved for one-letter initials;
    // an arbitrary sentence cannot be appended after a completed holder name.
    if let Some(initial) = token.strip_suffix('.') {
        return initial.chars().count() == 1 && initial.chars().all(char::is_uppercase);
    }
    if token.is_empty() {
        return false;
    }
    // Only name-shaped words, rather than unrestricted copyright-line text.
    let mut chars = token.chars();
    chars.next().is_some_and(char::is_uppercase)
        && chars.all(|c| c.is_alphabetic() || c == '-' || c == '\'')
        && !matches!(
            token.to_ascii_lowercase().as_str(),
            "no" | "not"
                | "permission"
                | "license"
                | "licensed"
                | "use"
                | "only"
                | "commercial"
                | "noncommercial"
                | "prohibited"
                | "except"
                | "without"
        )
}

fn body_matches(body: &str, expected: &str, id: &str) -> bool {
    if !id.starts_with("BSD-") {
        return body == expected;
    }
    // BSD list markers and the sqlparse authors/owner spellings are explicit
    // whole-body alternatives. No arbitrary holder or clause wildcard is used.
    let body = body
        .replace(
            "name of the authors nor",
            "name of the copyright holder nor",
        )
        .replace(
            "COPYRIGHT OWNER OR CONTRIBUTORS",
            "COPYRIGHT HOLDER OR CONTRIBUTORS",
        );
    body == expected
        || body
            == expected
                .replace("1. Redistributions", "* Redistributions")
                .replace("2. Redistributions", "* Redistributions")
                .replace("3. Neither", "* Neither")
}

#[cfg(test)]
mod tests {
    use super::identify;
    use proptest::prelude::*;
    use rstest::rstest;

    fn canonical(id: &str) -> &'static str {
        spdx::text::LICENSE_TEXTS
            .iter()
            .find(|(name, _)| *name == id)
            .unwrap()
            .1
    }

    #[rstest]
    #[case::mit("MIT")]
    #[case::bsd_two("BSD-2-Clause")]
    #[case::bsd_three("BSD-3-Clause")]
    #[case::isc("ISC")]
    #[case::apache("Apache-2.0")]
    fn recognizes_complete_canonical_text(#[case] id: &str) {
        let text = canonical(id);

        let actual = identify(text);

        assert_eq!(actual, Some(id));
    }

    #[rstest]
    #[case::restriction("Commercial use is prohibited.")]
    #[case::negation("This license does not apply.")]
    #[case::copyright_shaped("Copyright (c) 2026 Commercial use is prohibited.")]
    #[case::other_license("SPDX-License-Identifier: GPL-3.0-only")]
    fn rejects_appended_material(#[case] suffix: &str) {
        let text = format!("{}\n{suffix}", canonical("MIT"));

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[rstest]
    #[case::negated("Permission is hereby granted", "Permission is not hereby granted")]
    #[case::deleted("including without limitation", "")]
    #[case::duplicated(
        "subject to the following conditions:",
        "subject to the following conditions: subject to the following conditions:"
    )]
    fn rejects_altered_terms(#[case] from: &str, #[case] to: &str) {
        let text = canonical("MIT")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(from, to);

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[rstest]
    #[case::empty("")]
    #[case::identifier("MIT")]
    #[case::claim("This project uses the MIT license.")]
    fn rejects_incomplete_text(#[case] text: &str) {
        let actual = identify(text);

        assert_eq!(actual, None);
    }

    #[test]
    fn recognizes_sqlparse_bsd_variant() {
        // sqlparse 0.6.0 LICENSE uses these spelling/list variants of BSD-3-Clause.
        let text = canonical("BSD-3-Clause")
            .replace(
                "Copyright (c) <year> <owner>.",
                "Copyright (c) 2016, Andi Albrecht <albrecht.andi@gmail.com>\nAll rights reserved.",
            )
            .replace("1.", "*")
            .replace("2.", "*")
            .replace("3.", "*")
            .replace("name of the copyright holder", "name of the authors")
            .replace("COPYRIGHT HOLDER OR", "COPYRIGHT OWNER OR");

        let actual = identify(&text);

        assert_eq!(actual, Some("BSD-3-Clause"));
    }

    #[rstest]
    #[case::extra_line(
        "Copyright (c) 2026 Example.\nRedistribution Requires Consent.\nAll rights reserved."
    )]
    #[case::extra_sentence(
        "Copyright (c) 2026 Example. Redistribution Requires Consent.\nAll rights reserved."
    )]
    #[case::restriction_before_holder_end(
        "Copyright (c) 2026 Example. Redistribution Requires Consent\nAll rights reserved."
    )]
    fn rejects_bsd_restrictions_in_attribution(#[case] header: &str) {
        let text = canonical("BSD-3-Clause").replace("Copyright (c) <year> <owner>.", header);

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[rstest]
    #[case::negated("Copyright (c) 2026 No permission is granted.")]
    #[case::restriction("Copyright (c) 2026 Example; noncommercial use only")]
    #[case::extra_line("Copyright (c) 2026 Example\nPermission is not granted.")]
    fn rejects_terms_disguised_as_attribution(#[case] header: &str) {
        let text = canonical("MIT").replace("Copyright (c) <year> <copyright holders>", header);

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[test]
    fn rejects_reordered_paragraphs() {
        let text = canonical("MIT");
        let (prefix, rest) = text.split_once("The above copyright notice").unwrap();
        let (condition, disclaimer) = rest.split_once("THE SOFTWARE IS PROVIDED").unwrap();
        let text = format!(
            "{prefix}THE SOFTWARE IS PROVIDED{disclaimer}The above copyright notice{condition}"
        );

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[test]
    fn rejects_duplicate_license_text() {
        let text = format!("{}\n{}", canonical("MIT"), canonical("MIT"));

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[test]
    fn rejects_mixed_license_text() {
        let text = format!("{}\n{}", canonical("MIT"), canonical("ISC"));

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[test]
    fn recognizes_bsd_star_markers_with_canonical_attribution() {
        let text = canonical("BSD-3-Clause")
            .replace("1.", "*")
            .replace("2.", "*")
            .replace("3.", "*");

        let actual = identify(&text);

        assert_eq!(actual, Some("BSD-3-Clause"));
    }

    #[rstest]
    #[case::plain_name("Copyright (c) 2026 Example Author")]
    #[case::final_period("Copyright (c) 2026 Example Author.")]
    #[case::initial("Copyright (c) 2026 A. Author")]
    #[case::email("Copyright (c) 2026 Example Author <author@example.org>")]
    fn recognizes_single_line_attribution(#[case] header: &str) {
        let text = canonical("MIT").replace("Copyright (c) <year> <copyright holders>", header);

        let actual = identify(&text);

        assert_eq!(actual, Some("MIT"));
    }

    proptest! {
        #[test]
        fn whitespace_preserves_recognition(separator in prop::sample::select(vec![" ", "\n", "\r\n", "\t", " \n\t "])) {
            let text = canonical("MIT").split_whitespace().collect::<Vec<_>>().join(separator);

            let actual = identify(&text);

            prop_assert_eq!(actual, Some("MIT"));
        }

        #[test]
        fn simple_copyright_substitutions_preserve_recognition(year in 1900_u16..2100, holder in "Holder[a-z]{2,16}") {
            let text = canonical("MIT").replace("<year> <copyright holders>", &format!("{year} {holder}"));

            let actual = identify(&text);

            prop_assert_eq!(actual, Some("MIT"));
        }

        #[test]
        fn arbitrary_added_terms_are_rejected(word in "[a-z]{1,30}") {
            let text = format!("{}\nAdditional condition: {word}", canonical("MIT"));

            let actual = identify(&text);

            prop_assert_eq!(actual, None);
        }
    }
}
