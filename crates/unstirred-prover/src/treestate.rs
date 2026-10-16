#![doc = "The prover's only network request: GetTreeState for h0, which is a public block height. \
No transaction is ever fetched by txid, and no other RPC is called from this crate."]

use std::time::Duration;

use incrementalmerkletree::frontier::{Frontier, NonEmptyFrontier};
use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use unstirred_core::{Code, Network, Refusal};
use voting_crypto_deps::orchard;
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
use zcash_client_backend::proto::service::{BlockId, TreeState};

use crate::error::ProverError;

/// The depth of the Ironwood note commitment tree, which is Orchard-shaped.
pub const IRONWOOD_TREE_DEPTH: u8 = orchard::NOTE_COMMITMENT_TREE_DEPTH as u8;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub type IronwoodFrontier = Frontier<MerkleHashOrchard, IRONWOOD_TREE_DEPTH>;

/// The Ironwood note commitment tree as of one block height, as one source reported it. This is
/// the prover's working copy; the verifier pins the same root from its own root authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IronwoodTreeState {
    height: u32,
    tree: IronwoodFrontier,
}

impl IronwoodTreeState {
    pub fn from_parts(height: u32, tree: IronwoodFrontier) -> Self {
        Self { height, tree }
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// The note commitment root at this height.
    pub fn root(&self) -> MerkleHashOrchard {
        self.tree.root()
    }

    pub fn root_bytes(&self) -> [u8; 32] {
        self.tree.root().to_bytes()
    }

    /// The right edge of the tree, which witness generation needs. An empty tree has no edge, and
    /// no note can have existed at a height where the pool held nothing.
    pub fn frontier(&self) -> Result<NonEmptyFrontier<MerkleHashOrchard>, ProverError> {
        self.tree
            .value()
            .cloned()
            .ok_or(ProverError::IronwoodTreeEmpty(self.height))
    }
}

/// Reads the Ironwood note commitment tree at `height` from one lightwalletd server. This blocks,
/// and must not be called from inside an async runtime.
pub fn fetch_ironwood_tree_state(
    lightwalletd: &str,
    network: Network,
    height: u32,
) -> Result<IronwoodTreeState, ProverError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| unavailable(height, format!("no runtime could be started for the request: {e}")))?;
    runtime.block_on(get_tree_state(lightwalletd, network, height))
}

async fn get_tree_state(lightwalletd: &str, network: Network, height: u32) -> Result<IronwoodTreeState, ProverError> {
    let mut client = CompactTxStreamerClient::new(connect(lightwalletd, height).await?);
    let request = BlockId {
        height: u64::from(height),
        hash: Vec::new(),
    };
    let response = client
        .get_tree_state(request)
        .await
        .map_err(|status| unavailable(height, format!("GetTreeState failed: {status}")))?;
    ironwood_tree_state(response.get_ref(), network, height)
}

async fn connect(lightwalletd: &str, height: u32) -> Result<Channel, ProverError> {
    let uri: Uri = lightwalletd.parse().map_err(|e| ProverError::LightwalletdUrlInvalid {
        url: lightwalletd.to_owned(),
        detail: format!("{e}"),
    })?;
    let over_tls = uri.scheme_str() == Some("https");
    let endpoint = Endpoint::from(uri)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT);
    let endpoint = if over_tls {
        endpoint
            .tls_config(ClientTlsConfig::new().with_enabled_roots())
            .map_err(|e| ProverError::LightwalletdUrlInvalid {
                url: lightwalletd.to_owned(),
                detail: format!("TLS could not be configured: {e}"),
            })?
    } else {
        endpoint
    };
    endpoint
        .connect()
        .await
        .map_err(|e| unavailable(height, format!("{lightwalletd} could not be reached: {e}")))
}

/// The gate's h0 root source said nothing usable. This is inconclusive, never a guessed root: the
/// holder learns nothing and reveals nothing, and may retry or name another server.
fn unavailable(height: u32, detail: String) -> ProverError {
    Refusal::new(
        Code::TreeStateUnavailable,
        format!("the Ironwood tree state at height {height} could not be fetched: {detail}"),
    )
    .into()
}

/// Reads the Ironwood field of a tree state, after checking that the answer is for the height and
/// the network that were asked for.
fn ironwood_tree_state(
    response: &TreeState,
    network: Network,
    requested: u32,
) -> Result<IronwoodTreeState, ProverError> {
    let returned = u32::try_from(response.height).map_err(|_| ProverError::TreeStateHeightMismatch {
        requested,
        returned: response.height,
    })?;
    if returned != requested {
        return Err(ProverError::TreeStateHeightMismatch {
            requested,
            returned: response.height,
        });
    }
    require_same_network(&response.network, network)?;
    let tree = response.ironwood_tree().map_err(|e| ProverError::TreeStateUnreadable {
        height: requested,
        detail: format!("the Ironwood tree field did not parse: {e}"),
    })?;
    Ok(IronwoodTreeState::from_parts(requested, tree.to_frontier()))
}

/// lightwalletd reports "main" or "test"; longer spellings of the same name are accepted, a
/// different network is not.
fn require_same_network(reported: &str, expected: Network) -> Result<(), ProverError> {
    if reported.to_ascii_lowercase().starts_with(&expected.to_string()) {
        Ok(())
    } else {
        Err(ProverError::TreeStateWrongNetwork {
            expected,
            reported: reported.to_owned(),
        })
    }
}
