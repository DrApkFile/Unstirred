#![doc = "The HTTP transport the PIR client sends its encrypted queries over. `pir-client` keeps \
the transport abstract so consumers choose their own HTTP stack; this is ours, over the same \
hyper and rustls the lightwalletd client already brings in."]

use std::time::Duration;

use bytes::Bytes;
use http::{Method, Request};
use http_body_util::{BodyExt, Full};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use pir_client::{Transport, TransportFuture, TransportResponse};

use crate::error::ProverError;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

type Body = Full<Bytes>;

/// A plain HTTP and HTTPS client, with no cookies, no redirects and no retries: one request per
/// call, so the number of requests the prover makes is the number it is asked to make.
#[derive(Debug)]
pub struct HttpTransport {
    client: Client<HttpsConnector<HttpConnector>, Body>,
}

impl HttpTransport {
    /// Fails for any URL the PIR client could not use, so an unusable URL is a typed error before
    /// a single byte is sent.
    pub fn for_url(url: &str) -> Result<Self, ProverError> {
        let uri: http::Uri = url.parse().map_err(|e| ProverError::PirUrlInvalid {
            url: url.to_owned(),
            detail: format!("{e}"),
        })?;
        match uri.scheme_str() {
            Some("http") | Some("https") => {}
            other => {
                return Err(ProverError::PirUrlInvalid {
                    url: url.to_owned(),
                    detail: format!("the scheme is {other:?}; the PIR server speaks http or https"),
                })
            }
        }
        if uri.host().is_none() {
            return Err(ProverError::PirUrlInvalid {
                url: url.to_owned(),
                detail: "the URL names no host".to_owned(),
            });
        }
        Ok(Self::new())
    }

    fn new() -> Self {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        connector.set_connect_timeout(Some(CONNECT_TIMEOUT));
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .wrap_connector(connector);
        Self {
            client: Client::builder(TokioExecutor::new()).build(https),
        }
    }

    async fn send(&self, method: Method, url: &str, body: Vec<u8>) -> anyhow::Result<TransportResponse> {
        let request = Request::builder()
            .method(method)
            .uri(url)
            .body(Full::new(Bytes::from(body)))?;
        let response = self.client.request(request).await?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| value.to_str().ok().map(|value| (name.to_string(), value.to_owned())))
            .collect();
        let body = response.into_body().collect().await?.to_bytes().to_vec();
        Ok(TransportResponse { status, headers, body })
    }
}

impl Transport for HttpTransport {
    fn get<'a>(&'a self, url: &'a str) -> TransportFuture<'a> {
        Box::pin(async move { self.send(Method::GET, url, Vec::new()).await })
    }

    fn post<'a>(&'a self, url: &'a str, body: Vec<u8>) -> TransportFuture<'a> {
        Box::pin(async move { self.send(Method::POST, url, body).await })
    }
}
