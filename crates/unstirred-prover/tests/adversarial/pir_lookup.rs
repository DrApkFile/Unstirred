//! The private nullifier lookup under attack: a server that cannot be reached, a server that
//! will not answer, a server serving another root, a proof against another root, and a note
//! spent inside the window trying to prove itself unspent. **Synthetic:** the wallet and the
//! nullifier set are built here, the set being the delegation circuit's own reference tree.
//! Every request goes to a listener this test started on 127.0.0.1, or to 127.0.0.1:1 where
//! nothing listens. No nullifier may reach the wire in the clear.

use unstirred_core::{Code, Network};
use unstirred_prover::testing::{refusal_code, NoteA, TestResult, H1, NOTE_A_ZATOSHIS, NOTE_B_ZATOSHIS};
use unstirred_prover::{look_up_nullifiers, proves_absence, NoteNonMembership, NullifierLookup, ProverError};
use voting_circuits::delegation::{ImtProvider, SpacedLeafImtProvider};
use voting_crypto_deps::pasta_curves::group::ff::{Field, PrimeField};
use voting_crypto_deps::pasta_curves::Fp;

use crate::pir::held_wallet::{first, HeldWallet};
use crate::server::FakeServer;

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

/// A48: the answer a lookup returns carries the IMT path and the two leaf bounds, because the
/// circuit needs them as private witnesses — and shows neither of them when it is printed. The
/// bounds bracket the holder's own nullifier, so a log line or a panic message that spelled them
/// out would narrow it down to a range (LEAKS L5b). What `unstirred lookup` itself prints is the
/// recorded run, evidence/2026-10-16-lookup-real-testnet.txt: a root, positions and yes or no.
#[test]
fn a48_a_non_membership_answer_prints_neither_its_path_nor_its_bounds() -> TestResult {
    let held = HeldWallet::build(NoteA::StaysPut)?;
    let nullifier = first(&held.nullifiers_of_selected_notes()?)?;
    let nullifier_set = SpacedLeafImtProvider::new();
    let proof = nullifier_set
        .non_membership_proof(nullifier.value())
        .map_err(|e| format!("the reference tree gave no proof: {e}"))?;

    let secrets: Vec<String> = proof
        .nf_bounds
        .iter()
        .chain(proof.path.iter())
        .map(|field| hex::encode(field.to_repr()))
        .collect();
    assert!(
        !secrets.is_empty(),
        "a non-membership proof must carry bounds and a path, or this test is checking nothing"
    );

    let answer = NoteNonMembership {
        position: 7,
        proof,
        proven_unspent: true,
    };
    let lookup = NullifierLookup {
        height: H1,
        root: [0x11; 32],
        notes: vec![answer.clone()],
    };

    for shown in [format!("{answer:?}"), format!("{lookup:?}")] {
        for secret in &secrets {
            assert!(
                !shown.contains(secret.as_str()),
                "a printed lookup answer must not show a bound or a path element: {shown}"
            );
        }
        assert!(
            shown.contains("proven_unspent"),
            "it still has to say whether the note was proven unspent: {shown}"
        );
    }
    Ok(())
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}
