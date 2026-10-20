//! The note selection against a brute-force oracle (A32). **Synthetic and offline:** the notes
//! are made up from each case's own seed; no wallet, database or socket is involved. A failure
//! prints the seed that produced it.

mod properties {
    pub mod notes;
    pub mod oracle;
}

use properties::notes::random_wallet;
use properties::oracle::judge;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use unstirred_prover::select_held_notes;

/// Cases per property. Each one searches every subset of a wallet's notes, so this is the
/// count that keeps the file inside a second or two.
const CASES: u64 = 2_000;

/// A32: the selection picks a smallest set of notes that reaches the threshold. The oracle is an
/// exhaustive search over every subset, so it shares no code with the selection it checks.
#[test]
fn a32_the_selection_picks_a_smallest_set_reaching_the_threshold() {
    let mut reached_each_outcome = [0u32; 4];
    let failures: Vec<(u64, String)> = (0..CASES)
        .filter_map(|seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            let (notes, min_zatoshis) = random_wallet(&mut rng);
            let outcome = judge(&notes, min_zatoshis, &mut reached_each_outcome);
            outcome.err().map(|failure| (seed, failure))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "the selection disagreed with the oracle: {failures:#?}"
    );
    assert!(
        reached_each_outcome.iter().all(|count| *count > 0),
        "every outcome must be exercised; the counts were {reached_each_outcome:?}"
    );
}

/// A32: the same notes in another order are the same selection, so a wallet that lists its notes
/// differently still proposes the same bundle.
#[test]
fn a32_the_selection_does_not_depend_on_the_order_the_notes_arrive_in() {
    let failures: Vec<(u64, String)> = (0..CASES)
        .filter_map(|seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            let (notes, min_zatoshis) = random_wallet(&mut rng);
            let first = select_held_notes(&notes, min_zatoshis).map_err(|refusal| refusal.code);

            let mut shuffled = notes.clone();
            shuffled.unspent_at_h0.shuffle(&mut rng);
            shuffled.held_through_h1.shuffle(&mut rng);
            shuffled.unspent_at_h0.reverse();
            shuffled.held_through_h1.reverse();
            let again = select_held_notes(&shuffled, min_zatoshis).map_err(|refusal| refusal.code);

            (first != again).then(|| {
                (
                    seed,
                    format!("{first:?} became {again:?} when the notes were reordered"),
                )
            })
        })
        .collect();
    assert!(
        failures.is_empty(),
        "the selection depends on input order: {failures:#?}"
    );
}
