//! Finite alternatives within SPDX v3.29.0 BSD-2/3-Clause template fields.
//! Whole-body equality still checks every other word and all clause ordering.
use std::borrow::Cow;

/// A prepared BSD body. Construction and comparison share the same finite
/// normalization rules; callers never receive or supply a normalized string.
pub(super) struct Body(String);

impl Body {
    pub(super) fn new(canonical: &str) -> Self {
        Self(normalize(canonical).into_owned())
    }

    pub(super) fn matches(&self, input: &str) -> bool {
        normalize(input) == self.0
    }
}

/// Canonicalize only reviewed generic roles and known clause markers. Arbitrary
/// organizations and the upstream templates' wildcards are deliberately absent.
fn normalize(input: &str) -> Cow<'_, str> {
    let mut text = Cow::Borrowed(input);
    for (from, to) in [
        (
            "name of the copyright holders, nor those of its contributors",
            "name of the copyright holder nor the names of its contributors",
        ),
        ("name of the author nor", "name of the copyright holder nor"),
        (
            "name of the authors nor",
            "name of the copyright holder nor",
        ),
        (
            "name of the copyright owner nor",
            "name of the copyright holder nor",
        ),
        (
            "name of the copyright owners nor",
            "name of the copyright holder nor",
        ),
        (
            "name of the copyright holders nor",
            "name of the copyright holder nor",
        ),
        (
            "nor the names of their contributors",
            "nor the names of its contributors",
        ),
        (
            "nor the names of any contributors",
            "nor the names of its contributors",
        ),
        (
            "BY THE AUTHOR AND CONTRIBUTORS",
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
        ),
        (
            "BY THE AUTHORS AND CONTRIBUTORS",
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
        ),
        (
            "BY THE COPYRIGHT OWNER AND CONTRIBUTORS",
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
        ),
        (
            "BY THE COPYRIGHT OWNERS AND CONTRIBUTORS",
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
        ),
        (
            "BY THE COPYRIGHT HOLDER AND CONTRIBUTORS",
            "BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS",
        ),
        (
            "THE AUTHOR OR CONTRIBUTORS",
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
        ),
        (
            "THE AUTHORS OR CONTRIBUTORS",
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
        ),
        (
            "THE COPYRIGHT OWNER OR CONTRIBUTORS",
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
        ),
        (
            "THE COPYRIGHT OWNERS OR CONTRIBUTORS",
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
        ),
        (
            "THE COPYRIGHT HOLDERS OR CONTRIBUTORS",
            "THE COPYRIGHT HOLDER OR CONTRIBUTORS",
        ),
        ("1. Redistributions", "Redistributions"),
        ("2. Redistributions", "Redistributions"),
        ("* Redistributions", "Redistributions"),
        ("- Redistributions", "Redistributions"),
        ("3. Neither", "Neither"),
        ("* Neither", "Neither"),
        ("- Neither", "Neither"),
    ] {
        if text.contains(from) {
            text = Cow::Owned(text.replace(from, to));
        }
    }
    text
}
