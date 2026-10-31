//! Stable schema and exit precedence; acquisition counters never decide compliance.
use crate::{
    evidence::EvidenceSummary,
    model::{ContentHash, Error, RequestId, SourceId, SourceKind, safe_locator},
    policy::PolicyDecision,
    resolution::DependencyPath,
};
use serde::Serialize;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}
#[derive(Clone, Debug, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    #[serde(flatten)]
    pub error: Error,
}
impl Diagnostic {
    #[must_use]
    pub fn warning(error: Error) -> Self {
        Self {
            severity: Severity::Warning,
            error,
        }
    }
    #[must_use]
    pub fn error(error: Error) -> Self {
        Self {
            severity: Severity::Error,
            error,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Compliant,
    Violation,
    Incomplete,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct AcquisitionSummary {
    pub network_requests: u64,
    pub downloaded_bytes: u64,
    pub archive_extractions: u64,
    pub cache_hits: u64,
}
#[derive(Debug, Serialize)]
pub struct ArtifactReport {
    pub request_id: RequestId,
    pub locator: String,
    pub locked_hash: Option<ContentHash>,
    pub status: Status,
    pub evidence: Option<EvidenceSummary>,
    pub interpreted_expression: Option<String>,
    pub decision: Option<PolicyDecision>,
    pub diagnostics: Vec<Diagnostic>,
}
/// Safe display context for a source; fetch/authentication details remain private.
#[derive(Debug, Serialize)]
pub struct SourceSummary {
    pub kind: SourceKind,
    pub location: String,
}
impl From<&SourceId> for SourceSummary {
    fn from(source: &SourceId) -> Self {
        Self {
            kind: source.kind(),
            location: safe_locator(source.location()),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct PackageReport {
    pub package: String,
    pub source: SourceSummary,
    pub status: Status,
    pub artifacts: Vec<ArtifactReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub paths: Vec<DependencyPath>,
    pub origin_note: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    schema_version: u32,
    status: Status,
    packages: Vec<PackageReport>,
    diagnostics: Vec<Diagnostic>,
    acquisition: AcquisitionSummary,
}
impl Report {
    pub(crate) fn new(packages: Vec<PackageReport>, diagnostics: Vec<Diagnostic>) -> Self {
        let status = if diagnostics.iter().any(|d| d.severity == Severity::Error)
            || packages.iter().any(|p| p.status == Status::Incomplete)
        {
            Status::Incomplete
        } else if packages.iter().any(|p| p.status == Status::Violation) {
            Status::Violation
        } else {
            Status::Compliant
        };
        Self {
            schema_version: 1,
            status,
            packages,
            diagnostics,
            acquisition: AcquisitionSummary::default(),
        }
    }
    #[must_use]
    pub fn from_input_error(error: Error) -> Self {
        Self::new(vec![], vec![Diagnostic::error(error)])
    }
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }
    #[must_use]
    pub fn packages(&self) -> &[PackageReport] {
        &self.packages
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    #[must_use]
    pub fn acquisition(&self) -> &AcquisitionSummary {
        &self.acquisition
    }
    #[must_use]
    pub fn exit_code(&self, deny_warnings: bool) -> u8 {
        match self.status {
            Status::Incomplete => 2,
            Status::Violation => 1,
            Status::Compliant => u8::from(deny_warnings && self.has_warnings()),
        }
    }
    fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .chain(self.packages.iter().flat_map(|p| {
                p.diagnostics
                    .iter()
                    .chain(p.artifacts.iter().flat_map(|a| a.diagnostics.iter()))
            }))
            .any(|d| d.severity == Severity::Warning)
    }
    #[must_use]
    pub fn with_warnings(mut self, warnings: Vec<Diagnostic>) -> Self {
        self.diagnostics.extend(warnings.into_iter().map(|mut d| {
            d.severity = Severity::Warning;
            d
        }));
        self.diagnostics.sort_by(|a, b| {
            format!("{:?}:{}", a.error.code, a.error.message)
                .cmp(&format!("{:?}:{}", b.error.code, b.error.message))
        });
        self
    }
    #[must_use]
    pub fn with_acquisition_summary(mut self, summary: AcquisitionSummary) -> Self {
        self.acquisition = summary;
        self
    }
}
