#![doc = "The JSON-RPC call itself: one plain HTTP POST per method, with the timeout, the HTTP and \
JSON-RPC error shapes, and the result field. Zebra speaks JSON-RPC over HTTP, not gRPC, so this is \
hyper rather than tonic."]

use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{Method, Request};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::{TokioExecutor, TokioTimer};
use unstirred_core::Refusal;

use super::ZebraSource;
use crate::roots::{unavailable, RootSource};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

impl ZebraSource {
    pub(super) async fn call(
        &self,
        client: &Client<HttpConnector, Full<Bytes>>,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, Refusal> {
        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "unstirred",
            "method": method,
            "params": params,
        })
        .to_string();
        let request = Request::builder()
            .method(Method::POST)
            .uri(&self.rpc)
            .header(CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| {
                unavailable(
                    &self.describe(),
                    format!("the request to {method} could not be built: {e}"),
                )
            })?;
        let response = match tokio::time::timeout(REQUEST_TIMEOUT, client.request(request)).await {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => return Err(unavailable(&self.describe(), format!("{method} failed: {e}"))),
            Err(_) => {
                return Err(unavailable(
                    &self.describe(),
                    format!("{method} timed out after {REQUEST_TIMEOUT:?}"),
                ))
            }
        };
        if !response.status().is_success() {
            return Err(unavailable(
                &self.describe(),
                format!("{method} answered HTTP {}", response.status()),
            ));
        }
        let bytes = match response.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                return Err(unavailable(
                    &self.describe(),
                    format!("the {method} body could not be read: {e}"),
                ))
            }
        };
        let answer: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| unavailable(&self.describe(), format!("the {method} answer is not JSON: {e}")))?;
        if let Some(error) = answer.get("error").filter(|error| !error.is_null()) {
            return Err(unavailable(
                &self.describe(),
                format!("{method} returned an error: {error}"),
            ));
        }
        answer
            .get("result")
            .cloned()
            .ok_or_else(|| unavailable(&self.describe(), format!("the {method} answer has no result field")))
    }

    pub(super) fn jsonrpc_client(&self) -> Client<HttpConnector, Full<Bytes>> {
        Client::builder(TokioExecutor::new())
            .timer(TokioTimer::new())
            .build_http()
    }
}
