//! Project permissions and hash-bound overrides, independent of acquisition.
use crate::model::{
    ArtifactRequest, Code, ContentHash, Error, EvaluationDate, EvidenceRequirement, PackageId,
    SourceId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod unit;
pub use unit::PermissionUnit;

/// The SPDX identifier database compiled into this version.
pub const SPDX_LIST_VERSION: &str = spdx::identifiers::VERSION;

/// A validated Python SPDX expression. Unit equality never infers license compatibility.
#[derive(Clone)]
pub struct LicenseExpression {
    text: String,
    ast: spdx::Expression,
    units: BTreeSet<String>,
}
impl std::fmt::Debug for LicenseExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LicenseExpression")
            .field(&self.text)
            .finish()
    }
}
impl LicenseExpression {
    /// # Errors
    /// Rejects unknown identifiers, malformed expressions, and external document/addition references.
    pub fn parse(s: &str) -> Result<Self, Error> {
        let ast = spdx::Expression::parse_mode(
            s,
            spdx::ParseMode {
                allow_deprecated: true,
                ..spdx::ParseMode::STRICT
            },
        )
        .map_err(|_| Error::new(Code::UnknownLicense, "invalid SPDX expression"))?;
        for req in ast.requirements() {
            if let spdx::LicenseItem::Other(reference) = &req.req.license
                && reference.doc_ref.is_some()
            {
                return Err(Error::new(
                    Code::UnknownLicense,
                    "external SPDX document references are unsupported",
                ));
            }
            if req
                .req
                .addition
                .as_ref()
                .is_some_and(|a| !matches!(a, spdx::AdditionItem::Spdx(_)))
            {
                return Err(Error::new(
                    Code::UnknownLicense,
                    "custom SPDX addition references are unsupported",
                ));
            }
        }
        let units = ast.requirements().map(|r| r.req.to_string()).collect();
        Ok(Self {
            text: s.to_owned(),
            ast,
            units,
        })
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    /// Cached canonical string view for reporting and existing consumers.
    /// Policy permission storage uses validated `PermissionUnit` values.
    pub fn units(&self) -> &BTreeSet<String> {
        &self.units
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionOutcome {
    Permitted,
    Denied,
}
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct UnitDecision {
    pub unit: String,
    pub permitted: bool,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverrideKind {
    Exception,
    Clarification,
}
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct AppliedOverride {
    pub id: String,
    pub kind: OverrideKind,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct PolicyDecision {
    pub mode: PolicyMode,
    pub outcome: PermissionOutcome,
    pub units: Vec<UnitDecision>,
    pub overrides: Vec<AppliedOverride>,
}
impl PolicyDecision {
    #[must_use]
    pub fn is_permitted(&self) -> bool {
        self.outcome == PermissionOutcome::Permitted
    }
}

/// Global permission strategy, applied independently to every atomic SPDX unit.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Allow,
    Deny,
}
impl PolicyMode {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawPolicy {
    schema_version: u32,
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
    #[serde(default)]
    exceptions: Vec<RawException>,
    #[serde(default)]
    clarifications: Vec<RawClarification>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSource {
    kind: String,
    location: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawException {
    name: String,
    version: String,
    source: RawSource,
    allow: Vec<String>,
    reason: String,
    artifact_hash: Option<ContentHash>,
    expires: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawClarification {
    artifact_hash: ContentHash,
    expression: String,
    reason: String,
    expires: Option<String>,
}
#[derive(Debug, Clone)]
struct Exception {
    ordinal: usize,
    hash: Option<ContentHash>,
    allow: BTreeSet<PermissionUnit>,
    reason: String,
}
#[derive(Debug, Clone)]
pub struct Clarification {
    expression: LicenseExpression,
    reason: String,
}
impl Clarification {
    #[must_use]
    pub fn expression(&self) -> &LicenseExpression {
        &self.expression
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// One immutable workspace policy; validation includes all entries, even unmatched ones.
#[derive(Debug, Clone)]
pub struct Policy {
    mode: PolicyMode,
    units: BTreeSet<PermissionUnit>,
    exceptions: BTreeMap<PackageId, Vec<Exception>>,
    clarifications: BTreeMap<ContentHash, Clarification>,
}
fn check_entry(reason: &str, expires: Option<&str>, date: EvaluationDate) -> Result<(), Error> {
    if reason.trim().is_empty() {
        return Err(Error::new(
            Code::Configuration,
            "override reason must not be blank",
        ));
    }
    if let Some(expiry) = expires
        && EvaluationDate::parse(expiry)? < date
    {
        return Err(Error::new(
            Code::Expired,
            "policy contains an expired override",
        ));
    }
    Ok(())
}
fn permission_unit(text: &str, custom: bool) -> Result<PermissionUnit, Error> {
    let unit = PermissionUnit::parse(text)?;
    if !custom && unit.is_custom() {
        return Err(Error::new(
            Code::Configuration,
            "custom permissions require a hash-scoped exception",
        ));
    }
    Ok(unit)
}

impl Policy {
    /// Parse one workspace policy and validate all overrides on the supplied UTC day.
    /// Other pyproject tables are outside this policy's schema.
    /// # Errors
    /// Rejects missing/unknown fields, invalid permissions/selectors, duplicate clarifications,
    /// blank reasons, and expired overrides, including entries that match no dependency.
    pub fn from_pyproject(text: &str, date: EvaluationDate) -> Result<Self, Error> {
        let root: toml::Value = toml::from_str(text)
            .map_err(|_| Error::new(Code::Configuration, "invalid pyproject.toml"))?;
        let value = root
            .get("tool")
            .and_then(|v| v.get("taloon"))
            .ok_or_else(|| Error::new(Code::Configuration, "missing [tool.taloon] policy"))?;
        let raw: RawPolicy = value
            .clone()
            .try_into()
            .map_err(|_| Error::new(Code::Configuration, "invalid tool.taloon fields or schema"))?;
        if raw.schema_version != 1 {
            return Err(Error::new(
                Code::Configuration,
                "unsupported policy schema version",
            ));
        }
        let (mode, configured) = match (raw.allow, raw.deny) {
            (Some(units), None) => (PolicyMode::Allow, units),
            (None, Some(units)) => (PolicyMode::Deny, units),
            _ => {
                return Err(Error::new(
                    Code::Configuration,
                    "exactly one of allow or deny is required",
                ));
            }
        };
        let units = configured
            .iter()
            .map(|unit| permission_unit(unit, mode == PolicyMode::Deny))
            .collect::<Result<_, _>>()?;
        let mut exceptions: BTreeMap<PackageId, Vec<Exception>> = BTreeMap::new();
        for (position, e) in raw.exceptions.into_iter().enumerate() {
            check_entry(&e.reason, e.expires.as_deref(), date)?;
            let source = SourceId::parse(&e.source.kind, &e.source.location)?;
            let package = PackageId::parse(&e.name, &e.version, source)?;
            let allow = e
                .allow
                .iter()
                .map(|a| permission_unit(a, e.artifact_hash.is_some()))
                .collect::<Result<_, _>>()?;
            exceptions.entry(package).or_default().push(Exception {
                ordinal: position + 1,
                hash: e.artifact_hash,
                allow,
                reason: e.reason,
            });
        }
        let mut clarifications = BTreeMap::new();
        for c in raw.clarifications {
            check_entry(&c.reason, c.expires.as_deref(), date)?;
            let expression = LicenseExpression::parse(&c.expression)
                .map_err(|_| Error::new(Code::Configuration, "invalid clarification expression"))?;
            if clarifications
                .insert(
                    c.artifact_hash,
                    Clarification {
                        expression,
                        reason: c.reason,
                    },
                )
                .is_some()
            {
                return Err(Error::new(
                    Code::Configuration,
                    "duplicate clarification hash",
                ));
            }
        }
        Ok(Self {
            mode,
            units,
            exceptions,
            clarifications,
        })
    }
    /// Selected global strategy; the configured unit set remains private.
    #[must_use]
    pub fn mode(&self) -> PolicyMode {
        self.mode
    }
    #[must_use]
    /// Minimum proof required by hash-scoped policy entries for the locked content.
    pub fn minimum_evidence(&self, request: &ArtifactRequest) -> EvidenceRequirement {
        if let Some(hash) = request.hash()
            && (self.clarifications.contains_key(&hash)
                || self
                    .matching_exceptions(request, hash)
                    .any(|e| e.allow.iter().any(PermissionUnit::is_custom)))
        {
            return EvidenceRequirement::ArchiveVerified;
        }
        EvidenceRequirement::IndexAssociated
    }
    #[must_use]
    /// Lookup a content-bound replacement declaration; this does not grant permission.
    pub fn clarification(&self, hash: ContentHash) -> Option<&Clarification> {
        self.clarifications.get(&hash)
    }
    #[must_use]
    /// Evaluate exact units: deny entries prevail over scoped exceptions.
    /// Unlisted standard units are permitted in deny mode; custom units still need an exception.
    /// The caller must validate evidence and any clarification before invoking evaluation.
    pub fn evaluate(
        &self,
        request: &ArtifactRequest,
        hash: ContentHash,
        expression: &LicenseExpression,
    ) -> PolicyDecision {
        // Global units are immutable; only scoped matches need a per-call view.
        let matching: Vec<_> = self.matching_exceptions(request, hash).collect();
        let permits = |req: &spdx::LicenseReq| {
            let unit = req.to_string();
            let configured = self.units.contains(unit.as_str());
            let scoped = matching
                .iter()
                .any(|entry| entry.allow.contains(unit.as_str()));
            match self.mode {
                PolicyMode::Allow => configured || scoped,
                PolicyMode::Deny => {
                    !configured && (matches!(req.license, spdx::LicenseItem::Spdx { .. }) || scoped)
                }
            }
        };
        let overrides = matching
            .iter()
            .map(|entry| AppliedOverride {
                id: format!("exception[{}]", entry.ordinal),
                kind: OverrideKind::Exception,
                reason: entry.reason.clone(),
            })
            .collect();
        let permitted = expression.ast.evaluate(permits);
        PolicyDecision {
            mode: self.mode,
            outcome: if permitted {
                PermissionOutcome::Permitted
            } else {
                PermissionOutcome::Denied
            },
            units: expression
                .ast
                .requirements()
                .map(|requirement| (requirement.req.to_string(), permits(&requirement.req)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .map(|(unit, permitted)| UnitDecision { unit, permitted })
                .collect(),
            overrides,
        }
    }
    /// Package identity is owned by the map; ordered entries retain their file IDs.
    fn matching_exceptions<'a>(
        &'a self,
        request: &ArtifactRequest,
        hash: ContentHash,
    ) -> impl Iterator<Item = &'a Exception> {
        self.exceptions
            .get(request.package())
            .into_iter()
            .flatten()
            .filter(move |entry| entry.hash.is_none_or(|expected| expected == hash))
    }
    /// Called only after a complete run; all OR terms count as observed units.
    pub(crate) fn unused(
        &self,
        observations: &[(&ArtifactRequest, ContentHash, LicenseExpression)],
    ) -> Vec<Error> {
        // Retain only configured usage. A common all-used global-only policy
        // stops early, while exception/clarification checks share one input pass.
        let mut unused_units: BTreeSet<_> = self.units.iter().map(PermissionUnit::as_str).collect();
        let mut used_packages: BTreeMap<&PackageId, BTreeSet<ContentHash>> = BTreeMap::new();
        let mut unused_clarifications: BTreeSet<_> = self.clarifications.keys().copied().collect();
        for (request, hash, expression) in observations {
            if !unused_units.is_empty() {
                for unit in expression.units() {
                    unused_units.remove(unit.as_str());
                }
            }
            if !self.exceptions.is_empty() && self.exceptions.contains_key(request.package()) {
                used_packages
                    .entry(request.package())
                    .or_default()
                    .insert(*hash);
            }
            unused_clarifications.remove(hash);
            if unused_units.is_empty()
                && self.exceptions.is_empty()
                && unused_clarifications.is_empty()
            {
                break;
            }
        }
        let mut warnings: Vec<_> = unused_units
            .into_iter()
            .map(|unit| {
                Error::new(
                    Code::UnusedPermission,
                    match self.mode {
                        PolicyMode::Allow => format!("unused global permission: {unit}"),
                        PolicyMode::Deny => format!("unused global denial: {unit}"),
                    },
                )
            })
            .collect();
        let mut unused_exceptions = Vec::new();
        for (package, entries) in &self.exceptions {
            for entry in entries {
                if !used_packages
                    .get(package)
                    .is_some_and(|hashes| entry.hash.is_none_or(|hash| hashes.contains(&hash)))
                {
                    unused_exceptions.push(entry.ordinal);
                }
            }
        }
        // Map iteration groups by package; diagnostics retain original file order.
        unused_exceptions.sort_unstable();
        warnings.extend(unused_exceptions.into_iter().map(|ordinal| {
            Error::new(
                Code::UnusedPermission,
                format!("unused exception entry: {ordinal}"),
            )
        }));
        warnings.extend(unused_clarifications.into_iter().map(|hash| {
            Error::new(
                Code::UnusedPermission,
                format!("unused clarification: {hash}"),
            )
        }));
        warnings
    }
}
