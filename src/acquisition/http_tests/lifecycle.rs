//! Replay Quint's ITF operations through the real acquirer and inspection API.
use super::*;
use serde::Deserialize;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

const PARENT_URL: &str = "https://files.test/parent.whl";
const CHILD_URL: &str = "https://files.test/child.whl";

#[derive(Deserialize)]
struct State {
    cache: Vec<u8>,
    operation: Operation,
    report: ExpectedReport,
}

// This wire record mirrors independent switches in the Quint operation.
#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
struct Operation {
    offline: bool,
    clear: bool,
    revision: u8,
    delivery: Delivery,
    permitted: bool,
    duplicate: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Delivery {
    Current,
    Retry,
    Stale,
    Missing,
}

#[derive(Deserialize)]
struct ExpectedReport {
    parent: u8,
    child: u8,
    exit: u8,
    hits: u8,
    attempts: u8,
}

struct RetryOnce {
    routes: Fixture,
    retry_child: AtomicBool,
}

impl Transport for RetryOnce {
    fn send(
        &self,
        url: &url::Url,
        auth: Option<&Credentials>,
        accept: &str,
        limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error> {
        if url.as_str() == CHILD_URL && self.retry_child.swap(false, Ordering::Relaxed) {
            return Ok(Response {
                status: 503,
                location: None,
                content_type: String::new(),
                body: vec![],
                url: url.clone(),
            });
        }
        self.routes.send(url, auth, accept, limit, stats)
    }
}

fn wheel(name: &str, description: &str) -> Vec<u8> {
    let metadata = format!(
        "Metadata-Version: 2.4\nName: {name}\nVersion: 1.0\nLicense-Expression: MIT\n\n{description}\n"
    );
    crate::testfixtures::wheel(metadata.as_bytes())
}

#[derive(Clone, Copy)]
enum Topology {
    Chain,
    Siblings,
    MultiplePaths,
}

fn inspection(parent: &[u8], child: &[u8], permitted: bool, topology: Topology) -> Inspection {
    let (root_edges, parent_edges) = match topology {
        Topology::Chain => ("{name='parent'}", "{name='child'}"),
        Topology::Siblings => ("{name='parent'},{name='child'}", ""),
        Topology::MultiplePaths => ("{name='parent'},{name='child'}", "{name='child'}"),
    };
    let parent_hash = ContentHash::digest(parent);
    let child_hash = ContentHash::digest(child);
    let lock = format!(
        "version=1\nrevision=5\n\
         [[package]]\nname='workspace'\nsource={{virtual='.'}}\ndependencies=[{root_edges}]\n\
         [[package]]\nname='parent'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
         dependencies=[{parent_edges}]\nwheels=[{{url='{PARENT_URL}',hash='{parent_hash}'}}]\n\
         [[package]]\nname='child'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\n\
         wheels=[{{url='{CHILD_URL}',hash='{child_hash}'}}]"
    );
    let allowed = if permitted { "'MIT'" } else { "" };
    let config = format!("[tool.taloon]\nschema-version=1\nallow=[{allowed}]");
    Inspection::from_documents(&lock, &config, EvaluationDate::parse("2026-10-31").unwrap())
        .unwrap()
}

fn request<'a>(inspection: &'a Inspection, name: &str) -> &'a ArtifactRequest {
    inspection
        .requests()
        .iter()
        .find(|r| r.package().name() == name)
        .unwrap()
}

fn replay_step(
    state: &State,
    cache_root: &Path,
    parent: &[u8],
    children: &[Vec<u8>; 2],
    topology: Topology,
) {
    // Arrange
    let op = &state.operation;
    let revision = usize::from(op.revision - 1);
    let current = &children[revision];
    let inspection = inspection(parent, current, op.permitted, topology);
    assert_eq!(inspection.requests().len(), 2);
    let fixture = Fixture::default();
    fixture.add(PARENT_URL, 200, parent);
    match op.delivery {
        Delivery::Current | Delivery::Retry => fixture.add(CHILD_URL, 200, current),
        Delivery::Stale => fixture.add(CHILD_URL, 200, &children[1 - revision]),
        Delivery::Missing => fixture.add(CHILD_URL, 404, b""),
    }
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(cache_root, op.offline),
        RetryOnce {
            routes: fixture,
            retry_child: AtomicBool::new(matches!(op.delivery, Delivery::Retry)),
        },
    )
    .unwrap();
    if op.clear {
        acquirer.cache.clear().unwrap();
    }

    // Act
    let mut batch = acquirer.acquire(inspection.requests());
    if op.duplicate {
        batch.results.push(AcquisitionResult::failed(
            request(&inspection, "child").id(),
            Error::new(Code::Acquisition, "duplicate trace observation"),
        ));
    }
    let report = inspection.evaluate(batch.results);

    // Assert
    assert_eq!(batch.summary.cache_hits, u64::from(state.report.hits));
    assert_eq!(
        batch.summary.http.archive.requests,
        u64::from(state.report.attempts)
    );
    if op.offline {
        assert_eq!(batch.summary.network_requests, 0);
    }
    assert_eq!(report.exit_code(false), state.report.exit);
    assert_eq!(report.packages().len(), 2);
    for (name, expected) in [
        ("parent", &state.report.parent),
        ("child", &state.report.child),
    ] {
        let key = request(&inspection, name).package().key();
        let package = report.packages().iter().find(|p| p.package == key).unwrap();
        assert_eq!(package.status.exit_code(false, false), *expected);
        if name == "child" && *expected != 0 {
            let parent = request(&inspection, "parent").package().to_string();
            let child = request(&inspection, "child").package().to_string();
            assert!(
                package
                    .paths
                    .iter()
                    .any(|path| match (topology, path.steps.as_slice()) {
                        (Topology::Chain, [first, last]) =>
                            first.package == parent && last.package == child,
                        (Topology::Siblings | Topology::MultiplePaths, [only]) =>
                            only.package == child,
                        _ => false,
                    })
            );
        }
    }
    assert_cache(state, &acquirer, parent, children, topology);
}

fn assert_cache(
    state: &State,
    acquirer: &Acquirer,
    parent: &[u8],
    children: &[Vec<u8>; 2],
    topology: Topology,
) {
    let first = inspection(parent, &children[0], true, topology);
    let second = inspection(parent, &children[1], true, topology);
    let keys = [
        request(&first, "parent"),
        request(&first, "child"),
        request(&second, "child"),
    ];
    for (key, request) in (0_u8..).zip(keys) {
        let expected = state.cache.contains(&key);
        assert_eq!(
            acquirer
                .cache
                .get(request, request.hash().unwrap())
                .unwrap()
                .is_some(),
            expected
        );
    }
}

#[rstest::rstest]
fn quint_replays_acquisition_and_inspection_lifecycle(
    #[values(1, 2, 3, 4, 5, 6, 7, 8)] seed: usize,
    #[values(Topology::Chain, Topology::Siblings, Topology::MultiplePaths)] topology: Topology,
) {
    // Arrange
    let line = include_str!("../../../tests/fixtures/oracle/lifecycle.jsonl")
        .lines()
        .nth(seed - 1)
        .unwrap();
    let trace: Vec<State> = serde_json::from_str(line).unwrap();
    assert_eq!(trace.len(), 24);
    let parent = wheel("parent", "parent");
    let children = [wheel("child", "old"), wheel("child", "new")];
    let cache = tempfile::tempdir().unwrap();

    // Act and assert each transition against the generated state.
    for (step, state) in trace.iter().enumerate() {
        eprintln!("Quint seed {seed}, step {}", step + 1);
        replay_step(state, cache.path(), &parent, &children, topology);
    }
}
