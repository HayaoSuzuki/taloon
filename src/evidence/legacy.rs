//! Precise names and complete texts in legacy fields; SPDX expression syntax stays strict.
use crate::{model::Error, policy::LicenseExpression};

/// The only incomplete declarations that verified file text may complete.
/// Whole-value matching keeps added terms outside this extension.
#[derive(Clone, Copy)]
pub(super) enum FileConstraint {
    AnyRecognized,
    Bsd,
    Apache,
}
impl FileConstraint {
    pub(super) fn for_legacy(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("UNKNOWN") {
            Some(Self::AnyRecognized)
        } else if ["BSD", "BSD License"]
            .iter()
            .any(|label| value.eq_ignore_ascii_case(label))
        {
            Some(Self::Bsd)
        } else if [
            "Apache",
            "Apache License",
            "Apache Software License",
            "Apache 2",
        ]
        .iter()
        .any(|label| value.eq_ignore_ascii_case(label))
        {
            Some(Self::Apache)
        } else {
            None
        }
    }

    pub(super) fn accepts(self, expression: &LicenseExpression) -> bool {
        match self {
            Self::AnyRecognized => true,
            Self::Bsd => matches!(expression.text(), "BSD-2-Clause" | "BSD-3-Clause"),
            Self::Apache => expression.text() == "Apache-2.0",
        }
    }
}

// Whole-value precise names only. Generic family labels are ambiguous.
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
    // Reviewed subset of CycloneDX's mapping at 8e05e866d8073fc18b3b3c70a9316fd5a5999f81.
    // See docs/development/work/versioned-license-aliases/plan.md for scope and source.
    ("MPL 1.1", "MPL-1.1"),
    ("Mozilla Public License Version 1.1", "MPL-1.1"),
    ("MPL 2.0", "MPL-2.0"),
    ("Mozilla Public License, Version 2.0", "MPL-2.0"),
    ("The MIT License", "MIT"),
    ("Apache 2.0 License", "Apache-2.0"),
    ("Apache Software License, Version 2.0", "Apache-2.0"),
    ("Apache Software License 2.0", "Apache-2.0"),
    ("The Apache Software License, Version 2.0", "Apache-2.0"),
    ("Apache License (v2.0)", "Apache-2.0"),
    ("Apache License v2.0", "Apache-2.0"),
    ("Apache License Version 2.0", "Apache-2.0"),
    ("Apache Software License - Version 2.0", "Apache-2.0"),
    ("The Apache License, Version 2.0", "Apache-2.0"),
    ("Apache-2.0 License", "Apache-2.0"),
    ("BSD 2 Clause", "BSD-2-Clause"),
    ("The BSD 2-Clause License", "BSD-2-Clause"),
    ("The 2-Clause BSD License", "BSD-2-Clause"),
    ("BSD 3 Clause", "BSD-3-Clause"),
    ("The BSD 3-Clause License", "BSD-3-Clause"),
    ("CDDL 1.0", "CDDL-1.0"),
    (
        "Common Development and Distribution License (CDDL) v1.0",
        "CDDL-1.0",
    ),
    ("CDDL 1.1", "CDDL-1.1"),
    (
        "Common Development and Distribution License (CDDL) v1.1",
        "CDDL-1.1",
    ),
    ("EPL 1.0", "EPL-1.0"),
    ("Eclipse Public License (EPL) 1.0", "EPL-1.0"),
    ("Eclipse Public License v1.0", "EPL-1.0"),
    ("Eclipse Public License, Version 1.0", "EPL-1.0"),
    ("EPL 2.0", "EPL-2.0"),
    ("Eclipse Public License v2.0", "EPL-2.0"),
    ("Eclipse Public License, Version 2.0", "EPL-2.0"),
    ("CC0 1.0 Universal", "CC0-1.0"),
    (
        "GNU General Public License (GPL) version 2, or any later version",
        "GPL-2.0-or-later",
    ),
    (
        "GNU General Public License (GPL) version 2.0, or any later version",
        "GPL-2.0-or-later",
    ),
    ("GNU General Public License v2 or later", "GPL-2.0-or-later"),
    (
        "GNU Lesser General Public License (LGPL), version 2.1 or later",
        "LGPL-2.1-or-later",
    ),
    ("LGPL, v2.1 or later", "LGPL-2.1-or-later"),
    (
        "GNU Lesser General Public License (LGPL), version 3 or later",
        "LGPL-3.0-or-later",
    ),
    (
        "GNU Lesser General Public License (LGPL), version 3.0 or later",
        "LGPL-3.0-or-later",
    ),
];

fn unique_name<'a>(
    value: &str,
    candidates: impl Iterator<Item = (&'a str, &'a str)>,
) -> Option<&'a str> {
    let value = value.trim();
    let mut matches = candidates
        .filter(|(name, _)| name.eq_ignore_ascii_case(value))
        .map(|(_, id)| id);
    let result = matches.next()?;
    (!matches.any(|id| id != result)).then_some(result)
}

/// Interpret one unfolded legacy value. Complete-text matching additionally
/// requires the original value with continuation-line boundaries preserved.
/// Names and expressions use the unfolded form; arbitrary prose is not guessed.
pub(super) fn interpret(value: &str, raw_text: Option<&str>) -> Result<LicenseExpression, Error> {
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
    if let Some(id) = raw_text.and_then(super::text_match::identify) {
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
