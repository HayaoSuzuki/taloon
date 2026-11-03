use super::{
    http::{Credentials, Response, Transport},
    *,
};
use std::sync::{Arc, Mutex};
use taloon::{inspection::Inspection, model::EvaluationDate};
#[derive(Clone, Default)]
struct Fixture {
    routes: Arc<Mutex<std::collections::BTreeMap<String, Reply>>>,
    calls: Arc<Mutex<Vec<Call>>>,
    delay: std::time::Duration,
}
#[derive(Clone)]
struct Reply {
    status: u16,
    body: Vec<u8>,
    location: Option<String>,
    content_type: String,
}
struct Call {
    url: String,
    auth: Option<String>,
}
impl Fixture {
    fn add(&self, url: &str, status: u16, body: &[u8]) {
        self.routes.lock().unwrap().insert(
            url.into(),
            Reply {
                status,
                body: body.to_vec(),
                location: None,
                content_type: "application/vnd.pypi.simple.v1+json".into(),
            },
        );
    }
    fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}
impl Transport for Fixture {
    fn send(
        &self,
        url: &url::Url,
        auth: Option<&Credentials>,
        _accept: &str,
        limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error> {
        std::thread::sleep(self.delay);
        self.calls.lock().unwrap().push(Call {
            url: url.to_string(),
            auth: auth.map(|c| format!("{}:{}", c.login, c.password)),
        });
        let reply = self
            .routes
            .lock()
            .unwrap()
            .get(url.as_str())
            .cloned()
            .unwrap_or(Reply {
                status: 404,
                body: vec![],
                location: None,
                content_type: String::new(),
            });
        let Reply {
            status,
            body,
            location,
            content_type,
        } = reply;
        stats.downloaded_bytes += u64::try_from(body.len()).unwrap();
        if u64::try_from(body.len()).unwrap() > limit {
            return Err(http::failure());
        }
        Ok(Response {
            status,
            location,
            content_type,
            body,
            url: url.clone(),
        })
    }
}

#[test]
fn acquisition_progress_starts_and_ticks_while_transport_is_blocked() {
    use std::{sync::mpsc, time::Duration};
    struct Held {
        fixture: Fixture,
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        started: std::sync::atomic::AtomicBool,
    }
    impl Transport for Held {
        fn send(
            &self,
            url: &url::Url,
            auth: Option<&Credentials>,
            accept: &str,
            limit: u64,
            stats: &mut AcquisitionSummary,
        ) -> Result<Response, Error> {
            if !self.started.swap(true, std::sync::atomic::Ordering::SeqCst) {
                self.entered.send(()).unwrap();
                self.release.lock().unwrap().recv().unwrap();
            }
            self.fixture.send(url, auth, accept, limit, stats)
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let bytes =
        crate::testfixtures::wheel(&crate::testfixtures::metadata("License-Expression: MIT"));
    let fixture = Fixture::default();
    fixture.add("https://files.test/a.whl", 200, &bytes);
    fixture.add("https://files.test/b.whl", 200, &bytes);
    let requests = fixture_inspection(&bytes, false).requests().to_vec();
    let total = requests.len();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        Held {
            fixture,
            entered: entered_tx,
            release: Mutex::new(release_rx),
            started: std::sync::atomic::AtomicBool::new(false),
        },
    )
    .unwrap();
    let (progress_tx, progress_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        acquirer.acquire_with_progress(&requests, |progress| {
            let _ = progress_tx.send(progress);
        })
    });
    let first = progress_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!((first.completed, first.total), (0, total));
    entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let waiting = progress_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!((waiting.completed, waiting.total), (0, total));
    release_tx.send(()).unwrap();
    let batch = worker.join().unwrap();
    assert_eq!(batch.results.len(), total);
    let remaining: Vec<_> = progress_rx.try_iter().collect();
    assert_eq!(remaining.last().unwrap().completed, total);
    assert!(
        remaining
            .iter()
            .all(|p| p.completed <= p.total && p.total == total)
    );
    assert!(
        remaining
            .windows(2)
            .all(|p| p[0].completed <= p[1].completed)
    );
}

#[test]
fn acquisition_progress_accounts_for_panicked_workers_and_empty_scope() {
    struct Panicking;
    impl Transport for Panicking {
        fn send(
            &self,
            _url: &url::Url,
            _auth: Option<&Credentials>,
            _accept: &str,
            _limit: u64,
            stats: &mut AcquisitionSummary,
        ) -> Result<Response, Error> {
            stats.downloaded_bytes += 7;
            std::thread::sleep(std::time::Duration::from_millis(1));
            panic!("synthetic transport panic");
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let bytes =
        crate::testfixtures::wheel(&crate::testfixtures::metadata("License-Expression: MIT"));
    let inspection = fixture_inspection(&bytes, false);
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), Panicking)
            .unwrap();
    let mut progress = Vec::new();
    let batch = acquirer.acquire_with_progress(inspection.requests(), |p| progress.push(p));
    assert_eq!(batch.results.len(), inspection.requests().len());
    assert_eq!(batch.summary.network_requests, 1);
    assert_eq!(batch.summary.downloaded_bytes, 7);
    assert_eq!(batch.summary.http.index.requests, 1);
    assert_eq!(batch.summary.http.index.downloaded_bytes, 7);
    assert!(batch.summary.http.index.request_time_us >= 1000);
    assert_eq!(
        progress.last().unwrap().completed,
        inspection.requests().len()
    );
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 2);
    progress.clear();
    let batch = acquirer.acquire_with_progress(&[], |p| progress.push(p));
    assert!(batch.results.is_empty());
    assert_eq!(
        progress,
        vec![AcquisitionProgress {
            completed: 0,
            total: 0
        }]
    );
}
fn fixture_inspection(bytes: &[u8], required: bool) -> Inspection {
    let hash = ContentHash::digest(bytes);
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{hash}'}},{{url='https://files.test/b.whl',hash='{hash}'}}]"
    );
    let config = if required {
        format!(
            "[tool.taloon]\nschema-version=1\nallow=['MIT']\n[[tool.taloon.clarifications]]\nartifact-hash='{hash}'\nexpression='MIT'\nreason='reviewed'"
        )
    } else {
        "[tool.taloon]\nschema-version=1\nallow=['MIT']".into()
    };
    Inspection::from_documents(&lock, &config, EvaluationDate::parse("2026-10-31").unwrap())
        .unwrap()
}

#[test]
fn http_metrics_count_redirects_and_retries() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Sequenced(AtomicUsize);
    impl Transport for Sequenced {
        fn send(
            &self,
            url: &url::Url,
            _auth: Option<&Credentials>,
            _accept: &str,
            _limit: u64,
            stats: &mut AcquisitionSummary,
        ) -> Result<Response, Error> {
            let n = self.0.fetch_add(1, Ordering::SeqCst);
            let (response_code, size, location) = match n {
                0 => (429, 3, None),
                1 => (302, 2, Some("/final".into())),
                _ => (200, 4, None),
            };
            stats.downloaded_bytes += size;
            Ok(Response {
                status: response_code,
                body: vec![0; usize::try_from(size).unwrap()],
                location,
                content_type: String::new(),
                url: url.clone(),
            })
        }
    }
    let http = http::Http::new(
        Sequenced(AtomicUsize::new(0)),
        http::Authentication::default(),
    );
    let mut summary = AcquisitionSummary::default();
    http.get(
        RequestPurpose::Metadata,
        url::Url::parse("https://example.test/start").unwrap(),
        "test",
        64,
        &mut summary,
    )
    .unwrap();
    assert_eq!((summary.network_requests, summary.downloaded_bytes), (3, 9));
    assert_eq!(
        (
            summary.http.metadata.requests,
            summary.http.metadata.downloaded_bytes
        ),
        (3, 9)
    );
    assert_eq!(
        summary.http.index.requests + summary.http.archive.requests,
        0
    );
    assert!(summary.http.metadata.retry_delay_us >= 50_000);
}

#[test]
fn http_metrics_count_failed_partial_reads() {
    struct Partial;
    impl Transport for Partial {
        fn send(
            &self,
            _url: &url::Url,
            _auth: Option<&Credentials>,
            _accept: &str,
            _limit: u64,
            stats: &mut AcquisitionSummary,
        ) -> Result<Response, Error> {
            stats.downloaded_bytes += 7;
            std::thread::sleep(std::time::Duration::from_millis(1));
            Err(http::failure())
        }
    }
    let http = http::Http::new(Partial, http::Authentication::default());
    let mut summary = AcquisitionSummary::default();
    assert!(
        http.get(
            RequestPurpose::Archive,
            url::Url::parse("https://example.test/file").unwrap(),
            "test",
            64,
            &mut summary
        )
        .is_err()
    );
    assert_eq!(
        (summary.network_requests, summary.downloaded_bytes),
        (3, 21)
    );
    assert_eq!(
        (
            summary.http.archive.requests,
            summary.http.archive.downloaded_bytes
        ),
        (3, 21)
    );
    assert!(summary.http.archive.request_time_us >= 3000);
    assert!(summary.http.archive.retry_delay_us >= 150_000);
}

fn many_artifact_fixture(count: usize) -> (Fixture, Inspection, usize) {
    use std::fmt::Write as _;
    let transport = Fixture {
        delay: std::time::Duration::from_millis(10),
        ..Fixture::default()
    };
    let mut files = Vec::new();
    let mut lock = "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={registry='https://example.test/simple'}\nwheels=[".to_owned();
    for n in 0..count {
        let meta =
            crate::testfixtures::metadata(&format!("License-Expression: MIT\nX-Variant: {n}"));
        let archive = crate::testfixtures::wheel(&meta);
        let hash = ContentHash::digest(&archive);
        let metadata_hash = ContentHash::digest(&meta)
            .to_string()
            .replace("sha256:", "");
        let url = format!("https://files.test/artifact-{n}.whl");
        files.push(serde_json::json!({"url":url,"hashes":{"sha256":hash.to_string().replace("sha256:", "")},"core-metadata":{"sha256":metadata_hash}}));
        transport.add(&format!("{url}.metadata"), 200, &meta);
        transport.add(&url, 200, &archive);
        write!(lock, "{{url='{url}',hash='{hash}'}},").unwrap();
    }
    lock.push(']');
    let index =
        serde_json::to_vec(&serde_json::json!({"meta":{"api-version":"1.0"},"files":files}))
            .unwrap();
    let index_bytes = index.len();
    transport.add("https://example.test/simple/example/", 200, &index);
    let inspection = Inspection::from_documents(
        &lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        EvaluationDate::parse("2026-11-01").unwrap(),
    )
    .unwrap();
    (transport, inspection, index_bytes)
}

#[test]
fn one_index_snapshot_covers_all_concurrent_locked_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, index_bytes) = many_artifact_fixture(16);
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(inspection.requests());
    assert_eq!(batch.results.len(), 16);
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 0);
    assert_eq!(batch.summary.http.index.requests, 1);
    assert_eq!(batch.summary.index_parse_count, 1);
    assert_eq!(
        batch.summary.http.index.downloaded_bytes,
        u64::try_from(index_bytes).unwrap()
    );
    assert_eq!(batch.summary.http.metadata.requests, 16);
    assert_eq!(batch.summary.http.archive.requests, 0);
    assert_eq!(batch.summary.index_cache_hits, 15);
    assert!(batch.summary.index_wait_us > 0);
}

fn mutate_fixture_index(fixture: &Fixture, change: impl FnOnce(&mut serde_json::Value)) {
    let mut routes = fixture.routes.lock().unwrap();
    let reply = routes
        .get_mut("https://example.test/simple/example/")
        .unwrap();
    let mut document = serde_json::from_slice(&reply.body).unwrap();
    change(&mut document);
    reply.body = serde_json::to_vec(&document).unwrap();
}

#[test]
fn index_snapshot_is_fresh_per_call_and_warm_offline_bypasses_it() {
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, _) = many_artifact_fixture(8);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let first = acquirer.acquire(inspection.requests());
    assert_eq!(first.summary.http.index.requests, 1);
    assert_eq!(inspection.evaluate(first.results).exit_code(false), 0);
    let warm = acquirer.acquire(inspection.requests());
    assert_eq!(
        (
            warm.summary.network_requests,
            warm.summary.index_parse_count
        ),
        (0, 0)
    );
    let offline = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), true),
        fixture.clone(),
    )
    .unwrap();
    let cached = offline.acquire(inspection.requests());
    assert_eq!(
        (
            cached.summary.network_requests,
            cached.summary.index_parse_count
        ),
        (0, 0)
    );
    assert_eq!(cached.summary.cache_hits, 8);
    std::fs::remove_dir_all(temp.path().join("cache")).unwrap();
    mutate_fixture_index(&fixture, |document| {
        document["files"][0]
            .as_object_mut()
            .unwrap()
            .remove("core-metadata");
    });
    let fresh = acquirer.acquire(inspection.requests());
    assert_eq!(fresh.summary.http.index.requests, 1);
    assert_eq!(fresh.summary.http.archive.requests, 1);
    assert_eq!(fresh.summary.http.metadata.requests, 7);
    assert_eq!(inspection.evaluate(fresh.results).exit_code(false), 0);
}

#[test]
fn bad_bindings_do_not_invalidate_unrelated_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, _) = many_artifact_fixture(3);
    mutate_fixture_index(&fixture, |document| {
        document["files"][0]["hashes"]["sha256"] = serde_json::Value::String("0".repeat(64));
        let duplicate = document["files"][1].clone();
        document["files"].as_array_mut().unwrap().push(duplicate);
    });
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(inspection.requests());
    assert_eq!(
        (
            batch.summary.http.index.requests,
            batch.summary.index_parse_count
        ),
        (1, 1)
    );
    assert_eq!(batch.summary.http.metadata.requests, 1);
    assert_eq!(batch.summary.http.archive.requests, 2);
    assert_eq!(batch.warnings.len(), 2);
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 0);
}

#[test]
fn terminal_index_failure_is_shared_but_each_archive_is_verified() {
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, _) = many_artifact_fixture(8);
    fixture.add("https://example.test/simple/example/", 503, b"unavailable");
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(inspection.requests());
    assert_eq!(batch.summary.http.index.requests, 3);
    assert_eq!(batch.summary.index_parse_count, 0);
    assert_eq!(batch.summary.index_cache_hits, 7);
    assert_eq!(batch.summary.http.metadata.requests, 0);
    assert_eq!(batch.summary.http.archive.requests, 8);
    assert_eq!(batch.summary.archive_extractions, 8);
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 0);
}

#[test]
fn same_package_from_different_source_queries_keeps_separate_snapshots() {
    use std::fmt::Write as _;
    let temp = tempfile::tempdir().unwrap();
    let fixture = Fixture::default();
    let mut lock = "version=1\nrevision=5\n".to_owned();
    for tenant in ["a", "b"] {
        let metadata =
            crate::testfixtures::metadata(&format!("License-Expression: MIT\nX-Variant: {tenant}"));
        let bytes = crate::testfixtures::wheel(&metadata);
        let hash = ContentHash::digest(&bytes);
        let url = format!("https://files.test/{tenant}.whl");
        let index = serde_json::to_vec(&serde_json::json!({"files":[{"url":url,
            "hashes":{"sha256":hash.to_string().replace("sha256:","")},
            "core-metadata":{"sha256":ContentHash::digest(&metadata).to_string().replace("sha256:","")}}]})).unwrap();
        fixture.add(
            &format!("https://example.test/simple/example/?tenant={tenant}"),
            200,
            &index,
        );
        fixture.add(&format!("{url}.metadata"), 200, &metadata);
        fixture.add(&url, 200, &bytes);
        writeln!(lock,"[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://example.test/simple?tenant={tenant}'}}\nwheels=[{{url='{url}',hash='{hash}'}}]").unwrap();
    }
    let inspection = Inspection::from_documents(
        &lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        EvaluationDate::parse("2026-11-01").unwrap(),
    )
    .unwrap();
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(inspection.requests());
    assert_eq!(batch.summary.http.index.requests, 2);
    assert_eq!(batch.summary.http.metadata.requests, 2);
    assert_eq!(batch.summary.http.archive.requests, 0);
    assert_eq!(batch.summary.index_cache_hits, 0);
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 0);
}

#[test]
fn panicked_index_initializer_releases_waiters_and_keeps_traffic() {
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        time::Duration,
    };
    struct PanicOnce {
        fixture: Fixture,
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
        started: AtomicBool,
    }
    impl Transport for PanicOnce {
        fn send(
            &self,
            url: &url::Url,
            auth: Option<&Credentials>,
            accept: &str,
            limit: u64,
            stats: &mut AcquisitionSummary,
        ) -> Result<Response, Error> {
            if url.host_str() == Some("example.test") && !self.started.swap(true, Ordering::SeqCst)
            {
                stats.downloaded_bytes += 5;
                self.entered.send(()).unwrap();
                self.release.lock().unwrap().recv().unwrap();
                panic!("synthetic initializer panic");
            }
            self.fixture.send(url, auth, accept, limit, stats)
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, index_bytes) = many_artifact_fixture(16);
    let requests = inspection.requests().to_vec();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        PanicOnce {
            fixture,
            entered: entered_tx,
            release: Mutex::new(release_rx),
            started: AtomicBool::new(false),
        },
    )
    .unwrap();
    let (progress_tx, progress_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = acquirer.acquire_with_progress(&requests, |p| {
            let _ = progress_tx.send(p);
        });
        let _ = result_tx.send(result);
    });
    assert_eq!(
        progress_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .completed,
        0
    );
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        progress_rx
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .completed,
        0
    );
    release_tx.send(()).unwrap();
    let batch = result_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    worker.join().unwrap();
    assert_eq!(batch.results.len(), 16);
    assert_eq!(batch.summary.http.index.requests, 2);
    assert_eq!(
        batch.summary.http.index.downloaded_bytes,
        5 + u64::try_from(index_bytes).unwrap()
    );
    assert_eq!(batch.summary.index_parse_count, 1);
    assert!(batch.summary.index_wait_us > 0);
    assert_eq!(inspection.evaluate(batch.results).exit_code(false), 2);
}

#[test]
#[ignore = "controlled initial-acquisition comparison; three fresh evidence caches"]
fn cold_index_acquisition_benchmark() {
    for trial in 1..=3 {
        let temp = tempfile::tempdir().unwrap();
        let (fixture, inspection, index_bytes) = many_artifact_fixture(48);
        let acquirer = Acquirer::with_transport(
            AcquisitionOptions::testing(temp.path(), false),
            fixture.clone(),
        )
        .unwrap();
        let batch = acquirer.acquire(inspection.requests());
        assert_eq!(batch.results.len(), 48);
        assert_eq!(inspection.evaluate(batch.results).exit_code(false), 0);
        let summary = batch.summary;
        assert_eq!(summary.http.index.requests, 1);
        assert_eq!(summary.index_parse_count, 1);
        assert_eq!(summary.index_cache_hits, 47);
        assert_eq!(summary.http.metadata.requests, 48);
        assert_eq!(summary.http.archive.requests, 0);
        assert_eq!(summary.archive_extractions, 0);
        assert_eq!(summary.network_requests, 49);
        assert_eq!(fixture.count(), 49);
        assert_eq!(
            summary.http.index.downloaded_bytes,
            u64::try_from(index_bytes).unwrap()
        );
        assert_eq!(
            summary.downloaded_bytes,
            summary.http.index.downloaded_bytes + summary.http.metadata.downloaded_bytes
        );
        println!(
            "trial={trial}, index_response_bytes={index_bytes}, summary={}",
            serde_json::to_string(&summary).unwrap()
        );
    }
}
#[test]
fn sidecar_fallback_and_offline_keep_scope() {
    let temp = tempfile::tempdir().unwrap();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let mh = ContentHash::digest(&meta)
        .to_string()
        .replace("sha256:", "");
    let transport = Fixture::default();
    let index = format!(
        "{{\"meta\":{{\"api-version\":\"1.0\"}},\"files\":[{{\"url\":\"https://files.test/a.whl\",\"hashes\":{{\"sha256\":\"{}\"}},\"core-metadata\":{{\"sha256\":\"{mh}\"}}}},{{\"url\":\"https://files.test/b.whl\",\"hashes\":{{\"sha256\":\"{}\"}}}}]}}",
        hash.to_string().replace("sha256:", ""),
        hash.to_string().replace("sha256:", "")
    );
    transport.add(
        "https://example.test/simple/example/",
        200,
        index.as_bytes(),
    );
    transport.add("https://files.test/a.whl.metadata", 200, &meta);
    transport.add("https://files.test/a.whl", 200, &bytes);
    transport.add("https://files.test/b.whl", 200, &bytes);
    let i = fixture_inspection(&bytes, false);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        transport.clone(),
    )
    .unwrap();
    let batch = acquirer.acquire(i.requests());
    assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
    assert!(transport.count() > 0);
    // Separate cache containing only sidecar evidence: a stronger policy cannot promote it.
    let cache = EvidenceCache::new(temp.path().join("weak"));
    let r = &i.requests()[0];
    let e = Evidence::from_sidecar(
        r,
        taloon::evidence::SidecarAssociation::new(hash, hash, ContentHash::digest(&meta)).unwrap(),
        &meta,
    )
    .unwrap();
    cache.put(r, &e).unwrap();
    let mut options = AcquisitionOptions::testing(temp.path(), true);
    options.cache_dir = temp.path().join("weak");
    let before = transport.count();
    let offline = Acquirer::with_transport(options, transport.clone()).unwrap();
    let strong = fixture_inspection(&bytes, true);
    let batch = offline.acquire(strong.requests());
    assert_eq!(strong.evaluate(batch.results).exit_code(false), 2);
    assert_eq!(transport.count(), before);
    let mut online_options = AcquisitionOptions::testing(temp.path(), false);
    online_options.cache_dir = temp.path().join("weak");
    let online = Acquirer::with_transport(online_options, transport).unwrap();
    let batch = online.acquire(strong.requests());
    assert_eq!(batch.summary.archive_extractions, 1);
    assert_eq!(batch.summary.cache_hits, 1);
    assert_eq!(strong.evaluate(batch.results).exit_code(false), 0);
}

#[test]
fn redirects_and_errors_do_not_leak_or_skip() {
    let temp = tempfile::tempdir().unwrap();
    let auth_path = temp.path().join("netrc");
    std::fs::write(&auth_path,"# fixture\nmachine example.test login 'test user' password '秘密 token'\nmachine other.test login other password other-token\ndefault login default password must-not-send").unwrap();
    let fixture = Fixture::default();
    fixture.add("https://example.test/start?token=secret", 302, b"");
    fixture
        .routes
        .lock()
        .unwrap()
        .get_mut("https://example.test/start?token=secret")
        .unwrap()
        .location = Some("/same".into());
    fixture.add("https://example.test/same", 302, b"");
    fixture
        .routes
        .lock()
        .unwrap()
        .get_mut("https://example.test/same")
        .unwrap()
        .location = Some("https://other.test/final".into());
    fixture.add("https://other.test/final", 200, b"ok");
    let http = http::Http::new(
        fixture.clone(),
        http::Authentication::from_file(Some(&auth_path)).unwrap(),
    );
    let mut stats = AcquisitionSummary::default();
    let response = http
        .get(
            RequestPurpose::Archive,
            url::Url::parse("https://inline:password@example.test/start?token=secret#fragment")
                .unwrap(),
            "test",
            64,
            &mut stats,
        )
        .unwrap();
    assert_eq!(response.body, b"ok");
    let calls = fixture.calls.lock().unwrap();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].auth.as_deref(), Some("test user:秘密 token"));
    assert_eq!(calls[1].auth.as_deref(), Some("test user:秘密 token"));
    assert!(calls[2].auth.is_none());
    assert!(!calls[0].url.contains("inline"));
    drop(calls);
    fixture.add("https://example.test/downgrade", 302, b"");
    fixture
        .routes
        .lock()
        .unwrap()
        .get_mut("https://example.test/downgrade")
        .unwrap()
        .location = Some("http://other.test/insecure".into());
    let before = fixture.count();
    assert!(
        http.get(
            RequestPurpose::Archive,
            url::Url::parse("https://example.test/downgrade").unwrap(),
            "test",
            64,
            &mut stats
        )
        .is_err()
    );
    assert_eq!(fixture.count(), before + 1);
    fixture.add(
        "https://example.test/unavailable?secret=token",
        503,
        b"retry",
    );
    let before = fixture.count();
    let error = http
        .get(
            RequestPurpose::Archive,
            url::Url::parse("https://example.test/unavailable?secret=token").unwrap(),
            "test",
            64,
            &mut stats,
        )
        .err()
        .unwrap();
    assert_eq!(fixture.count(), before + 3);
    assert!(!error.to_string().contains("secret"));
    assert!(!error.to_string().contains("token"));
    let before = fixture.count();
    assert!(
        http.get(
            RequestPurpose::Archive,
            url::Url::parse("https://missing.test/").unwrap(),
            "test",
            64,
            &mut stats
        )
        .is_err()
    );
    assert_eq!(fixture.count(), before + 1);
    assert!(fixture.calls.lock().unwrap().last().unwrap().auth.is_none());
}
#[test]
fn index_variants_require_exact_url_and_hash_associations() {
    let archive = ContentHash::digest(b"archive");
    let meta = ContentHash::digest(b"metadata");
    let archive_hex = archive.to_string().replace("sha256:", "");
    let meta_hex = meta.to_string().replace("sha256:", "");
    let artifact = url::Url::parse("https://files.test/a%2Bb.whl?signature=secret").unwrap();
    for attribute in ["data-core-metadata", "data-dist-info-metadata"] {
        let text = format!(
            "<base href='https://files.test/'><a href='a%2Bb.whl?signature=secret#sha256={archive_hex}' {attribute}='sha256={meta_hex}'>wheel</a>"
        );
        let response = Response {
            status: 200,
            location: None,
            content_type: "text/html".into(),
            body: text.into_bytes(),
            url: url::Url::parse("https://index.test/simple/example/").unwrap(),
        };
        let sidecar = index::sidecar(&response, &artifact, archive)
            .unwrap()
            .unwrap()
            .0;
        assert_eq!(
            sidecar.as_str(),
            "https://files.test/a%2Bb.whl.metadata?signature=secret"
        );
        let different = url::Url::parse("https://files.test/a%2Bb.whl?signature=other").unwrap();
        assert!(
            index::sidecar(&response, &different, archive)
                .unwrap()
                .is_none()
        );
        assert!(index::sidecar(&response, &artifact, ContentHash::digest(b"wrong")).is_err());
    }
    for attribute in ["core-metadata", "dist-info-metadata"] {
        let text = format!(
            "{{\"meta\":{{\"api-version\":\"1.0\"}},\"files\":[{{\"url\":\"https://files.test/a%2Bb.whl?signature=secret\",\"hashes\":{{\"sha256\":\"{archive_hex}\"}},\"{attribute}\":{{\"sha256\":\"{meta_hex}\"}}}}]}}"
        );
        let response = Response {
            status: 200,
            location: None,
            content_type: "application/json".into(),
            body: text.into_bytes(),
            url: artifact.clone(),
        };
        assert!(
            index::sidecar(&response, &artifact, archive)
                .unwrap()
                .is_some()
        );
    }
    let text = format!(
        "<script>const example=\"<a href='{artifact}#sha256={archive_hex}' data-core-metadata='sha256={meta_hex}'>\";</script>"
    );
    let response = Response {
        status: 200,
        location: None,
        content_type: "text/html".into(),
        body: text.into_bytes(),
        url: artifact.clone(),
    };
    assert!(
        index::sidecar(&response, &artifact, archive)
            .unwrap()
            .is_none()
    );
}
#[test]
fn fallback_preserves_failures_and_counts_every_request() {
    let temp = tempfile::tempdir().unwrap();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let i = fixture_inspection(&bytes, false);
    let fixture = Fixture::default();
    fixture.add("https://files.test/a.whl", 200, &bytes);
    fixture.add("https://files.test/b.whl", 200, b"corrupt");
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(i.requests());
    assert_eq!(batch.results.len(), 2);
    // First verified archive is reusable for identical locked content at another URL.
    assert_eq!(batch.summary.archive_extractions, 1);
    assert_eq!(batch.summary.cache_hits, 1);
    assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
    let temp = tempfile::tempdir().unwrap();
    let fixture = Fixture::default();
    fixture.add("https://files.test/a.whl", 200, b"corrupt");
    fixture.add("https://files.test/b.whl", 404, b"");
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    let batch = acquirer.acquire(i.requests());
    assert_eq!(batch.results.len(), 2);
    assert_eq!(i.evaluate(batch.results).exit_code(false), 2);
}
#[test]
fn real_client_offline_never_reaches_network() {
    let temp = tempfile::tempdir().unwrap();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let i = fixture_inspection(&bytes, false);
    let acquirer = Acquirer::new(AcquisitionOptions::testing(temp.path(), true)).unwrap();
    let batch = acquirer.acquire(i.requests());
    assert_eq!(batch.summary.network_requests, 0);
    assert_eq!(i.evaluate(batch.results).exit_code(false), 2);
}
proptest::proptest! {
 #![proptest_config(proptest::test_runner::Config::with_cases(256))]
 #[test]
 fn signed_query_association_is_exact(token in "[A-Za-z0-9]{1,32}"){
 let hash=ContentHash::digest(b"archive");let hex=hash.to_string().replace("sha256:","");let url=url::Url::parse(&format!("https://files.test/a.whl?token={token}")).unwrap();let response=Response{status:200,location:None,content_type:"text/html".into(),body:format!("<a href='{url}#sha256={hex}' data-core-metadata='sha256={hex}'>a</a>").into_bytes(),url:url.clone()};
 proptest::prop_assert!(index::sidecar(&response,&url,hash).unwrap().is_some());let wrong=url::Url::parse(&format!("https://files.test/a.whl?token={token}x")).unwrap();proptest::prop_assert!(index::sidecar(&response,&wrong,hash).unwrap().is_none());let secret=format!("token={token}");proptest::prop_assert!(!taloon::model::safe_locator(url.as_str()).contains(&secret));
 }
}

#[test]
#[ignore = "requires loopback sockets"]
fn real_https_transport_rejects_bad_tls_and_times_out() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        time::{Duration, Instant},
    };
    for stall in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut count = 0;
            while count < 3 && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        count += 1;
                        stream
                            .set_read_timeout(Some(Duration::from_millis(200)))
                            .unwrap();
                        let mut hello = [0; 1024];
                        let _ = stream.read(&mut hello);
                        if stall {
                            std::thread::sleep(Duration::from_millis(200));
                        } else {
                            let _ =
                                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("fixture listener: {e}"),
                }
            }
            count
        });
        let transport = http::HttpsTransport::with_timeout(Duration::from_millis(100)).unwrap();
        let client = http::Http::new(transport, http::Authentication::default());
        let mut counters = AcquisitionSummary::default();
        let error = client
            .get(
                RequestPurpose::Archive,
                url::Url::parse(&format!("https://{address}/?signed=secret")).unwrap(),
                "test",
                64,
                &mut counters,
            )
            .err()
            .unwrap();
        assert_eq!(counters.network_requests, 3);
        assert!(!error.to_string().contains("secret"));
        assert_eq!(server.join().unwrap(), 3);
    }
}

#[test]
fn oversized_and_malformed_index_responses_fall_back_to_archives() {
    let temp = tempfile::tempdir().unwrap();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let i = fixture_inspection(&bytes, false);
    let fixture = Fixture::default();
    fixture.add("https://example.test/simple/example/", 200, b"not JSON");
    fixture.add("https://files.test/a.whl", 200, &bytes);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let batch = acquirer.acquire(i.requests());
    assert!(!batch.warnings.is_empty());
    assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
    fixture.add("https://example.test/oversized", 200, b"too much");
    let http = http::Http::new(fixture, http::Authentication::default());
    let mut stats = AcquisitionSummary::default();
    assert!(
        http.get(
            RequestPurpose::Archive,
            url::Url::parse("https://example.test/oversized").unwrap(),
            "test",
            2,
            &mut stats
        )
        .is_err()
    );
    assert_eq!(stats.network_requests, 3);
}

#[test]
fn one_failure_does_not_remove_another_artifacts_success() {
    let temp = tempfile::tempdir().unwrap();
    let bytes =
        crate::testfixtures::wheel(&crate::testfixtures::metadata("License-Expression: MIT"));
    let other =
        crate::testfixtures::wheel(&crate::testfixtures::metadata("License-Expression: ISC"));
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://example.test/simple'}}\nwheels=[{{url='https://files.test/a.whl',hash='{}'}},{{url='https://files.test/b.whl',hash='{}'}}]",
        ContentHash::digest(&bytes),
        ContentHash::digest(&other)
    );
    let i = Inspection::from_documents(
        &lock,
        "[tool.taloon]\nschema-version=1\nallow=['MIT']",
        EvaluationDate::parse("2026-10-31").unwrap(),
    )
    .unwrap();
    let transport = Fixture::default();
    transport.add("https://files.test/a.whl", 200, &bytes);
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), transport)
            .unwrap();
    let batch = acquirer.acquire(i.requests());
    assert_eq!(batch.results.len(), 2);
    let report = i.evaluate(batch.results);
    assert_eq!(report.exit_code(false), 2);
    assert_eq!(
        report.packages()[0]
            .artifacts
            .iter()
            .filter(|a| a.status == taloon::report::Status::Compliant)
            .count(),
        1
    );
}

#[test]
fn unhashed_unavailable_or_corrupt_sidecars_fall_back_without_permission_shortcuts() {
    for advertisement in [
        serde_json::Value::Bool(true),
        serde_json::json!({"sha256":ContentHash::digest(b"missing").to_string().replace("sha256:","")}),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let meta = crate::testfixtures::metadata("License-Expression: MIT");
        let bytes = crate::testfixtures::wheel(&meta);
        let hash = ContentHash::digest(&bytes);
        let i = fixture_inspection(&bytes, false);
        let fixture = Fixture::default();
        let doc = serde_json::json!({"meta":{"api-version":"1.0"},"files":[{"url":"https://files.test/a.whl","hashes":{"sha256":hash.to_string().replace("sha256:","")},"core-metadata":advertisement}]});
        fixture.add(
            "https://example.test/simple/example/",
            200,
            &serde_json::to_vec(&doc).unwrap(),
        );
        fixture.add("https://files.test/a.whl.metadata", 200, b"corrupt");
        fixture.add("https://files.test/a.whl", 200, &bytes);
        let acquirer =
            Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture)
                .unwrap();
        let batch = acquirer.acquire(i.requests());
        assert_eq!(batch.summary.archive_extractions, 1);
        assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
    }
}

#[test]
fn two_distinct_sidecar_associations_survive_offline_warmup() {
    let temp = tempfile::tempdir().unwrap();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let i = fixture_inspection(&bytes, false);
    let fixture = Fixture::default();
    let hash = ContentHash::digest(&bytes)
        .to_string()
        .replace("sha256:", "");
    let mh = ContentHash::digest(&meta)
        .to_string()
        .replace("sha256:", "");
    let files:Vec<_>=["a","b"].iter().map(|name|serde_json::json!({"url":format!("https://files.test/{name}.whl"),"hashes":{"sha256":hash},"core-metadata":{"sha256":mh}})).collect();
    let doc = serde_json::json!({"meta":{"api-version":"1.0"},"files":files});
    fixture.add(
        "https://example.test/simple/example/",
        200,
        &serde_json::to_vec(&doc).unwrap(),
    );
    for name in ["a", "b"] {
        fixture.add(
            &format!("https://files.test/{name}.whl.metadata"),
            200,
            &meta,
        );
    }
    let online = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let batch = online.acquire(i.requests());
    assert_eq!(batch.summary.archive_extractions, 0);
    assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
    let before = fixture.count();
    let offline = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), true),
        fixture.clone(),
    )
    .unwrap();
    let batch = offline.acquire(i.requests());
    assert_eq!(batch.summary.network_requests, 0);
    assert_eq!(fixture.count(), before);
    assert_eq!(batch.summary.cache_hits, 2);
    assert_eq!(i.evaluate(batch.results).exit_code(false), 0);
}

#[test]
fn description_acquires_only_selected_remote_artifacts_and_reuses_evidence() {
    use taloon::description::Description;
    let temp = tempfile::tempdir().unwrap();
    let fixture = Fixture::default();
    let meta = crate::testfixtures::metadata("License-Expression: MIT");
    let bytes = crate::testfixtures::wheel(&meta);
    let hash = ContentHash::digest(&bytes);
    let mh = ContentHash::digest(&meta)
        .to_string()
        .replace("sha256:", "");
    let digest = hash.to_string().replace("sha256:", "");
    let lock = format!(
        "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\nsource={{registry='https://index.test/simple'}}\nwheels=[{{url='https://files.test/selected.whl',hash='{hash}'}}]\n[[package]]\nname='unrelated'\nversion='1.0'\nsource={{registry='https://other.test/simple'}}\nwheels=[{{url='https://other.test/unrelated.whl',hash='{hash}'}}]"
    );
    fixture.add("https://index.test/simple/example/",200,format!("{{\"files\":[{{\"url\":\"https://files.test/selected.whl\",\"hashes\":{{\"sha256\":\"{digest}\"}},\"core-metadata\":{{\"sha256\":\"{mh}\"}}}}]}}").as_bytes());
    fixture.add("https://files.test/selected.whl.metadata", 200, &meta);
    let description = Description::from_uv_lock(&lock, "example").unwrap();
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let batch = acquirer.acquire(description.requests());
    assert_eq!(batch.summary.network_requests, 2);
    assert_eq!(description.evaluate(batch.results).exit_code(), 0);
    let urls: Vec<_> = fixture
        .calls
        .lock()
        .unwrap()
        .iter()
        .map(|c| c.url.clone())
        .collect();
    assert_eq!(
        urls,
        [
            "https://index.test/simple/example/",
            "https://files.test/selected.whl.metadata"
        ]
    );
    let offline = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), true),
        fixture.clone(),
    )
    .unwrap();
    let batch = offline.acquire(description.requests());
    assert_eq!(batch.summary.cache_hits, 1);
    assert_eq!(batch.summary.network_requests, 0);
    assert_eq!(description.evaluate(batch.results).exit_code(), 0);
    assert_eq!(fixture.count(), 2);
}

#[rstest::rstest]
#[case::null(serde_json::Value::Null)]
#[case::number(serde_json::json!(17))]
#[case::array(serde_json::json!([]))]
#[case::hash_array(serde_json::json!(["valid metadata hash"]))]
fn malformed_json_index_does_not_reduce_coverage(#[case] advertisement: serde_json::Value) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, inspection, _) = many_artifact_fixture(2);
    mutate_fixture_index(&fixture, |document| {
        let value = if advertisement
            .as_array()
            .is_some_and(|values| !values.is_empty())
        {
            serde_json::json!([document["files"][0]["core-metadata"]["sha256"].clone()])
        } else {
            advertisement
        };
        document["files"][0]["core-metadata"] = value;
    });
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();

    // Act
    let batch = acquirer.acquire(inspection.requests());
    let report = inspection
        .evaluate(batch.results)
        .with_warnings(batch.warnings);

    // Assert
    assert_eq!(report.exit_code(false), 0);
    assert_eq!(report.packages()[0].artifacts.len(), 2);
    assert!(
        report.packages()[0]
            .artifacts
            .iter()
            .all(|a| a.evidence.as_ref().unwrap().strength == EvidenceRequirement::ArchiveVerified)
    );
    assert_eq!(batch.summary.http.index.requests, 1);
    assert_eq!(batch.summary.http.metadata.requests, 0);
    assert_eq!(batch.summary.http.archive.requests, 2);
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .filter(|d| d.error.code == Code::InvalidMetadata)
            .count(),
        2
    );
}

fn bundled_fixture(fields: &str, text: &str) -> (Fixture, ArtifactRequest, Vec<u8>) {
    let metadata = crate::testfixtures::metadata(fields);
    let bytes = crate::testfixtures::wheel_with_files(
        &metadata,
        &[("example-1.0.dist-info/licenses/LICENSE", text.as_bytes())],
    );
    let request = fixture_inspection(&bytes, false).requests()[0].clone();
    let fixture = Fixture::default();
    fixture.add("https://example.test/simple/example/",200,&serde_json::to_vec(&serde_json::json!({"meta":{"api-version":"1.0"},"files":[{"url":"https://files.test/a.whl","hashes":{"sha256":ContentHash::digest(&bytes).to_string().replace("sha256:","")},"core-metadata":{"sha256":ContentHash::digest(&metadata).to_string().replace("sha256:","")}}]})).unwrap());
    fixture.add("https://files.test/a.whl.metadata", 200, &metadata);
    fixture.add("https://files.test/a.whl", 200, &bytes);
    (fixture, request, metadata)
}
fn acquire_bundled(
    acquirer: &Acquirer,
    request: &ArtifactRequest,
) -> (Evidence, AcquisitionSummary) {
    let catalog = index::IndexCatalog::new(std::slice::from_ref(request), &acquirer.http);
    let mut summary = AcquisitionSummary::default();
    let evidence = acquirer
        .acquire_remote(request, &catalog, &mut summary, &mut Vec::new())
        .unwrap();
    (evidence, summary)
}
#[rstest::rstest]
#[case::declared("License-Expression: MIT\nLicense-File: LICENSE", 0, "MIT")]
#[case::legacy_name("License: MIT License\nLicense-File: LICENSE", 0, "MIT")]
#[case::mpl_alias("License: MPL 2.0\nLicense-File: LICENSE", 0, "MPL-2.0")]
#[case::apache_alias(
    "License: Apache License (v2.0)\nLicense-File: LICENSE",
    0,
    "Apache-2.0"
)]
#[case::gpl_later_alias(
    "License: GNU General Public License v2 or later\nLicense-File: LICENSE",
    0,
    "GPL-2.0-or-later"
)]
#[case::legacy_text(&format!("License: {}\nLicense-File: LICENSE", spdx::license_id("MIT").unwrap().text().replace('\n', "\n ")), 0, "MIT")]
#[case::absent("License-File: LICENSE", 1, "MIT")]
fn bundled_fallback_only_fetches_archives_for_absent_declarations(
    #[case] fields: &str,
    #[case] archives: u64,
    #[case] expected: &str,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) = bundled_fixture(fields, spdx::license_id("MIT").unwrap().text());
    let archive_size = fixture.routes.lock().unwrap()["https://files.test/a.whl"]
        .body
        .len() as u64;
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), false), fixture).unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 1);
    assert_eq!(summary.http.metadata.requests, 1);
    assert_eq!(summary.http.archive.requests, archives);
    assert_eq!(
        summary.http.archive.downloaded_bytes,
        archives * archive_size
    );
    assert_eq!(evidence.license().unwrap().text(), expected);
}
#[rstest::rstest]
#[case::recognized(spdx::license_id("MIT").unwrap().text(),Some("MIT"))]
#[case::unrecognized("custom terms", None)]
#[case::apache_terms(&crate::testfixtures::license_text::Variant::ApacheTerms.text(), Some("Apache-2.0"))]
#[case::apache_appendix(&crate::testfixtures::license_text::Variant::ApacheAppendix.text(), Some("Apache-2.0"))]
#[case::mit_names(&crate::testfixtures::license_text::Variant::MitNames.text(), Some("MIT"))]
#[case::mit_contact(&crate::testfixtures::license_text::Variant::MitContact.text(), Some("MIT"))]
#[case::mpl_locator(&crate::testfixtures::license_text::Variant::MplLocator.text(), Some("MPL-2.0"))]
#[case::bsd_plural(&crate::testfixtures::license_text::Variant::BsdPlural.text(), Some("BSD-3-Clause"))]
#[case::mit_bare(&crate::testfixtures::license_text::Variant::MitBare.text(), Some("MIT"))]
#[case::mit_markdown(&crate::testfixtures::license_text::Variant::MitMarkdown.text(), Some("MIT"))]
#[case::mit_yearless(&crate::testfixtures::license_text::Variant::MitYearless.text(), Some("MIT"))]
#[case::mit_dated_owners(&crate::testfixtures::license_text::Variant::MitDatedOwners.text(), Some("MIT"))]
#[case::mit_attribution_contacts(&crate::testfixtures::license_text::Variant::MitAttributionContacts.text(), Some("MIT"))]
fn completed_bundled_scan_uses_no_network_on_next_run(
    #[case] text: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) = bundled_fixture("License-File: LICENSE", text);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let _ = acquire_bundled(&acquirer, &request);
    let before = fixture.count();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(fixture.count(), before);
    assert_eq!(summary.network_requests, 0);
    assert_eq!(summary.cache_hits, 1);
    assert_eq!(summary.downloaded_bytes, 0);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected
    );
}

#[rstest::rstest]
#[case::apache_terms(crate::testfixtures::license_text::Variant::ApacheTerms, "Apache-2.0")]
#[case::apache_appendix(
    crate::testfixtures::license_text::Variant::ApacheAppendix,
    "Apache-2.0"
)]
#[case::mit_names(crate::testfixtures::license_text::Variant::MitNames, "MIT")]
#[case::mit_contact(crate::testfixtures::license_text::Variant::MitContact, "MIT")]
#[case::mpl_locator(crate::testfixtures::license_text::Variant::MplLocator, "MPL-2.0")]
#[case::bsd_plural(crate::testfixtures::license_text::Variant::BsdPlural, "BSD-3-Clause")]
#[case::mit_bare(crate::testfixtures::license_text::Variant::MitBare, "MIT")]
#[case::mit_markdown(crate::testfixtures::license_text::Variant::MitMarkdown, "MIT")]
#[case::mit_yearless(crate::testfixtures::license_text::Variant::MitYearless, "MIT")]
#[case::mit_dated_owners(crate::testfixtures::license_text::Variant::MitDatedOwners, "MIT")]
#[case::mit_attribution_contacts(
    crate::testfixtures::license_text::Variant::MitAttributionContacts,
    "MIT"
)]
fn complete_legacy_text_cache_needs_no_new_acquisition(
    #[case] variant: crate::testfixtures::license_text::Variant,
    #[case] expected: &str,
    #[values(false, true)] offline: bool,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let fields = format!("License: {}", variant.text().replace('\n', "\n "));
    let (fixture, request, metadata) = bundled_fixture(&fields, "unused file");
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), offline),
        fixture.clone(),
    )
    .unwrap();
    let hash = request.hash().unwrap();
    let evidence = Evidence::from_sidecar(
        &request,
        taloon::evidence::SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata))
            .unwrap(),
        &metadata,
    )
    .unwrap();
    acquirer.cache.put(&request, &evidence).unwrap();
    // Act
    let (restored, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(restored.license().unwrap().text(), expected);
    assert_eq!(restored.content_hash(), hash);
    assert_eq!(
        restored.summary().metadata_hash,
        ContentHash::digest(&metadata)
    );
    assert!(restored.requirement_satisfied(EvidenceRequirement::IndexAssociated));
    assert!(!restored.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert_eq!(summary.network_requests, 0);
    assert_eq!(summary.downloaded_bytes, 0);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(summary.cache_hits, 1);
    assert_eq!(fixture.count(), 0);
}
#[rstest::rstest]
#[case::online(false, 1, Some("MIT"))]
#[case::offline(true, 0, None)]
fn cached_sidecar_upgrade_does_not_refetch_metadata(
    #[case] offline: bool,
    #[case] archives: u64,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, metadata) = bundled_fixture(
        "License-File: LICENSE",
        spdx::license_id("MIT").unwrap().text(),
    );
    let acquirer =
        Acquirer::with_transport(AcquisitionOptions::testing(temp.path(), offline), fixture)
            .unwrap();
    let hash = request.hash().unwrap();
    let sidecar = Evidence::from_sidecar(
        &request,
        taloon::evidence::SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata))
            .unwrap(),
        &metadata,
    )
    .unwrap();
    acquirer.cache.put(&request, &sidecar).unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 0);
    assert_eq!(summary.http.metadata.requests, 0);
    assert_eq!(summary.http.archive.requests, archives);
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected
    );
}

#[rstest::rstest]
#[case::mit("MIT License", "MIT")]
#[case::mpl11("MPL 1.1", "MPL-1.1")]
#[case::mpl11_title("Mozilla Public License Version 1.1", "MPL-1.1")]
#[case::mpl20("MPL 2.0", "MPL-2.0")]
#[case::mpl20_title("Mozilla Public License, Version 2.0", "MPL-2.0")]
#[case::apache("Apache License (v2.0)", "Apache-2.0")]
#[case::bsd2("BSD 2 Clause", "BSD-2-Clause")]
#[case::bsd3("The BSD 3-Clause License", "BSD-3-Clause")]
#[case::cddl("CDDL 1.1", "CDDL-1.1")]
#[case::epl("EPL 2.0", "EPL-2.0")]
#[case::cc0("CC0 1.0 Universal", "CC0-1.0")]
#[case::gpl_later("GNU General Public License v2 or later", "GPL-2.0-or-later")]
#[case::lgpl_later("LGPL, v2.1 or later", "LGPL-2.1-or-later")]
#[case::zpl_versioned("ZPL 2.1", "ZPL-2.1")]
#[case::bsl_versioned("BSL 1.0", "BSL-1.0")]
#[case::mpl_v_prefix("Mozilla Public License v1.0", "MPL-1.0")]
fn cached_legacy_names_need_no_new_evidence(
    #[case] value: &str,
    #[case] expected: &str,
    #[values(false, true)] offline: bool,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, metadata) = bundled_fixture(
        &format!("License: {value}"),
        spdx::license_id("MIT").unwrap().text(),
    );
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), offline),
        fixture.clone(),
    )
    .unwrap();
    let hash = request.hash().unwrap();
    let evidence = Evidence::from_sidecar(
        &request,
        taloon::evidence::SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata))
            .unwrap(),
        &metadata,
    )
    .unwrap();
    acquirer.cache.put(&request, &evidence).unwrap();
    // Act
    let (restored, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(restored.license().unwrap().text(), expected);
    assert_eq!(restored.content_hash(), hash);
    let summary_evidence = restored.summary();
    assert_eq!(
        summary_evidence.metadata_hash,
        ContentHash::digest(&metadata)
    );
    assert_eq!(summary_evidence.original_license, vec![value.to_owned()]);
    assert_eq!(
        summary_evidence.strength,
        EvidenceRequirement::IndexAssociated
    );
    assert_eq!(
        summary_evidence.license_origin,
        Some(taloon::evidence::LicenseOrigin::LegacyDeclaration)
    );
    assert_eq!(summary.network_requests, 0);
    assert_eq!(summary.downloaded_bytes, 0);
    assert_eq!(summary.cache_hits, 1);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(fixture.count(), 0);
}

#[rstest::rstest]
#[case::unknown("UNKNOWN", "MIT")]
#[case::bsd_short("BSD", "BSD-2-Clause")]
#[case::bsd_name("BSD License", "BSD-3-Clause")]
#[case::apache_short("Apache", "Apache-2.0")]
#[case::apache_name("Apache License", "Apache-2.0")]
#[case::apache_software("Apache Software License", "Apache-2.0")]
#[case::apache_two("Apache 2", "Apache-2.0")]
#[case::trimmed_mixed_case("  bSd LiCeNsE  ", "BSD-3-Clause")]
fn legacy_supplement_downloads_one_verified_archive(#[case] legacy: &str, #[case] expected: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) = bundled_fixture(
        &format!("License: {legacy}\nLicense-File: LICENSE"),
        spdx::license_id(expected).unwrap().text(),
    );
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 1);
    assert_eq!(summary.http.metadata.requests, 1);
    assert_eq!(summary.http.archive.requests, 1);
    assert_eq!(summary.archive_extractions, 1);
    assert_eq!(fixture.count(), 3);
    assert_eq!(evidence.license().unwrap().text(), expected);
    assert!(evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
}

#[rstest::rstest]
#[case::bsd_with_apache("BSD", spdx::license_id("Apache-2.0").unwrap().text())]
#[case::apache_with_mit("Apache", spdx::license_id("MIT").unwrap().text())]
#[case::unknown_custom("UNKNOWN", "Custom redistribution conditions apply.")]
fn legacy_supplement_negative_scan_is_reused_from_cache(#[case] legacy: &str, #[case] text: &str) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) =
        bundled_fixture(&format!("License: {legacy}\nLicense-File: LICENSE"), text);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let (first, first_summary) = acquire_bundled(&acquirer, &request);
    assert_eq!(first_summary.http.archive.requests, 1);
    assert_eq!(first.license().unwrap_err().code, Code::UnknownLicense);
    let before = fixture.count();
    // Act
    let (cached, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(cached.license().unwrap_err().code, Code::UnknownLicense);
    assert!(!cached.needs_license_files());
    assert!(cached.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert_eq!(summary.cache_hits, 1);
    assert_eq!(summary.network_requests, 0);
    assert_eq!(summary.downloaded_bytes, 0);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(fixture.count(), before);
}

#[rstest::rstest]
#[case::unknown_online("UNKNOWN", "MIT", false, 1, Some("MIT"))]
#[case::bsd_online("BSD License", "BSD-3-Clause", false, 1, Some("BSD-3-Clause"))]
#[case::apache_online("Apache 2", "Apache-2.0", false, 1, Some("Apache-2.0"))]
#[case::unknown_offline("UNKNOWN", "MIT", true, 0, None)]
#[case::bsd_offline("BSD", "BSD-2-Clause", true, 0, None)]
#[case::apache_offline("Apache", "Apache-2.0", true, 0, None)]
fn legacy_supplement_cached_sidecar_respects_network_mode(
    #[case] legacy: &str,
    #[case] bundled: &str,
    #[case] offline: bool,
    #[case] archives: u64,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, metadata) = bundled_fixture(
        &format!("License: {legacy}\nLicense-File: LICENSE"),
        spdx::license_id(bundled).unwrap().text(),
    );
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), offline),
        fixture.clone(),
    )
    .unwrap();
    let hash = request.hash().unwrap();
    let sidecar = Evidence::from_sidecar(
        &request,
        taloon::evidence::SidecarAssociation::new(hash, hash, ContentHash::digest(&metadata))
            .unwrap(),
        &metadata,
    )
    .unwrap();
    acquirer.cache.put(&request, &sidecar).unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 0);
    assert_eq!(summary.http.metadata.requests, 0);
    assert_eq!(summary.http.archive.requests, archives);
    assert_eq!(summary.archive_extractions, archives);
    assert_eq!(fixture.count() as u64, archives);
    assert_eq!(evidence.needs_license_files(), offline);
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected,
    );
}

#[rstest::rstest]
#[case::valid_expression("License-Expression: Apache-2.0\nLicense: UNKNOWN", Some("Apache-2.0"))]
#[case::invalid_expression("License-Expression: not-a-license\nLicense: UNKNOWN", None)]
#[case::repeated_expression(
    "License-Expression: MIT\nLicense-Expression: MIT\nLicense: UNKNOWN",
    None
)]
#[case::repeated_legacy("License: UNKNOWN\nLicense: UNKNOWN", None)]
#[case::additional_conditions(
    "License: BSD License; redistribution requires separate permission",
    None
)]
fn legacy_supplement_keeps_ineligible_sidecars_on_metadata_path(
    #[case] fields: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) = bundled_fixture(fields, spdx::license_id("MIT").unwrap().text());
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 1);
    assert_eq!(summary.http.metadata.requests, 1);
    assert_eq!(summary.http.archive.requests, 0);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(fixture.count(), 2);
    assert!(!evidence.needs_license_files());
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected,
    );
}

#[rstest::rstest]
#[case::expression("License-Expression: Apache-2.0\nLicense: UNKNOWN", Some("Apache-2.0"))]
#[case::additional_conditions(
    "License: BSD License; redistribution requires separate permission",
    None
)]
fn legacy_supplement_uses_fresh_archive_declaration(
    #[case] archive_fields: &str,
    #[case] expected: Option<&str>,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) =
        bundled_fixture(archive_fields, spdx::license_id("MIT").unwrap().text());
    let sidecar_metadata = crate::testfixtures::metadata("License: UNKNOWN\nLicense-File: LICENSE");
    let sidecar_hash = ContentHash::digest(&sidecar_metadata)
        .to_string()
        .replace("sha256:", "");
    let mut index_document: serde_json::Value = serde_json::from_slice(
        &fixture.routes.lock().unwrap()["https://example.test/simple/example/"].body,
    )
    .unwrap();
    index_document["files"][0]["core-metadata"]["sha256"] = sidecar_hash.into();
    fixture.add(
        "https://example.test/simple/example/",
        200,
        &serde_json::to_vec(&index_document).unwrap(),
    );
    fixture.add("https://files.test/a.whl.metadata", 200, &sidecar_metadata);
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    // Act
    let (evidence, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(summary.http.index.requests, 1);
    assert_eq!(summary.http.metadata.requests, 1);
    assert_eq!(summary.http.archive.requests, 1);
    assert_eq!(summary.archive_extractions, 1);
    assert_eq!(fixture.count(), 3);
    assert!(evidence.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert!(!evidence.needs_license_files());
    assert_eq!(
        evidence
            .license()
            .ok()
            .as_ref()
            .map(taloon::policy::LicenseExpression::text),
        expected,
    );
}

#[rstest::rstest]
#[case::unknown("UNKNOWN", "MIT")]
#[case::bsd("BSD", "BSD-2-Clause")]
#[case::apache("Apache License", "Apache-2.0")]
fn legacy_supplement_positive_scan_is_reused_from_cache(
    #[case] legacy: &str,
    #[case] expected: &str,
) {
    // Arrange
    let temp = tempfile::tempdir().unwrap();
    let (fixture, request, _) = bundled_fixture(
        &format!("License: {legacy}\nLicense-File: LICENSE"),
        spdx::license_id(expected).unwrap().text(),
    );
    let acquirer = Acquirer::with_transport(
        AcquisitionOptions::testing(temp.path(), false),
        fixture.clone(),
    )
    .unwrap();
    let (_, first_summary) = acquire_bundled(&acquirer, &request);
    assert_eq!(first_summary.http.archive.requests, 1);
    let before = fixture.count();
    // Act
    let (cached, summary) = acquire_bundled(&acquirer, &request);
    // Assert
    assert_eq!(cached.license().unwrap().text(), expected);
    assert!(!cached.needs_license_files());
    assert!(cached.requirement_satisfied(EvidenceRequirement::ArchiveVerified));
    assert_eq!(summary.cache_hits, 1);
    assert_eq!(summary.network_requests, 0);
    assert_eq!(summary.downloaded_bytes, 0);
    assert_eq!(summary.archive_extractions, 0);
    assert_eq!(fixture.count(), before);
}
