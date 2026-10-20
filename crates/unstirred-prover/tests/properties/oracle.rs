//! The oracle A32 checks the selection against: the smallest subset that reaches the
//! threshold, found by trying every subset. Brute force on purpose, so it cannot repeat the
//! selection's own mistake.

use unstirred_core::{Code, Refusal};
use unstirred_prover::{select_held_notes, Selection, WindowNotes, MAX_NOTES_PER_PROOF};

pub fn judge(notes: &WindowNotes, min_zatoshis: u64, reached: &mut [u32; 4]) -> Result<(), String> {
    let held: Vec<u64> = notes.held_through_h1.iter().map(|note| note.value_zatoshis).collect();
    let total: u128 = held.iter().map(|value| u128::from(*value)).sum();
    let fewest = fewest_notes_reaching(&held, min_zatoshis);
    let outcome = select_held_notes(notes, min_zatoshis);

    if notes.unspent_at_h0.is_empty() {
        reached[0] += 1;
        return expect_code(&outcome, Code::CoinsTooNew);
    }
    if total < u128::from(min_zatoshis) {
        reached[1] += 1;
        return expect_code(&outcome, Code::InsufficientHeldBalance);
    }
    let fewest = fewest.ok_or_else(|| format!("{total} zatoshis reach {min_zatoshis} but no subset does"))?;
    if fewest > MAX_NOTES_PER_PROOF {
        reached[2] += 1;
        return expect_code(&outcome, Code::TooManyNotesNeeded);
    }
    reached[3] += 1;
    let selection = outcome.map_err(|refusal| format!("expected a selection of {fewest}, got {refusal}"))?;
    expect_smallest(&selection, notes, min_zatoshis, fewest)
}

pub fn expect_smallest(
    selection: &Selection,
    notes: &WindowNotes,
    min_zatoshis: u64,
    fewest: usize,
) -> Result<(), String> {
    if selection.notes.len() != fewest {
        return Err(format!(
            "the selection took {} notes where {fewest} suffice",
            selection.notes.len()
        ));
    }
    if selection.total_zatoshis < u128::from(min_zatoshis) {
        return Err(format!(
            "the selection totals {} zatoshis, short of {min_zatoshis}",
            selection.total_zatoshis
        ));
    }
    let summed: u128 = selection.notes.iter().map(|note| u128::from(note.value_zatoshis)).sum();
    if summed != selection.total_zatoshis {
        return Err(format!(
            "the reported total {} is not the sum {summed}",
            selection.total_zatoshis
        ));
    }
    let foreign = selection
        .notes
        .iter()
        .find(|note| !notes.held_through_h1.contains(note));
    match foreign {
        Some(note) => Err(format!("the selection took a note the wallet did not hold: {note:?}")),
        None => Ok(()),
    }
}

/// The oracle: the size of the smallest subset whose values reach the threshold, found by trying
/// every subset. Deliberately brute force, so it cannot repeat the selection's own mistake.
pub fn fewest_notes_reaching(values: &[u64], min_zatoshis: u64) -> Option<usize> {
    (0..1u32 << values.len())
        .filter(|mask| subset_total(values, *mask) >= u128::from(min_zatoshis))
        .map(|mask| mask.count_ones() as usize)
        .min()
}

pub fn subset_total(values: &[u64], mask: u32) -> u128 {
    values
        .iter()
        .enumerate()
        .filter(|(index, _)| mask & (1 << index) != 0)
        .map(|(_, value)| u128::from(*value))
        .sum()
}

pub fn expect_code(outcome: &Result<Selection, Refusal>, code: Code) -> Result<(), String> {
    match outcome {
        Err(refusal) if refusal.code == code => Ok(()),
        Err(refusal) => Err(format!("expected {code}, got {}", refusal.code)),
        Ok(selection) => Err(format!(
            "expected {code}, got a selection of {} notes",
            selection.notes.len()
        )),
    }
}
