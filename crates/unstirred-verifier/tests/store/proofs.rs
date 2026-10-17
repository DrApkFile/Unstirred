//! Admission of real K=12 packages. **Synthetic:** the wallet and the nullifier set are built in
//! this process. The proofs, the challenge check and the marker transaction are the real ones.

use std::collections::BTreeSet;
use std::error::Error;

use unstirred_core::Code;
use unstirred_verifier::store::{Store, CHALLENGE_TTL_SECONDS};
use unstirred_verifier::{admit, Verdict};

use super::fixture::{corpus, Made};
use super::support::{clock_at, expect_code, file_bytes, require_absent, snapshot, tamper_proof, TestResult, NOW};

/// A05: the same coins, proved twice for one gate, are admitted once. The second proof is
/// refused and that refusal writes nothing, including leaving its own challenge unconsumed.
/// Replaying the admitted package is the same refusal.
#[test]
fn a05_the_same_coins_twice_at_one_gate_are_refused_and_the_refusal_writes_nothing() -> TestResult {
    let made = corpus()?;
    let overlap = shared_markers(&made.primary.markers, &made.same_coins.markers);
    assert!(
        !overlap.is_empty(),
        "the two proofs of the same coins must share a marker, or this test does not exercise A05"
    );
    assert_distinct(&made.primary.markers)?;

    let (_directory, mut store) = super::support::open_gate()?;
    let clock = clock_at(NOW);
    store.record(&made.primary.challenge, &made.primary.manifest.context, &clock)?;
    store.record(&made.same_coins.challenge, &made.same_coins.manifest.context, &clock)?;

    let first = admit_made(&mut store, &made.primary, NOW)?;
    assert_eq!(
        first,
        Verdict::Admitted,
        "the first proof of these coins must be admitted, got {first:?}"
    );
    let after_first = snapshot(&store)?;

    let second = admit_made(&mut store, &made.same_coins, NOW)?;
    expect_code(second, Code::AlreadyUsedInContext)?;
    assert_eq!(snapshot(&store)?, after_first, "the refusal must write nothing");
    assert_unconsumed(&store, &made.same_coins.challenge, &made.same_coins.manifest.context)?;

    let replay = admit_made(&mut store, &made.primary, NOW)?;
    expect_code(replay, Code::AlreadyUsedInContext)?;
    assert_eq!(
        snapshot(&store)?,
        after_first,
        "replaying the admitted package must write nothing"
    );
    Ok(())
}

/// A05c: two admissions of the same package at once. Exactly one commits.
#[test]
fn a05c_two_concurrent_admissions_of_the_same_markers_admit_exactly_one() -> TestResult {
    let made = corpus()?;
    let primary = &made.primary;
    let (_directory, store) = super::support::open_gate()?;
    store.record(&primary.challenge, &primary.manifest.context, &clock_at(NOW))?;
    let path = store.path().to_path_buf();
    drop(store);

    let mut handles = Vec::new();
    for _ in 0..2 {
        let path = path.clone();
        let manifest = primary.manifest.clone();
        let package = primary.package_json.clone();
        let roots = primary.roots;
        let tip = primary.tip;
        handles.push(std::thread::spawn(move || {
            let mut store = Store::open(&path).map_err(|error| error.to_string())?;
            admit(&manifest, &package, &roots, tip, &mut store, &clock_at(NOW)).map_err(|error| error.to_string())
        }));
    }
    let mut verdicts = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(verdict) => verdicts.push(verdict?),
            Err(_) => return Err("a concurrent admission panicked".into()),
        }
    }
    let admitted = verdicts
        .iter()
        .filter(|verdict| matches!(verdict, Verdict::Admitted))
        .count();
    let already_used = verdicts
        .iter()
        .filter(|verdict| matches!(verdict, Verdict::Refused(refusal) if refusal.code == Code::AlreadyUsedInContext))
        .count();
    assert_eq!(admitted, 1, "exactly one of the two races may admit, got {verdicts:?}");
    assert_eq!(
        already_used, 1,
        "the loser must be ALREADY_USED_IN_CONTEXT, got {verdicts:?}"
    );

    let store = Store::open(&path)?;
    assert_eq!(store.markers()?.len(), 5, "the losing admission must not add markers");
    assert_eq!(store.admissions()?.len(), 1, "only the winner records an admission");
    let challenges = store.challenges()?;
    assert_eq!(challenges.len(), 1);
    assert_eq!(
        challenges[0].consumed_at,
        Some(NOW),
        "the winning admission consumes the challenge"
    );
    Ok(())
}

/// A07: an expired challenge is refused before the proof is checked, and the refusal writes
/// nothing. The proof is tampered so a check that ran the circuit would say PROOF_INVALID.
#[test]
fn a07_an_expired_challenge_is_refused_before_the_proof_and_writes_nothing() -> TestResult {
    let made = corpus()?;
    let (_directory, mut store) = super::support::open_gate()?;
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
    let (_directory, mut store) = super::support::open_gate()?;
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
    let (_directory, mut store) = super::support::open_gate()?;
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
    let (_directory, mut store) = super::support::open_gate()?;
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

/// A08: the same coins at two gates are both admitted, and the marker sets share nothing.
#[test]
fn a08_the_same_coins_in_two_contexts_are_both_admitted_with_disjoint_markers() -> TestResult {
    let made = corpus()?;
    let primary = marker_set(&made.primary.markers);
    let other = marker_set(&made.other_room.markers);
    assert!(
        primary.is_disjoint(&other),
        "two contexts of the same coins must not share a marker"
    );
    assert_distinct(&made.other_room.markers)?;

    let (_directory, mut store) = super::support::open_gate()?;
    let clock = clock_at(NOW);
    store.record(&made.primary.challenge, &made.primary.manifest.context, &clock)?;
    store.record(&made.other_room.challenge, &made.other_room.manifest.context, &clock)?;
    let first = admit_made(&mut store, &made.primary, NOW)?;
    let second = admit_made(&mut store, &made.other_room, NOW)?;
    assert_eq!(first, Verdict::Admitted, "holders-room must admit, got {first:?}");
    assert_eq!(second, Verdict::Admitted, "second-room must admit, got {second:?}");

    let markers = store.markers()?;
    assert_eq!(markers.len(), 10, "both gates keep their own five markers");
    let stored_primary: BTreeSet<_> = markers
        .iter()
        .filter(|row| row.context == made.primary.manifest.context)
        .map(|row| row.marker)
        .collect();
    let stored_other: BTreeSet<_> = markers
        .iter()
        .filter(|row| row.context == made.other_room.manifest.context)
        .map(|row| row.marker)
        .collect();
    assert_eq!(stored_primary, primary);
    assert_eq!(stored_other, other);
    assert!(stored_primary.is_disjoint(&stored_other));
    let admissions = store.admissions()?;
    assert_eq!(admissions.len(), 2);
    assert!(admissions
        .iter()
        .all(|row| row.verdict == "ADMITTED" && row.code.is_none()));
    Ok(())
}

/// A02b: the file holds the challenge hash, the markers and the verdict. It does not hold the
/// raw challenge, the package, a note position or a note value.
#[test]
fn a02b_a_database_dump_contains_no_ip_package_body_position_or_value() -> TestResult {
    let made = corpus()?;
    let primary = &made.primary;
    let (_directory, mut store) = super::support::open_gate()?;
    store.record(&primary.challenge, &primary.manifest.context, &clock_at(NOW))?;
    let verdict = admit_made(&mut store, primary, NOW)?;
    assert_eq!(
        verdict,
        Verdict::Admitted,
        "the dump is of an admitted package, got {verdict:?}"
    );
    store.checkpoint()?;

    let schema = store.schema_sql()?;
    let tables = schema.matches("CREATE TABLE").count();
    assert_eq!(
        tables, 3,
        "the file has challenges, markers and admissions, got:\n{schema}"
    );
    let lowered = schema.to_ascii_lowercase();
    for word in ["ip", "package", "proof", "position", "value", "nullifier"] {
        assert!(!lowered.contains(word), "the schema must not mention {word}:\n{schema}");
    }
    assert_no_leaked_integers(&store, primary)?;

    let bytes = file_bytes(store.path())?;
    require_absent(&bytes, &primary.challenge, "the raw challenge")?;
    require_absent(
        &bytes,
        hex::encode(primary.challenge).as_bytes(),
        "the challenge in hex",
    )?;
    require_absent(&bytes, primary.proof_slice.as_bytes(), "a slice of the proof body")?;
    require_absent(&bytes, primary.session_pubkey.as_bytes(), "the session pubkey")?;
    for value in &primary.values {
        let text = value.to_string();
        if text.len() >= 6 {
            require_absent(&bytes, text.as_bytes(), "a note value")?;
        }
    }
    for position in &primary.positions {
        let text = position.to_string();
        if text.len() >= 6 {
            require_absent(&bytes, text.as_bytes(), "a note position")?;
        }
    }
    if primary.proof_slice.len() < 32 {
        return Err("the proof slice was too short to be a meaningful canary".into());
    }
    Ok(())
}

fn admit_made(store: &mut Store, made: &Made, now: i64) -> Result<Verdict, Box<dyn Error>> {
    Ok(admit(
        &made.manifest,
        &made.package_json,
        &made.roots,
        made.tip,
        store,
        &clock_at(now),
    )?)
}

fn shared_markers(left: &[[u8; 32]; 5], right: &[[u8; 32]; 5]) -> BTreeSet<[u8; 32]> {
    let right = marker_set(right);
    marker_set(left).intersection(&right).copied().collect()
}

fn marker_set(markers: &[[u8; 32]; 5]) -> BTreeSet<[u8; 32]> {
    markers.iter().copied().collect()
}

fn assert_distinct(markers: &[[u8; 32]; 5]) -> TestResult {
    if marker_set(markers).len() == markers.len() {
        Ok(())
    } else {
        Err("one proof produced a repeated marker, so a unique conflict would not mean already used".into())
    }
}

fn assert_unconsumed(store: &Store, challenge: &[u8; 32], context: &str) -> TestResult {
    match store.lookup(challenge, context)? {
        Some(row) if row.consumed_at.is_none() => Ok(()),
        Some(row) => Err(format!("the challenge was consumed at {:?}", row.consumed_at).into()),
        None => Err("the challenge is not stored for this context".into()),
    }
}

fn assert_no_leaked_integers(store: &Store, made: &Made) -> TestResult {
    let forbidden: Vec<i64> = made
        .positions
        .iter()
        .copied()
        .chain(made.values.iter().copied())
        .filter_map(|value| i64::try_from(value).ok())
        .collect();
    let mut stored = Vec::new();
    for row in store.challenges()? {
        stored.push(row.issued_at);
        stored.push(row.expires_at);
        if let Some(consumed) = row.consumed_at {
            stored.push(consumed);
        }
    }
    for row in store.markers()? {
        stored.push(row.admitted_at);
    }
    for row in store.admissions()? {
        stored.push(row.at);
    }
    for value in stored {
        if forbidden.contains(&value) {
            return Err(format!("a stored integer equals a note position or value ({value})").into());
        }
    }
    Ok(())
}
