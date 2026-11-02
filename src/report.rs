//! Stable schema and exit precedence; acquisition counters never decide compliance.
use crate::{
    evidence::EvidenceSummary,
    model::{ContentHash, Error, RequestId, SourceId, SourceKind, safe_locator},
    policy::{PolicyDecision, PolicyMode},
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
pub use taloon_decisions::accounting::{AcquisitionSummary, HttpSummary, TransferSummary};
pub use taloon_decisions::status::Status;
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
#[derive(Clone, Debug, Serialize)]
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
pub type PackageReport = taloon_decisions::status::PackageOutcome<
    String,
    SourceSummary,
    Vec<ArtifactReport>,
    Vec<Diagnostic>,
    Vec<DependencyPath>,
    Option<String>,
>;
#[derive(Debug, Serialize)]
pub struct Report {
    schema_version: u32,
    status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_mode: Option<PolicyMode>,
    packages: Vec<PackageReport>,
    diagnostics: Vec<Diagnostic>,
    acquisition: AcquisitionSummary,
}
impl Report {
    pub(crate) fn new(packages: Vec<PackageReport>, diagnostics: Vec<Diagnostic>) -> Self {
        let input_error = diagnostics.iter().any(|d| d.severity == Severity::Error);
        let status = taloon_decisions::status::aggregate(&packages, input_error);
        Self {
            schema_version: 1,
            policy_mode: None,
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
    /// Selected strategy, absent when policy validation did not complete.
    #[must_use]
    pub fn policy_mode(&self) -> Option<PolicyMode> {
        self.policy_mode
    }
    pub(crate) fn with_policy_mode(mut self, mode: PolicyMode) -> Self {
        self.policy_mode = Some(mode);
        self
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
        let has_warnings = self.status == Status::Compliant && deny_warnings && self.has_warnings();
        self.status.exit_code(deny_warnings, has_warnings)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Code, SourceKind};
    use proptest::prelude::*;
    use rstest::rstest;

    fn package(status: Status) -> PackageReport {
        PackageReport {
            package: "fixture".into(),
            source: SourceSummary {
                kind: SourceKind::Registry,
                location: "https://example.test/simple".into(),
            },
            status,
            artifacts: Vec::new(),
            diagnostics: Vec::new(),
            paths: Vec::new(),
            origin_note: None,
        }
    }

    #[test]
    fn package_report_preserves_named_field_serialization() {
        // Arrange
        let package = package(Status::Violation);
        let expected = serde_json::json!({
            "package": "fixture",
            "source": { "kind": "registry", "location": "https://example.test/simple" },
            "status": "violation",
            "artifacts": [],
            "diagnostics": [],
            "paths": [],
            "origin_note": null,
        });
        // Act
        let actual = serde_json::to_value(package).unwrap();
        // Assert
        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case::compliant(Status::Compliant, "compliant")]
    #[case::violation(Status::Violation, "violation")]
    #[case::incomplete(Status::Incomplete, "incomplete")]
    fn status_serialization_is_unchanged(#[case] status: Status, #[case] expected: &str) {
        // Arrange: a public report status.
        // Act
        let actual = serde_json::to_value(status).unwrap();
        // Assert
        assert_eq!(actual, expected);
    }

    proptest! {
        #[test]
        fn report_aggregation_matches_independent_rank_oracle(
            ranks in prop::collection::vec(0_u8..=2, 0..100),
            input_error in any::<bool>(),
            warning in any::<bool>(),
            deny in any::<bool>(),
        ) {
            // Arrange
            let packages = ranks.iter().map(|rank| package(match rank {
                0 => Status::Compliant,
                1 => Status::Violation,
                _ => Status::Incomplete,
            })).collect();
            let mut diagnostics = Vec::new();
            if input_error {
                diagnostics.push(Diagnostic::error(Error::new(Code::Configuration, "fixture")));
            }
            if warning {
                diagnostics.push(Diagnostic::warning(Error::new(Code::UnusedPermission, "fixture")));
            }
            let expected = ranks.into_iter().chain([
                u8::from(input_error) * 2,
                u8::from(deny && warning),
            ]).max().unwrap();
            // Act
            let actual = Report::new(packages, diagnostics).exit_code(deny);
            // Assert
            prop_assert_eq!(actual, expected);
        }
    }
}
