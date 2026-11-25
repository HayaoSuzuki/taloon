//! A DFS enumeration oracle checks BFS shortest paths without reproducing BFS.
use proptest::prelude::*;
use rstest::rstest;
use std::fmt::Write as _;
use taloon::resolution::{DependencyPath, Resolution};

fn graph_lock(edges: &[Vec<usize>]) -> String {
    let mut text = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='p000'}]\n".to_owned();
    for (node, outgoing) in edges.iter().enumerate() {
        writeln!(text, "[[package]]\nname='p{node:03}'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\ndependencies=[").unwrap();
        for next in outgoing {
            writeln!(text, "{{name='p{next:03}'}},").unwrap();
        }
        text.push_str("]\n");
    }
    text
}
fn nodes(path: &DependencyPath) -> Vec<usize> {
    path.steps
        .iter()
        .map(|step| {
            step.package
                .split_once("==")
                .unwrap()
                .0
                .trim_start_matches('p')
                .parse()
                .unwrap()
        })
        .collect()
}
fn enumerate(
    edges: &[Vec<usize>],
    target: usize,
    prefix: &mut Vec<usize>,
    paths: &mut Vec<Vec<usize>>,
) {
    let node = *prefix.last().unwrap();
    if node == target {
        paths.push(prefix.clone());
        return;
    }
    for next in &edges[node] {
        if !prefix.contains(next) {
            prefix.push(*next);
            enumerate(edges, target, prefix, paths);
            prefix.pop();
        }
    }
}
#[rstest]
#[case::chain(vec![vec![1], vec![2], vec![]], Some(vec![0,1,2]))]
#[case::cyclic_tie(vec![vec![1,2], vec![0,3], vec![3], vec![1]], Some(vec![0,1,3]))]
#[case::unreachable(vec![vec![1], vec![0], vec![]], None)]
fn paths_preserve_shortest_order_and_cycles(
    #[case] edges: Vec<Vec<usize>>,
    #[case] expected: Option<Vec<usize>>,
) {
    // Arrange
    let lock = graph_lock(&edges);
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    let target = resolution.targets().last().unwrap().package();
    // Act
    let paths = resolution.paths(target);
    // Assert
    assert_eq!(paths.first().map(nodes), expected);
    assert!(paths.len() <= 1);
}
#[test]
fn cyclic_extra_states_preserve_the_activated_branch_and_annotations() {
    // Arrange
    let text = "version=1\nrevision=5\n[[package]]\nname='root'\nsource={virtual='.'}\ndependencies=[{name='p000',extra=['a']},{name='p000',extra=['b']}]\n[[package]]\nname='p000'\nversion='1.0'\nsource={registry='https://example.test/simple'}\n[package.optional-dependencies]\na=[{name='p001'}]\nb=[{name='p002',marker=\"sys_platform == 'linux'\"}]\n[[package]]\nname='p001'\nversion='1.0'\nsource={registry='https://example.test/simple'}\ndependencies=[{name='p000',extra=['a']}]\n[[package]]\nname='p002'\nversion='1.0'\nsource={registry='https://example.test/simple'}\nresolution-markers=[\"python_full_version >= '3.14'\"]";
    let resolution = Resolution::from_uv_lock(text).unwrap();
    let target = resolution.targets().last().unwrap().package();
    // Act
    let paths = resolution.paths(target);
    // Assert: one annotated path distinguishes the two visits to the same package.
    assert_eq!(paths.len(), 1);
    assert_eq!(nodes(&paths[0]), vec![0, 2]);
    assert_eq!(paths[0].steps[0].via, ["extra:b"]);
    assert_eq!(paths[0].steps[1].via, ["extra:b"]);
    assert_eq!(
        paths[0].steps[1].marker.as_deref(),
        Some("sys_platform == 'linux'")
    );
    assert_eq!(
        paths[0].steps[1].package_markers,
        ["python_full_version >= '3.14'"]
    );
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]
    #[test]
    fn shortest_path_matches_exhaustive_simple_path_enumeration(
        masks in prop::collection::vec(any::<u8>(), 2..8),
    ) {
        // Arrange: enumerate every simple path, then compare length and lexical order.
        let count = masks.len();
        let edges: Vec<Vec<usize>> = masks.iter().map(|mask| (0..count).filter(|next| mask & (1 << next) != 0).collect()).collect();
        let mut all_paths = Vec::new();
        enumerate(&edges, count - 1, &mut vec![0], &mut all_paths);
        all_paths.sort_by(|a,b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        let expected = all_paths.into_iter().next();
        let lock = graph_lock(&edges);
        let resolution = Resolution::from_uv_lock(&lock).unwrap();
        let target = resolution.targets().last().unwrap().package();
        // Act
        let actual = resolution.paths(target);
        // Assert
        prop_assert_eq!(actual.first().map(nodes), expected);
        prop_assert!(actual.len() <= 1);
    }
}

#[derive(serde::Deserialize)]
struct ExtrasCase {
    scenario: ExtrasScenario,
    path: Vec<u8>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtrasScenario {
    root_extra: bool,
    return_extra: bool,
    shortcut: bool,
}

fn extras_lock(scenario: &ExtrasScenario) -> String {
    let root_activation = if scenario.root_extra {
        ",extra=['feature']"
    } else {
        ""
    };
    let return_activation = if scenario.return_extra {
        ",extra=['feature']"
    } else {
        ""
    };
    let shortcut = if scenario.shortcut {
        ",{name='target'}"
    } else {
        ""
    };
    format!(
        "version=1\nrevision=5\n\
         [[package]]\nname='root'\nsource={{virtual='.'}}\n\
         dependencies=[{{name='hub'{root_activation}}}{shortcut}]\n\
         [[package]]\nname='hub'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
         dependencies=[{{name='bridge'}}]\n\
         [package.optional-dependencies]\nfeature=[{{name='target'}}]\n\
         [[package]]\nname='bridge'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
         dependencies=[{{name='hub'{return_activation}}}]\n\
         [[package]]\nname='target'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}"
    )
}

#[rstest]
fn quint_replays_extras(#[values(0, 1, 2, 3, 4, 5, 6, 7)] index: usize) {
    // Arrange
    let row = include_str!("fixtures/oracle/extras.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: ExtrasCase = serde_json::from_str(row).unwrap();
    let resolution = Resolution::from_uv_lock(&extras_lock(&case.scenario)).unwrap();
    let target = resolution
        .targets()
        .iter()
        .find(|target| target.package().name() == "target")
        .unwrap();

    // Act
    let paths = resolution.paths(target.package());

    // Assert
    let actual: Vec<Vec<u8>> = paths
        .iter()
        .map(|path| {
            assert_eq!(path.entry, "root");
            path.steps
                .iter()
                .map(|step| match step.package.split_once("==").unwrap().0 {
                    "hub" => {
                        if step.via.iter().any(|label| label == "extra:feature") {
                            2
                        } else {
                            0
                        }
                    }
                    "bridge" => 1,
                    "target" => {
                        let expected: &[&str] = if case.scenario.shortcut {
                            &[]
                        } else {
                            &["extra:feature"]
                        };
                        assert_eq!(step.via, expected);
                        3
                    }
                    package => panic!("unexpected package in extras path: {package}"),
                })
                .collect()
        })
        .collect();
    let expected: Vec<_> = Some(case.path)
        .filter(|path| !path.is_empty())
        .into_iter()
        .collect();
    assert_eq!(actual, expected);
}

#[derive(serde::Deserialize)]
struct PathTieCase {
    scenario: PathTieScenario,
    path: Vec<TieStep>,
    development: Vec<TieStep>,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PathTieScenario {
    mode: usize,
    base_edge: bool,
    shortcut: bool,
    reversed: bool,
}

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct TieStep {
    package: String,
    marker: String,
    via: Vec<String>,
}

fn path_tie_lock(scenario: &PathTieScenario) -> String {
    const LINUX: &str = ",marker=\"sys_platform == 'linux'\"";
    const WINDOWS: &str = ",marker=\"sys_platform == 'win32'\"";
    let (marker_a, marker_b) = [
        (LINUX, LINUX),
        (LINUX, WINDOWS),
        (WINDOWS, LINUX),
        (WINDOWS, ""),
    ][scenario.mode];
    let extras = if scenario.reversed {
        "['b','a','a']"
    } else {
        "['a','b']"
    };
    let mut roots = vec![
        format!("{{name='hub',extra={extras}{marker_a}}}"),
        format!("{{name='hub',extra=['b']{marker_b}}}"),
        "{name='other'}".to_owned(),
    ];
    if scenario.shortcut {
        roots.push("{name='target'}".to_owned());
    }
    if scenario.reversed {
        roots.reverse();
    }
    let root = format!(
        "[[package]]\nname='root'\nsource={{virtual='.'}}\n\
         dependencies=[{}]\n[package.dev-dependencies]\ndev=[{{name='other'}}]\n",
        roots.join(",")
    );
    let base = if scenario.base_edge {
        format!("{{name='target'{WINDOWS}}}")
    } else {
        String::new()
    };
    let hub = format!(
        "[[package]]\nname='hub'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
         dependencies=[{base}]\n[package.optional-dependencies]\n\
         b=[{{name='target'{LINUX}}}]\na=[{{name='target'{WINDOWS}}}]\n"
    );
    let other = "[[package]]\nname='other'\nversion='1.0'\nsource={registry='https://example.test/simple'}\ndependencies=[{name='target'}]\n";
    let leaf = "[[package]]\nname='target'\nversion='1.0'\nsource={registry='https://example.test/simple'}\n";
    let mut records = [root.as_str(), hub.as_str(), other, leaf];
    if scenario.reversed {
        records.reverse();
    }
    format!("version=1\nrevision=5\n{}", records.concat())
}

#[rstest]
fn quint_replays_path_ties(
    #[values(0, 1, 2, 3)] mode: usize,
    #[values(false, true)] base_edge: bool,
    #[values(false, true)] shortcut: bool,
    #[values(false, true)] reversed: bool,
) {
    // Arrange
    let index = ((mode * 2 + usize::from(base_edge)) * 2 + usize::from(shortcut)) * 2
        + usize::from(reversed);
    let row = include_str!("fixtures/oracle/path-ties.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: PathTieCase = serde_json::from_str(row).unwrap();
    assert_eq!(
        case.scenario,
        PathTieScenario {
            mode,
            base_edge,
            shortcut,
            reversed
        }
    );
    let resolution = Resolution::from_uv_lock(&path_tie_lock(&case.scenario)).unwrap();
    let target = resolution
        .targets()
        .iter()
        .find(|target| target.package().name() == "target")
        .unwrap();

    // Act
    let paths = resolution.paths(target.package());

    // Assert
    let actual: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let steps: Vec<_> = path
                .steps
                .into_iter()
                .map(|step| TieStep {
                    package: step.package.split_once("==").unwrap().0.to_owned(),
                    marker: step.marker.unwrap_or_default(),
                    via: step.via,
                })
                .collect();
            (path.entry, steps)
        })
        .collect();
    let expected = [
        ("root".to_owned(), case.path),
        ("root [group:dev]".to_owned(), case.development),
    ];
    assert_eq!(actual, expected);
}
