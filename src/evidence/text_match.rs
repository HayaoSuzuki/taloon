//! Conservative, complete-text matching of selected SPDX license texts.

use std::sync::LazyLock;

#[cfg(test)]
mod variants_tests;

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
                return apache_matches(original, &text, canonical);
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

/// SPDX marks the application appendix optional and its identifying field
/// replaceable. Every mandatory term and every other appendix word stays exact.
fn apache_matches(original: &str, text: &str, canonical: &str) -> bool {
    let (terms, _) = canonical.split_once(" APPENDIX:").expect("Apache appendix");
    if text == terms {
        return true;
    }
    let (before, after) = canonical
        .split_once("[yyyy] [name of copyright owner]")
        .expect("Apache identifying field");
    let Some(field) = text
        .strip_prefix(before)
        .and_then(|s| s.strip_suffix(after))
    else {
        return false;
    };
    // A field cannot consume extra lines of conditions, even after unfolding.
    let mut notices = original
        .lines()
        .filter_map(|line| line.trim().strip_prefix("Copyright "));
    let Some(notice) = notices.next() else {
        return false;
    };
    if notices.next().is_some() || field != words(notice) {
        return false;
    }
    let identifying = if let Some(bracketed) = notice.strip_prefix('[') {
        let Some((year, holder)) = bracketed
            .strip_suffix(']')
            .and_then(|s| s.split_once("] ["))
        else {
            return false;
        };
        format!("Copyright {year} {holder}")
    } else {
        format!("Copyright {notice}")
    };
    copyright_line(&identifying)
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
        ("MIT", "MIT License" | "The MIT License (MIT)") | ("ISC", "ISC License")
    ) {
        let Some(copyright) = lines.next() else {
            return false;
        };
        header = copyright;
    }
    // Only separate copyright notices and an optional final BSD reservation
    // belong to the substituted header. Never flatten extra lines into names.
    let mut count = 0;
    loop {
        count += 1;
        if count > 32 || !copyright_line(header) {
            return false;
        }
        match lines.next() {
            None => return true,
            Some("All rights reserved.") if id.starts_with("BSD-") => {
                return lines.next().is_none();
            }
            Some(next) => header = next,
        }
    }
}

fn copyright_line(line: &str) -> bool {
    let Some(notice) = line.strip_prefix("Copyright ") else {
        return false;
    };
    let notice = notice.strip_prefix("(c) ").unwrap_or(notice);
    let Some(holder_offset) =
        notice.find(|c: char| !(c.is_ascii_digit() || c == '-' || c == ',' || c.is_whitespace()))
    else {
        return false;
    };
    let (years, holder) = notice.split_at(holder_offset);
    if !years.ends_with(char::is_whitespace) {
        return false;
    }
    // A trailing comma retains the established single-year notice spelling.
    let years = years.trim().strip_suffix(',').unwrap_or(years.trim());
    let valid_year = |part: &str| part.len() == 4 && part.bytes().all(|b| b.is_ascii_digit());
    let mut count = 0;
    for year in years.split(',').map(str::trim) {
        count += 1;
        if count > 32
            || !(valid_year(year)
                || year
                    .split_once('-')
                    .is_some_and(|(a, b)| valid_year(a) && valid_year(b)))
        {
            return false;
        }
    }
    holder_name(holder)
}

fn corporate_abbreviation(token: &str) -> bool {
    matches!(token, "Inc" | "Ltd" | "Corp" | "Co")
}

fn holder_name(holder: &str) -> bool {
    let holder = holder.strip_suffix('.').unwrap_or(holder);
    let Some(holder) = without_contact_origin(holder) else {
        return false;
    };
    if two_person_names(holder) {
        return true;
    }
    let mut tokens = holder.split_whitespace().peekable();
    let mut named = false;
    let mut final_connector = false;
    while let Some(token) = tokens.next() {
        final_connector = matches!(token, "and" | "the" | "of");
        if final_connector
            || matches!(
                token,
                "contributors" | "authors" | "developers" | "maintainers" | "team"
            )
        {
            continue;
        }
        // A comma belongs only to a corporate suffix, never an arbitrary clause.
        let name = if let Some(name) = token.strip_suffix(',') {
            if !tokens
                .peek()
                .is_some_and(|next| corporate_abbreviation(next))
            {
                return false;
            }
            name
        } else {
            token
        };
        if !holder_token(name) {
            return false;
        }
        if name.starts_with('<') {
            return named && tokens.peek().is_none();
        }
        named = true;
    }
    named && !final_connector
}

/// A narrow two-person attribution, not an arbitrary comma-delimited clause.
fn two_person_names(holder: &str) -> bool {
    let Some((first, second)) = holder.split_once(',') else {
        return false;
    };
    [first, second].into_iter().all(|person| {
        let mut words = person.split_whitespace();
        matches!((words.next(), words.next(), words.next()), (Some(given), Some(family), None)
            if !given.starts_with('<') && !family.starts_with('<')
                && holder_token(given) && holder_token(family))
    })
}

/// Accept a final parenthesized web origin as contact notation, never fetch it.
/// Paths, credentials and parameters cannot carry extra terms through this rule.
fn without_contact_origin(holder: &str) -> Option<&str> {
    let Some((name, contact)) = holder.rsplit_once(" (") else {
        return Some(holder);
    };
    let contact = contact.strip_suffix(')')?;
    if contact.chars().any(char::is_whitespace) {
        return None;
    }
    let url = url::Url::parse(contact).ok()?;
    // Require a literal origin spelling: URL parsing can erase dot segments or
    // normalize backslashes, which must not conceal a non-root input path.
    let origin = url.origin().ascii_serialization();
    (matches!(url.scheme(), "http" | "https")
        && contact.strip_suffix('/').unwrap_or(contact) == origin
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none())
    .then_some(name)
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
                | "consent"
                | "required"
                | "requires"
                | "redistribution"
                | "restricted"
                | "restrictions"
                | "must"
                | "shall"
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

    fn with_attribution(id: &str, header: &str) -> String {
        let marker = match id {
            "MIT" => "Permission is hereby granted",
            "ISC" => "Permission to use",
            _ => "Redistribution and use",
        };
        let (_, body) = canonical(id).split_once(marker).unwrap();
        format!("{header}\n\n{marker}{body}")
    }

    #[rstest]
    #[case::mit("MIT")]
    #[case::bsd_two("BSD-2-Clause")]
    #[case::bsd_three("BSD-3-Clause")]
    #[case::isc("ISC")]
    fn recognizes_separate_copyright_lines(#[case] id: &str) {
        let text = with_attribution(
            id,
            "Copyright (c) 2025 Example Author\nCopyright 2026 Other Author",
        );

        let actual = identify(&text);

        assert_eq!(actual, Some(id));
    }

    #[rstest]
    #[case::year_list("Copyright (c) 2022, 2024, 2025-2026 Example Author")]
    #[case::compact_year_list("Copyright (c) 2022,2024,2025-2026 Example Author")]
    #[case::corporation("Copyright (c) 2026 Example, Inc.")]
    #[case::limited("Copyright (c) 2026 Example Ltd.")]
    #[case::corporate_limited("Copyright (c) 2026 Example, Ltd.")]
    #[case::corporate_company("Copyright (c) 2026 Example, Co.")]
    #[case::corporate_corp("Copyright (c) 2026 Example, Corp.")]
    #[case::roles("Copyright (c) 2026 Example and the contributors")]
    #[case::team("Copyright (c) 2026 The Example team of authors and developers")]
    #[case::maintainers("Copyright (c) 2026 Example maintainers")]
    fn recognizes_structured_attribution(#[case] header: &str) {
        let text = with_attribution("MIT", header);

        let actual = identify(&text);

        assert_eq!(actual, Some("MIT"));
    }

    #[rstest]
    #[case::mit("MIT")]
    #[case::bsd("BSD-3-Clause")]
    #[case::isc("ISC")]
    fn recognizes_maximum_copyright_lines(#[case] id: &str) {
        let header = vec!["Copyright (c) 2026 Example Author"; 32].join("\n");
        let text = with_attribution(id, &header);

        let actual = identify(&text);

        assert_eq!(actual, Some(id));
    }

    #[test]
    fn rejects_excess_copyright_lines() {
        let header = vec!["Copyright (c) 2026 Example Author"; 33].join("\n");
        let text = with_attribution("MIT", &header);

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[rstest]
    #[case::corporate_sentence("Copyright (c) 2026 Example, Inc. Redistribution Requires Consent.")]
    #[case::lowercase_sentence("Copyright (c) 2026 Example and permission is withheld")]
    #[case::extra_line(
        "Copyright (c) 2026 Example\nRedistribution Requires Consent.\nCopyright (c) 2025 Other Author"
    )]
    #[case::same_line_copyright("Copyright (c) 2026 Example Copyright (c) 2025 Other Author")]
    #[case::year_gap("Copyright (c) 2022,,2026 Example")]
    #[case::year_short("Copyright (c) 2022, 26 Example")]
    #[case::year_open_range("Copyright (c) 2022- Example")]
    #[case::roles_only("Copyright (c) 2026 contributors and maintainers")]
    #[case::trailing_connector("Copyright (c) 2026 Example and")]
    #[case::unknown_corporate_suffix("Copyright (c) 2026 Example, Other.")]
    #[case::comma_clause("Copyright (c) 2026 Example, Redistribution Requires Consent")]
    #[case::reservation_between_notices(
        "Copyright (c) 2026 Example\nAll rights reserved.\nCopyright (c) 2025 Other Author"
    )]
    #[case::repeated_reservation(
        "Copyright (c) 2026 Example\nAll rights reserved.\nAll rights reserved."
    )]
    #[case::preamble("Provided under these terms\nCopyright (c) 2026 Example")]
    #[case::reservation_before_copyright("All rights reserved.\nCopyright (c) 2026 Example")]
    fn rejects_unstructured_attribution(#[case] header: &str) {
        let text = with_attribution("BSD-3-Clause", header);

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[rstest]
    #[case::maximum(32, Some("MIT"))]
    #[case::excess(33, None)]
    fn bounds_year_lists(#[case] count: usize, #[case] expected: Option<&str>) {
        let years = vec!["2026"; count].join(", ");
        let text = with_attribution("MIT", &format!("Copyright (c) {years} Example"));

        let actual = identify(&text);

        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case::mit("MIT")]
    #[case::isc("ISC")]
    fn rejects_reservation_for_non_bsd_texts(#[case] id: &str) {
        let text = with_attribution(id, "Copyright (c) 2026 Example\nAll rights reserved.");

        let actual = identify(&text);

        assert_eq!(actual, None);
    }

    #[test]
    fn recognizes_bsd_reservation_after_multiple_notices() {
        let text = with_attribution(
            "BSD-3-Clause",
            "Copyright (c) 2025 Example\nCopyright (c) 2026 Other Author\nAll rights reserved.",
        );

        let actual = identify(&text);

        assert_eq!(actual, Some("BSD-3-Clause"));
    }

    proptest! {
        #[test]
        fn structured_notices_preserve_recognition(
            years in prop::collection::vec(1900_u16..2100, 2..10),
            names in prop::collection::vec("Author[a-z]{2,12}", 2..8),
            id in prop::sample::select(vec!["MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC"]),
        ) {
            let years = years.iter().map(u16::to_string).collect::<Vec<_>>().join(", ");
            let header = names.iter().map(|name| format!("Copyright (c) {years} {name} and contributors")).collect::<Vec<_>>().join("\n");
            let text = with_attribution(id, &header);

            let actual = identify(&text);

            prop_assert_eq!(actual, Some(id));
        }

        #[test]
        fn additional_lines_between_notices_are_rejected(word in "[A-Z][a-z]{2,12}", id in prop::sample::select(vec!["MIT", "BSD-2-Clause", "BSD-3-Clause", "ISC"])) {
            let text = with_attribution(id, &format!("Copyright (c) 2025 Example\n{word} Requires Consent.\nCopyright (c) 2026 Other Author"));

            let actual = identify(&text);

            prop_assert_eq!(actual, None);
        }

        #[test]
        fn sentences_after_corporate_names_are_rejected(name in "Company[a-z]{2,12}", word in "[A-Z][a-z]{2,12}") {
            let text = with_attribution("MIT", &format!("Copyright (c) 2026 {name}, Inc. {word} Requires Consent."));

            let actual = identify(&text);

            prop_assert_eq!(actual, None);
        }

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
