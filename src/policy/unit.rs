//! Atomic permissions validated by the existing expression owner and SPDX parser.
use super::LicenseExpression;
use crate::model::{Code, Error};
use std::{borrow::Borrow, cmp::Ordering, fmt};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Standard,
    CustomReference,
}

/// One validated license or license/WITH permission, never an AND/OR expression.
///
/// Identity and ordering use canonical spelling, preserving supported deprecated
/// identifiers and suffix distinctions. This keeps equality, ordering and borrowed string lookup coherent
/// without depending on upstream requirement comparison details. Classification
/// comes from the parsed SPDX variant; the catalogue remains owned by spdx.
#[derive(Clone, Debug)]
pub struct PermissionUnit {
    canonical: String,
    kind: Kind,
}
impl PermissionUnit {
    /// Validate an atomic Python permission using the current SPDX database.
    /// Custom references are valid units; Policy separately restricts their scope.
    /// # Errors
    /// Rejects invalid/compound permissions, external document references and
    /// unsupported custom additions with a configuration diagnostic.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let expression = LicenseExpression::parse(text)
            .map_err(|_| Error::new(Code::Configuration, "invalid permission unit"))?;
        if expression.ast.iter().count() != 1 {
            return Err(Error::new(
                Code::Configuration,
                "permission entries must be single license/WITH units",
            ));
        }
        let requirement = expression
            .ast
            .requirements()
            .next()
            .ok_or_else(|| Error::new(Code::Configuration, "empty permission"))?;
        let kind = match &requirement.req.license {
            spdx::LicenseItem::Spdx { .. } => Kind::Standard,
            spdx::LicenseItem::Other(_) => Kind::CustomReference,
        };
        let canonical = expression
            .units
            .into_iter()
            .next()
            .ok_or_else(|| Error::new(Code::Configuration, "empty permission"))?;
        Ok(Self { canonical, kind })
    }

    /// Canonical spelling used by policy matching and stable report ordering.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Whether the parsed license is a local custom reference, including WITH.
    #[must_use]
    pub fn is_custom(&self) -> bool {
        matches!(self.kind, Kind::CustomReference)
    }
}
impl Borrow<str> for PermissionUnit {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for PermissionUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
impl PartialEq for PermissionUnit {
    fn eq(&self, other: &Self) -> bool {
        self.canonical == other.canonical
    }
}
impl Eq for PermissionUnit {}
impl Ord for PermissionUnit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.canonical.cmp(&other.canonical)
    }
}
impl PartialOrd for PermissionUnit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
