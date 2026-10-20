#![doc = "Reading the Ironwood root out of one tree-state answer, and the refusal a source that \
cannot serve this gate produces. Both sources share this file, so the height, network and \
Ironwood-tree rules are one code path."]

use unstirred_core::{Code, Network, Refusal};
use zcash_client_backend::proto::service::TreeState;

/// Reads the Ironwood root out of a tree state after checking the answer is for the height and
/// the network that were asked for, and that it carries an Ironwood tree at all. A wrong height
/// or network, or a tree state without the Ironwood pool, means this source cannot serve this
/// gate, which is a root-source failure, not a refusal of the holder.
pub(crate) fn ironwood_root_at(response: &TreeState, network: Network, requested: u32) -> Result<[u8; 32], String> {
    let returned = u32::try_from(response.height)
        .map_err(|_| format!("the server reported height {}, which does not fit", response.height))?;
    if returned != requested {
        return Err(format!(
            "the server answered a tree state for height {returned}, not the gate's h0 {requested}"
        ));
    }
    if !response.network.to_ascii_lowercase().starts_with(&network.to_string()) {
        return Err(format!(
            "the server reports network {:?}, not {:?}",
            response.network, network
        ));
    }
    if response.ironwood_tree.is_empty() {
        return Err(
            "the answer carries no Ironwood tree, so this source cannot serve an Ironwood-height gate".to_owned(),
        );
    }
    let tree = response
        .ironwood_tree()
        .map_err(|e| format!("the Ironwood tree field did not parse: {e}"))?;
    Ok(tree.to_frontier().root().to_bytes())
}

/// One source said nothing usable. Unknown is never success: no verdict of any kind is produced
/// from a guessed root, and the caller must report INCONCLUSIVE.
pub(crate) fn unavailable(source: &str, detail: impl Into<String>) -> Refusal {
    Refusal::new(
        Code::RootSourceUnavailable,
        format!("root source {source}: {}", detail.into()),
    )
}
