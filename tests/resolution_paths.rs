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
