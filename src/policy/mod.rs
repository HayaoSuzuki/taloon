//! Project permissions and hash-bound overrides, independent of acquisition.
use crate::model::{
    ArtifactRequest, Code, ContentHash, Error, EvaluationDate, EvidenceRequirement, PackageId,
    SourceId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawPolicy {
    schema_version: u32,
    allow: Vec<String>,
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
    package: PackageId,
    hash: Option<ContentHash>,
    allow: BTreeSet<String>,
    reason: String,
}
impl Exception {
    fn matches(&self, request: &ArtifactRequest, hash: ContentHash) -> bool {
        self.package == *request.package() && self.hash.is_none_or(|h| h == hash)
    }
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
    allow: BTreeSet<String>,
    exceptions: Vec<Exception>,
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
fn permission_unit(text: &str, custom: bool) -> Result<String, Error> {
    let expr = LicenseExpression::parse(text)
        .map_err(|_| Error::new(Code::Configuration, "invalid permission unit"))?;
    if expr.ast.iter().count() != 1 {
        return Err(Error::new(
            Code::Configuration,
            "permission entries must be single license/WITH units",
        ));
    }
    let unit = expr
        .units
        .iter()
        .next()
        .ok_or_else(|| Error::new(Code::Configuration, "empty permission"))?;
    if !custom && unit.contains("LicenseRef-") {
        return Err(Error::new(
            Code::Configuration,
            "custom permissions require a hash-scoped exception",
        ));
    }
    Ok(unit.clone())
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
        let allow = raw
            .allow
            .iter()
            .map(|a| permission_unit(a, false))
            .collect::<Result<_, _>>()?;
        let mut exceptions = Vec::new();
        for e in raw.exceptions {
            check_entry(&e.reason, e.expires.as_deref(), date)?;
            let source = SourceId::parse(&e.source.kind, &e.source.location)?;
            let package = PackageId::parse(&e.name, &e.version, source)?;
            let allow = e
                .allow
                .iter()
                .map(|a| permission_unit(a, e.artifact_hash.is_some()))
                .collect::<Result<_, _>>()?;
            exceptions.push(Exception {
                package,
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
            allow,
            exceptions,
            clarifications,
        })
    }
    #[must_use]
    /// Minimum proof required by hash-scoped policy entries for the locked content.
    pub fn minimum_evidence(&self, request: &ArtifactRequest) -> EvidenceRequirement {
        if let Some(hash) = request.hash()
            && (self.clarifications.contains_key(&hash)
                || self.exceptions.iter().any(|e| {
                    e.matches(request, hash) && e.allow.iter().any(|a| a.contains("LicenseRef-"))
                }))
        {
            return EvidenceRequirement::ArchiveVerified;
        }
        EvidenceRequirement::IndexAssociated
    }
    #[must_use]
    /// Lookup a content-bound replacement declaration; this does not grant permission.
    pub fn clarification(
        &self,
        _request: &ArtifactRequest,
        hash: ContentHash,
    ) -> Option<&Clarification> {
        self.clarifications.get(&hash)
    }
    #[must_use]
    /// Evaluate explicit permission units with additive, exact-identity exceptions.
    /// The caller must validate evidence and any clarification before invoking evaluation.
    pub fn evaluate(
        &self,
        request: &ArtifactRequest,
        hash: ContentHash,
        expression: &LicenseExpression,
    ) -> PolicyDecision {
        let mut allowed = self.allow.clone();
        let mut overrides = Vec::new();
        for (index, e) in self.exceptions.iter().enumerate() {
            if e.matches(request, hash) {
                allowed.extend(e.allow.iter().cloned());
                overrides.push(AppliedOverride {
                    id: format!("exception[{}]", index + 1),
                    kind: OverrideKind::Exception,
                    reason: e.reason.clone(),
                });
            }
        }
        let permitted = expression
            .ast
            .evaluate(|req| allowed.contains(&req.to_string()));
        PolicyDecision {
            outcome: if permitted {
                PermissionOutcome::Permitted
            } else {
                PermissionOutcome::Denied
            },
            units: expression
                .units
                .iter()
                .map(|unit| UnitDecision {
                    unit: unit.clone(),
                    permitted: allowed.contains(unit),
                })
                .collect(),
            overrides,
        }
    }
    /// Called only after a complete run; all OR terms count as observed units.
    pub(crate) fn unused(
        &self,
        observations: &[(&ArtifactRequest, ContentHash, LicenseExpression)],
    ) -> Vec<Error> {
        let mut warnings = Vec::new();
        for unit in &self.allow {
            if !observations
                .iter()
                .any(|(_, _, expression)| expression.units().contains(unit))
            {
                warnings.push(Error::new(
                    Code::UnusedPermission,
                    format!("unused global permission: {unit}"),
                ));
            }
        }
        for (index, exception) in self.exceptions.iter().enumerate() {
            if !observations
                .iter()
                .any(|(request, hash, _)| exception.matches(request, *hash))
            {
                warnings.push(Error::new(
                    Code::UnusedPermission,
                    format!("unused exception entry: {}", index + 1),
                ));
            }
        }
        for hash in self.clarifications.keys() {
            if !observations.iter().any(|(_, observed, _)| observed == hash) {
                warnings.push(Error::new(
                    Code::UnusedPermission,
                    format!("unused clarification: {hash}"),
                ));
            }
        }
        warnings
    }
}
