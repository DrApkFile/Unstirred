//! The private nullifier lookup at h1. The wallet and its notes are synthetic (see
//! `harness/mod.rs`); so is the nullifier set, which is built here with the delegation circuit's
//! own reference Indexed Merkle Tree rather than fetched from a real `nf-server`. No real
//! testnet lookup is recorded yet.
//!
//! Every request in this file goes to a listener this test started on 127.0.0.1, or to
//! 127.0.0.1:1 where nothing listens. Running it makes no request to anyone else.

mod harness;
mod server;

use harness::{manifest, refusal_code, NoteA, SyntheticWallet, TestResult, H0, H1, NOTE_A_ZATOSHIS, NOTE_B_ZATOSHIS};
use server::FakeServer;
use unstirred_core::{Code, Network};
use unstirred_prover::{
    eligible_notes, look_up_nullifiers, proves_absence, NoteNullifier, ProverError, ReadOnlyWallet,
};
use voting_circuits::delegation::{ImtProvider, SpacedLeafImtProvider};
use voting_crypto_deps::pasta_curves::group::ff::{Field, PrimeField};
use voting_crypto_deps::pasta_curves::Fp;

/// Nothing answers on this port, so the lookup cannot happen at all.
const NOTHING_LISTENS: &str = "http://127.0.0.1:1";

#[test]
fn a15_an_unreachable_pir_server_is_inconclusive_and_no_proof_is_made() -> TestResult {
    let held = HeldWallet::build(NoteA::StaysPut)?;
    let nullifiers = held.nullifiers_of_selected_notes()?;

    let refused = look_up_nullifiers(NOTHING_LISTENS, Network::Test, H1, None, &nullifiers);

    assert_eq!(
        refused.err().and_then(refusal_code),
        Some(Code::PirUnavailable),
        "a PIR server that cannot be reached is inconclusive and retryable, and no proof is made"
    );
    Ok(())
}

#[test]
fn a15_a_pir_server_that_fails_is_refused_and_no_nullifier_is_ever_sent_in_the_clear() -> TestResult {
    let held = HeldWallet::build(NoteA::StaysPut)?;
    let nullifiers = held.nullifiers_of_selected_notes()?;
    let server = FakeServer::answering(503, "the dataset is being rebuilt")?;

    let refused = look_up_nullifiers(&server.url(), Network::Test, H1, None, &nullifiers);

    assert_eq!(
        refused.err().and_then(refusal_code),
        Some(Code::PirUnavailable),
        "a PIR server that will not answer is inconclusive, and the prover does not ask any other way"
    );
    let sent = server.requests();
    assert!(!sent.is_empty(), "the lookup must have asked the server something");
    for nullifier in &nullifiers {
        let bytes = nullifier.value().to_repr();
        for request in &sent {
            assert!(
                !contains(request, &bytes) && !contains(request, hex::encode(bytes).as_bytes()),
                "a nullifier reached the wire in the clear"
            );
        }
    }
    Ok(())
}

#[test]
fn a15b_a_server_serving_any_root_but_the_expected_h1_root_is_refused_before_any_query() -> TestResult {
    let held = HeldWallet::build(NoteA::StaysPut)?;
    let nullifiers = held.nullifiers_of_selected_notes()?;
    let serving = [0x11; 32];
    let expected = [0x22; 32];
    let server = FakeServer::serving_root(&hex::encode(serving), H1)?;

    let refused = look_up_nullifiers(&server.url(), Network::Test, H1, Some(expected), &nullifiers);

    assert!(
        matches!(
            refused,
            Err(ProverError::PirRootMismatch { ref expected, ref returned })
                if *expected == hex::encode([0x22; 32]) && *returned == hex::encode(serving)
        ),
        "{refused:?}"
    );
    assert_eq!(
        server.requests().len(),
        1,
        "only the public /root request may be made before the root is agreed"
    );
    Ok(())
}

/// The same rule one level down: a proof is only evidence under the root the gate pinned at h1.
#[test]
fn a15b_a_proof_against_another_root_proves_nothing_at_the_expected_root() -> TestResult {
    let held = HeldWallet::build(NoteA::StaysPut)?;
    let nullifier = first(&held.nullifiers_of_selected_notes()?)?;
    let nullifier_set = SpacedLeafImtProvider::new();
    let proof = nullifier_set
        .non_membership_proof(nullifier.value())
        .map_err(|e| format!("the reference tree gave no proof: {e}"))?;

    assert!(proves_absence(&proof, nullifier.value()));

    let mut against_another_root = proof.clone();
    against_another_root.root += Fp::ONE;

    assert!(
        !proves_absence(&against_another_root, nullifier.value()),
        "a path that does not hash to the proof's own root proves nothing"
    );
    Ok(())
}

#[test]
fn a04_a_note_spent_inside_the_window_has_no_non_membership_proof_at_h1() -> TestResult {
    let wallet = HeldWallet::build(NoteA::SpentInsideTheWindow)?;
    let spent = wallet.nullifier_of_note_worth(NOTE_A_ZATOSHIS)?;
    let stayed_put = wallet.nullifier_of_note_worth(NOTE_B_ZATOSHIS)?;
    let nullifier_set_at_h1 = SpacedLeafImtProvider::with_extra_nullifiers(&[spent.value()]);

    let for_the_held_note = nullifier_set_at_h1
        .non_membership_proof(stayed_put.value())
        .map_err(|e| format!("the note that stayed put must have a non-membership proof: {e}"))?;
    assert!(proves_absence(&for_the_held_note, stayed_put.value()));

    assert!(
        nullifier_set_at_h1.non_membership_proof(spent.value()).is_err(),
        "a nullifier that is in the set at h1 has no non-membership proof"
    );
    assert!(
        !proves_absence(&for_the_held_note, spent.value()),
        "another note's proof must not pass for the spent note"
    );
    Ok(())
}

/// A synthetic wallet on disk, opened the way the CLI opens the holder's wallet.
struct HeldWallet {
    _directory: tempfile::TempDir,
    wallet: ReadOnlyWallet<zcash_protocol::local_consensus::LocalNetwork>,
    account: zcash_client_sqlite::AccountUuid,
}

impl HeldWallet {
    fn build(note_a: NoteA) -> Result<Self, Box<dyn std::error::Error>> {
        let synthetic = SyntheticWallet::build(note_a)?;
        let directory = tempfile::tempdir()?;
        synthetic.write_to(directory.path())?;
        let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network())?;
        let account = synthetic.account()?;
        Ok(Self {
            _directory: directory,
            wallet,
            account,
        })
    }

    fn nullifiers_of_selected_notes(&self) -> Result<Vec<NoteNullifier>, Box<dyn std::error::Error>> {
        let selected = eligible_notes(&self.wallet, self.account, &manifest(H0, H1)?)?.selection;
        let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
        Ok(self.wallet.ironwood_nullifiers(self.account, H0, &positions)?)
    }

    /// Notes are named by value here because that is what the harness fixes; the nullifier is
    /// derived from the wallet's own viewing key, as the prover does it.
    fn nullifier_of_note_worth(&self, value_zatoshis: u64) -> Result<NoteNullifier, Box<dyn std::error::Error>> {
        let notes = self
            .wallet
            .window_notes(self.account, manifest(H0, H1)?.window)?
            .unspent_at_h0;
        let note = notes
            .iter()
            .find(|note| note.value_zatoshis == value_zatoshis)
            .ok_or_else(|| format!("the wallet holds no note worth {value_zatoshis} zatoshis at h0"))?;
        first(&self.wallet.ironwood_nullifiers(self.account, H0, &[note.position])?)
    }
}

fn first(nullifiers: &[NoteNullifier]) -> Result<NoteNullifier, Box<dyn std::error::Error>> {
    nullifiers
        .first()
        .copied()
        .ok_or_else(|| "no nullifier was derived".into())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}
