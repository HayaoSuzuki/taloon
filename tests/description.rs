mod common;
use std::collections::BTreeSet;
use taloon::{
    description::Description,
    evidence::{Evidence, SidecarAssociation},
    inspection::AcquisitionResult,
    model::ContentHash,
    resolution::Resolution,
};

fn package(name: &str, version: &str, source: &str, fields: &str) -> String {
    let hash = ContentHash::digest(b"locked archive");
    format!(
        "\n[[package]]\nname='{name}'\nversion='{version}'\nsource={{{source}}}\nwheels=[{{url='https://files.test/{name}-{version}.whl',hash='{hash}'}}]\n{fields}\n"
    )
}
fn graph_lock() -> String {
    let mut lock = "version=1\nrevision=5\n[manifest]\nmembers=['app','member']\n".to_owned();
    lock.push_str("[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}\ndependencies=[{name='a',version='1.0'}]\n[package.dev-dependencies]\ntest=[{name='dev'}]\n");
    lock.push_str("[[package]]\nname='member'\nversion='1.0'\nsource={editable='member'}\ndependencies=[{name='d'}]\n");
    lock.push_str(&package("a", "1.0", "registry='https://index.test/simple'", "dependencies=[{name='b',marker=\"python_version >= '3.14'\"}]\n[package.optional-dependencies]\nfeature=[{name='c'}]\n[package.dev-dependencies]\ntest=[{name='dev'}]"));
    lock.push_str(&package(
        "a",
        "2.0",
        "registry='https://other.test/simple'",
        "",
    ));
    lock.push_str(&package(
        "b",
        "1.0",
        "registry='https://index.test/simple'",
        "dependencies=[{name='a',version='1.0'},{name='member'}]",
    ));
    for name in ["c", "d", "dev", "unrelated"] {
        lock.push_str(&package(
            name,
            "1.0",
            "registry='https://index.test/simple'",
            "",
        ));
    }
    lock
}
fn names(description: &Description) -> BTreeSet<String> {
    description
        .requests()
        .iter()
        .map(|r| r.package().name().to_owned())
        .collect()
}
fn observations(description: &Description, licenses: &[Option<&str>]) -> Vec<AcquisitionResult> {
    description
        .requests()
        .iter()
        .enumerate()
        .map(|(n, r)| {
            let declaration = licenses[n % licenses.len()]
                .map_or_else(String::new, |s| format!("License-Expression: {s}\n"));
            let meta = format!(
                "Metadata-Version: 2.4\nName: {}\nVersion: {}\n{declaration}\n",
                r.package().name(),
                r.package().version()
            );
            let hash = r.hash().unwrap();
            let evidence = Evidence::from_sidecar(
                r,
                SidecarAssociation::new(hash, hash, ContentHash::digest(meta.as_bytes())).unwrap(),
                meta.as_bytes(),
            )
            .unwrap();
            AcquisitionResult::observed(r.id(), evidence)
        })
        .collect()
}
#[test]
fn description_selects_all_identities_extras_markers_and_workspace_routes() {
    // Arrange
    let lock = graph_lock();
    // Act
    let description = Description::from_uv_lock(&lock, "A").unwrap();
    let report = description.evaluate(observations(&description, &[Some("MIT")]));
    // Assert
    assert_eq!(
        names(&description),
        BTreeSet::from_iter(["a", "b", "c", "d"].map(str::to_owned))
    );
    assert_eq!(description.requests().len(), 5);
    assert_eq!(report.exit_code(), 0);
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(value["selection_mode"], "dependency_closure");
    assert_eq!(value["roots"].as_array().unwrap().len(), 2);
    assert_eq!(value["packages"].as_array().unwrap().len(), 5);
    assert!(
        value["graph"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["package"]["name"] == "member" && n["workspace"] == true)
    );
    assert!(
        value["graph"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["via"]["kind"] == "extra" && e["via"]["name"] == "feature")
    );
    assert!(
        value["graph"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| !e["marker"].is_null())
    );
}
#[rstest::rstest]
#[case::project("app")]
#[case::member("member")]
fn project_and_member_description_exactly_match_whole_external_inventory(#[case] query: &str) {
    // Arrange
    let lock = graph_lock();
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    let expected: BTreeSet<_> = resolution
        .targets()
        .iter()
        .flat_map(taloon::resolution::Target::requests)
        .map(taloon::model::ArtifactRequest::id)
        .collect();
    // Act
    let description = Description::from_uv_lock(&lock, query).unwrap();
    let report =
        serde_json::to_value(description.evaluate(observations(&description, &[Some("MIT")])))
            .unwrap();
    // Assert
    assert_eq!(
        description
            .requests()
            .iter()
            .map(taloon::model::ArtifactRequest::id)
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(report["selection_mode"], "all_external");
    assert!(
        report["graph"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["via"]["kind"] == "development")
    );
    assert!(!names(&description).contains("app"));
    assert!(!names(&description).contains("member"));
}
#[test]
fn description_preserves_different_unknown_and_custom_declarations() {
    // Arrange
    let hash = ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}},{{url='https://files.test/c.whl',hash='{hash}'}}]"
    );
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let results = observations(&description, &[Some("MIT"), Some("BSD-3-Clause"), None]);
    // Act
    let report = description.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(), 2);
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(
        value["packages"][0]["licenses"].as_array().unwrap().len(),
        2
    );
    assert_eq!(value["packages"][0]["unknown_artifacts"], 1);
}

#[test]
fn description_preserves_custom_declaration() {
    // Arrange
    let hash = ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}},{{url='https://files.test/c.whl',hash='{hash}'}}]"
    );
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    // Act
    let report = description.evaluate(observations(&description, &[Some("LicenseRef-Local")]));
    // Assert
    assert_eq!(report.exit_code(), 0);
    assert_eq!(
        serde_json::to_value(report).unwrap()["packages"][0]["licenses"][0]["expression"],
        "LicenseRef-Local"
    );
}
#[test]
fn missing_results_are_incomplete() {
    // Arrange
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    // Act
    let report = description.evaluate(vec![]);
    // Assert
    assert_eq!(report.exit_code(), 2);
}

#[test]
fn duplicate_results_are_incomplete() {
    // Arrange
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    let mut duplicate = observations(&description, &[Some("MIT")]);
    duplicate.extend(observations(&description, &[Some("MIT")]));
    // Act
    let report = description.evaluate(duplicate);
    // Assert
    assert_eq!(report.exit_code(), 2);
}

#[test]
fn foreign_results_are_incomplete() {
    // Arrange
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    let other = Description::from_uv_lock(&graph_lock(), "unrelated").unwrap();
    let mut foreign = observations(&description, &[Some("MIT")]);
    foreign.extend(observations(&other, &[Some("MIT")]));
    // Act
    let report = description.evaluate(foreign);
    // Assert
    assert_eq!(report.exit_code(), 2);
}

#[test]
fn wrong_identity_results_are_incomplete() {
    // Arrange
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    let evidence = Evidence::from_archive(
        &common::request(
            &common::wheel(&common::metadata("License-Expression: MIT")),
            taloon::model::ArtifactFormat::Wheel,
        ),
        &common::wheel(&common::metadata("License-Expression: MIT")),
    )
    .unwrap();
    let mut results = observations(&description, &[Some("MIT")]);
    results[0] = AcquisitionResult::observed(description.requests()[0].id(), evidence);
    // Act
    let report = description.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(), 2);
}

#[test]
fn result_order_preserves_description_report() {
    // Arrange
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    let normal = observations(&description, &[Some("MIT")]);
    let mut reverse = observations(&description, &[Some("MIT")]);
    reverse.reverse();
    // Act
    let forward = description.evaluate(normal);
    let backward = description.evaluate(reverse);
    // Assert
    assert_eq!(
        serde_json::to_value(forward).unwrap(),
        serde_json::to_value(backward).unwrap()
    );
}

#[rstest::rstest]
#[case::missing("missing")]
#[case::version_selector("a==1.0")]
fn unknown_description_query_is_rejected(#[case] query: &str) {
    // Arrange
    let lock = graph_lock();
    // Act
    let result = Description::from_uv_lock(&lock, query);
    // Assert
    assert!(result.is_err());
}

#[test]
fn unsupported_targets_remain_incomplete() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='external'\nversion='1.0'\nsource={git='https://git.test/repo'}";
    let description = Description::from_uv_lock(lock, "external").unwrap();
    // Act
    let report = description.evaluate(vec![]);
    // Assert
    assert_eq!(description.requests().len(), 0);
    assert_eq!(report.exit_code(), 2);
    assert_eq!(
        serde_json::to_value(report).unwrap()["packages"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn workspace_name_collision_selects_all_and_keeps_external_identity() {
    // Arrange
    let mut lock = graph_lock();
    lock.push_str(&package(
        "app",
        "2.0",
        "registry='https://third.test/simple'",
        "",
    ));
    // Act
    let description = Description::from_uv_lock(&lock, "APP").unwrap();
    let value =
        serde_json::to_value(description.evaluate(observations(&description, &[Some("MIT")])))
            .unwrap();
    // Assert
    assert!(names(&description).contains("unrelated"));
    assert!(names(&description).contains("app"));
    assert_eq!(value["selection_mode"], "all_external");
    assert_eq!(value["roots"].as_array().unwrap().len(), 2);
    assert_eq!(
        value["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["name"] == "app")
            .count(),
        1
    );
}
#[rstest::rstest]
#[case::prose("not a license")]
#[case::external_reference("LicenseRef-External:Document")]
#[case::injected_field("MIT\nLicense-Expression: BSD-3-Clause")]
fn invalid_declarations_stay_unknown(#[case] expression: &str) {
    // Arrange
    let lock = package("example", "1.0", "registry='https://index.test/simple'", "");
    let description =
        Description::from_uv_lock(&format!("version=1\nrevision=5\n{lock}"), "example").unwrap();
    let results = observations(&description, &[Some(expression)]);
    // Act
    let report = description.evaluate(results);
    // Assert
    assert_eq!(report.exit_code(), 2);
    let report = serde_json::to_value(report).unwrap();
    assert_eq!(report["packages"][0]["unknown_artifacts"], 1);
    assert_eq!(
        report["packages"][0]["artifacts"][0]["evidence"]["strength"],
        "index_associated"
    );
}

#[test]
fn empty_project_completes() {
    // Arrange
    let lock =
        "version=1\nrevision=5\n[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}";
    let description = Description::from_uv_lock(lock, "app").unwrap();
    // Act
    let report = description.evaluate(vec![]);
    // Assert
    assert_eq!(report.exit_code(), 0);
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionCase {
    schema: u32,
    id: String,
    edges: Vec<SelectionEdge>,
    roots: Vec<usize>,
    expected: Vec<usize>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionEdge {
    source: usize,
    target: usize,
    kind: SelectionKind,
}
#[derive(serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SelectionKind {
    Normal,
    Extra,
    Development,
}

fn selection_fixture(index: usize) -> (SelectionCase, String) {
    let line = include_str!("fixtures/oracle/selection.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: SelectionCase = serde_json::from_str(line).unwrap();
    let mut lock = "version=1\nrevision=5\n".to_owned();
    let name = |n: usize| {
        if case.roots.contains(&n) {
            "query".to_owned()
        } else {
            format!("node{n}")
        }
    };
    for node in 0..4 {
        let group = |kind: SelectionKind| {
            case.edges
                .iter()
                .filter(|e| e.source == node && e.kind == kind)
                .map(|e| format!("{{name='{}',version='{}.0'}}", name(e.target), e.target + 1))
                .collect::<Vec<_>>()
                .join(",")
        };
        let fields = format!(
            "dependencies=[{}]\n[package.optional-dependencies]\nfeature=[{}]\n[package.dev-dependencies]\ntest=[{}]",
            group(SelectionKind::Normal),
            group(SelectionKind::Extra),
            group(SelectionKind::Development)
        );
        let source = if node == 3 {
            "virtual='.'"
        } else {
            "registry='https://index.test/simple'"
        };
        lock.push_str(&package(
            &name(node),
            &format!("{}.0", node + 1),
            source,
            &fields,
        ));
    }
    (case, lock)
}

#[rstest::rstest]
#[case::selection_0(0)]
#[case::selection_1(1)]
#[case::selection_2(2)]
#[case::selection_3(3)]
#[case::selection_5(5)]
#[case::selection_6(6)]
#[case::selection_7(7)]
#[case::selection_8(8)]
#[case::selection_10(10)]
#[case::selection_11(11)]
#[case::selection_12(12)]
#[case::selection_13(13)]
#[case::selection_15(15)]
#[case::selection_16(16)]
#[case::selection_17(17)]
#[case::selection_18(18)]
#[case::selection_20(20)]
#[case::selection_21(21)]
#[case::selection_22(22)]
#[case::selection_23(23)]
#[case::selection_25(25)]
#[case::selection_26(26)]
#[case::selection_27(27)]
#[case::selection_28(28)]
#[case::selection_30(30)]
#[case::selection_31(31)]
#[case::selection_32(32)]
#[case::selection_33(33)]
#[case::selection_35(35)]
#[case::selection_36(36)]
#[case::selection_37(37)]
#[case::selection_38(38)]
#[case::selection_40(40)]
#[case::selection_41(41)]
#[case::selection_42(42)]
#[case::selection_43(43)]
#[case::selection_45(45)]
#[case::selection_46(46)]
#[case::selection_47(47)]
#[case::selection_48(48)]
fn lean_selection_corpus_matches_public_description_requests(#[case] index: usize) {
    // Arrange
    let (case, lock) = selection_fixture(index);
    assert_eq!(case.schema, 1);
    // Act
    let description = Description::from_uv_lock(&lock, "query").unwrap();
    // Assert
    let mut observed: Vec<_> = description
        .requests()
        .iter()
        .map(|r| {
            r.package()
                .version()
                .split('.')
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap()
                - 1
        })
        .collect();
    observed.sort_unstable();
    observed.dedup();
    assert_eq!(observed, case.expected, "{}", case.id);
}

#[rstest::rstest]
#[case::selection_4(4)]
#[case::selection_9(9)]
#[case::selection_14(14)]
#[case::selection_19(19)]
#[case::selection_24(24)]
#[case::selection_29(29)]
#[case::selection_34(34)]
#[case::selection_39(39)]
#[case::selection_44(44)]
#[case::selection_49(49)]
fn lean_selection_without_roots_rejects_query(#[case] index: usize) {
    // Arrange
    let (case, lock) = selection_fixture(index);
    assert_eq!(case.schema, 1);
    // Act
    let result = Description::from_uv_lock(&lock, "query");
    // Assert
    assert!(result.is_err(), "{}", case.id);
}

#[test]
fn hashless_local_observations_cannot_be_substituted_between_artifacts() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='a.whl'}\nwheels=[{path='a.whl'},{path='b.whl'}]";
    let description = Description::from_uv_lock(lock, "example").unwrap();
    let requests = description.requests();
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let evidence = Evidence::from_archive(&requests[0], &bytes).unwrap();
    // Act
    let report = description.evaluate(vec![
        AcquisitionResult::observed(requests[0].id(), evidence.clone()),
        AcquisitionResult::observed(requests[1].id(), evidence),
    ]);
    // Assert
    assert_eq!(report.exit_code(), 2);
    assert_eq!(report.packages()[0].unknown_artifacts, 1);
}

#[test]
fn hashless_local_inspection_rejects_substituted_artifacts() {
    // Arrange
    let lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='a.whl'}\nwheels=[{path='a.whl'},{path='b.whl'}]";
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let date = taloon::model::EvaluationDate::parse("2026-11-01").unwrap();
    let inspection = taloon::inspection::Inspection::from_documents(
        lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        date,
    )
    .unwrap();
    let evidence = Evidence::from_archive(&inspection.requests()[0], &bytes).unwrap();
    // Act
    let report = inspection.evaluate(vec![
        AcquisitionResult::observed(inspection.requests()[0].id(), evidence.clone()),
        AcquisitionResult::observed(inspection.requests()[1].id(), evidence),
    ]);
    // Assert
    assert_eq!(report.exit_code(false), 2);
}
#[test]
fn digest_pinned_archive_observations_can_still_be_reused_by_content_identity() {
    // Arrange
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let hash = ContentHash::digest(&bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}}]"
    );
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let evidence = Evidence::from_archive(&description.requests()[0], &bytes).unwrap();
    // Act
    let report = description.evaluate(
        description
            .requests()
            .iter()
            .map(|r| AcquisitionResult::observed(r.id(), evidence.clone()))
            .collect(),
    );
    // Assert
    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.packages()[0].licenses[0].artifacts.len(), 2);
}

fn generated_graph_lock(
    size: usize,
    edges: &BTreeSet<(usize, usize, u8)>,
    reverse: bool,
) -> String {
    let mut lock = "version=1\nrevision=5\n".to_owned();
    let mut nodes: Vec<_> = (0..size).collect();
    if reverse {
        nodes.reverse();
    }
    for node in nodes {
        let group = |kind| {
            let mut deps: Vec<_> = edges
                .iter()
                .filter(|&&(from, _, k)| from == node && k == kind)
                .map(|&(_, to, _)| {
                    format!("{{name='node{to}',marker=\"python_version >= '3.14'\"}}")
                })
                .collect();
            if reverse {
                deps.reverse();
            }
            deps.join(",")
        };
        let fields = format!(
            "dependencies=[{}]\n[package.optional-dependencies]\nfeature=[{}]\n[package.dev-dependencies]\ntest=[{}]",
            group(0),
            group(1),
            group(2)
        );
        lock.push_str(&package(
            &format!("node{node}"),
            "1.0",
            "registry='https://index.test/simple'",
            &fields,
        ));
    }
    lock
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn generated_selection_matches_independent_transitive_closure(
        size in 1_usize..8,
        edges in prop::collection::vec((0_usize..8,0_usize..8,0_u8..3),0..40)
    ) {
        // Arrange
        let edges:BTreeSet<_>=edges.into_iter().map(|(a,b,k)|(a%size,b%size,k)).collect();
        let mut reachable=vec![vec![false;size];size];
        for (n,row) in reachable.iter_mut().enumerate(){row[n]=true;}
        for &(from,to,kind) in &edges {if kind<2 {reachable[from][to]=true;}}
        // Floyd-Warshall is intentionally different from the production queue walk.
        for via in 0..size {for from in 0..size {for to in 0..size {
            reachable[from][to]|=reachable[from][via]&&reachable[via][to];
        }}}
        let expected:BTreeSet<_>=(0..size).filter(|&n|reachable[0][n]).map(|n|format!("node{n}")).collect();
        // Act
        let description=Description::from_uv_lock(&generated_graph_lock(size, &edges, false),"NODE0").unwrap();
        // Assert
        prop_assert_eq!(names(&description),expected);
    }
    #[test]
    fn generated_description_is_invariant_under_input_order(
        size in 1_usize..8,
        edges in prop::collection::vec((0_usize..8,0_usize..8,0_u8..3),0..40)
    ) {
        // Arrange
        let edges: BTreeSet<_> = edges.into_iter().map(|(a,b,k)| (a % size,b % size,k)).collect();
        let description = Description::from_uv_lock(&generated_graph_lock(size, &edges, false), "node0").unwrap();
        let shuffled = Description::from_uv_lock(&generated_graph_lock(size, &edges, true), "node0").unwrap();
        let normal = observations(&description, &[Some("MIT")]);
        let mut reversed = observations(&shuffled, &[Some("MIT")]);
        reversed.reverse();
        // Act
        let forward = description.evaluate(normal);
        let backward = shuffled.evaluate(reversed);
        // Assert
        prop_assert_eq!(serde_json::to_value(forward).unwrap(), serde_json::to_value(backward).unwrap());
    }
}

#[test]
fn selection_adapter_covers_the_complete_corpus() {
    // Arrange
    let corpus = include_str!("fixtures/oracle/selection.jsonl");
    // Act
    let row_count = corpus.lines().count();
    // Assert: update the named row cases when regenerating the corpus.
    assert_eq!(row_count, 50);
}
