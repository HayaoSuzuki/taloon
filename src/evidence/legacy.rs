//! Precise names and complete texts in legacy fields; SPDX expression syntax stays strict.
use crate::{model::Error, policy::LicenseExpression};
mod expression;

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
    // Additional spellings: docs/development/work/license-interpretation-extensions/plan.md.
    ("Mozilla Public License v1.0", "MPL-1.0"),
    ("Mozilla Public License v1.1", "MPL-1.1"),
    ("Mozilla Public License v2.0", "MPL-2.0"),
    ("Mozilla Public License Version 2.0", "MPL-2.0"),
    ("ZPL 1.1", "ZPL-1.1"),
    ("Zope Public License Version 1.1", "ZPL-1.1"),
    ("ZPL 2.0", "ZPL-2.0"),
    ("Zope Public License Version 2.0", "ZPL-2.0"),
    ("ZPL 2.1", "ZPL-2.1"),
    ("Zope Public License Version 2.1", "ZPL-2.1"),
    ("BSL 1.0", "BSL-1.0"),
    ("Boost Software License Version 1.0", "BSL-1.0"),
    ("Eclipse Public License Version 2.0", "EPL-2.0"),
    ("CDDL v1.0", "CDDL-1.0"),
    ("CDDL v1.1", "CDDL-1.1"),
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
    longest_unique_name(
        candidates
            .filter(|(name, _)| name.eq_ignore_ascii_case(value))
            .map(|(name, id)| (name.len(), id)),
    )
    .map(|(_, id)| id)
}

enum NameIdentity<'a> {
    Unique(&'a str),
    Ambiguous,
}

/// Resolve the longest matching spelling in one pass. Ambiguity at that length
/// cannot fall back to a shorter name; a longer spelling supersedes shorter conflicts.
fn longest_unique_name<'a>(
    candidates: impl Iterator<Item = (usize, &'a str)>,
) -> Option<(usize, &'a str)> {
    let mut best: Option<(usize, NameIdentity<'a>)> = None;
    for (length, id) in candidates {
        match &mut best {
            Some((best_length, _)) if *best_length > length => {}
            Some((best_length, identity)) if *best_length == length => {
                if matches!(identity, NameIdentity::Unique(previous) if *previous != id) {
                    *identity = NameIdentity::Ambiguous;
                }
            }
            _ => best = Some((length, NameIdentity::Unique(id))),
        }
    }
    match best? {
        (length, NameIdentity::Unique(id)) => Some((length, id)),
        (_, NameIdentity::Ambiguous) => None,
    }
}

fn names() -> impl Iterator<Item = (&'static str, &'static str)> {
    spdx::identifiers::LICENSES
        .iter()
        .map(|license| (license.full_name, license.name))
        .chain(ALIASES.iter().copied())
}

/// Interpret one unfolded legacy value. Complete-text matching additionally
/// requires the original value with continuation-line boundaries preserved.
/// Names and expressions use the unfolded form; arbitrary prose is not guessed.
pub(super) fn interpret(value: &str, raw_text: Option<&str>) -> Result<LicenseExpression, Error> {
    let error = match LicenseExpression::parse(value) {
        Ok(expression) => return Ok(expression),
        Err(error) => error,
    };
    if let Some(id) = unique_name(value, names()) {
        return LicenseExpression::parse(id);
    }
    if let Some(expression) = expression::parse(value) {
        return Ok(expression);
    }
    if let Some(id) = raw_text.and_then(super::text_match::identify) {
        return LicenseExpression::parse(id);
    }
    Err(error)
}

#[cfg(test)]
mod tests {
    use super::{longest_unique_name, unique_name};
    use proptest::prelude::*;
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

    // Independent batch oracle: select the maximum length, then collect all
    // distinct identities at that length instead of using incremental state.
    fn batch_resolution<'a>(candidates: &[(usize, &'a str)]) -> Option<(usize, &'a str)> {
        let length = candidates.iter().map(|(length, _)| *length).max()?;
        let ids: std::collections::BTreeSet<_> = candidates
            .iter()
            .filter(|(candidate_length, _)| *candidate_length == length)
            .map(|(_, id)| *id)
            .collect();
        (ids.len() == 1).then(|| (length, *ids.first().unwrap()))
    }

    proptest! {
        #[test]
        fn incremental_resolution_agrees_with_batch_identity_sets(
            entries in prop::collection::vec((0_usize..64, 0_usize..3), 0..128),
        ) {
            // Arrange
            let candidates: Vec<_> = entries.into_iter()
                .map(|(length, id)| (length, ["MIT", "ISC", "BSD-2-Clause"][id])).collect();
            let expected = batch_resolution(&candidates);
            // Act
            let actual = longest_unique_name(candidates.into_iter());
            // Assert
            prop_assert_eq!(actual, expected);
        }
    }
}
