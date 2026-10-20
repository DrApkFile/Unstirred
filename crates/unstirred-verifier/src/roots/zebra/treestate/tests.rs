//! The z_gettreestate parse, against answers in Zebra's own response shape: one derived from a
//! synthetic Ironwood tree, one its real height-0 snapshot. **Synthetic:** no node is contacted.

use std::error::Error;

use incrementalmerkletree::frontier::{CommitmentTree, Frontier};
use unstirred_prover::testing::{NoteA, SyntheticWallet, H0};
use unstirred_prover::IronwoodTreeState;
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use voting_crypto_deps::orchard::NOTE_COMMITMENT_TREE_DEPTH;
use zcash_primitives::merkle_tree::write_commitment_tree;

use super::proto_tree_state;
use crate::roots::ironwood_root_at;
use unstirred_core::{Network, Refusal};
use zcash_client_backend::proto::service::TreeState;

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
