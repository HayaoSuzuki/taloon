//! JSON preserves declaration values while escaping terminal-disruptive scalars.
use serde::Serialize;
use std::io::Write;

struct TerminalSafe;
impl serde_json::ser::Formatter for TerminalSafe {
    fn write_string_fragment<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> std::io::Result<()> {
        let mut copied = 0;
        for (index, character) in fragment.char_indices() {
            if super::needs_terminal_escape(character) {
                writer.write_all(&fragment.as_bytes()[copied..index])?;
                for unit in character.encode_utf16(&mut [0; 2]) {
                    write!(writer, "\\u{unit:04x}")?;
                }
                copied = index + character.len_utf8();
            }
        }
        writer.write_all(&fragment.as_bytes()[copied..])
    }
}

pub(super) fn encode<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(128);
    value.serialize(&mut serde_json::Serializer::with_formatter(
        &mut bytes,
        TerminalSafe,
    ))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::encode;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn json_escaping_preserves_values(value in any::<String>()) {
            // Arrange: arbitrary Unicode including terminal controls.
            // Act
            let bytes = encode(&value).unwrap();
            let decoded: String = serde_json::from_slice(&bytes).unwrap();
            // Assert
            prop_assert_eq!(decoded, value);
        }

        #[test]
        fn json_encoding_excludes_literal_terminal_controls(value in any::<String>()) {
            // Arrange: the forbidden scalar ranges are independent of the encoder predicate.
            // Act
            let bytes = encode(&value).unwrap();
            // Assert
            let text = std::str::from_utf8(&bytes).unwrap();
            prop_assert!(!text.chars().any(|character| matches!(u32::from(character),
                0..=31 | 127..=159 | 0x202a..=0x202e | 0x2066..=0x2069)));
        }
    }
}
