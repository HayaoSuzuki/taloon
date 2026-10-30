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
