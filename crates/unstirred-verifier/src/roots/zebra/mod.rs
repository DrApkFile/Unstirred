#![doc = "One root source: a Zebra node's JSON-RPC, read over plain HTTP (z_gettreestate for the \
h0 tree, getblockchaininfo for the chain name, getblockcount for the tip). The response shape is \
zebra-rpc 7.0.0-rc.0's `GetTreestateResponse` (`zebra-rpc/src/methods/trees.rs`)."]

mod jsonrpc;
mod treestate;

use hyper::Uri;
use unstirred_core::{Network, Refusal};

use self::treestate::proto_tree_state;
use super::{ironwood_root_at, unavailable, RootSource, RootsFromServer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZebraSource {
    pub(super) rpc: String,
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
