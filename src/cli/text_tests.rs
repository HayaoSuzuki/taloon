//! Output preservation and private ownership contracts for terminal rendering.
use super::safe;
use proptest::prelude::*;
use rstest::rstest;
use std::{borrow::Cow, fmt::Write as _, hint::black_box, time::Instant};

#[rstest]
#[case::empty("", "")]
#[case::unicode("日本語 café 🦀", "日本語 café 🦀")]
#[case::quotes("\"'\\", "\"'\\")]
#[case::newline("a\nb", "a\\nb")]
#[case::tab("a\tb", "a\\tb")]
#[case::carriage_return("\r", "\\r")]
#[case::nul("\0", "\\u{0}")]
#[case::escape("\u{1b}[31m", "\\u{1b}[31m")]
#[case::delete("\u{7f}", "\\u{7f}")]
#[case::c1("\u{85}", "\\u{85}")]
#[case::bidi_embedding("\u{202a}", "\\u{202a}")]
#[case::bidi_override_end("\u{202e}", "\\u{202e}")]
#[case::bidi_isolate("\u{2066}", "\\u{2066}")]
#[case::bidi_isolate_end("\u{2069}", "\\u{2069}")]
#[case::outside_bidi_ranges("\u{2029}\u{202f}\u{2065}\u{206a}", "\u{2029}\u{202f}\u{2065}\u{206a}")]
fn terminal_escaping_preserves_existing_output(#[case] input: &str, #[case] expected: &str) {
    // Arrange: input and expected output are independently specified above.
    // Act
    let result = safe(input);
    // Assert
    assert_eq!(result, expected);
}

// Numeric control ranges and explicit short escapes form a separate oracle;
// do not use the implementation's predicate or escape_default iterator here.
fn expected_text(input: &str) -> String {
    let mut out = String::new();
    for c in input.chars() {
        match u32::from(c) {
            9 => out.push_str("\\t"),
            10 => out.push_str("\\n"),
            13 => out.push_str("\\r"),
            n @ (0..=31 | 127..=159 | 0x202a..=0x202e | 0x2066..=0x2069) => {
                write!(out, "\\u{{{n:x}}}").unwrap();
            }
            _ => out.push(c),
        }
    }
    out
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn terminal_text_matches_numeric_unicode_oracle(
        characters in prop::collection::vec(any::<char>(), 0..256),
    ) {
        // Arrange
        let input: String = characters.into_iter().collect();
        let expected = expected_text(&input);
        // Act
        let result = safe(&input);
        // Assert
        prop_assert_eq!(result, expected);
    }
}

#[rstest]
#[case::ascii("https://files.test/example-1.0.whl")]
#[case::unicode("日本語の説明とライセンス 🦀")]
#[case::escaped("description\nlicense\tMIT\u{202e}")]
#[ignore = "opt-in release measurement, no CI elapsed-time threshold"]
fn terminal_text_rendering(#[case] pattern: &str) {
    // Arrange: fixture construction is outside timing.
    let input = pattern.repeat(1024);
    for sample in 1..=5 {
        // Act
        let started = Instant::now();
        for _ in 0..100 {
            black_box(safe(black_box(&input)));
        }
        let elapsed_us = started.elapsed().as_micros();
        // Assert
        assert_ne!(input, "");
        println!(
            "pattern={pattern:?} bytes={} iterations=100 sample={sample} elapsed_us={elapsed_us}",
            input.len()
        );
    }
}

#[rstest]
#[case::empty("")]
#[case::ascii("example-1.0.whl")]
#[case::unicode("日本語 café 🦀")]
#[case::quotes("\"'\\")]
#[case::joiner("a\u{200d}b")]
#[case::outside_bidi_ranges("\u{2029}\u{202f}\u{2065}\u{206a}")]
fn unchanged_terminal_text_is_borrowed(#[case] input: &str) {
    // Arrange: every case is unchanged by the terminal escaping policy.
    // Act
    let result = safe(input);
    // Assert
    assert!(matches!(result, Cow::Borrowed(value) if value == input));
}
