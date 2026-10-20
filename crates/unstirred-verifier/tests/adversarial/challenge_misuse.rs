//! Challenges presented when they may not be: expired, never issued, issued for another
//! context, or already consumed; and the proof that fails after the challenge check passed.
//! **Synthetic:** the packages come from `store/fixture.rs`. None of these may write a row.

use unstirred_core::Code;
use unstirred_verifier::admit;
use unstirred_verifier::store::CHALLENGE_TTL_SECONDS;

/// The challenge the consumed-challenge attack replays.
const CONSUMED: [u8; 32] = [0xc1; 32];

use crate::store::assertions::{admit_made, assert_unconsumed};
use crate::store::fixture::corpus;
use crate::store::support::{
    admission, clock_at, distinct_markers, expect_admitted, expect_code, expect_store_code, snapshot, tamper_proof,
    TestResult, NOW,
};

/// A07: an expired challenge is refused before the proof is checked, and the refusal writes
/// nothing. The proof is tampered so a check that ran the circuit would say PROOF_INVALID.
#[test]
fn a07_an_expired_challenge_is_refused_before_the_proof_and_writes_nothing() -> TestResult {
    let made = corpus()?;
    let (_directory, mut store) = crate::store::support::open_gate()?;
    store.record(&made.primary.challenge, &made.primary.manifest.context, &clock_at(NOW))?;
    let before = snapshot(&store)?;
    let tampered = tamper_proof(&made.primary.package_json)?;
    let expired_at = NOW + CHALLENGE_TTL_SECONDS;

    let verdict = admit(
        &made.primary.manifest,
        &tampered,
        &made.primary.roots,
        made.primary.tip,
        &mut store,
        &clock_at(expired_at),
    )?;
    expect_code(verdict, Code::ChallengeUnknownOrExpired)?;
    assert_eq!(snapshot(&store)?, before, "an expired challenge must write nothing");
    assert_unconsumed(&store, &made.primary.challenge, &made.primary.manifest.context)?;
    Ok(())
}

/// A challenge this gate never issued is the same refusal, before the proof, with no write.
#[test]
fn a07_an_unknown_challenge_is_refused_and_writes_nothing() -> TestResult {
    let made = corpus()?;
    let (_directory, mut store) = crate::store::support::open_gate()?;
    let before = snapshot(&store)?;
    let verdict = admit_made(&mut store, &made.primary, NOW)?;
    expect_code(verdict, Code::ChallengeUnknownOrExpired)?;
    assert_eq!(snapshot(&store)?, before, "an unknown challenge must write nothing");
    Ok(())
}

/// A challenge issued for another context is absent at this one: same code, no write.
#[test]
fn a07_a_challenge_issued_for_another_context_is_refused_and_writes_nothing() -> TestResult {
    let made = corpus()?;
    let (_directory, mut store) = crate::store::support::open_gate()?;
    store.record(&made.primary.challenge, "other-room", &clock_at(NOW))?;
    let before = snapshot(&store)?;
    let verdict = admit_made(&mut store, &made.primary, NOW)?;
    expect_code(verdict, Code::ChallengeUnknownOrExpired)?;
    assert_eq!(
        snapshot(&store)?,
        before,
        "a challenge for another context must write nothing here"
    );
    let rows = store.challenges()?;
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].consumed_at.is_none(),
        "the other context's challenge stays unconsumed"
    );
    Ok(())
}

/// A11 at the gate: a tampered proof is PROOF_INVALID and leaves the open challenge unconsumed.
#[test]
fn a11_an_invalid_proof_writes_nothing_and_leaves_the_challenge_open() -> TestResult {
    let made = corpus()?;
    let (_directory, mut store) = crate::store::support::open_gate()?;
    store.record(&made.primary.challenge, &made.primary.manifest.context, &clock_at(NOW))?;
    let before = snapshot(&store)?;
    let tampered = tamper_proof(&made.primary.package_json)?;
    let verdict = admit(
        &made.primary.manifest,
        &tampered,
        &made.primary.roots,
        made.primary.tip,
        &mut store,
        &clock_at(NOW),
    )?;
    expect_code(verdict, Code::ProofInvalid)?;
    assert_eq!(snapshot(&store)?, before, "an invalid proof must write nothing");
    assert_unconsumed(&store, &made.primary.challenge, &made.primary.manifest.context)?;
    Ok(())
}

/// A07, store half: a challenge that was already consumed, presented with markers this gate has
/// not stored, is CHALLENGE_UNKNOWN_OR_EXPIRED. The refusal rolls back, so the file is unchanged.
#[test]
fn a07_a_consumed_challenge_with_new_markers_is_refused_and_writes_nothing() -> TestResult {
    let (_directory, mut store) = crate::store::support::open_gate()?;
    let clock = clock_at(NOW);
    store.record(&CONSUMED, "holders-room", &clock)?;
    expect_admitted(store.commit_admission(&admission("holders-room", CONSUMED, distinct_markers(0xc1), NOW))?)?;
    let before = snapshot(&store)?;

    let outcome = store.commit_admission(&admission("holders-room", CONSUMED, distinct_markers(0xd1), NOW))?;
    expect_store_code(outcome, Code::ChallengeUnknownOrExpired)?;
    assert_eq!(
        snapshot(&store)?,
        before,
        "refusing a consumed challenge must write nothing"
    );
    Ok(())
}
