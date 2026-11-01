//! Policy-free locked license observations and complete selected-scope accounting.
use crate::{
    evidence::{Evidence, EvidenceSummary},
    inspection::AcquisitionResult,
    model::{ArtifactRequest, Code, Error, RequestId, normalize_name, safe_locator},
    report::{AcquisitionSummary, Diagnostic, Severity},
    resolution::{DependencyGraph, PackageIdentity, Resolution, SelectionMode, Target},
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptionStatus {
    Complete,
    Incomplete,
}
#[derive(Debug, Serialize)]
pub struct DescribedArtifact {
    pub request_id: RequestId,
    pub locator: String,
    pub status: DescriptionStatus,
    pub evidence: Option<EvidenceSummary>,
    pub expression: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Debug, Serialize)]
pub struct LicenseGroup {
    pub expression: String,
    pub artifacts: Vec<RequestId>,
}
#[derive(Debug, Serialize)]
pub struct DescribedPackage {
    #[serde(flatten)]
    pub package: PackageIdentity,
    pub status: DescriptionStatus,
    pub licenses: Vec<LicenseGroup>,
    pub unknown_artifacts: usize,
    pub artifacts: Vec<DescribedArtifact>,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Debug, Serialize)]
pub struct DescriptionReport {
    schema_version: u32,
    command: &'static str,
    query: String,
    selection_mode: Option<SelectionMode>,
    roots: Vec<PackageIdentity>,
    status: DescriptionStatus,
    packages: Vec<DescribedPackage>,
    graph: DependencyGraph,
    diagnostics: Vec<Diagnostic>,
    acquisition: AcquisitionSummary,
}
impl DescriptionReport {
    #[must_use]
    pub fn from_input_error(query: &str, error: Error) -> Self {
        Self {
            schema_version: 1,
            command: "describe",
            query: normalize_name(query).unwrap_or_else(|_| safe_locator(query)),
            selection_mode: None,
            roots: vec![],
            status: DescriptionStatus::Incomplete,
            packages: vec![],
            graph: DependencyGraph::default(),
            diagnostics: vec![Diagnostic::error(error)],
            acquisition: AcquisitionSummary::default(),
        }
    }
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        if self.status == DescriptionStatus::Complete {
            0
        } else {
            2
        }
    }
    #[must_use]
    pub fn status(&self) -> DescriptionStatus {
        self.status
    }
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }
    #[must_use]
    pub fn packages(&self) -> &[DescribedPackage] {
        &self.packages
    }
    #[must_use]
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }
    #[must_use]
    pub fn roots(&self) -> &[PackageIdentity] {
        &self.roots
    }
    #[must_use]
    pub fn selection_mode(&self) -> Option<SelectionMode> {
        self.selection_mode
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
    pub fn with_acquisition_summary(mut self, summary: AcquisitionSummary) -> Self {
        self.acquisition = summary;
        self
    }
    #[must_use]
    pub fn with_warnings(mut self, warnings: Vec<Diagnostic>) -> Self {
        self.diagnostics.extend(warnings);
        self.refresh_status();
        self
    }
    fn refresh_status(&mut self) {
        self.status = if self
            .packages
            .iter()
            .any(|p| p.status == DescriptionStatus::Incomplete)
            || self
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error)
        {
            DescriptionStatus::Incomplete
        } else {
            DescriptionStatus::Complete
        };
    }
}
/// Select before acquisition. No policy, environment evaluation or I/O is involved.
pub struct Description {
    query: String,
    mode: SelectionMode,
    roots: Vec<PackageIdentity>,
    targets: Vec<Target>,
    requests: Vec<ArtifactRequest>,
    graph: DependencyGraph,
}
impl Description {
    /// # Errors
    /// Rejects invalid locks/names and names absent from the complete locked graph.
    pub fn from_uv_lock(lock: &str, name: &str) -> Result<Self, Error> {
        let resolution = Resolution::from_uv_lock(lock)?;
        let selected = resolution.select(name)?;
        let targets: Vec<_> = selected.targets.into_iter().cloned().collect();
        let requests = targets.iter().flat_map(Target::requests).cloned().collect();
        Ok(Self {
            query: selected.query,
            mode: selected.mode,
            roots: selected.roots,
            targets,
            requests,
            graph: selected.graph,
        })
    }
    #[must_use]
    pub fn requests(&self) -> &[ArtifactRequest] {
        &self.requests
    }
    /// Preserve every selected target and reject missing, duplicate, foreign or
    /// rebound observations. Different valid declarations never imply permission.
    #[must_use]
    pub fn evaluate(&self, results: Vec<AcquisitionResult>) -> DescriptionReport {
        let mut received: BTreeMap<RequestId, Vec<Result<Evidence, Error>>> = BTreeMap::new();
        for result in results {
            let (id, result) = result.into_parts();
            received.entry(id).or_default().push(result);
        }
        let mut packages = Vec::new();
        for target in &self.targets {
            let mut artifacts = Vec::new();
            let mut licenses: BTreeMap<String, Vec<RequestId>> = BTreeMap::new();
            for request in target.requests() {
                let result = match received.remove(&request.id()) {
                    None => Err(Error::new(
                        Code::MissingResult,
                        "no acquisition result for expected artifact",
                    )),
                    Some(mut results) if results.len() == 1 => results.remove(0),
                    Some(_) => Err(Error::new(
                        Code::InvalidResult,
                        "duplicate results for expected artifact",
                    )),
                };
                let artifact = describe_artifact(request, result);
                if let Some(expression) = &artifact.expression {
                    licenses
                        .entry(expression.clone())
                        .or_default()
                        .push(request.id());
                }
                artifacts.push(artifact);
            }
            artifacts.sort_by_key(|a| a.request_id);
            let unknown_artifacts = artifacts.iter().filter(|a| a.expression.is_none()).count();
            let diagnostics: Vec<_> = target
                .issues()
                .iter()
                .cloned()
                .map(Diagnostic::error)
                .collect();
            let status = if unknown_artifacts > 0 || !diagnostics.is_empty() {
                DescriptionStatus::Incomplete
            } else {
                DescriptionStatus::Complete
            };
            let licenses = licenses
                .into_iter()
                .map(|(expression, mut artifacts)| {
                    artifacts.sort();
                    LicenseGroup {
                        expression,
                        artifacts,
                    }
                })
                .collect();
            packages.push(DescribedPackage {
                package: target.package().into(),
                status,
                licenses,
                unknown_artifacts,
                artifacts,
                diagnostics,
            });
        }
        packages.sort_by(|a, b| a.package.id.cmp(&b.package.id));
        let roots = self.roots.clone();
        let graph = self.graph.clone();
        let diagnostics = received
            .keys()
            .map(|_| {
                Diagnostic::error(Error::new(
                    Code::InvalidResult,
                    "result does not belong to this description",
                ))
            })
            .collect();
        let mut report = DescriptionReport {
            schema_version: 1,
            command: "describe",
            query: self.query.clone(),
            selection_mode: Some(self.mode),
            roots,
            status: DescriptionStatus::Complete,
            packages,
            graph,
            diagnostics,
            acquisition: AcquisitionSummary::default(),
        };
        report.refresh_status();
        report
    }
}
fn describe_artifact(
    request: &ArtifactRequest,
    result: Result<Evidence, Error>,
) -> DescribedArtifact {
    let mut artifact = DescribedArtifact {
        request_id: request.id(),
        locator: request.locator().display(),
        status: DescriptionStatus::Incomplete,
        evidence: None,
        expression: None,
        diagnostics: vec![],
    };
    let interpreted = (|| {
        let evidence = result?;
        let evidence = Evidence::from_record(request, evidence.to_record())?;
        let summary = evidence.summary();
        artifact
            .diagnostics
            .extend(summary.warnings.iter().cloned().map(Diagnostic::warning));
        artifact.evidence = Some(summary);
        if !evidence.requirement_satisfied(request.requirement()) {
            return Err(Error::new(
                Code::InsufficientEvidence,
                "observation does not satisfy the required proof strength",
            ));
        }
        artifact.expression = Some(evidence.license()?.text().into());
        artifact.status = DescriptionStatus::Complete;
        Ok(())
    })();
    if let Err(error) = interpreted {
        artifact.diagnostics.push(Diagnostic::error(error));
    }
    artifact
}
