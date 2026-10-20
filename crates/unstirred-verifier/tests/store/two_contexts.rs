//! The same coins at two gates. **Synthetic:** both packages come from `store/fixture.rs`.
//! Each context derives its own round id, so the two proofs share no marker.

use std::collections::BTreeSet;

use unstirred_verifier::Verdict;

use super::assertions::{admit_made, assert_distinct, marker_set};
use super::fixture::corpus;
use super::support::{clock_at, TestResult, NOW};

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
