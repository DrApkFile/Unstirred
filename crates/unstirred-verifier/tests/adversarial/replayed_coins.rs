//! The same coins offered twice at one gate, and two admissions of one package at once.
//! **Synthetic:** the wallet and the nullifier set come from `store/fixture.rs`; the proofs,
//! the challenge check and the marker transaction are the real ones. Every refusal here has to
//! leave the file exactly as it was.

use unstirred_core::Code;
use unstirred_verifier::store::Store;
use unstirred_verifier::{admit, Verdict};

use crate::store::assertions::{admit_made, assert_distinct, assert_unconsumed, shared_markers};
use crate::store::fixture::corpus;
use crate::store::support::{clock_at, expect_code, snapshot, TestResult, NOW};

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

    let (_directory, mut store) = crate::store::support::open_gate()?;
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
    let (_directory, store) = crate::store::support::open_gate()?;
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
