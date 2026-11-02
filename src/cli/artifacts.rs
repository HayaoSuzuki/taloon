//! Shared artifact presentation; grouping never participates in evaluation.
use super::safe;
use std::{collections::BTreeMap, fmt::Write as _};
use taloon::{
    description::{DescribedArtifact, DescriptionStatus},
    evidence::{EvidenceSummary, LicenseOrigin},
    model::{Code, LicenseProblem},
    policy::PolicyDecision,
    report::{ArtifactReport, Diagnostic, Status},
};

/// Borrow just the reporting facts shared by check and describe.
pub(super) struct Artifact<'a> {
    locator: &'a str,
    status: &'static str,
    expression: Option<&'a str>,
    evidence: Option<&'a EvidenceSummary>,
    decision: Option<&'a PolicyDecision>,
    diagnostics: &'a [Diagnostic],
}
impl<'a> From<&'a ArtifactReport> for Artifact<'a> {
    fn from(value: &'a ArtifactReport) -> Self {
        Self {
            locator: &value.locator,
            status: match value.status {
                Status::Compliant => "Compliant",
                Status::Violation => "Violation",
                Status::Incomplete => "Incomplete",
            },
            expression: value.interpreted_expression.as_deref(),
            evidence: value.evidence.as_ref(),
            decision: value.decision.as_ref(),
            diagnostics: &value.diagnostics,
        }
    }
}
impl<'a> From<&'a DescribedArtifact> for Artifact<'a> {
    fn from(value: &'a DescribedArtifact) -> Self {
        Self {
            locator: &value.locator,
            status: match value.status {
                DescriptionStatus::Complete => "Complete",
                DescriptionStatus::Incomplete => "Incomplete",
            },
            expression: value.expression.as_deref(),
            evidence: value.evidence.as_ref(),
            decision: None,
            diagnostics: &value.diagnostics,
        }
    }
}

fn remedy(problem: LicenseProblem) -> &'static str {
    match problem {
        LicenseProblem::MissingDeclaration => {
            "Acquire bundled evidence online if available, or review an exact artifact for a hash-bound clarification."
        }
        LicenseProblem::RepeatedDeclaration => {
            "Review the conflicting/repeated metadata fields; request corrected metadata or add a reviewed hash-bound clarification."
        }
        LicenseProblem::InvalidExpression => {
            "Review License-Expression syntax/identifiers; a legacy License cannot replace it. Use corrected metadata or a reviewed hash-bound clarification."
        }
        LicenseProblem::UnrecognizedLegacy => {
            "Review the original License and bundled terms before adding a hash-bound clarification; do not discard additional conditions."
        }
        LicenseProblem::LicenseFilesUnavailable => {
            "Inspect selected license paths and reading limits with --verbose; review the exact artifact before a hash-bound clarification."
        }
        LicenseProblem::UnrecognizedLicenseText => {
            "Review the complete bundled text; unsupported wording is not permission. Use a reviewed hash-bound clarification if appropriate."
        }
        LicenseProblem::ConflictingLicenseFiles => {
            "Review every selected file and their relationship; do not choose one license or invent AND/OR. A hash-bound clarification needs that review."
        }
        LicenseProblem::LegacyFamilyMismatch => {
            "Review the disagreement between the legacy family and bundled text before a hash-bound clarification."
        }
    }
}
fn declaration(out: &mut String, name: &str, value: &str, verbose: bool) {
    let end = (!verbose).then(|| value.char_indices().nth(200)).flatten();
    let preview = end.map_or(value, |(end, _)| &value[..end]);
    let _ = writeln!(
        out,
        "    {name}: {}{}",
        safe(preview),
        if end.is_some() {
            "... (full value: --verbose or --format json)"
        } else {
            ""
        }
    );
}
fn body(artifact: &Artifact<'_>, verbose: bool) -> String {
    let mut out = String::new();
    if let Some(e) = artifact.evidence {
        let _ = writeln!(out, "    evidence: {:?}", e.strength);
        if verbose {
            let _ = writeln!(out, "    observed {}", e.content_hash);
            let _ = writeln!(out, "    metadata {}", e.metadata_hash);
            for file in &e.license_files {
                let _ = writeln!(
                    out,
                    "    file {} {} {:?}: {:?}",
                    safe(&file.path),
                    file.content_hash,
                    file.kind,
                    file.matched_expression
                );
            }
        }
        if e.license_origin == Some(LicenseOrigin::LegacySupplement) {
            let _ = writeln!(
                out,
                "    license source: legacy declaration supplemented by bundled files"
            );
        } else if let Some(origin) = e.license_origin {
            let _ = writeln!(out, "    license source: {origin:?}");
        }
        for value in &e.original_expression {
            declaration(&mut out, "License-Expression", value, verbose);
        }
        for value in &e.original_license {
            declaration(&mut out, "License", value, verbose);
        }
        if e.original_expression.is_empty() && e.original_license.is_empty() {
            let _ = writeln!(out, "    declarations: absent");
        }
    }
    if let Some(expression) = artifact.expression {
        let _ = writeln!(out, "    license: {}", safe(expression));
    }
    if let Some(decision) = artifact.decision {
        for unit in &decision.units {
            let permission = if unit.permitted {
                "permitted"
            } else {
                "denied"
            };
            let _ = writeln!(out, "    {permission} unit: {}", safe(&unit.unit));
        }
        for applied in &decision.overrides {
            let _ = writeln!(
                out,
                "    {:?} {}: {}",
                applied.kind,
                safe(&applied.id),
                safe(&applied.reason)
            );
        }
    }
    for d in artifact.diagnostics {
        let _ = writeln!(
            out,
            "    {:?} {:?}: {}",
            d.severity,
            d.error.code,
            safe(&d.error.message)
        );
        if let Some(problem) = d.error.license_problem {
            let _ = writeln!(out, "    cause: {problem:?}\n    next: {}", remedy(problem));
        } else if d.error.code == Code::Acquisition {
            let _ = writeln!(
                out,
                "    next: Check artifact access and credentials; retry without --offline if cached evidence is missing."
            );
        }
    }
    out
}

// Full redacted declarations prevent preview/terminal-escape collisions. The displayed body
// additionally distinguishes status-independent proof/origin, diagnostics and
// policy decisions. Artifact identity/hash deliberately stays out of the key.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct GroupKey<'a> {
    status: &'static str,
    declarations: Option<(&'a [String], &'a [String])>,
    raw_context: Vec<&'a str>,
    body: String,
}
impl<'a> Artifact<'a> {
    fn key(&self) -> GroupKey<'a> {
        let mut raw_context: Vec<_> = self
            .diagnostics
            .iter()
            .map(|d| d.error.message.as_str())
            .collect();
        if let Some(decision) = self.decision {
            raw_context.extend(decision.units.iter().map(|u| u.unit.as_str()));
            for applied in &decision.overrides {
                raw_context.extend([applied.id.as_str(), applied.reason.as_str()]);
            }
        }
        GroupKey {
            status: self.status,
            declarations: self.evidence.map(|e| {
                (
                    e.original_expression.as_slice(),
                    e.original_license.as_slice(),
                )
            }),
            raw_context,
            body: body(self, false),
        }
    }
}

/// Group within one package/source only; JSON keeps the original full report.
pub(super) fn render<'a>(
    out: &mut String,
    artifacts: impl Iterator<Item = Artifact<'a>>,
    total: usize,
    verbose: bool,
) {
    if verbose {
        for artifact in artifacts {
            let _ = writeln!(
                out,
                "  {} artifact: {}",
                artifact.status,
                safe(artifact.locator)
            );
            out.push_str(&body(&artifact, true));
        }
        return;
    }
    let mut groups: BTreeMap<GroupKey<'_>, (usize, &str)> = BTreeMap::new();
    for artifact in artifacts {
        groups
            .entry(artifact.key())
            .and_modify(|(count, _)| *count += 1)
            .or_insert((1, artifact.locator));
    }
    for (key, (count, locator)) in groups {
        let _ = writeln!(out, "  {}: {count}/{total} artifacts", key.status);
        let _ = writeln!(
            out,
            "    example artifact: {} (all artifacts: --verbose or --format json)",
            safe(locator)
        );
        out.push_str(&key.body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use taloon::model::{ContentHash, EvidenceRequirement};

    fn evidence(strength: EvidenceRequirement) -> EvidenceSummary {
        EvidenceSummary {
            content_hash: ContentHash::digest(b"archive"),
            metadata_hash: ContentHash::digest(b"metadata"),
            strength,
            metadata_version: "2.4".into(),
            original_expression: Vec::new(),
            original_license: vec!["custom terms".into()],
            license_origin: None,
            license_files: Vec::new(),
            warnings: Vec::new(),
        }
    }
    fn artifact(evidence: &EvidenceSummary) -> Artifact<'_> {
        Artifact {
            locator: "https://example.test/a.whl",
            status: "Incomplete",
            expression: None,
            evidence: Some(evidence),
            decision: None,
            diagnostics: &[],
        }
    }
    #[rstest]
    #[case::same_proof(EvidenceRequirement::IndexAssociated, 1)]
    #[case::different_proof(EvidenceRequirement::ArchiveVerified, 2)]
    fn grouping_preserves_proof_strength(
        #[case] other: EvidenceRequirement,
        #[case] groups: usize,
    ) {
        // Arrange
        let first = evidence(EvidenceRequirement::IndexAssociated);
        let second = evidence(other);
        let artifacts = [artifact(&first), artifact(&second)];
        let mut out = String::new();
        // Act
        render(&mut out, artifacts.into_iter(), 2, false);
        // Assert
        assert_eq!(out.matches("example artifact:").count(), groups, "{out}");
    }
    #[rstest]
    #[case::same_status("Incomplete", 1)]
    #[case::different_status("Violation", 2)]
    fn grouping_preserves_artifact_status(#[case] status: &'static str, #[case] groups: usize) {
        // Arrange
        let observed = evidence(EvidenceRequirement::ArchiveVerified);
        let first = artifact(&observed);
        let mut second = artifact(&observed);
        second.status = status;
        let mut out = String::new();
        // Act
        render(&mut out, [first, second].into_iter(), 2, false);
        // Assert
        assert_eq!(out.matches("example artifact:").count(), groups, "{out}");
    }
}
