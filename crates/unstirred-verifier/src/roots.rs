#![doc = "The verifier's only network access (PRD section 4: roots only): the Ironwood \
note-commitment root at h0 and the chain tip, both from one lightwalletd server. The two-server \
agreement rule and the nf-server `verify-root` check for the h1 nullifier-set root are milestone \
M2; in M1 the operator pins the h1 root. Every failure here is INCONCLUSIVE: a verifier that \
cannot read its own root source cannot decide anything."]

use std::time::Duration;

use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use unstirred_core::{Code, Network, Refusal};
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
use zcash_client_backend::proto::service::{BlockId, ChainSpec, TreeState};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// What one root-source read returned, and where it came from, so a report can say exactly which
/// server every root and the tip came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootsFromServer {
    pub server: String,
    pub nc_root_at_h0: [u8; 32],
    pub h0: u32,
    pub tip: u32,
}

/// Reads the Ironwood note-commitment root at `h0` and the chain tip from one lightwalletd
/// server. Blocks; must not be called from inside an async runtime. Any failure — unreachable,
/// wrong height, wrong network, unreadable answer — is `ROOT_SOURCE_UNAVAILABLE` (inconclusive),
/// never a guessed or partial value.
pub fn fetch_h0_root_and_tip(lightwalletd: &str, network: Network, h0: u32) -> Result<RootsFromServer, Refusal> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| {
            unavailable(
                lightwalletd,
                format!("no runtime could be started for the request: {e}"),
            )
        })?;
    runtime.block_on(read_roots(lightwalletd, network, h0))
}

async fn read_roots(lightwalletd: &str, network: Network, h0: u32) -> Result<RootsFromServer, Refusal> {
    let mut client = CompactTxStreamerClient::new(connect(lightwalletd).await?);
    let tree_state = client
        .get_tree_state(BlockId {
            height: u64::from(h0),
            hash: Vec::new(),
        })
        .await
        .map_err(|status| unavailable(lightwalletd, format!("GetTreeState({h0}) failed: {status}")))?
        .into_inner();
    let nc_root_at_h0 =
        ironwood_root_at(&tree_state, network, h0).map_err(|detail| unavailable(lightwalletd, detail))?;
    let tip = latest_tip(&mut client, lightwalletd).await?;
    Ok(RootsFromServer {
        server: lightwalletd.to_owned(),
        nc_root_at_h0,
        h0,
        tip,
    })
}

async fn connect(lightwalletd: &str) -> Result<Channel, Refusal> {
    let uri: Uri = lightwalletd
        .parse()
        .map_err(|e| unavailable(lightwalletd, format!("the URL does not parse: {e}")))?;
    let over_tls = uri.scheme_str() == Some("https");
    let endpoint = Endpoint::from(uri)
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT);
    let endpoint = if over_tls {
        endpoint
            .tls_config(ClientTlsConfig::new().with_enabled_roots())
            .map_err(|e| unavailable(lightwalletd, format!("TLS could not be configured: {e}")))?
    } else {
        endpoint
    };
    endpoint
        .connect()
        .await
        .map_err(|e| unavailable(lightwalletd, format!("the server could not be reached: {e}")))
}

/// Reads the Ironwood root out of a tree state after checking the answer is for the height and
/// the network that were asked for. A wrong height or network means this server cannot serve
/// this gate, which is a root-source failure, not a refusal of the holder.
fn ironwood_root_at(response: &TreeState, network: Network, requested: u32) -> Result<[u8; 32], String> {
    let returned = u32::try_from(response.height)
        .map_err(|_| format!("the server reported height {}, which does not fit", response.height))?;
    if returned != requested {
        return Err(format!(
            "the server answered GetTreeState({requested}) with height {returned}"
        ));
    }
    if !response.network.to_ascii_lowercase().starts_with(&network.to_string()) {
        return Err(format!(
            "the server reports network {:?}, not {:?}",
            response.network, network
        ));
    }
    let tree = response
        .ironwood_tree()
        .map_err(|e| format!("the Ironwood tree field did not parse: {e}"))?;
    Ok(tree.to_frontier().root().to_bytes())
}

async fn latest_tip(client: &mut CompactTxStreamerClient<Channel>, lightwalletd: &str) -> Result<u32, Refusal> {
    let block = client
        .get_latest_block(ChainSpec {})
        .await
        .map_err(|status| unavailable(lightwalletd, format!("GetLatestBlock failed: {status}")))?
        .into_inner();
    u32::try_from(block.height).map_err(|_| {
        unavailable(
            lightwalletd,
            format!("the server reported tip {}, which does not fit", block.height),
        )
    })
}

/// The verifier's own root source said nothing usable. Unknown is never success: no verdict of
/// any kind is produced from a guessed root, and the caller must report INCONCLUSIVE.
fn unavailable(lightwalletd: &str, detail: impl Into<String>) -> Refusal {
    Refusal::new(
        Code::RootSourceUnavailable,
        format!("root source {lightwalletd}: {}", detail.into()),
    )
}
