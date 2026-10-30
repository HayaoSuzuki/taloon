//! Bounded HTTPS requests; redirects are followed here with explicit credential scope.
use std::{collections::BTreeMap, io::Read, path::Path, time::Duration};
use taloon::{
    model::{Code, Error},
    report::AcquisitionSummary,
};
use url::Url;
#[derive(Clone)]
pub(super) struct Credentials {
    pub login: String,
    pub password: String,
}
pub(super) struct Response {
    pub status: u16,
    pub location: Option<String>,
    pub content_type: String,
    pub body: Vec<u8>,
    pub url: Url,
}
pub(super) trait Transport: Send + Sync {
    fn send(
        &self,
        url: &Url,
        auth: Option<&Credentials>,
        accept: &str,
        limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error>;
}
pub(super) fn failure() -> Error {
    Error::new(
        Code::Acquisition,
        "HTTPS acquisition failed (transport, timeout, or response)",
    )
}
pub(super) struct HttpsTransport {
    client: reqwest::blocking::Client,
}
impl HttpsTransport {
    pub(super) fn new() -> Result<Self, Error> {
        Self::with_timeout(Duration::from_secs(30))
    }
    pub(super) fn with_timeout(timeout: Duration) -> Result<Self, Error> {
        let client = reqwest::blocking::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .referer(false)
            .tls_sslkeylogfile(false)
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!("taloon/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| failure())?;
        Ok(Self { client })
    }
}
impl Transport for HttpsTransport {
    fn send(
        &self,
        url: &Url,
        auth: Option<&Credentials>,
        accept: &str,
        limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error> {
        let mut request = self
            .client
            .get(url.clone())
            .header(reqwest::header::ACCEPT, accept);
        if let Some(auth) = auth {
            request = request.basic_auth(&auth.login, Some(&auth.password));
        }
        let mut response = request.send().map_err(|_| failure())?;
        let code = response.status().as_u16();
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|s| s.to_str().ok())
            .map(str::to_owned);
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|s| s.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let mut body = Vec::new();
        let mut buffer = [0; 16 * 1024];
        loop {
            let read = response.read(&mut buffer).map_err(|_| failure())?;
            if read == 0 {
                break;
            }
            stats.downloaded_bytes += u64::try_from(read).unwrap_or(u64::MAX);
            if u64::try_from(body.len() + read).unwrap_or(u64::MAX) > limit {
                return Err(Error::new(
                    Code::Acquisition,
                    "HTTPS response exceeds its input limit",
                ));
            }
            body.extend_from_slice(&buffer[..read]);
        }
        Ok(Response {
            status: code,
            location,
            content_type,
            body,
            url: url.clone(),
        })
    }
}

#[derive(Default)]
pub(super) struct Authentication {
    hosts: BTreeMap<String, Credentials>,
}
enum MachineScope {
    Absent,
    Default,
    Host(String),
}
struct Entry {
    scope: MachineScope,
    login: Option<String>,
    password: Option<String>,
}
impl Entry {
    fn save(self, hosts: &mut BTreeMap<String, Credentials>) -> Result<(), Error> {
        if let (MachineScope::Host(host), Some(login), Some(password)) =
            (self.scope, self.login, self.password)
            && hosts
                .insert(host.to_ascii_lowercase(), Credentials { login, password })
                .is_some()
        {
            return Err(invalid_netrc());
        }
        Ok(())
    }
}
fn invalid_netrc() -> Error {
    Error::new(
        Code::Configuration,
        "invalid netrc fields (machine/login/password/account supported)",
    )
}
impl Authentication {
    pub(super) fn from_file(path: Option<&Path>) -> Result<Self, Error> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let file = std::fs::File::open(path)
            .map_err(|_| Error::new(Code::Configuration, "cannot read selected netrc file"))?;
        let mut text = String::new();
        file.take(1024 * 1024 + 1)
            .read_to_string(&mut text)
            .map_err(|_| Error::new(Code::Configuration, "cannot read UTF-8 netrc file"))?;
        if text.len() > 1024 * 1024 {
            return Err(Error::new(Code::Configuration, "netrc file exceeds 1 MiB"));
        }
        Self::parse(&text)
    }
    fn parse(text: &str) -> Result<Self, Error> {
        let tokens = shlex::split(text).ok_or_else(invalid_netrc)?;
        let mut tokens = tokens.into_iter();
        let mut hosts = BTreeMap::new();
        let mut entry = Entry {
            scope: MachineScope::Absent,
            login: None,
            password: None,
        };
        while let Some(token) = tokens.next() {
            match token.as_str() {
                "machine" | "default" => {
                    entry.save(&mut hosts)?;
                    let scope = if token == "machine" {
                        MachineScope::Host(tokens.next().ok_or_else(invalid_netrc)?)
                    } else {
                        MachineScope::Default
                    };
                    entry = Entry {
                        scope,
                        login: None,
                        password: None,
                    };
                }
                "login" | "password" | "account" => {
                    if matches!(entry.scope, MachineScope::Absent) {
                        return Err(invalid_netrc());
                    }
                    let value = tokens.next().ok_or_else(invalid_netrc)?;
                    let replaced = match token.as_str() {
                        "login" => entry.login.replace(value),
                        "password" => entry.password.replace(value),
                        _ => None,
                    };
                    if replaced.is_some() {
                        return Err(invalid_netrc());
                    }
                }
                _ => return Err(invalid_netrc()),
            }
        }
        entry.save(&mut hosts)?;
        Ok(Self { hosts })
    }
    fn for_url(&self, url: &Url) -> Option<&Credentials> {
        url.host_str().and_then(|host| self.hosts.get(host))
    }
}
pub(super) struct Http {
    transport: Box<dyn Transport>,
    auth: Authentication,
}
impl Http {
    pub(super) fn new<T: Transport + 'static>(transport: T, auth: Authentication) -> Self {
        Self {
            transport: Box::new(transport),
            auth,
        }
    }
    pub(super) fn get(
        &self,
        mut url: Url,
        accept: &str,
        limit: u64,
        stats: &mut AcquisitionSummary,
    ) -> Result<Response, Error> {
        // Never let reqwest derive Authorization from URL userinfo. Only explicit host credentials apply.
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_fragment(None);
        let origin = url.origin();
        let mut auth_allowed = true;
        for _ in 0..6 {
            if url.scheme() != "https" {
                return Err(Error::new(
                    Code::UnsupportedSource,
                    "only HTTPS acquisition is supported",
                ));
            }
            let auth = if auth_allowed {
                self.auth.for_url(&url)
            } else {
                None
            };
            let mut attempts = 0;
            let response = loop {
                stats.network_requests += 1;
                let result = self.transport.send(&url, auth, accept, limit, stats);
                match result {
                    Ok(response)
                        if !(response.status == 429 || response.status >= 500) || attempts == 2 =>
                    {
                        break response;
                    }
                    Err(e) if attempts == 2 => return Err(e),
                    _ => {
                        attempts += 1;
                        std::thread::sleep(Duration::from_millis(50 * attempts));
                    }
                }
            };
            if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
                let next = url
                    .join(response.location.as_deref().ok_or_else(failure)?)
                    .map_err(|_| failure())?;
                auth_allowed &= next.origin() == origin;
                url = next;
                let _ = url.set_username("");
                let _ = url.set_password(None);
                url.set_fragment(None);
                continue;
            }
            if response.status != 200 {
                return Err(failure());
            }
            return Ok(response);
        }
        Err(Error::new(
            Code::Acquisition,
            "HTTPS redirect limit exceeded",
        ))
    }
}
