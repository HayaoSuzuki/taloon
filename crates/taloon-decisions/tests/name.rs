use proptest::prelude::*;
use rstest::rstest;
use taloon_decisions::name::normalize;

#[rstest]
#[case::single(b"A", Some(&b"a"[..]))]
#[case::digits(b"09", Some(&b"09"[..]))]
#[case::mixed(b"Foo._--BAR_bAz", Some(&b"foo-bar-baz"[..]))]
#[case::empty(b"", None)]
#[case::leading(b".a", None)]
#[case::trailing(b"a_", None)]
#[case::space(b"a b", None)]
#[case::nul(b"a\0b", None)]
#[case::unicode("café".as_bytes(), None)]
fn named_normalization(#[case] input: &[u8], #[case] expected: Option<&[u8]>) {
    // Arrange: each case supplies the complete expected decision.
    // Act
    let actual = normalize(input);
    // Assert
    assert_eq!(actual.as_deref(), expected);
}

proptest! {
    #[test]
    fn generated_components_have_independent_canonical_join(
        parts in prop::collection::vec("[a-zA-Z0-9]{1,20}", 1..20),
        separators in prop::collection::vec("[-_.]{1,12}", 19),
    ) {
        // Arrange: a component grammar determines the expected output directly.
        let expected = parts.join("-").to_ascii_lowercase();
        let mut input = parts[0].clone();
        for (part, separator) in parts.iter().skip(1).zip(separators) {
            input.push_str(&separator);
            input.push_str(part);
        }
        // Act
        let actual = normalize(input.as_bytes());
        // Assert
        prop_assert_eq!(actual.as_deref(), Some(expected.as_bytes()));
    }

    #[test]
    fn arbitrary_bytes_obey_domain(input in prop::collection::vec(any::<u8>(), 0..160)) {
        // Arrange: the domain is checked independently with the standard ASCII API.
        let accepted = input.first().is_some_and(u8::is_ascii_alphanumeric)
            && input.last().is_some_and(u8::is_ascii_alphanumeric)
            && input.iter().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(b));
        // Act
        let actual = normalize(&input);
        // Assert
        prop_assert_eq!(actual.is_some(), accepted);
    }

    #[test]
    fn accepted_output_is_idempotent(input in "[a-zA-Z0-9]([-_.a-zA-Z0-9]{0,150}[a-zA-Z0-9])?") {
        // Arrange
        let first = normalize(input.as_bytes()).expect("generated domain is valid");
        // Act
        let second = normalize(&first);
        // Assert
        prop_assert_eq!(second.as_deref(), Some(first.as_slice()));
    }

    #[test]
    fn accepted_output_does_not_grow(input in "[a-zA-Z0-9]([-_.a-zA-Z0-9]{0,150}[a-zA-Z0-9])?") {
        // Arrange: input length bounds the possible number of retained bytes.
        let bound = input.len();
        // Act
        let output = normalize(input.as_bytes()).expect("generated domain is valid");
        // Assert
        prop_assert!(output.len() <= bound);
    }
}
