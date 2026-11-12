//! Description input/output. Selection and declaration meaning stay in the core.
use super::{Format, acquire, acquisition_summary, project_root, read_document, safe};
use serde::Serialize;
use std::{collections::BTreeMap, fmt::Write as _, io::Write as _, path::Path};
use taloon::{
    description::{Description, DescriptionReport, DescriptionStatus},
    evidence::LicenseOrigin,
    model::Error,
    report::Diagnostic,
    resolution::{DependencyGraph, DependencyKind, GraphEdge, PackageIdentity, SelectionMode},
};

pub(super) fn run(
    project: Option<&Path>,
    package: &str,
    offline: bool,
) -> Result<DescriptionReport, Error> {
    let root = project_root(project)?;
    let lock = read_document(
        &root.join("uv.lock"),
        64 * 1024 * 1024,
        "cannot read UTF-8 uv.lock",
    )?;
    let description = Description::from_uv_lock(&lock, package)?;
    let batch = acquire(root, offline, description.requests())?;
    Ok(description
        .evaluate(batch.results)
        .with_warnings(batch.warnings)
        .with_acquisition_summary(batch.summary))
}
#[derive(Serialize)]
struct Envelope<'a> {
    #[serde(flatten)]
    report: &'a DescriptionReport,
    exit_code: u8,
}
pub(super) fn emit(report: &DescriptionReport, format: Format, verbose: bool) -> u8 {
    let encoded = match format {
        Format::Json => super::json::encode(&Envelope {
            report,
            exit_code: report.exit_code(),
        })
        .map_err(|_| ()),
        Format::Text => Ok(text(report, verbose).into_bytes()),
    };
    encoded.map_or_else(super::serialization_failure, |mut bytes| {
        bytes.push(b'\n');
        if std::io::stdout().lock().write_all(&bytes).is_err() {
            let _ = writeln!(std::io::stderr().lock(), "cannot write taloon description");
            return 2;
        }
        report.exit_code()
    })
}
fn identity(package: &PackageIdentity) -> String {
    format!(
        "{} {} [{:?}: {}]",
        safe(&package.name),
        safe(&package.version),
        package.source.kind,
        safe(&package.source.location)
    )
}
fn diagnostics(out: &mut String, values: &[Diagnostic], indent: &str) {
    for diagnostic in values {
        let _ = writeln!(
            out,
            "{indent}{:?} {:?}: {}",
            diagnostic.severity,
            diagnostic.error.code,
            safe(&diagnostic.error.message)
        );
    }
}
fn conditions(marker: Option<&str>, extras: &[String]) -> String {
    let mut text = String::new();
    if let Some(marker) = marker {
        let _ = write!(text, "; marker: {}", safe(marker));
    }
    if !extras.is_empty() {
        let _ = write!(text, "; requested extras: {}", safe(&extras.join(", ")));
    }
    text
}
// Index relationships once so a large flat inventory does not rescan all
// graph edges for every package. This affects rendering only, never selection.
struct GraphDisplay<'a> {
    nodes: BTreeMap<&'a str, &'a PackageIdentity>,
    outgoing: BTreeMap<&'a str, Vec<&'a GraphEdge>>,
}
impl<'a> GraphDisplay<'a> {
    fn new(graph: &'a DependencyGraph) -> Self {
        let nodes = graph
            .nodes
            .iter()
            .map(|n| (n.package.id.as_str(), &n.package))
            .collect();
        let mut outgoing: BTreeMap<&str, Vec<&GraphEdge>> = BTreeMap::new();
        for edge in &graph.edges {
            outgoing.entry(edge.from.as_str()).or_default().push(edge);
        }
        Self { nodes, outgoing }
    }
    fn dependencies(&self, out: &mut String, from: &str) {
        for edge in self.outgoing.get(from).into_iter().flatten() {
            let via = match &edge.via {
                DependencyKind::Normal => "normal".into(),
                DependencyKind::Extra { name } => format!("extra: {}", safe(name)),
                DependencyKind::Development { name } => format!("development: {}", safe(name)),
            };
            if let Some(target) = self.nodes.get(edge.to.as_str()) {
                let _ = writeln!(
                    out,
                    "  -> {} [{}{}]",
                    identity(target),
                    via,
                    conditions(edge.marker.as_deref(), &edge.extras)
                );
            }
        }
    }
}
fn render_origins(
    out: &mut String,
    graph: &GraphDisplay<'_>,
    origins: &[taloon::resolution::GraphOrigin],
) {
    for origin in origins {
        let _ = writeln!(out, "\norigin: {}", safe(&origin.entry));
        for edge in &origin.edges {
            if let Some(target) = graph.nodes.get(edge.to.as_str()) {
                let _ = writeln!(
                    out,
                    "  -> {} [{}{}]",
                    identity(target),
                    safe(&edge.via.join(", ")),
                    conditions(edge.marker.as_deref(), &edge.extras)
                );
            }
        }
    }
}
fn text(report: &DescriptionReport, verbose: bool) -> String {
    let mut out = String::new();
    let scope = match report.selection_mode() {
        Some(SelectionMode::AllExternal) => "all external packages",
        Some(SelectionMode::DependencyClosure) => "package and dependencies",
        None => "invalid input",
    };
    let status = match report.status() {
        DescriptionStatus::Complete => "complete",
        DescriptionStatus::Incomplete => "incomplete",
    };
    let _ = writeln!(
        out,
        "taloon describe: {} ({scope}; {status})",
        safe(report.query())
    );
    let _ = writeln!(
        out,
        "Locked alternatives across all extras/environments; no permission evaluation."
    );
    for root in report.roots() {
        let _ = writeln!(out, "root: {}", identity(root));
    }
    let graph = GraphDisplay::new(report.graph());
    for package in report.packages() {
        let _ = writeln!(out, "\n{}", identity(&package.package));
        for group in &package.licenses {
            let _ = writeln!(
                out,
                "  {}: {}/{} artifacts",
                safe(&group.expression),
                group.artifacts.len(),
                package.artifacts.len()
            );
        }
        if package.unknown_artifacts > 0 {
            let _ = writeln!(
                out,
                "  Unknown: {}/{} artifacts",
                package.unknown_artifacts,
                package.artifacts.len()
            );
        }
        super::artifacts::render(
            &mut out,
            package
                .artifacts
                .iter()
                .filter(|a| {
                    verbose
                        || !a.diagnostics.is_empty()
                        || a.evidence.as_ref().is_some_and(|e| {
                            e.license_origin == Some(LicenseOrigin::LegacySupplement)
                        })
                })
                .map(super::artifacts::Artifact::from),
            package.artifacts.len(),
            verbose,
        );
        diagnostics(&mut out, &package.diagnostics, "  ");
        graph.dependencies(&mut out, &package.package.id);
    }
    for node in &report.graph().nodes {
        if node.workspace {
            let _ = writeln!(
                out,
                "\nworkspace context (own license excluded): {}",
                identity(&node.package)
            );
            graph.dependencies(&mut out, &node.package.id);
        }
        if !node.markers.is_empty() {
            let _ = writeln!(
                out,
                "resolution markers for {}: {}",
                identity(&node.package),
                safe(&node.markers.join("; "))
            );
        }
    }
    render_origins(&mut out, &graph, &report.graph().origins);
    diagnostics(&mut out, report.diagnostics(), "");
    acquisition_summary(&mut out, report.acquisition());
    out
}

#[cfg(test)]
mod graph_rendering_tests {
    use super::*;
    #[test]
    fn absent_dependency_target_is_not_rendered_as_a_dangling_link() {
        // Arrange
        let graph = DependencyGraph {
            edges: vec![GraphEdge {
                from: "source".into(),
                to: "missing".into(),
                via: DependencyKind::Normal,
                marker: None,
                extras: Vec::new(),
            }],
            ..DependencyGraph::default()
        };
        let display = GraphDisplay::new(&graph);
        let mut text = String::new();
        // Act
        display.dependencies(&mut text, "source");
        // Assert
        assert_eq!(text, "");
    }
    #[test]
    fn absent_origin_target_preserves_origin_without_a_dangling_link() {
        // Arrange
        let graph = DependencyGraph::default();
        let display = GraphDisplay::new(&graph);
        let origins = [taloon::resolution::GraphOrigin {
            entry: "root".into(),
            edges: vec![taloon::resolution::OriginEdge {
                to: "missing".into(),
                via: Vec::new(),
                marker: None,
                extras: Vec::new(),
            }],
        }];
        let mut text = String::new();
        // Act
        render_origins(&mut text, &display, &origins);
        // Assert
        assert_eq!(text, "\norigin: root\n");
    }
}
