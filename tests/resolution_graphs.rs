//! Generated lock graphs checked against reverse distance relaxation.
use proptest::prelude::*;
use std::fmt::Write as _;
use taloon::{inspection::Inspection, model::EvaluationDate, resolution::DependencyPath};

const EXTRAS: [&str; 2] = ["a", "b"];
const CONTEXTS: [&str; 2] = ["extra:a", "extra:b"];
const MARKERS: &[&str] = &["sys_platform == 'linux'", "sys_platform == 'win32'"];

#[derive(Clone, Copy, Debug)]
struct Edge {
    to: Position,
    marker: Option<&'static str>,
}

#[derive(Clone, Copy, Debug)]
struct Position {
    package: usize,
    extras: u8,
}

#[derive(Debug)]
struct Package {
    base: Vec<Edge>,
    optional: [Vec<Edge>; 2],
    marker: Option<&'static str>,
    record_order: u8,
}

#[derive(Debug)]
struct Graph {
    packages: Vec<Package>,
    root_a: Vec<Edge>,
    root_b: Vec<Edge>,
    dev: Vec<Edge>,
    optional: Vec<Edge>,
    manifest: Vec<Edge>,
}

fn edge_list(count: usize) -> impl Strategy<Value = Vec<Edge>> {
    prop::collection::vec(
        (
            0..count,
            0_u8..4,
            prop::option::of(prop::sample::select(MARKERS)),
        )
            .prop_map(|(package, extras, marker)| Edge {
                to: Position { package, extras },
                marker,
            }),
        0..4,
    )
}

fn graphs() -> impl Strategy<Value = Graph> {
    (2_usize..=5).prop_flat_map(|count| {
        let package = (
            edge_list(count),
            edge_list(count),
            edge_list(count),
            prop::option::of(prop::sample::select(MARKERS)),
            any::<u8>(),
        )
            .prop_map(|(base, a, b, marker, record_order)| Package {
                base,
                optional: [a, b],
                marker,
                record_order,
            });
        (
            prop::collection::vec(package, count),
            edge_list(count),
            edge_list(count),
            edge_list(count),
            edge_list(count),
            edge_list(count),
        )
            .prop_map(|(mut packages, root_a, root_b, dev, optional, manifest)| {
                // This external record has no incoming edge in any generated entry.
                packages.push(Package {
                    base: vec![],
                    optional: [vec![], vec![]],
                    marker: None,
                    record_order: 0,
                });
                Graph {
                    packages,
                    root_a,
                    root_b,
                    dev,
                    optional,
                    manifest,
                }
            })
    })
}

fn extras(mask: u8) -> impl Iterator<Item = &'static str> {
    EXTRAS
        .into_iter()
        .enumerate()
        .filter_map(move |(index, name)| (mask & (1 << index) != 0).then_some(name))
}

fn write_edges(lock: &mut String, field: &str, edges: &[Edge]) {
    writeln!(lock, "{field}=[").unwrap();
    for edge in edges.iter().rev() {
        write!(lock, "{{name='p{}'", edge.to.package).unwrap();
        if let Some(marker) = edge.marker {
            write!(lock, ",marker={marker:?}").unwrap();
        }
        let selected: Vec<_> = extras(edge.to.extras).collect();
        writeln!(lock, ",extra={selected:?}}},").unwrap();
    }
    lock.push_str("]\n");
}

impl Graph {
    fn lock(&self) -> String {
        let mut lock =
            "version=1\nrevision=5\n[manifest]\nmembers=['root-b']\n[manifest.dependency-groups]\n"
                .to_owned();
        write_edges(&mut lock, "lint", &self.manifest);
        lock.push_str("[[package]]\nname='root-a'\nsource={virtual='.'}\n");
        write_edges(&mut lock, "dependencies", &self.root_a);
        lock.push_str("[package.dev-dependencies]\n");
        write_edges(&mut lock, "test", &self.dev);
        lock.push_str("[[package]]\nname='root-b'\nsource={editable='packages/b'}\n");
        write_edges(&mut lock, "dependencies", &self.root_b);
        lock.push_str("[package.optional-dependencies]\n");
        write_edges(&mut lock, "docs", &self.optional);

        let mut records: Vec<_> = self.packages.iter().enumerate().collect();
        records.sort_by_key(|(_, package)| package.record_order);
        for (node, package) in records {
            let hash = "1".repeat(64);
            writeln!(
                lock,
                "[[package]]\nname='p{node}'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
                 wheels=[{{url='https://files.test/p{node}.whl',hash='sha256:{hash}'}}]"
            )
            .unwrap();
            if let Some(marker) = package.marker {
                writeln!(lock, "resolution-markers=[{marker:?}]").unwrap();
            }
            write_edges(&mut lock, "dependencies", &package.base);
            lock.push_str("[package.optional-dependencies]\n");
            for (name, edges) in EXTRAS.into_iter().zip(&package.optional) {
                write_edges(&mut lock, name, edges);
            }
        }
        lock
    }

    fn next(&self, from: Position) -> impl Iterator<Item = (usize, &Edge, Option<&'static str>)> {
        let package = &self.packages[from.package];
        std::iter::once((0, package.base.as_slice(), None))
            .chain(
                package
                    .optional
                    .iter()
                    .enumerate()
                    .filter(move |(index, _)| from.extras & (1 << index) != 0)
                    .map(|(index, edges)| (index + 1, edges.as_slice(), Some(CONTEXTS[index]))),
            )
            .flat_map(|(branch, edges, context)| {
                edges.iter().map(move |edge| (branch, edge, context))
            })
    }

    fn distances(&self, target: usize) -> Vec<[Option<usize>; 4]> {
        // Each pass allows one more edge toward the target. Shortest paths in
        // the expanded graph never need to repeat a package/extra state.
        let mut distance = vec![[None; 4]; self.packages.len()];
        distance[target] = [Some(0); 4];
        for _ in 0..self.packages.len() * 4 {
            let mut next = distance.clone();
            for (package, row) in next.iter_mut().enumerate() {
                for (extras, value) in (0_u8..4).zip(row) {
                    let via_edge = self
                        .next(Position { package, extras })
                        .filter_map(|(_, edge, _)| {
                            distance[edge.to.package][usize::from(edge.to.extras)].map(|d| d + 1)
                        })
                        .min();
                    *value = (*value).into_iter().chain(via_edge).min();
                }
            }
            if next == distance {
                break;
            }
            distance = next;
        }
        distance
    }

    fn expected_paths(&self, target: usize) -> Vec<Path> {
        let distance = self.distances(target);
        [
            ("root-a", self.root_a.as_slice(), None),
            ("root-a [group:test]", &self.dev, Some("group:test")),
            ("root-b", &self.root_b, None),
            ("root-b [extra:docs]", &self.optional, Some("extra:docs")),
            ("workspace [group:lint]", &self.manifest, Some("group:lint")),
        ]
        .into_iter()
        .filter_map(|(entry, edges, context)| {
            let mut winner = choose(edges.iter().map(|edge| (0, edge, context)), &distance)?;
            let mut steps = Vec::new();
            loop {
                let (edge, via) = winner;
                steps.push(Step {
                    package: format!("p{}", edge.to.package),
                    marker: edge.marker.map(str::to_owned),
                    via,
                    package_markers: self.packages[edge.to.package]
                        .marker
                        .map(str::to_owned)
                        .into_iter()
                        .collect(),
                });
                if edge.to.package == target {
                    return Some(Path {
                        entry: entry.to_owned(),
                        steps,
                    });
                }
                winner = choose(self.next(edge.to), &distance).unwrap();
            }
        })
        .collect()
    }
}

fn choose<'a>(
    candidates: impl Iterator<Item = (usize, &'a Edge, Option<&'static str>)>,
    distance: &[[Option<usize>; 4]],
) -> Option<(&'a Edge, Vec<String>)> {
    candidates
        .filter_map(|(branch, edge, context)| {
            let remaining = distance[edge.to.package][usize::from(edge.to.extras)]?;
            let mut via: Vec<_> = extras(edge.to.extras)
                .map(|name| format!("extra:{name}"))
                .chain(context.map(str::to_owned))
                .collect();
            via.sort();
            Some(((remaining, branch, edge.to.package, edge.marker, via), edge))
        })
        .min_by(|(a, _), (b, _)| a.cmp(b))
        .map(|((_, _, _, _, via), edge)| (edge, via))
}

#[derive(Debug, PartialEq, Eq)]
struct Path {
    entry: String,
    steps: Vec<Step>,
}

#[derive(Debug, PartialEq, Eq)]
struct Step {
    package: String,
    marker: Option<String>,
    via: Vec<String>,
    package_markers: Vec<String>,
}

fn diagnostic_paths(paths: &[DependencyPath]) -> Vec<Path> {
    paths
        .iter()
        .map(|path| Path {
            entry: path.entry.clone(),
            steps: path
                .steps
                .iter()
                .map(|step| Step {
                    package: step.package.split_once("==").unwrap().0.to_owned(),
                    marker: step.marker.clone(),
                    via: step.via.clone(),
                    package_markers: step.package_markers.clone(),
                })
                .collect(),
        })
        .collect()
}

fn inspection(graph: &Graph) -> Inspection {
    Inspection::from_documents(
        &graph.lock(),
        "[tool.taloon]\nschema-version=1\nallow=[]",
        EvaluationDate::parse("2026-10-31").unwrap(),
    )
    .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn inspection_inventory_includes_every_external_record(graph in graphs()) {
        // Arrange
        let expected: Vec<_> = (0..graph.packages.len())
            .map(|node| format!("p{node}"))
            .collect();

        // Act
        let inspection = inspection(&graph);

        // Assert
        let actual: Vec<_> = inspection.requests().iter()
            .map(|request| request.package().name())
            .collect();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn diagnostic_paths_match_the_reverse_distance_oracle(graph in graphs()) {
        // Arrange
        let expected: Vec<_> = (0..graph.packages.len())
            .map(|node| (format!("p{node}"), graph.expected_paths(node)))
            .collect();
        let inspection = inspection(&graph);

        // Act
        let report = inspection.evaluate(vec![]);

        // Assert
        let actual: Vec<_> = report.packages().iter()
            .map(|package| (
                package.package.split_once('@').unwrap().0.to_owned(),
                diagnostic_paths(&package.paths),
            ))
            .collect();
        prop_assert_eq!(actual, expected);
    }
}
