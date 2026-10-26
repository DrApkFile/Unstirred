#![doc = "The other side of the same five routes: what `unstirred knock` uses to fetch a room's \
manifest and a challenge, and to post a package. It lives beside the service so a route cannot be \
renamed on one side only. Calls block and must not be made from inside an async runtime."]

mod answers;

use std::time::Duration;

use bytes::Bytes;
use http::{Method, Request};
use http_body_util::{BodyExt, Full};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use unstirred_core::{parse_manifest, Manifest};

pub use answers::{AdvertisedServices, Answer, GateVerdict, IssuedChallenge};

use crate::router::{room_path, ADMIT, CHALLENGE, LOG, MANIFEST, ROOMS};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Why talking to a gate failed. None of these is a verdict: a gate that answers REFUSED has
/// answered, and that comes back as an [`Answer`].
#[derive(Debug, thiserror::Error)]
pub enum GateClientError {
    /// The gate URL is not one this client can address.
    #[error("the gate URL {url:?} cannot be used: {detail}")]
    Url {
        /// The URL as given.
        url: String,
        /// What is wrong with it.
        detail: String,
    },
    /// No async runtime could be started for the request.
    #[error("the gate could not be started up against: {detail}")]
    Runtime {
        /// What the runtime builder reported.
        detail: String,
    },
    /// The gate could not be reached at all.
    #[error("the gate at {url} could not be reached: {detail}")]
    Unreachable {
        /// The URL that was tried.
        url: String,
        /// What the transport reported.
        detail: String,
    },
    /// The gate answered with a status this route does not use.
    #[error("the gate answered {status} for {path}: {body}")]
    Status {
        /// The status it answered.
        status: u16,
        /// The path that was asked for.
        path: String,
        /// The body, which usually names the problem.
        body: String,
    },
    /// The body is not the shape the route promises. Never guessed at.
    #[error("the gate's answer for {path} is not what that route returns: {detail}")]
    Unreadable {
        /// The path that was asked for.
        path: String,
        /// What the reader could not make of it.
        detail: String,
    },
}

/// One gate, addressed by its base URL. No cookies, no redirects and no retries: one request per
/// call, so a knock makes exactly the requests it is written to make.
#[derive(Debug)]
pub struct GateClient {
    base: String,
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    runtime: tokio::runtime::Runtime,
}

impl GateClient {
    /// A client for the gate at this base URL. The URL is checked here; nothing is dialled
    /// until a request is made.
    pub fn new(url: &str) -> Result<Self, GateClientError> {
        let uri: http::Uri = url.parse().map_err(|e| GateClientError::Url {
            url: url.to_owned(),
            detail: format!("{e}"),
        })?;
        match (uri.scheme_str(), uri.host()) {
            (Some("http") | Some("https"), Some(_)) => {}
            (scheme, _) => {
                return Err(GateClientError::Url {
                    url: url.to_owned(),
                    detail: format!("a gate is addressed by http or https and a host, not {scheme:?}"),
                })
            }
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| GateClientError::Runtime { detail: format!("{e}") })?;
        Ok(Self {
            base: url.trim_end_matches('/').to_owned(),
            client: Client::builder(TokioExecutor::new()).build(https_or_http()),
            runtime,
        })
    }

    /// `GET /rooms`, as answered. The rooms this gate hosts, and any services it advertises.
    pub fn rooms(&self) -> Result<Answer, GateClientError> {
        self.request(Method::GET, ROOMS, Vec::new())
    }

    /// The public service addresses this gate advertises, if it advertises any. One request, and
    /// a gate that advertises nothing is answered as two `None`s rather than as an error (A49).
    pub fn services(&self) -> Result<AdvertisedServices, GateClientError> {
        let answer = self.require_ok(Method::GET, ROOMS, Vec::new())?;
        answers::read_services(&answer.body, ROOMS)
    }

    /// The room's manifest, parsed by `unstirred-core`: a gate that published text core refuses
    /// is a gate this holder cannot prove against, and that is said here rather than at proving
    /// time.
    pub fn manifest(&self, room: &str) -> Result<Manifest, GateClientError> {
        let path = room_path(room, MANIFEST);
        let answer = self.require_ok(Method::GET, &path, Vec::new())?;
        parse_manifest(&answer.body).map_err(|refusal| GateClientError::Unreadable {
            path,
            detail: format!("{refusal}"),
        })
    }

    /// Asks the room for a challenge. `unstirred knock` reaches this only once the wallet has
    /// something to prove, so a holder who does not qualify never appears in a gate's records.
    pub fn challenge(&self, room: &str) -> Result<IssuedChallenge, GateClientError> {
        let path = room_path(room, CHALLENGE);
        let answer = self.require_ok(Method::POST, &path, Vec::new())?;
        answers::read_challenge(&answer.body, &path)
    }

    /// Posts one package. Every answer the gate gives to this route is returned, including a
    /// refusal: a REFUSED verdict is an answer, not a transport failure.
    pub fn admit(&self, room: &str, package_json: &str) -> Result<Answer, GateClientError> {
        self.request(Method::POST, &room_path(room, ADMIT), package_json.as_bytes().to_vec())
    }

    /// `GET /rooms/<room>/log`, as answered: the committed admissions and the markers held.
    pub fn room_log(&self, room: &str) -> Result<Answer, GateClientError> {
        self.request(Method::GET, &room_path(room, LOG), Vec::new())
    }

    /// One request to one path of this gate, with the status and body exactly as answered.
    pub fn request(&self, method: Method, path: &str, body: Vec<u8>) -> Result<Answer, GateClientError> {
        let url = format!("{}{path}", self.base);
        let request = Request::builder()
            .method(method)
            .uri(&url)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| GateClientError::Url {
                url: url.clone(),
                detail: format!("{e}"),
            })?;
        self.runtime.block_on(async {
            let response = self
                .client
                .request(request)
                .await
                .map_err(|e| GateClientError::Unreachable {
                    url: url.clone(),
                    detail: format!("{e}"),
                })?;
            let status = response.status().as_u16();
            let collected = response
                .into_body()
                .collect()
                .await
                .map_err(|e| GateClientError::Unreachable {
                    url: url.clone(),
                    detail: format!("the answer could not be read: {e}"),
                })?;
            Ok(Answer {
                status,
                body: String::from_utf8_lossy(&collected.to_bytes()).into_owned(),
            })
        })
    }

    fn require_ok(&self, method: Method, path: &str, body: Vec<u8>) -> Result<Answer, GateClientError> {
        let answer = self.request(method, path, body)?;
        if answer.status == 200 {
            Ok(answer)
        } else {
            Err(GateClientError::Status {
                status: answer.status,
                path: path.to_owned(),
                body: answer.body,
            })
        }
    }
}

fn https_or_http() -> HttpsConnector<HttpConnector> {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    connector.set_connect_timeout(Some(CONNECT_TIMEOUT));
    hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .wrap_connector(connector)
}
