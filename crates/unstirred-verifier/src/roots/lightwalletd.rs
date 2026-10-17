#![doc = "One root source: a lightwalletd server, read over gRPC (GetTreeState for the h0 tree, \
GetLatestBlock for the tip)."]

use std::time::Duration;

use tonic::transport::{Channel, ClientTlsConfig, Endpoint, Uri};
use unstirred_core::{Network, Refusal};
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
use zcash_client_backend::proto::service::{BlockId, ChainSpec};

use super::{ironwood_root_at, unavailable, RootSource, RootsFromServer};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightwalletdSource {
    server: String,
}

impl LightwalletdSource {
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_owned(),
        }
    }
}

impl RootSource for LightwalletdSource {
    fn describe(&self) -> String {
        format!("lwd:{}", self.server)
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
        runtime.block_on(self.read_roots_async(network, h0))
    }
}

impl LightwalletdSource {
    async fn read_roots_async(&self, network: Network, h0: u32) -> Result<RootsFromServer, Refusal> {
        let mut client = CompactTxStreamerClient::new(self.connect().await?);
        let tree_state = client
            .get_tree_state(BlockId {
                height: u64::from(h0),
                hash: Vec::new(),
            })
            .await
            .map_err(|status| unavailable(&self.describe(), format!("GetTreeState({h0}) failed: {status}")))?
            .into_inner();
        let nc_root_at_h0 =
            ironwood_root_at(&tree_state, network, h0).map_err(|detail| unavailable(&self.describe(), detail))?;
        let tip = self.latest_tip(&mut client).await?;
        Ok(RootsFromServer {
            server: self.describe(),
            nc_root_at_h0,
            h0,
            tip,
        })
    }

    async fn connect(&self) -> Result<Channel, Refusal> {
        let uri: Uri = self
            .server
            .parse()
            .map_err(|e| unavailable(&self.describe(), format!("the URL does not parse: {e}")))?;
        let over_tls = uri.scheme_str() == Some("https");
        let endpoint = Endpoint::from(uri)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT);
        let endpoint = if over_tls {
            endpoint
                .tls_config(ClientTlsConfig::new().with_enabled_roots())
                .map_err(|e| unavailable(&self.describe(), format!("TLS could not be configured: {e}")))?
        } else {
            endpoint
        };
        endpoint
            .connect()
            .await
            .map_err(|e| unavailable(&self.describe(), format!("the server could not be reached: {e}")))
    }

    async fn latest_tip(&self, client: &mut CompactTxStreamerClient<Channel>) -> Result<u32, Refusal> {
        let block = client
            .get_latest_block(ChainSpec {})
            .await
            .map_err(|status| unavailable(&self.describe(), format!("GetLatestBlock failed: {status}")))?
            .into_inner();
        u32::try_from(block.height).map_err(|_| {
            unavailable(
                &self.describe(),
                format!("the server reported tip {}, which does not fit", block.height),
            )
        })
    }
}
