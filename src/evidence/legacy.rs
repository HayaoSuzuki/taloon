//! Precise names in the legacy free-text field, without changing expression syntax.
use crate::{model::Error, policy::LicenseExpression};

// Whole-value, versioned names only. Generic BSD/Apache labels are ambiguous.
const ALIASES: &[(&str, &str)] = &[
    ("Apache License, Version 2.0", "Apache-2.0"),
    ("Apache 2.0", "Apache-2.0"),
    ("BSD 2-Clause License", "BSD-2-Clause"),
    ("BSD 3-Clause License", "BSD-3-Clause"),
    ("BSD 2-Clause", "BSD-2-Clause"),
    ("BSD 3-Clause", "BSD-3-Clause"),
    ("2-Clause BSD License", "BSD-2-Clause"),
    ("3-Clause BSD License", "BSD-3-Clause"),
    ("The MIT License (MIT)", "MIT"),
];

fn unique_name<'a>(
    value: &str,
    candidates: impl Iterator<Item = (&'a str, &'a str)>,
) -> Option<&'a str> {
    let mut matches = candidates
        .filter(|(name, _)| name.eq_ignore_ascii_case(value.trim()))
        .map(|(_, id)| id);
    let result = matches.next()?;
    (!matches.any(|id| id != result)).then_some(result)
}

pub(super) fn interpret(value: &str) -> Result<LicenseExpression, Error> {
    let error = match LicenseExpression::parse(value) {
        Ok(expression) => return Ok(expression),
        Err(error) => error,
    };
    let names = spdx::identifiers::LICENSES
        .iter()
        .map(|license| (license.full_name, license.name))
        .chain(ALIASES.iter().copied());
    if let Some(id) = unique_name(value, names) {
        return LicenseExpression::parse(id);
    }
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::unique_name;
    use rstest::rstest;
    #[rstest]
    #[case::unique(&[("Example License","MIT")],Some("MIT"))]
    #[case::repeated_same_identity(&[("Example License","MIT"),("example license","MIT")],Some("MIT"))]
    #[case::ambiguous(&[("Example License","MIT"),("example license","ISC")],None)]
    #[case::substring(&[("Modified Example License","MIT")],None)]
    fn name_lookup_requires_one_identity(
        #[case] names: &[(&str, &str)],
        #[case] expected: Option<&str>,
    ) {
        // Arrange
        let input = "Example License";
        // Act
        let matched = unique_name(input, names.iter().copied());
        // Assert
        assert_eq!(matched, expected);
    }
}
