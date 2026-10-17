//! Synthetic wallet reads: which Ironwood notes the window sees. See `unstirred_prover::testing` for what
//! "synthetic" means here.

use unstirred_core::Code;
use unstirred_prover::testing::{
    manifest, refusal_code, NoteA, SyntheticWallet, TestResult, ACTIVATION, CHAIN_TIP, H0, H1, NOTE_A_ZATOSHIS,
    NOTE_B_ZATOSHIS,
};
use unstirred_prover::{eligible_notes, ReadOnlyWallet};

#[test]
fn a17_lists_the_ironwood_notes_held_across_the_window_with_value_and_mined_height() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let found = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?;

    let mut values: Vec<u64> = found
        .notes
        .held_through_h1
        .iter()
        .map(|note| note.value_zatoshis)
        .collect();
    values.sort_unstable();
    assert_eq!(values, vec![NOTE_B_ZATOSHIS, NOTE_A_ZATOSHIS]);
    assert!(
        found
            .notes
            .held_through_h1
            .iter()
            .all(|note| note.mined_height >= ACTIVATION && note.mined_height <= H0),
        "{:?}",
        found.notes.held_through_h1
    );
    assert_eq!(found.selection.notes.len(), 1, "{:?}", found.selection);
    assert_eq!(found.selection.total_zatoshis, u128::from(NOTE_A_ZATOSHIS));
    assert_eq!(found.chain_tip, CHAIN_TIP);
    assert_eq!(found.fully_scanned, H1);
    Ok(())
}

#[test]
fn a03_notes_mined_after_h0_are_refused_with_coins_too_new() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let before_any_note = manifest(ACTIVATION - 1, H1)?;
    let refused = eligible_notes(&wallet, synthetic.account()?, &before_any_note);

    assert_eq!(
        refused.err().and_then(refusal_code),
        Some(Code::CoinsTooNew),
        "an h0 below every note's mined height must be COINS_TOO_NEW"
    );
    Ok(())
}

#[test]
fn a18_a_wallet_scanned_short_of_h1_is_refused_before_any_note_is_read() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let past_the_scan = manifest(H0, H1 + 10)?;
    let refused = eligible_notes(&wallet, synthetic.account()?, &past_the_scan);

    assert_eq!(refused.err().and_then(refusal_code), Some(Code::WalletNotSyncedPastH1));
    Ok(())
}

#[test]
fn a04_a_note_spent_inside_the_window_drops_out_of_the_held_set() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::SpentInsideTheWindow)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;

    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let found = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?;

    assert!(
        found
            .notes
            .unspent_at_h0
            .iter()
            .any(|note| note.value_zatoshis == NOTE_A_ZATOSHIS),
        "the spent note was still unspent at h0: {:?}",
        found.notes.unspent_at_h0
    );
    assert_eq!(
        found
            .notes
            .held_through_h1
            .iter()
            .map(|note| note.value_zatoshis)
            .collect::<Vec<u64>>(),
        vec![NOTE_B_ZATOSHIS],
        "only the note that never moved may count as held"
    );
    assert_eq!(found.selection.total_zatoshis, u128::from(NOTE_B_ZATOSHIS));
    Ok(())
}
