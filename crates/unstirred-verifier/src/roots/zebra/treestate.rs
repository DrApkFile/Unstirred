#![doc = "The z_gettreestate answer: Zebra's own response shape, turned into the tree-state type the \
lightwalletd source already reads so both sources share the height, network and Ironwood-tree rules."]

#[cfg(test)]
mod tests;

use serde::Deserialize;
use unstirred_core::Refusal;
use zcash_client_backend::proto::service::TreeState;

use crate::roots::unavailable;

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
pub(super) fn proto_tree_state(
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
