//! Expected requests, complete coverage accounting, and policy evaluation without I/O.
use crate::{
    evidence::Evidence,
    model::{ArtifactRequest, Code, Error, EvaluationDate, EvidenceRequirement, RequestId},
    policy::{AppliedOverride, LicenseExpression, OverrideKind, Policy},
    report::{ArtifactReport, Diagnostic, PackageReport, Report, Status},
    resolution::Resolution,
};
use std::collections::BTreeMap;
/// An association is required for each requested artifact, even when bytes are shared.
pub struct AcquisitionResult {
    id: RequestId,
    result: Result<Evidence, Error>,
}
impl AcquisitionResult {
    #[must_use]
    pub fn observed(id: RequestId, evidence: Evidence) -> Self {
        Self {
            id,
            result: Ok(evidence),
        }
    }
    #[must_use]
    pub fn failed(id: RequestId, error: Error) -> Self {
        Self {
            id,
            result: Err(error),
        }
    }
}
pub struct Inspection {
    resolution: Resolution,
    policy: Policy,
    requests: Vec<ArtifactRequest>,
}
impl Inspection {
    /// # Errors
    /// Rejects unsupported lock documents and invalid or expired policy entries.
    pub fn from_documents(
        lock: &str,
        pyproject: &str,
        date: EvaluationDate,
    ) -> Result<Self, Error> {
        let resolution = Resolution::from_uv_lock(lock)?;
        let policy = Policy::from_pyproject(pyproject, date)?;
        let requests = resolution
            .targets()
            .iter()
            .flat_map(crate::resolution::Target::requests)
            .map(|r| r.clone().requiring(policy.minimum_evidence(r)))
            .collect();
        Ok(Self {
            resolution,
            policy,
            requests,
        })
    }
    #[must_use]
    pub fn requests(&self) -> &[ArtifactRequest] {
        &self.requests
    }
    /// Account for exactly one result per expected request and evaluate every target.
    /// Observations are rebound to their original identity and minimum proof requirement;
    /// missing, duplicate, foreign, or insufficient results make the report incomplete.
    #[must_use]
    pub fn evaluate(&self, results: Vec<AcquisitionResult>) -> Report {
        let mut received: BTreeMap<RequestId, Vec<Result<Evidence, Error>>> = BTreeMap::new();
        for r in results {
            received.entry(r.id).or_default().push(r.result);
        }
        let mut packages = Vec::new();
        let mut interpreted = Vec::new();
        for target in self.resolution.targets() {
            let mut artifacts = Vec::new();
            for request in target.requests() {
                let result = match received.remove(&request.id()) {
                    None => Err(Error::new(
                        Code::MissingResult,
                        "no acquisition result for expected artifact",
                    )),
                    Some(mut values) if values.len() == 1 => values.remove(0),
                    Some(_) => Err(Error::new(
                        Code::InvalidResult,
                        "duplicate results for expected artifact",
                    )),
                };
                let artifact = self.evaluate_artifact(request, result, &mut interpreted);
                artifacts.push(artifact);
            }
            artifacts.sort_by_key(|a| a.request_id);
            let diagnostics: Vec<_> = target
                .issues()
                .iter()
                .cloned()
                .map(Diagnostic::error)
                .collect();
            let status = if !diagnostics.is_empty()
                || artifacts.iter().any(|a| a.status == Status::Incomplete)
            {
                Status::Incomplete
            } else if artifacts.iter().any(|a| a.status == Status::Violation) {
                Status::Violation
            } else {
                Status::Compliant
            };
            let paths = if status == Status::Compliant {
                vec![]
            } else {
                self.resolution.paths(target.package())
            };
            let origin_note = (status != Status::Compliant && paths.is_empty())
                .then(|| "No unambiguous dependency origin is recorded in the lock.".to_owned());
            packages.push(PackageReport {
                package: target.package().key(),
                source: target.package().source().into(),
                status,
                artifacts,
                diagnostics,
                paths,
                origin_note,
            });
        }
        packages.sort_by(|a, b| a.package.cmp(&b.package));
        let diagnostics = received
            .keys()
            .map(|_| {
                Diagnostic::error(Error::new(
                    Code::InvalidResult,
                    "result does not belong to this inspection",
                ))
            })
            .collect();
        let report = Report::new(packages, diagnostics);
        if report.status() == Status::Incomplete {
            report
        } else {
            report.with_warnings(
                self.policy
                    .unused(&interpreted)
                    .into_iter()
                    .map(Diagnostic::warning)
                    .collect(),
            )
        }
    }
    fn evaluate_artifact<'a>(
        &self,
        request: &'a ArtifactRequest,
        result: Result<Evidence, Error>,
        interpreted: &mut Vec<(
            &'a ArtifactRequest,
            crate::model::ContentHash,
            LicenseExpression,
        )>,
    ) -> ArtifactReport {
        let mut report = ArtifactReport {
            request_id: request.id(),
            locator: request.locator().display(),
            locked_hash: request.hash(),
            status: Status::Incomplete,
            evidence: None,
            interpreted_expression: None,
            decision: None,
            diagnostics: vec![],
        };
        let evaluation = (|| {
            // Rebind observations through the evidence owner: caller-supplied result IDs are not proof.
            let evidence = result?;
            let evidence = Evidence::from_record(request, evidence.to_record())?;
            report.evidence = Some(evidence.summary());
            report.diagnostics.extend(
                evidence
                    .summary()
                    .warnings
                    .into_iter()
                    .map(Diagnostic::warning),
            );
            let hash = evidence.content_hash();
            let clarification = self.policy.clarification(request, hash);
            let need = if clarification.is_some() {
                EvidenceRequirement::ArchiveVerified
            } else {
                self.policy.minimum_evidence(request)
            };
            if !evidence.requirement_satisfied(need) {
                return Err(Error::new(
                    Code::InsufficientEvidence,
                    "observation does not satisfy the required proof strength",
                ));
            }
            let expression = if let Some(c) = clarification {
                c.expression().clone()
            } else {
                evidence.license()?
            };
            let mut decision = self.policy.evaluate(request, hash, &expression);
            if let Some(c) = clarification {
                decision.overrides.push(AppliedOverride {
                    id: format!("clarification:{hash}"),
                    kind: OverrideKind::Clarification,
                    reason: c.reason().into(),
                });
            }
            report.status = if decision.is_permitted() {
                Status::Compliant
            } else {
                Status::Violation
            };
            report.interpreted_expression = Some(expression.text().into());
            report.decision = Some(decision);
            interpreted.push((request, hash, expression));
            Ok(())
        })();
        if let Err(e) = evaluation {
            report.diagnostics.push(Diagnostic::error(e));
        }
        report
    }
}
