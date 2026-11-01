//! Complete locked scope selection. Graph/extra/workspace knowledge stays here.
use super::{Edge, Resolution, Target};
use crate::{
    model::{Code, Error, PackageId, normalize_name},
    report::SourceSummary,
};
use serde::Serialize;
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    AllExternal,
    DependencyClosure,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DependencyKind {
    Normal,
    Extra { name: String },
    Development { name: String },
}
/// Redacted identity for output; the stable ID hashes fetching/source details.
#[derive(Clone, Debug, Serialize)]
pub struct PackageIdentity {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: SourceSummary,
}
impl From<&PackageId> for PackageIdentity {
    fn from(id: &PackageId) -> Self {
        Self {
            id: id.key(),
            name: id.name().into(),
            version: id.version(),
            source: id.source().into(),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct GraphNode {
    pub package: PackageIdentity,
    pub workspace: bool,
    pub markers: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub via: DependencyKind,
    pub marker: Option<String>,
    pub extras: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct OriginEdge {
    pub to: String,
    pub via: Vec<String>,
    pub marker: Option<String>,
    pub extras: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct GraphOrigin {
    pub entry: String,
    pub edges: Vec<OriginEdge>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub origins: Vec<GraphOrigin>,
}
pub(crate) struct Selection<'a> {
    pub query: String,
    pub mode: SelectionMode,
    pub roots: Vec<PackageIdentity>,
    pub targets: Vec<&'a Target>,
    pub graph: DependencyGraph,
}
impl Resolution {
    /// Union all matching identities. Workspace matches select the full external
    /// inventory; other matches traverse every normal/extra branch, never dev.
    /// Markers annotate alternatives rather than selecting the host environment.
    pub(crate) fn select(&self, name: &str) -> Result<Selection<'_>, Error> {
        let query = normalize_name(name)?;
        let roots: Vec<_> = self
            .graph
            .keys()
            .filter(|id| id.name() == query)
            .cloned()
            .collect();
        if roots.is_empty() {
            return Err(Error::new(
                Code::Configuration,
                "package name is not present in uv.lock",
            ));
        }
        let mode = if roots.iter().any(|r| self.roots.contains(r)) {
            SelectionMode::AllExternal
        } else {
            SelectionMode::DependencyClosure
        };
        let mut selected = BTreeSet::new();
        let mut pending = VecDeque::from(roots.clone());
        if mode == SelectionMode::AllExternal {
            selected.extend(self.graph.keys().cloned());
        } else {
            while let Some(id) = pending.pop_front() {
                if !selected.insert(id.clone()) {
                    continue;
                }
                if let Some(node) = self.graph.get(&id) {
                    pending.extend(
                        node.edges
                            .iter()
                            .chain(node.extras.values().flatten())
                            .map(|e| e.to.clone()),
                    );
                }
            }
        }
        let targets = self
            .targets
            .iter()
            .filter(|t| selected.contains(t.package()))
            .collect();
        let graph = self.selected_graph(&selected, mode);
        Ok(Selection {
            query,
            mode,
            roots: roots.iter().map(PackageIdentity::from).collect(),
            targets,
            graph,
        })
    }
    fn selected_graph(
        &self,
        selected: &BTreeSet<PackageId>,
        mode: SelectionMode,
    ) -> DependencyGraph {
        let mut graph = DependencyGraph::default();
        for id in selected {
            let node = &self.graph[id];
            let mut markers = node.markers.clone();
            markers.sort();
            markers.dedup();
            graph.nodes.push(GraphNode {
                package: id.into(),
                workspace: self.roots.contains(id),
                markers,
            });
            let mut edges: Vec<(&Edge, DependencyKind)> = node
                .edges
                .iter()
                .map(|e| (e, DependencyKind::Normal))
                .collect();
            for (name, group) in &node.extras {
                edges.extend(
                    group
                        .iter()
                        .map(|e| (e, DependencyKind::Extra { name: name.clone() })),
                );
            }
            if mode == SelectionMode::AllExternal && self.roots.contains(id) {
                for (name, group) in &node.development {
                    edges.extend(
                        group
                            .iter()
                            .map(|e| (e, DependencyKind::Development { name: name.clone() })),
                    );
                }
            }
            for (edge, via) in edges {
                if selected.contains(&edge.to) {
                    let mut extras = edge.extras.clone();
                    extras.sort();
                    extras.dedup();
                    graph.edges.push(GraphEdge {
                        from: id.key(),
                        to: edge.to.key(),
                        via,
                        marker: edge.marker.clone(),
                        extras,
                    });
                }
            }
        }
        graph.edges.sort_by(|a, b| {
            (&a.from, &a.to, &a.via, &a.marker, &a.extras)
                .cmp(&(&b.from, &b.to, &b.via, &b.marker, &b.extras))
        });
        graph.edges.dedup_by(|a, b| {
            a.from == b.from
                && a.to == b.to
                && a.via == b.via
                && a.marker == b.marker
                && a.extras == b.extras
        });
        if mode == SelectionMode::AllExternal {
            for entry in &self.entries {
                let mut edges: Vec<_> = entry
                    .edges
                    .iter()
                    .map(|e| {
                        let mut extras = e.extras.clone();
                        extras.sort();
                        extras.dedup();
                        OriginEdge {
                            to: e.to.key(),
                            via: e.via.clone(),
                            marker: e.marker.clone(),
                            extras,
                        }
                    })
                    .collect();
                edges.sort_by(|a, b| {
                    (&a.to, &a.via, &a.marker, &a.extras)
                        .cmp(&(&b.to, &b.via, &b.marker, &b.extras))
                });
                graph.origins.push(GraphOrigin {
                    entry: entry.label.clone(),
                    edges,
                });
            }
        }
        graph
    }
}
