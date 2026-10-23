//! Two proofs of the same coins at the same gate. **Synthetic:** both packages come from
//! `store/fixture.rs`. What a gate may match on is a real note's marker; what it may not match on
//! is a key, and what carries no information at all is the padding.

use unstirred_core::ProofPackage;

use super::assertions::shared_markers;
use super::fixture::corpus;
use super::support::TestResult;

/// A47: `alpha` is drawn fresh for every proof, so the randomized key `rk`, the session key and
/// the blinded value commitment all differ between two proofs of the very same notes (LEAKS L7).
/// What repeats is one marker per real note — which is what single use is built on — while the
/// rest of the five are the circuit's own padding nullifiers and differ per proof.
#[test]
fn a47_two_proofs_of_the_same_coins_repeat_one_marker_per_note_and_no_key() -> TestResult {
    let made = corpus()?;
    let first = ProofPackage::parse(&made.primary.package_json).map_err(|e| format!("{e:?}"))?;
    let again = ProofPackage::parse(&made.same_coins.package_json).map_err(|e| format!("{e:?}"))?;

    let real_notes = made.primary.positions.len();
    assert!(real_notes > 0, "the fixture must prove at least one real note");
    assert_eq!(
        shared_markers(&made.primary.markers, &made.same_coins.markers).len(),
        real_notes,
        "two proofs of the same notes must repeat exactly one marker per real note: that collision \
         is what makes the second attempt ALREADY_USED_IN_CONTEXT, and the other markers are the \
         circuit's padding"
    );
    assert_ne!(
        first.instance.rk_x, again.instance.rk_x,
        "rk is randomized per proof: two proofs of the same coins must not carry the same key"
    );
    assert_ne!(
        first.session_pubkey, again.session_pubkey,
        "every proof carries its own session key"
    );
    assert_ne!(
        first.instance.van_comm, again.instance.van_comm,
        "the value commitment is blinded per proof, so the same balance does not commit the same way twice"
    );
    Ok(())
}
