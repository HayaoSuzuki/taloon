//! Locked package/artifact coverage and annotated graph paths.
use crate::model::{
    ArtifactFormat, ArtifactLocator, ArtifactRequest, Code, ContentHash, Error, PackageId,
    SourceId, SourceKind, normalize_name,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
/// One external locked package, including unsupported/missing-evidence outcomes.
pub struct Target {
    package: PackageId,
    requests: Vec<ArtifactRequest>,
    issues: Vec<Error>,
}
impl Target {
    #[must_use]
    pub fn package(&self) -> &PackageId {
        &self.package
    }
    #[must_use]
    pub fn requests(&self) -> &[ArtifactRequest] {
        &self.requests
    }
    #[must_use]
    pub fn issues(&self) -> &[Error] {
        &self.issues
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct PathStep {
    pub package: String,
    pub marker: Option<String>,
    pub via: Vec<String>,
    pub package_markers: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct DependencyPath {
    pub entry: String,
    pub steps: Vec<PathStep>,
}
#[derive(Clone)]
struct Edge {
    to: PackageId,
    marker: Option<String>,
    via: Vec<String>,
    extras: Vec<String>,
}
struct Entry {
    label: String,
    edges: Vec<Edge>,
}
struct Node {
    edges: Vec<Edge>,
    markers: Vec<String>,
    extras: BTreeMap<String, Vec<Edge>>,
}
#[derive(Clone, Copy)]
enum GroupKind {
    Development,
    Extra,
}
impl GroupKind {
    fn label(self) -> &'static str {
        match self {
            Self::Development => "group",
            Self::Extra => "extra",
        }
    }
}
/// Complete locked inventory and conditional origins for a supported wire schema.
pub struct Resolution {
    targets: Vec<Target>,
    graph: BTreeMap<PackageId, Node>,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct RawLock {
    version: u32,
    revision: Option<u32>,
    #[serde(default)]
    package: Vec<RawPackage>,
    #[serde(default)]
    manifest: RawManifest,
}
#[derive(Default, Deserialize)]
struct RawManifest {
    #[serde(default)]
    members: Vec<String>,
    #[serde(default, rename = "dependency-groups")]
    dependency_groups: BTreeMap<String, Vec<RawDependency>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawPackage {
    name: String,
    version: Option<String>,
    source: BTreeMap<String, String>,
    #[serde(default)]
    resolution_markers: Vec<String>,
    #[serde(default)]
    dependencies: Vec<RawDependency>,
    #[serde(default)]
    dev_dependencies: BTreeMap<String, Vec<RawDependency>>,
    #[serde(default)]
    optional_dependencies: BTreeMap<String, Vec<RawDependency>>,
    #[serde(default)]
    wheels: Vec<RawArtifact>,
    sdist: Option<RawArtifact>,
}
#[derive(Deserialize)]
struct RawDependency {
    name: String,
    version: Option<String>,
    source: Option<BTreeMap<String, String>>,
    marker: Option<String>,
    specifier: Option<String>,
    #[serde(default, alias = "extras")]
    extra: Vec<String>,
}
#[derive(Deserialize)]
struct RawArtifact {
    url: Option<String>,
    path: Option<String>,
    hash: Option<String>,
}

fn source(raw: &BTreeMap<String, String>) -> Result<SourceId, Error> {
    if raw.len() != 1 {
        return Err(Error::new(
            Code::LockFormat,
            "a package source must have one recognized kind",
        ));
    }
    let (kind, location) = raw
        .first_key_value()
        .ok_or_else(|| Error::new(Code::LockFormat, "missing source"))?;
    SourceId::parse(kind, location)
}
fn root_path(source: &SourceId) -> bool {
    matches!(source.kind(), SourceKind::Editable | SourceKind::Virtual)
        && std::path::Path::new(source.location())
            .components()
            .all(|c| c == std::path::Component::CurDir)
}
fn local_source(kind: SourceKind) -> bool {
    matches!(
        kind,
        SourceKind::Editable | SourceKind::Directory | SourceKind::Virtual
    )
}
fn edges(
    raw: &[RawDependency],
    packages: &[PackageId],
    context: Option<&str>,
) -> Result<Vec<Edge>, Error> {
    let mut result = Vec::new();
    for dep in raw {
        let name = normalize_name(&dep.name)?;
        let selected_source = dep.source.as_ref().map(source).transpose()?;
        let mut matches = Vec::new();
        for p in packages {
            if p.name() != name || selected_source.as_ref().is_some_and(|s| s != p.source()) {
                continue;
            }
            if let Some(version) = &dep.version
                && PackageId::parse(&name, version, p.source().clone())? != *p
            {
                continue;
            }
            if let Some(constraint) = &dep.specifier
                && !p.satisfies(constraint)?
            {
                continue;
            }
            matches.push(p.clone());
        }
        if matches.len() != 1 {
            return Err(Error::new(
                Code::LockFormat,
                "dependency edge has no unique locked package",
            ));
        }
        let mut extras = dep
            .extra
            .iter()
            .map(|name| normalize_name(name))
            .collect::<Result<Vec<_>, _>>()?;
        extras.sort();
        extras.dedup();
        let mut via = extras
            .iter()
            .map(|s| format!("extra:{s}"))
            .collect::<Vec<_>>();
        if let Some(context) = context {
            via.push(context.into());
        }
        via.sort();
        result.push(Edge {
            to: matches.remove(0),
            marker: dep.marker.clone(),
            via,
            extras,
        });
    }
    result.sort_by(|a, b| (&a.to, &a.marker, &a.via).cmp(&(&b.to, &b.marker, &b.via)));
    Ok(result)
}
fn artifact(
    package: &PackageId,
    raw: &RawArtifact,
    format: ArtifactFormat,
) -> Result<ArtifactRequest, Error> {
    let hash = raw.hash.as_deref().map(ContentHash::parse).transpose()?;
    let locator = match package.source().kind() {
        SourceKind::Registry => ArtifactLocator::registry(
            package.source().location(),
            raw.url
                .as_deref()
                .ok_or_else(|| Error::new(Code::MissingArtifact, "registry artifact lacks URL"))?,
        )?,
        SourceKind::Url => {
            ArtifactLocator::url(raw.url.as_deref().unwrap_or(package.source().location()))?
        }
        SourceKind::Path => {
            ArtifactLocator::local(raw.path.as_deref().unwrap_or(package.source().location()))?
        }
        _ => {
            return Err(Error::new(
                Code::UnsupportedSource,
                "source has no supported archive evidence",
            ));
        }
    };
    ArtifactRequest::new(package.clone(), locator, format, hash)
}
fn target(id: PackageId, p: &RawPackage) -> Target {
    let mut requests = Vec::new();
    let mut issues = Vec::new();
    if !matches!(
        id.source().kind(),
        SourceKind::Registry | SourceKind::Url | SourceKind::Path
    ) {
        issues.push(Error::new(
            Code::UnsupportedSource,
            "external source has no supported distribution archive",
        ));
        return Target {
            package: id,
            requests,
            issues,
        };
    }
    for (format, a) in p
        .wheels
        .iter()
        .map(|a| (ArtifactFormat::Wheel, a))
        .chain(p.sdist.iter().map(|a| (ArtifactFormat::Sdist, a)))
    {
        match artifact(&id, a, format) {
            Ok(request) => requests.push(request),
            Err(error) => issues.push(error),
        }
    }
    if requests.is_empty() && issues.is_empty() && id.source().kind() == SourceKind::Path {
        let format = if std::path::Path::new(id.source().location())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("whl"))
        {
            ArtifactFormat::Wheel
        } else {
            ArtifactFormat::Sdist
        };
        match artifact(
            &id,
            &RawArtifact {
                url: None,
                path: None,
                hash: None,
            },
            format,
        ) {
            Ok(request) => requests.push(request),
            Err(error) => issues.push(error),
        }
    }
    if requests.is_empty() && issues.is_empty() {
        issues.push(Error::new(
            Code::MissingArtifact,
            "package has no recorded distribution archives",
        ));
    }
    let mut seen = BTreeSet::new();
    if requests.iter().any(|r| !seen.insert(r.id())) {
        issues.push(Error::new(
            Code::LockFormat,
            "duplicate artifact association",
        ));
    }
    Target {
        package: id,
        requests,
        issues,
    }
}

struct Records {
    packages: BTreeMap<PackageId, RawPackage>,
    roots: BTreeSet<PackageId>,
    groups: BTreeMap<String, Vec<RawDependency>>,
}
fn records(raw: RawLock) -> Result<Records, Error> {
    if raw.version != 1 || raw.revision != Some(5) {
        return Err(Error::new(
            Code::LockFormat,
            "supported uv.lock schema is version 1 revision 5",
        ));
    }
    let members = raw
        .manifest
        .members
        .iter()
        .map(|n| normalize_name(n))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut records = BTreeMap::new();
    let mut roots = BTreeSet::new();
    for p in raw.package {
        let src = source(&p.source)?;
        let is_root = root_path(&src)
            || (local_source(src.kind()) && members.contains(&normalize_name(&p.name)?));
        let version = p
            .version
            .as_deref()
            .or(if is_root { Some("0") } else { None })
            .ok_or_else(|| Error::new(Code::LockFormat, "external package lacks version"))?;
        let id = PackageId::parse(&p.name, version, src)?;
        if is_root {
            roots.insert(id.clone());
        }
        if records.insert(id, p).is_some() {
            return Err(Error::new(
                Code::LockFormat,
                "duplicate locked package identity",
            ));
        }
    }
    for name in &members {
        if roots.iter().filter(|p| p.name() == name).count() > 1 {
            return Err(Error::new(
                Code::LockFormat,
                "ambiguous workspace membership",
            ));
        }
    }

    Ok(Records {
        packages: records,
        roots,
        groups: raw.manifest.dependency_groups,
    })
}

impl Resolution {
    /// # Errors
    /// Rejects unsupported or invalid lock records.
    pub fn from_uv_lock(text: &str) -> Result<Self, Error> {
        let raw: RawLock = toml::from_str(text)
            .map_err(|_| Error::new(Code::LockFormat, "invalid uv.lock fields"))?;
        let Records {
            packages: records,
            roots,
            groups,
        } = records(raw)?;
        let packages = records.keys().cloned().collect::<Vec<_>>();
        let mut graph = BTreeMap::new();
        let mut entries = Vec::new();
        for (name, requirements) in groups {
            let context = format!("group:{name}");
            // Manifest requirements are origin hints, not resolved edges. Ambiguous hints
            // leave a missing-path explanation; they never narrow the locked target set.
            if let Ok(group_edges) = edges(&requirements, &packages, Some(&context)) {
                entries.push(Entry {
                    label: format!("workspace [{context}]"),
                    edges: group_edges,
                });
            }
        }
        let mut targets = Vec::new();
        for (id, p) in records {
            let node_edges = edges(&p.dependencies, &packages, None)?;
            let mut extras = BTreeMap::new();
            if roots.contains(&id) {
                entries.push(Entry {
                    label: id.name().to_owned(),
                    edges: node_edges.clone(),
                });
            }
            for (prefix, groups) in [
                (GroupKind::Development, &p.dev_dependencies),
                (GroupKind::Extra, &p.optional_dependencies),
            ] {
                for (name, deps) in groups {
                    let context = format!("{}:{name}", prefix.label());
                    let group_edges = edges(deps, &packages, Some(&context))?;
                    if roots.contains(&id) {
                        entries.push(Entry {
                            label: format!("{} [{context}]", id.name()),
                            edges: group_edges.clone(),
                        });
                    }
                    if matches!(prefix, GroupKind::Extra)
                        && extras.insert(normalize_name(name)?, group_edges).is_some()
                    {
                        return Err(Error::new(Code::LockFormat, "duplicate normalized extra"));
                    }
                }
            }
            graph.insert(
                id.clone(),
                Node {
                    edges: node_edges,
                    markers: p.resolution_markers.clone(),
                    extras,
                },
            );
            if roots.contains(&id) {
                continue;
            }
            targets.push(target(id, &p));
        }
        entries.sort_by(|a, b| a.label.cmp(&b.label));
        Ok(Self {
            targets,
            graph,
            entries,
        })
    }
    #[must_use]
    /// All external records, including unreachable and unsupported sources.
    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
    #[must_use]
    /// One shortest annotated path per known entry point. Empty means no path can
    /// be reconstructed; it never removes the package from target coverage.
    pub fn paths(&self, package: &PackageId) -> Vec<DependencyPath> {
        let mut paths = Vec::new();
        for entry in &self.entries {
            let mut seen = BTreeSet::new();
            let mut queue = VecDeque::new();
            for edge in &entry.edges {
                queue.push_back((edge.clone(), Vec::new()));
            }
            while let Some((edge, mut steps)) = queue.pop_front() {
                if !seen.insert((edge.to.clone(), edge.extras.clone())) {
                    continue;
                }
                steps.push(PathStep {
                    package: edge.to.to_string(),
                    marker: edge.marker,
                    via: edge.via,
                    package_markers: self
                        .graph
                        .get(&edge.to)
                        .map_or_else(Vec::new, |n| n.markers.clone()),
                });
                if edge.to == *package {
                    paths.push(DependencyPath {
                        entry: entry.label.clone(),
                        steps,
                    });
                    break;
                }
                if let Some(next) = self.graph.get(&edge.to) {
                    for edge in next.edges.iter().chain(
                        edge.extras
                            .iter()
                            .filter_map(|name| next.extras.get(name))
                            .flatten(),
                    ) {
                        if !seen.contains(&(edge.to.clone(), edge.extras.clone())) {
                            queue.push_back((edge.clone(), steps.clone()));
                        }
                    }
                }
            }
        }
        paths
    }
}
