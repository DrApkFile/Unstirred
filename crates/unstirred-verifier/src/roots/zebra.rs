#![doc = "One root source: a Zebra node's JSON-RPC, read over plain HTTP (z_gettreestate for the \
h0 tree, getblockchaininfo for the chain name, getblockcount for the tip). The response shape is \
zebra-rpc 7.0.0-rc.0's `GetTreestateResponse` (`zebra-rpc/src/methods/trees.rs`)."]

use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::CONTENT_TYPE;
use hyper::{Method, Request, Uri};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::{TokioExecutor, TokioTimer};
use serde::Deserialize;
use unstirred_core::{Network, Refusal};
use zcash_client_backend::proto::service::TreeState;

use super::{ironwood_root_at, unavailable, RootSource, RootsFromServer};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZebraSource {
    rpc: String,
}

impl ZebraSource {
    pub fn new(rpc: &str) -> Result<Self, String> {
        let uri = rpc
            .parse::<Uri>()
            .map_err(|e| format!("zebra RPC URL {rpc:?} does not parse: {e}"))?;
        if uri.scheme_str() != Some("http") {
            return Err(format!(
                "zebra root source {rpc:?} must be plain http; this build speaks no TLS to JSON-RPC"
            ));
        }
        Ok(Self { rpc: rpc.to_owned() })
    }

    async fn read(&self, network: Network, h0: u32) -> Result<RootsFromServer, Refusal> {
        let client = self.jsonrpc_client();
        let info = self.call(&client, "getblockchaininfo", &serde_json::json!([])).await?;
        let chain = info
            .get("chain")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| unavailable(&self.describe(), "the getblockchaininfo answer has no chain field"))?;
        if !chain.to_ascii_lowercase().starts_with(&network.to_string()) {
            return Err(unavailable(
                &self.describe(),
                format!("the node reports chain {chain:?}, not {:?}", network),
            ));
        }
        let answer = self
            .call(&client, "z_gettreestate", &serde_json::json!([h0.to_string()]))
            .await?;
        let tree_state = proto_tree_state(&self.describe(), &answer, chain, h0)?;
        let nc_root_at_h0 =
            ironwood_root_at(&tree_state, network, h0).map_err(|detail| unavailable(&self.describe(), detail))?;
        let tip = self.call(&client, "getblockcount", &serde_json::json!([])).await?;
        let tip = u32::try_from(tip.as_u64().ok_or_else(|| {
            unavailable(
                &self.describe(),
                format!("getblockcount returned {tip}, which is not a height"),
            )
        })?)
        .map_err(|_| unavailable(&self.describe(), "getblockcount returned a height that does not fit"))?;
        Ok(RootsFromServer {
            server: self.describe(),
            nc_root_at_h0,
            h0,
            tip,
        })
    }

    async fn call(
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

    fn jsonrpc_client(&self) -> Client<HttpConnector, Full<Bytes>> {
        Client::builder(TokioExecutor::new())
            .timer(TokioTimer::new())
            .build_http()
    }
}

impl RootSource for ZebraSource {
    fn describe(&self) -> String {
        format!("zebra:{}", self.rpc)
    }

    fn read_roots(&self, network: Network, h0: u32) -> Result<RootsFromServer, Refusal> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                unavailable(
                    &self.describe(),
                    format!("no runtime could be started for the request: {e}"),
                )
            })?;
        runtime.block_on(self.read(network, h0))
    }
}

/// zebra-rpc 7.0.0-rc.0, `zebra-rpc/src/methods/trees.rs`: `GetTreestateResponse` carries
/// `hash`, `height`, `time` and one optional treestate per pool, each
/// `{"commitments": {"finalRoot": <hex>, "finalState": <hex>}}` (`Commitments`, serde renames).
/// `ironwood` is present only from NU6.3. Only the fields the verifier uses are named here; the
/// rest are accepted and ignored, so a future Zebra can add fields without breaking this parse.
#[derive(Deserialize)]
struct ZebraTreestate {
    height: u64,
    #[serde(default)]
    sapling: Option<ZebraPoolTrees>,
    #[serde(default)]
    orchard: Option<ZebraPoolTrees>,
    #[serde(default)]
    ironwood: Option<ZebraPoolTrees>,
}

#[derive(Deserialize)]
struct ZebraPoolTrees {
    #[serde(default)]
    commitments: ZebraCommitments,
}

#[derive(Deserialize, Default)]
struct ZebraCommitments {
    #[serde(rename = "finalState", default)]
    final_state: Option<String>,
}

/// Turns one z_gettreestate answer into the same tree-state type the lightwalletd source reads,
/// so both sources share the height, network and Ironwood-tree rules in `ironwood_root_at`. The
/// root itself comes from `finalState`, the serialized commitment tree; `finalRoot` is not read,
/// because both fields come from the same server, so one cannot check the other.
fn proto_tree_state(
    source: &str,
    answer: &serde_json::Value,
    chain: &str,
    requested: u32,
) -> Result<TreeState, Refusal> {
    let parsed: ZebraTreestate = serde_json::from_value(answer.clone()).map_err(|e| {
        unavailable(
            source,
            format!("the z_gettreestate answer does not match Zebra's response shape: {e}"),
        )
    })?;
    let ironwood_state = parsed
        .ironwood
        .and_then(|trees| trees.commitments.final_state)
        .filter(|state| !state.is_empty())
        .ok_or_else(|| {
            unavailable(
                source,
                format!(
                    "the z_gettreestate answer for height {requested} carries no Ironwood tree; \
                     a node without the Ironwood pool at that height cannot serve this gate"
                ),
            )
        })?;
    Ok(TreeState {
        network: chain.to_owned(),
        height: parsed.height,
        hash: String::new(),
        time: 0,
        sapling_tree: parsed
            .sapling
            .and_then(|trees| trees.commitments.final_state)
            .unwrap_or_default(),
        orchard_tree: parsed
            .orchard
            .and_then(|trees| trees.commitments.final_state)
            .unwrap_or_default(),
        ironwood_tree: ironwood_state,
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use incrementalmerkletree::frontier::{CommitmentTree, Frontier};
    use unstirred_prover::testing::{NoteA, SyntheticWallet, H0};
    use unstirred_prover::IronwoodTreeState;
    use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
    use voting_crypto_deps::orchard::NOTE_COMMITMENT_TREE_DEPTH;
    use zcash_primitives::merkle_tree::write_commitment_tree;

    use super::*;

    type TestResult = Result<(), Box<dyn Error>>;

    fn refusal_detail(outcome: Result<TreeState, Refusal>) -> Result<TreeState, String> {
        outcome.map_err(|refusal| refusal.detail)
    }

    fn expect_rejected(outcome: Result<[u8; 32], String>, expected: &str) -> TestResult {
        match outcome {
            Ok(root) => Err(format!("the answer was used and returned a root {root:?}").into()),
            Err(detail) => {
                assert!(
                    detail.contains(expected),
                    "the refusal must name {expected:?}, got: {detail}"
                );
                Ok(())
            }
        }
    }

    /// Zebra's own height-0 snapshot (`z_get_treestate_by_hash@custom_testnet.snap` in
    /// zebra-rpc 7.0.0-rc.0), which has no `ironwood` key: the shape is genuine Zebra output.
    fn zebra_height_zero_snapshot() -> serde_json::Value {
        serde_json::json!({
            "hash": "05a60a92d99d85997cce3b87616c089f6124d7342af37106edc76126334a2c38",
            "height": 0,
            "time": 1477648033,
            "sapling": {"commitments": {}},
            "orchard": {"commitments": {}},
        })
    }

    fn synthetic_ironwood_fixture() -> Result<(IronwoodTreeState, String), Box<dyn Error>> {
        let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
        let state = synthetic.tree_state_at(H0)?;
        let frontier = Frontier::try_from(state.frontier()?)
            .map_err(|e| format!("the Ironwood tree does not fit the depth: {e:?}"))?;
        let tree = CommitmentTree::from_frontier(&frontier);
        let mut bytes = Vec::new();
        write_commitment_tree::<MerkleHashOrchard, _, { NOTE_COMMITMENT_TREE_DEPTH as u8 }>(&tree, &mut bytes)?;
        Ok((state, hex::encode(bytes)))
    }

    /// A10b, parser half: an answer in Zebra's own response shape carries the same Ironwood root
    /// the lightwalletd tree-state type reports for the same tree.
    #[test]
    fn a10b_a_zebra_answer_in_zebras_own_response_shape_parses_to_the_same_root() -> TestResult {
        let (state, final_state) = synthetic_ironwood_fixture()?;
        let answer = serde_json::json!({
            "hash": "05a60a92d99d85997cce3b87616c089f6124d7342af37106edc76126334a2c38",
            "height": H0,
            "time": 1477648033,
            "sapling": {"commitments": {}},
            "orchard": {"commitments": {}},
            "ironwood": {"commitments": {
                "finalRoot": hex::encode(state.root_bytes()),
                "finalState": final_state,
            }},
        });

        let parsed = refusal_detail(proto_tree_state("zebra:fixture", &answer, "test", H0))?;
        let root = ironwood_root_at(&parsed, Network::Test, H0)?;
        assert_eq!(
            root,
            state.root_bytes(),
            "the z_gettreestate answer must carry the same root its tree serializes to"
        );
        Ok(())
    }

    /// A10b: Zebra's genuine height-0 answer has no `ironwood` key, and a source that answers
    /// without the Ironwood pool is refused before any root is read, never a root of zeros.
    #[test]
    fn a10b_zebras_own_height_zero_answer_without_ironwood_is_unusable() -> TestResult {
        let outcome = proto_tree_state("zebra:fixture", &zebra_height_zero_snapshot(), "test", 0);
        let detail = match outcome {
            Ok(_) => return Err("an answer without an Ironwood tree was accepted".into()),
            Err(refusal) => refusal.detail,
        };
        assert!(
            detail.contains("no Ironwood tree"),
            "the refusal must name the missing Ironwood tree, got: {detail}"
        );
        Ok(())
    }

    /// A10b: a tree state that answers for another height is rejected before any root is read.
    #[test]
    fn a10b_a_zebra_answer_for_another_height_is_rejected() -> TestResult {
        let (_state, final_state) = synthetic_ironwood_fixture()?;
        let answer = serde_json::json!({
            "hash": "05a60a92d99d85997cce3b87616c089f6124d7342af37106edc76126334a2c38",
            "height": 0,
            "time": 1477648033,
            "sapling": {"commitments": {}},
            "orchard": {"commitments": {}},
            "ironwood": {"commitments": {"finalState": final_state}},
        });
        let parsed = refusal_detail(proto_tree_state("zebra:fixture", &answer, "test", 4_505_790))?;
        expect_rejected(ironwood_root_at(&parsed, Network::Test, 4_505_790), "height 0")
    }

    /// A10b: a tree state that reports the wrong network is rejected before any root is read.
    #[test]
    fn a10b_a_zebra_answer_for_another_network_is_rejected() -> TestResult {
        let (_state, final_state) = synthetic_ironwood_fixture()?;
        let answer = serde_json::json!({
            "hash": "05a60a92d99d85997cce3b87616c089f6124d7342af37106edc76126334a2c38",
            "height": H0,
            "time": 1477648033,
            "sapling": {"commitments": {}},
            "orchard": {"commitments": {}},
            "ironwood": {"commitments": {"finalState": final_state}},
        });

        let parsed = refusal_detail(proto_tree_state("zebra:fixture", &answer, "main", H0))?;
        expect_rejected(ironwood_root_at(&parsed, Network::Test, H0), "network")
    }
}
