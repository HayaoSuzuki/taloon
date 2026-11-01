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
    let description = Description::from_uv_lock(&graph_lock(), "A").unwrap();
    assert_eq!(
        names(&description),
        BTreeSet::from_iter(["a", "b", "c", "d"].map(str::to_owned))
    );
    assert_eq!(description.requests().len(), 5);
    let report = description.evaluate(observations(&description, &[Some("MIT")]));
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
#[test]
fn project_and_member_description_exactly_match_whole_external_inventory() {
    let lock = graph_lock();
    let resolution = Resolution::from_uv_lock(&lock).unwrap();
    let expected: BTreeSet<_> = resolution
        .targets()
        .iter()
        .flat_map(taloon::resolution::Target::requests)
        .map(taloon::model::ArtifactRequest::id)
        .collect();
    for query in ["app", "member"] {
        let description = Description::from_uv_lock(&lock, query).unwrap();
        assert_eq!(
            description
                .requests()
                .iter()
                .map(taloon::model::ArtifactRequest::id)
                .collect::<BTreeSet<_>>(),
            expected
        );
        let report =
            serde_json::to_value(description.evaluate(observations(&description, &[Some("MIT")])))
                .unwrap();
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
}
#[test]
fn description_preserves_different_unknown_and_custom_declarations() {
    let hash = ContentHash::digest(b"archive");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}},{{url='https://files.test/c.whl',hash='{hash}'}}]"
    );
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let results = observations(&description, &[Some("MIT"), Some("BSD-3-Clause"), None]);
    let report = description.evaluate(results);
    assert_eq!(report.exit_code(), 2);
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(
        value["packages"][0]["licenses"].as_array().unwrap().len(),
        2
    );
    assert_eq!(value["packages"][0]["unknown_artifacts"], 1);
    let report = description.evaluate(observations(&description, &[Some("LicenseRef-Local")]));
    assert_eq!(report.exit_code(), 0);
    assert_eq!(
        serde_json::to_value(report).unwrap()["packages"][0]["licenses"][0]["expression"],
        "LicenseRef-Local"
    );
}
#[test]
fn description_accounts_missing_duplicate_foreign_and_wrong_identity_results() {
    let description = Description::from_uv_lock(&graph_lock(), "a").unwrap();
    assert_eq!(description.evaluate(vec![]).exit_code(), 2);
    let mut duplicate = observations(&description, &[Some("MIT")]);
    duplicate.extend(observations(&description, &[Some("MIT")]));
    assert_eq!(description.evaluate(duplicate).exit_code(), 2);
    let other = Description::from_uv_lock(&graph_lock(), "unrelated").unwrap();
    let mut foreign = observations(&description, &[Some("MIT")]);
    foreign.extend(observations(&other, &[Some("MIT")]));
    assert_eq!(description.evaluate(foreign).exit_code(), 2);
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
    assert_eq!(description.evaluate(results).exit_code(), 2);
    let normal = observations(&description, &[Some("MIT")]);
    let mut reverse = observations(&description, &[Some("MIT")]);
    reverse.reverse();
    assert_eq!(
        serde_json::to_value(description.evaluate(normal)).unwrap(),
        serde_json::to_value(description.evaluate(reverse)).unwrap()
    );
}
#[test]
fn unknown_names_fail_and_unsupported_targets_remain_incomplete() {
    assert!(Description::from_uv_lock(&graph_lock(), "missing").is_err());
    assert!(Description::from_uv_lock(&graph_lock(), "a==1.0").is_err());
    let lock = "version=1\nrevision=5\n[[package]]\nname='external'\nversion='1.0'\nsource={git='https://git.test/repo'}";
    let description = Description::from_uv_lock(lock, "external").unwrap();
    assert_eq!(description.requests().len(), 0);
    let report = description.evaluate(vec![]);
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
    let mut lock = graph_lock();
    lock.push_str(&package(
        "app",
        "2.0",
        "registry='https://third.test/simple'",
        "",
    ));
    let description = Description::from_uv_lock(&lock, "APP").unwrap();
    assert!(names(&description).contains("unrelated"));
    assert!(names(&description).contains("app"));
    let value =
        serde_json::to_value(description.evaluate(observations(&description, &[Some("MIT")])))
            .unwrap();
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
#[test]
fn invalid_declarations_stay_unknown_and_empty_project_completes() {
    let lock = package("example", "1.0", "registry='https://index.test/simple'", "");
    let description =
        Description::from_uv_lock(&format!("version=1\nrevision=5\n{lock}"), "example").unwrap();
    for expression in [
        "not a license",
        "LicenseRef-External:Document",
        "MIT\nLicense-Expression: BSD-3-Clause",
    ] {
        let report = description.evaluate(observations(&description, &[Some(expression)]));
        assert_eq!(report.exit_code(), 2);
        let report = serde_json::to_value(report).unwrap();
        assert_eq!(report["packages"][0]["unknown_artifacts"], 1);
        assert_eq!(
            report["packages"][0]["artifacts"][0]["evidence"]["strength"],
            "index_associated"
        );
    }
    let empty =
        "version=1\nrevision=5\n[[package]]\nname='app'\nversion='1.0'\nsource={virtual='.'}";
    assert_eq!(
        Description::from_uv_lock(empty, "app")
            .unwrap()
            .evaluate(vec![])
            .exit_code(),
        0
    );
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn generated_selection_matches_independent_transitive_closure(
        size in 1_usize..8,
        edges in prop::collection::vec((0_usize..8,0_usize..8,0_u8..3),0..40)
    ) {
        let edges:BTreeSet<_>=edges.into_iter().map(|(a,b,k)|(a%size,b%size,k)).collect();
        let mut reachable=vec![vec![false;size];size];
        for (n,row) in reachable.iter_mut().enumerate(){row[n]=true;}
        for &(from,to,kind) in &edges {if kind<2 {reachable[from][to]=true;}}
        // Floyd-Warshall is intentionally different from the production queue walk.
        for via in 0..size {for from in 0..size {for to in 0..size {
            reachable[from][to]|=reachable[from][via]&&reachable[via][to];
        }}}
        let expected:BTreeSet<_>=(0..size).filter(|&n|reachable[0][n]).map(|n|format!("node{n}")).collect();
        let build=|reverse:bool| {
            let mut lock="version=1\nrevision=5\n".to_owned();
            let mut nodes:Vec<_>=(0..size).collect();if reverse {nodes.reverse();}
            for node in nodes {
                let group=|kind| {
                    let mut deps:Vec<_>=edges.iter().filter(|&&(from,_,k)|from==node&&k==kind).map(|&(_,to,_)|format!("{{name='node{to}',marker=\"python_version >= '3.14'\"}}")).collect();
                    if reverse {deps.reverse();}deps.join(",")
                };
                let fields=format!("dependencies=[{}]\n[package.optional-dependencies]\nfeature=[{}]\n[package.dev-dependencies]\ntest=[{}]",group(0),group(1),group(2));
                lock.push_str(&package(&format!("node{node}"),"1.0","registry='https://index.test/simple'",&fields));
            }
            lock
        };
        let description=Description::from_uv_lock(&build(false),"NODE0").unwrap();
        prop_assert_eq!(names(&description),expected);
        let shuffled=Description::from_uv_lock(&build(true),"node0").unwrap();
        let report=description.evaluate(observations(&description,&[Some("MIT")]));
        let mut results=observations(&shuffled,&[Some("MIT")]);results.reverse();
        prop_assert_eq!(serde_json::to_value(report).unwrap(),serde_json::to_value(shuffled.evaluate(results)).unwrap());
    }
}

#[test]
fn lean_selection_corpus_matches_public_description_requests() {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Case {
        schema: u32,
        id: String,
        edges: Vec<Edge>,
        roots: Vec<usize>,
        expected: Vec<usize>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Edge {
        source: usize,
        target: usize,
        kind: Kind,
    }
    #[derive(serde::Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Kind {
        Normal,
        Extra,
        Development,
    }
    let mut count = 0;
    for line in include_str!("fixtures/oracle/selection.jsonl").lines() {
        let case: Case = serde_json::from_str(line).unwrap();
        assert_eq!(case.schema, 1);
        let mut lock = "version=1\nrevision=5\n".to_owned();
        let name = |n: usize| {
            if case.roots.contains(&n) {
                "query".to_owned()
            } else {
                format!("node{n}")
            }
        };
        for node in 0..4 {
            let group = |kind: Kind| {
                case.edges
                    .iter()
                    .filter(|e| e.source == node && e.kind == kind)
                    .map(|e| format!("{{name='{}',version='{}.0'}}", name(e.target), e.target + 1))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let fields = format!(
                "dependencies=[{}]\n[package.optional-dependencies]\nfeature=[{}]\n[package.dev-dependencies]\ntest=[{}]",
                group(Kind::Normal),
                group(Kind::Extra),
                group(Kind::Development)
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
        let description = Description::from_uv_lock(&lock, "query");
        if case.roots.is_empty() {
            assert!(description.is_err(), "{}", case.id);
        } else {
            let description = description.unwrap();
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
        count += 1;
    }
    assert_eq!(count, 50);
}

#[test]
fn hashless_local_observations_cannot_be_substituted_between_artifacts() {
    let lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={path='a.whl'}\nwheels=[{path='a.whl'},{path='b.whl'}]";
    let description = Description::from_uv_lock(lock, "example").unwrap();
    let requests = description.requests();
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let evidence = Evidence::from_archive(&requests[0], &bytes).unwrap();
    let report = description.evaluate(vec![
        AcquisitionResult::observed(requests[0].id(), evidence.clone()),
        AcquisitionResult::observed(requests[1].id(), evidence),
    ]);
    assert_eq!(report.exit_code(), 2);
    assert_eq!(report.packages()[0].unknown_artifacts, 1);
    let date = taloon::model::EvaluationDate::parse("2026-11-01").unwrap();
    let inspection = taloon::inspection::Inspection::from_documents(
        lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        date,
    )
    .unwrap();
    let evidence = Evidence::from_archive(&inspection.requests()[0], &bytes).unwrap();
    let report = inspection.evaluate(vec![
        AcquisitionResult::observed(inspection.requests()[0].id(), evidence.clone()),
        AcquisitionResult::observed(inspection.requests()[1].id(), evidence),
    ]);
    assert_eq!(report.exit_code(false), 2);
}
#[test]
fn digest_pinned_archive_observations_can_still_be_reused_by_content_identity() {
    let bytes = common::wheel(&common::metadata("License-Expression: MIT"));
    let hash = ContentHash::digest(&bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}}]"
    );
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let evidence = Evidence::from_archive(&description.requests()[0], &bytes).unwrap();
    let report = description.evaluate(
        description
            .requests()
            .iter()
            .map(|r| AcquisitionResult::observed(r.id(), evidence.clone()))
            .collect(),
    );
    assert_eq!(report.exit_code(), 0);
    assert_eq!(report.packages()[0].licenses[0].artifacts.len(), 2);
}
