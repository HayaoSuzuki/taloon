//! Replay redirects and retry bursts at the production HTTP wrapper boundary.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Case {
    scenario: Scenario,
    outcome: String,
    calls: Vec<Call>,
    bytes: u64,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct Scenario {
    route: usize,
    burst: usize,
    terminal: usize,
}

#[derive(Deserialize)]
struct Call {
    endpoint: usize,
    authenticated: bool,
    response: i16,
    redirect: i8,
}

fn endpoint(index: usize) -> &'static str {
    [
        "https://example.test/start",
        "https://example.test/same",
        "https://other.test/hop",
        "https://example.test:444/hop",
        "https://example.test/returned",
    ][index]
}

fn location(index: i8) -> Option<String> {
    let value = match index {
        -1 => return None,
        0 => "/start#ignored",
        1 => "/same#ignored",
        2 => "https://decorative:ignored@other.test/hop#ignored",
        3 => "https://example.test:444/hop#ignored",
        4 => "https://example.test/returned#ignored",
        5 => "http://example.test/downgrade",
        _ => unreachable!(),
    };
    Some(value.to_owned())
}

struct Script {
    calls: Arc<Vec<Call>>,
    received: Arc<Mutex<Vec<CallRecord>>>,
}

#[derive(Debug, PartialEq, Eq)]
struct CallRecord {
    url: String,
    credentials: Option<String>,
}

impl Transport for Script {
    fn send(
        &self,
        url: &url::Url,
        auth: Option<&Credentials>,
        _accept: &str,
        _limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error> {
        let mut received = self.received.lock().unwrap();
        let scripted = &self.calls[received.len()];
        received.push(CallRecord {
            url: url.to_string(),
            credentials: auth.map(|value| format!("{}:{}", value.login, value.password)),
        });
        if scripted.response == -1 {
            stats.downloaded_bytes += 3;
            return Err(Error::new(Code::Acquisition, "fixture transport failure"));
        }
        stats.downloaded_bytes += 2;
        Ok(Response {
            status: u16::try_from(scripted.response).unwrap(),
            location: location(scripted.redirect),
            content_type: "application/octet-stream".into(),
            body: b"ok".to_vec(),
            url: url.clone(),
        })
    }
}

fn check_outcome(result: Result<Response, Error>, outcome: &str, last: &Call) {
    if outcome == "success" {
        let response = result.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.url.as_str(), endpoint(last.endpoint));
        assert_eq!(response.body, b"ok");
        return;
    }
    let error = result.err().unwrap();
    let (code, message) = match outcome {
        "downgrade" => (
            Code::UnsupportedSource,
            "only HTTPS acquisition is supported",
        ),
        "redirect_limit" => (Code::Acquisition, "HTTPS redirect limit exceeded"),
        "transport" => (Code::Acquisition, "fixture transport failure"),
        "terminal" => (
            Code::Acquisition,
            "HTTPS acquisition failed (transport, timeout, or response)",
        ),
        value => panic!("unknown outcome in the HTTP trace: {value}"),
    };
    assert_eq!(error.code, code);
    assert_eq!(error.message, message);
}

#[rstest::rstest]
fn quint_replays_http_routing(
    #[values(0, 1, 2, 3, 4, 5)] route: usize,
    #[values(0, 1, 2, 3)] burst: usize,
    #[values(0, 1, 2, 3)] terminal: usize,
) {
    // Arrange
    let index = (route * 4 + burst) * 4 + terminal;
    let row = include_str!("../../../tests/fixtures/oracle/http-routing.jsonl")
        .lines()
        .nth(index)
        .unwrap();
    let case: Case = serde_json::from_str(row).unwrap();
    assert_eq!(
        case.scenario,
        Scenario {
            route,
            burst,
            terminal
        }
    );
    let temp = tempfile::tempdir().unwrap();
    let netrc = temp.path().join("netrc");
    std::fs::write(&netrc, "machine example.test login original password fixture\nmachine other.test login other password fixture").unwrap();
    let calls = Arc::new(case.calls);
    let received = Arc::new(Mutex::new(Vec::new()));
    let http = http::Http::new(
        Script {
            calls: Arc::clone(&calls),
            received: Arc::clone(&received),
        },
        http::Authentication::from_file(Some(&netrc)).unwrap(),
    );
    let start = url::Url::parse("https://decorative:ignored@example.test/start#ignored").unwrap();
    let mut summary = AcquisitionSummary::default();

    // Act
    let result = http.get(
        RequestPurpose::Archive,
        start,
        "application/octet-stream",
        64,
        &mut summary,
    );

    // Assert
    let expected: Vec<_> = calls
        .iter()
        .map(|call| CallRecord {
            url: endpoint(call.endpoint).to_owned(),
            credentials: call.authenticated.then(|| "original:fixture".to_owned()),
        })
        .collect();
    assert_eq!(*received.lock().unwrap(), expected);
    check_outcome(result, &case.outcome, calls.last().unwrap());
    let requests = u64::try_from(calls.len()).unwrap();
    assert_eq!(summary.network_requests, requests);
    assert_eq!(summary.http.archive.requests, requests);
    assert_eq!(summary.downloaded_bytes, case.bytes);
    assert_eq!(summary.http.archive.downloaded_bytes, case.bytes);
}
