//! Incomplete-over-violation precedence and its process exit interpretation.
#[cfg(creusot)]
use creusot_std::prelude::*;

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

/// Package report storage with application-owned payload types.
/// Aggregation borrows these records directly without extracting statuses.
#[cfg_attr(not(creusot), derive(Debug))]
#[cfg_attr(all(feature = "serde", not(creusot)), derive(serde::Serialize))]
pub struct PackageOutcome<P, S, A, D, L, N> {
    pub package: P,
    pub source: S,
    pub status: Status,
    pub artifacts: A,
    pub diagnostics: D,
    pub paths: L,
    pub origin_note: N,
}

/// Maximum package status, with input errors and incomplete packages absorbing.
#[must_use]
#[cfg_attr(creusot, ensures(
    (result == Status::Incomplete) == (input_error || exists<i: Int>
        0 <= i && i < packages@.len() && packages@[i].status == Status::Incomplete)
))]
#[cfg_attr(creusot, ensures(
    (result == Status::Compliant) == (!input_error && forall<i: Int>
        0 <= i && i < packages@.len() ==> packages@[i].status == Status::Compliant)
))]
#[cfg_attr(creusot, ensures(
    (result == Status::Violation) == (!input_error
        && (forall<i: Int> 0 <= i && i < packages@.len() ==> packages@[i].status != Status::Incomplete)
        && (exists<i: Int> 0 <= i && i < packages@.len() && packages@[i].status == Status::Violation))
))]
pub fn aggregate<P, S, A, D, L, N>(
    packages: &[PackageOutcome<P, S, A, D, L, N>],
    input_error: bool,
) -> Status {
    if input_error {
        return Status::Incomplete;
    }
    let mut status = Status::Compliant;
    let mut index = 0;
    #[cfg_attr(creusot, invariant(index@ <= packages@.len()))]
    #[cfg_attr(creusot, invariant(status != Status::Incomplete))]
    #[cfg_attr(creusot, invariant(forall<i: Int>
        0 <= i && i < index@ ==> packages@[i].status != Status::Incomplete
    ))]
    #[cfg_attr(creusot, invariant((status == Status::Compliant) == (forall<i: Int>
        0 <= i && i < index@ ==> packages@[i].status == Status::Compliant
    )))]
    #[cfg_attr(creusot, invariant((status == Status::Violation) == (exists<i: Int>
        0 <= i && i < index@ && packages@[i].status == Status::Violation
    )))]
    #[cfg_attr(creusot, variant(packages@.len() - index@))]
    while index < packages.len() {
        status = match packages[index].status {
            Status::Incomplete => return Status::Incomplete,
            Status::Violation => Status::Violation,
            Status::Compliant => status,
        };
        index += 1;
    }
    status
}
