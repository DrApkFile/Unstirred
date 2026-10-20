//! Synthetic witness rebuilds: a note's authentication path at a height the wallet has long
//! since passed. See `unstirred_prover::testing` for what "synthetic" means here. The forged
//! frontier is in `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod wrong_frontier;
}

use unstirred_core::{Code, Network};
use unstirred_prover::testing::{
    manifest, refusal_code, NoteA, SyntheticWallet, TestResult, H0, H1, NOTE_A_HEIGHT, NOTE_B_HEIGHT, NOTE_B_ZATOSHIS,
};
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, witnesses_at, HeldNote, ProverError, ReadOnlyWallet,
};
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

/// Nothing answers on this port, so the request fails. It is a local address: running the test
/// makes no request to anyone else.
#[test]
fn a21_an_unreachable_tree_state_source_is_inconclusive_not_a_guessed_root() -> TestResult {
    let unreachable = fetch_ironwood_tree_state("http://127.0.0.1:1", Network::Test, H0);

    assert_eq!(
        unreachable.err().and_then(refusal_code),
        Some(Code::TreeStateUnavailable),
        "an h0 root source that cannot be reached is inconclusive and retryable, never a root the prover made up"
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
