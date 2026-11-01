//! Redact URL tokens in declaration prose without treating the prose as a URL.
use crate::model::safe_locator;

/// Preserve prose/email addresses; apply locator protection only to URL tokens.
/// Malformed URL tokens are redacted individually. Terminal escaping belongs to
/// the CLI, while JSON escaping belongs to the serializer.
pub(crate) fn redact_urls(value: &str) -> String {
    let mut output = String::new();
    let mut copied = 0;
    let mut search = 0;
    while let Some(relative) = value[search..].find("://") {
        let colon = search + relative;
        let start = value[..colon]
            .char_indices()
            .rev()
            .find(|(_, c)| !c.is_ascii_alphanumeric() && !matches!(c, '+' | '-' | '.'))
            .map_or(0, |(index, c)| index + c.len_utf8());
        let end = value[colon + 3..]
            .find(|c: char| c.is_whitespace())
            .map_or(value.len(), |offset| colon + 3 + offset);
        let token = &value[start..end];
        let url = token.trim_end_matches(['.', ',', ';', '!', ')', '}', '>', '\'', '"', '`']);
        output.push_str(&value[copied..start]);
        output.push_str(&safe_locator(url));
        output.push_str(&token[url.len()..]);
        copied = end;
        search = end;
    }
    if copied == 0 {
        value.to_owned()
    } else {
        output.push_str(&value[copied..]);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::redact_urls;
    use proptest::prelude::*;
    use rstest::rstest;

    #[rstest]
    #[case::normal("Contact author@example.org?", "Contact author@example.org?")]
    #[case::url(
        "See <https://alice:password@example.org/license?token=hidden#private>.",
        "See <https://example.org/license?redacted>."
    )]
    #[case::malformed(
        "See https://alice:password@/license now",
        "See [invalid/redacted locator] now"
    )]
    #[case::bad_scheme(
        "See .https://alice:password@example.org now",
        "See [invalid/redacted locator] now"
    )]
    #[case::scheme_missing(
        "See ://alice:password@example.org now",
        "See [invalid/redacted locator] now"
    )]
    #[case::ipv6(
        "See https://alice:password@[::1]/license?token=hidden now",
        "See https://[::1]/license?redacted now"
    )]
    #[case::quotes(
        "See https://alice:pass'word@example.org/license now",
        "See https://example.org/license now"
    )]
    #[case::multiple(
        "A https://alice:password@a.test/x B https://bob:password@b.test/y?secret=hidden",
        "A https://a.test/x B https://b.test/y?redacted"
    )]
    fn prose_url_redaction(#[case] input: &str, #[case] expected: &str) {
        // Arrange: prose with one independently specified expected observation.
        // Act
        let output = redact_urls(input);
        // Assert
        assert_eq!(output, expected);
    }

    proptest! {
        #[test]
        fn ordinary_email_prose_is_unchanged(user in "[a-zA-Z0-9]{1,40}", text in "[a-zA-Z ?.,]{0,80}") {
            // Arrange
            let input = format!("{text} <{user}@example.org>");
            // Act
            let actual = redact_urls(&input);
            // Assert
            prop_assert_eq!(actual, input);
        }

        #[test]
        fn embedded_credentials_are_never_reported(password in "[a-zA-Z0-9'<>]{1,40}", query in "[a-zA-Z0-9]{1,40}") {
            // Arrange
            let input = format!("Contact author@example.org see https://alice:{password}@example.org/license?token={query} now");
            // Act
            let actual = redact_urls(&input);
            // Assert
            prop_assert_eq!(actual, "Contact author@example.org see https://example.org/license?redacted now");
        }
    }
}
