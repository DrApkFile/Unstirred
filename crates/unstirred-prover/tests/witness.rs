//! Synthetic witness rebuilds: a note's authentication path at a height the wallet has long since
//! passed. See `harness/mod.rs` for what "synthetic" means here.

mod harness;

use harness::{manifest, NoteA, SyntheticWallet, TestResult, H0, H1, NOTE_A_HEIGHT, NOTE_B_HEIGHT, NOTE_B_ZATOSHIS};
use incrementalmerkletree::frontier::{Frontier, NonEmptyFrontier};
use unstirred_core::Network;
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, witnesses_at, HeldNote, IronwoodTreeState, ProverError, ReadOnlyWallet,
    Witnesses,
};
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use zcash_client_sqlite::error::SqliteClientError;

#[test]
fn a19_witnesses_rebuilt_at_h0_verify_against_the_h0_root() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;
    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let selected = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?.selection;
    let tree_state = synthetic.tree_state_at(H0)?;

    assert!(
        H0 < wallet.fully_scanned_height()?,
        "h0 must be below the wallet's scanned tip for this to be a historical witness"
    );

    let witnesses = witnesses_at(&wallet, &tree_state, &selected.notes)?;

    assert_eq!(witnesses.height, H0);
    assert_eq!(witnesses.root, tree_state.root_bytes());
    assert_eq!(witnesses.notes.len(), selected.notes.len());
    for (witness, note) in witnesses.notes.iter().zip(&selected.notes) {
        assert_eq!(witness.note, *note);
        assert_eq!(u64::from(witness.path.position()), note.position);
        assert_eq!(
            witness.path_root, witnesses.root,
            "the path for tree position {} does not hash to the h0 root",
            note.position
        );
    }
    Ok(())
}

#[test]
fn a19_both_held_notes_witness_against_the_same_h0_root() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;
    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let held = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?
        .notes
        .held_through_h1;
    let tree_state = synthetic.tree_state_at(H0)?;

    let witnesses = witnesses_at(&wallet, &tree_state, &held)?;

    assert_eq!(witnesses.notes.len(), 2);
    assert!(
        witnesses
            .notes
            .iter()
            .all(|witness| witness.path_root == tree_state.root_bytes()),
        "{witnesses:?}"
    );
    Ok(())
}

#[test]
fn a03_a_note_mined_after_h0_gets_no_witness_at_h0() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;
    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let held = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?
        .notes
        .held_through_h1;
    let newer_note = note_worth(&held, NOTE_B_ZATOSHIS)?;
    assert_eq!(newer_note.mined_height, NOTE_B_HEIGHT);

    let before_it_existed = synthetic.tree_state_at(NOTE_A_HEIGHT)?;
    let refused = witnesses_at(&wallet, &before_it_existed, &[newer_note]);

    assert!(
        matches!(
            refused,
            Err(ProverError::WalletUnreadable(ref error))
                if matches!(**error, SqliteClientError::HistoricalWitnessUnavailable { .. })
        ),
        "{refused:?}"
    );
    Ok(())
}

#[test]
fn a20_a_frontier_that_is_not_the_wallets_tree_at_h0_is_rejected() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;
    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let selected = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?.selection;
    let honest = synthetic.tree_state_at(H0)?;

    let forged = tampered_leaf(&honest)?;
    assert_ne!(forged.root_bytes(), honest.root_bytes());

    let refused: Result<Witnesses, ProverError> = witnesses_at(&wallet, &forged, &selected.notes);

    assert!(
        matches!(
            refused,
            Err(ProverError::WalletUnreadable(ref error))
                if matches!(**error, SqliteClientError::HistoricalFrontierInvalid(_))
        ),
        "a forged frontier must be rejected as inconsistent with the wallet's own tree: {refused:?}"
    );
    Ok(())
}

/// Nothing answers on this port, so the request fails. It is a local address: running the test
/// makes no request to anyone else.
#[test]
fn a21_an_unreachable_tree_state_source_is_a_typed_error_not_a_guessed_root() -> TestResult {
    let unreachable = fetch_ironwood_tree_state("http://127.0.0.1:1", Network::Test, H0);

    assert!(
        matches!(
            unreachable,
            Err(ProverError::TreeStateUnavailable { height, .. }) if height == H0
        ),
        "{unreachable:?}"
    );
    Ok(())
}

fn note_worth(notes: &[HeldNote], value_zatoshis: u64) -> Result<HeldNote, Box<dyn std::error::Error>> {
    notes
        .iter()
        .find(|note| note.value_zatoshis == value_zatoshis)
        .copied()
        .ok_or_else(|| format!("the wallet holds no note worth {value_zatoshis} zatoshis").into())
}

/// The same frontier with a different leaf: the shape and height are right, the contents are not.
fn tampered_leaf(honest: &IronwoodTreeState) -> Result<IronwoodTreeState, Box<dyn std::error::Error>> {
    let real = honest.frontier()?;
    let (position, leaf, ommers) = real.into_parts();
    let forged_leaf = MerkleHashOrchard::from_bytes(&[1u8; 32])
        .into_option()
        .ok_or("the test's stand-in leaf is not a canonical field element")?;
    assert_ne!(forged_leaf.to_bytes(), leaf.to_bytes());
    let forged = NonEmptyFrontier::from_parts(position, forged_leaf, ommers)
        .map_err(|e| format!("the test's forged frontier is malformed: {e:?}"))?;
    let tree = Frontier::try_from(forged).map_err(|e| format!("the test's forged frontier is too deep: {e:?}"))?;
    Ok(IronwoodTreeState::from_parts(honest.height(), tree))
}
