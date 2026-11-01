//! Incomplete-over-violation precedence and its process exit interpretation.
#[cfg(creusot)]
use creusot_std::prelude::ensures;

/// Inspection outcome independent of diagnostics, rendering and acquisition.
#[cfg_attr(not(creusot), derive(Clone, Copy, Debug, Eq, PartialEq))]
#[cfg_attr(all(feature = "serde", not(creusot)), derive(serde::Serialize))]
#[cfg_attr(all(feature = "serde", not(creusot)), serde(rename_all = "snake_case"))]
pub enum Status {
    Compliant,
    Violation,
    Incomplete,
}

impl Status {
    /// Combine outcomes without allowing a violation to hide missing inspection.
    #[must_use]
    #[cfg_attr(creusot, ensures(
        (result == Status::Incomplete) == (self == Status::Incomplete || other == Status::Incomplete)
    ))]
    #[cfg_attr(creusot, ensures(
        (result == Status::Compliant) == (self == Status::Compliant && other == Status::Compliant)
    ))]
    #[cfg_attr(creusot, ensures(
        (result == Status::Violation) == (
            self != Status::Incomplete && other != Status::Incomplete
            && (self == Status::Violation || other == Status::Violation)
        )
    ))]
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Incomplete, _) | (_, Self::Incomplete) => Self::Incomplete,
            (Self::Violation, _) | (_, Self::Violation) => Self::Violation,
            (Self::Compliant, Self::Compliant) => Self::Compliant,
        }
    }

    /// Warnings can turn compliance into exit 1, but cannot hide incompleteness.
    #[must_use]
    #[cfg_attr(creusot, ensures(result <= 2u8))]
    #[cfg_attr(creusot, ensures((result == 2u8) == (self == Status::Incomplete)))]
    #[cfg_attr(creusot, ensures(
        (result == 0u8) == (self == Status::Compliant && !(deny_warnings && has_warnings))
    ))]
    #[cfg_attr(creusot, ensures(
        (result == 1u8) == (self == Status::Violation
            || (self == Status::Compliant && deny_warnings && has_warnings))
    ))]
    pub fn exit_code(self, deny_warnings: bool, has_warnings: bool) -> u8 {
        match self {
            Self::Incomplete => 2,
            Self::Violation => 1,
            Self::Compliant => u8::from(deny_warnings && has_warnings),
        }
    }
}
