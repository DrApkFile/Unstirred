#![doc = "What a PIR server says about the snapshot it serves, read from `GET /root` before any \
query is sent. These are public facts about a public dataset: asking for them says nothing about \
the holder. `pir-client` checks the pool and the dataset version at connect time but not the \
network or the height, so this does."]

use std::sync::Arc;

use pir_client::{Transport as _, COMPILED_PIR_LAYOUT};
use pir_types::{PirLayout, RootInfo};
use unstirred_core::{Code, Network, Refusal};
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::Fp;

use crate::error::ProverError;
use crate::transport::HttpTransport;

/// The one snapshot a lookup is allowed to be about: this gate's h1, and the nullifier-set root
/// at that height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PirDataset {
    pub height: u32,
    pub root: [u8; 32],
}

/// Reads a 32-byte root from the hex a gate owner or `nf-server verify-root` prints.
pub fn root_from_hex(hex_root: &str) -> Result<[u8; 32], ProverError> {
    let bytes = hex::decode(hex_root.trim()).map_err(|_| ProverError::ExpectedRootMalformed(hex_root.to_owned()))?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| ProverError::ExpectedRootMalformed(hex_root.to_owned()))
}

/// Reads `GET /root` and refuses any snapshot that is not this gate's: wrong pool, wrong dataset
/// version, wrong network, wrong height, or a tree layout this build does not speak.
pub(crate) fn describe(
    transport: &Arc<HttpTransport>,
    base: &str,
    network: Network,
    h1: u32,
) -> Result<PirDataset, ProverError> {
    let url = format!("{base}/root");
    let body = get(transport, &url)?;
    let info: RootInfo = serde_json::from_slice(&body).map_err(|e| ProverError::PirDatasetUnreadable {
        url: url.clone(),
        detail: format!("the /root answer did not parse: {e}"),
    })?;
    if !pir_types::is_current_dataset(&info.nullifier_pool, info.dataset_version) {
        return Err(ProverError::PirDatasetWrongPool {
            pool: info.nullifier_pool,
            version: info.dataset_version,
        });
    }
    if info.zcash_network.as_str() != network.to_string() {
        return Err(ProverError::PirDatasetWrongNetwork {
            expected: network,
            reported: info.zcash_network.to_string(),
        });
    }
    if info.height != Some(u64::from(h1)) {
        return Err(ProverError::PirDatasetWrongHeight {
            expected: h1,
            reported: match info.height {
                Some(height) => height.to_string(),
                None => "an unstated height".to_owned(),
            },
        });
    }
    if info.pir_layout != COMPILED_PIR_LAYOUT {
        return Err(ProverError::PirLayoutMismatch {
            expected: layout(COMPILED_PIR_LAYOUT),
            reported: layout(info.pir_layout),
        });
    }
    Ok(PirDataset {
        height: h1,
        root: canonical_root(&info.circuit_root)?,
    })
}

/// A failed request is unavailability, which is inconclusive and retryable. It is never a reason
/// to ask for a nullifier any other way.
fn get(transport: &Arc<HttpTransport>, url: &str) -> Result<Vec<u8>, ProverError> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| unavailable(format!("no runtime could be started for the request: {e}")))?;
    let response = runtime
        .block_on(transport.get(url))
        .map_err(|e| unavailable(format!("{url} could not be reached: {e}")))?;
    if (200..300).contains(&response.status) {
        Ok(response.body)
    } else {
        Err(unavailable(format!("{url} answered HTTP {}", response.status)))
    }
}

fn canonical_root(hex_root: &str) -> Result<[u8; 32], ProverError> {
    let bytes = root_from_hex(hex_root).map_err(|_| ProverError::PirRootNotCanonical {
        root: hex_root.to_owned(),
    })?;
    Option::<Fp>::from(Fp::from_repr(bytes))
        .map(|_| bytes)
        .ok_or_else(|| ProverError::PirRootNotCanonical {
            root: hex_root.to_owned(),
        })
}

fn layout(layout: PirLayout) -> String {
    format!(
        "depth {} as {} public and {} private layers, polynomial degree {}",
        layout.pir_depth, layout.tier0_layers, layout.tier1_layers, layout.poly_len
    )
}

/// A source the lookup needs could not be reached. Inconclusive, retryable, and never a reason
/// to ask for a nullifier any other way.
pub(crate) fn unavailable(detail: String) -> ProverError {
    Refusal::new(Code::PirUnavailable, detail).into()
}
