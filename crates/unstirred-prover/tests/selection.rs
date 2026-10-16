use unstirred_core::Code;
use unstirred_prover::{select_held_notes, HeldNote, Selection, WindowNotes, MAX_NOTES_PER_PROOF};

const THRESHOLD: u64 = 12_500_000;

fn note(value_zatoshis: u64, position: u32) -> HeldNote {
    HeldNote {
        value_zatoshis,
        mined_height: 4_505_700 + position,
        position: u64::from(position),
    }
}

fn all_held(notes: &[HeldNote]) -> WindowNotes {
    WindowNotes {
        unspent_at_h0: notes.to_vec(),
        held_through_h1: notes.to_vec(),
    }
}

fn refusal_code(notes: &WindowNotes) -> Option<Code> {
    select_held_notes(notes, THRESHOLD).err().map(|r| r.code)
}

#[test]
fn a17_picks_the_fewest_notes_that_reach_the_threshold() {
    let notes = all_held(&[note(4_000_000, 1), note(10_000_000, 2), note(3_000_000, 3)]);
    assert_eq!(
        select_held_notes(&notes, THRESHOLD),
        Ok(Selection {
            notes: vec![note(10_000_000, 2), note(4_000_000, 1)],
            total_zatoshis: 14_000_000,
        })
    );
}

#[test]
fn a17_one_note_above_the_threshold_is_selected_alone() {
    let notes = all_held(&[note(100_000_000, 7), note(12_500_000, 8)]);
    assert_eq!(
        select_held_notes(&notes, THRESHOLD),
        Ok(Selection {
            notes: vec![note(100_000_000, 7)],
            total_zatoshis: 100_000_000,
        })
    );
}

#[test]
fn a17_equal_values_are_broken_by_tree_position_so_the_bundle_is_stable() {
    let notes = all_held(&[note(THRESHOLD, 9), note(THRESHOLD, 4)]);
    assert_eq!(
        select_held_notes(&notes, THRESHOLD),
        Ok(Selection {
            notes: vec![note(THRESHOLD, 4)],
            total_zatoshis: u128::from(THRESHOLD),
        })
    );
}

#[test]
fn a17_exactly_five_notes_reaching_the_threshold_is_still_one_proof() {
    let five: Vec<HeldNote> = (1..=5).map(|position| note(2_500_000, position)).collect();
    let selection = select_held_notes(&all_held(&five), THRESHOLD);
    assert_eq!(selection.as_ref().map(|s| s.notes.len()), Ok(MAX_NOTES_PER_PROOF));
    assert_eq!(
        selection.map(|s| s.total_zatoshis),
        Ok(u128::from(THRESHOLD)),
        "five notes of 2,500,000 are exactly the threshold"
    );
}

#[test]
fn a03_refuses_coins_too_new_when_no_note_existed_at_h0() {
    let nothing_yet = WindowNotes {
        unspent_at_h0: vec![],
        held_through_h1: vec![],
    };
    assert_eq!(refusal_code(&nothing_yet), Some(Code::CoinsTooNew));
}

#[test]
fn a18_refuses_insufficient_held_balance_when_the_notes_that_stayed_put_fall_short() {
    let small = all_held(&[note(1_000_000, 1), note(2_000_000, 2)]);
    assert_eq!(refusal_code(&small), Some(Code::InsufficientHeldBalance));
}

#[test]
fn a18_a_note_spent_inside_the_window_does_not_count_towards_the_threshold() {
    let big = note(100_000_000, 1);
    let small = note(1_000_000, 2);
    let spent_in_window = WindowNotes {
        unspent_at_h0: vec![big, small],
        held_through_h1: vec![small],
    };
    assert_eq!(
        refusal_code(&spent_in_window),
        Some(Code::InsufficientHeldBalance),
        "the note spent inside the window must not be counted"
    );
}

#[test]
fn a18_refuses_too_many_notes_needed_when_more_than_five_are_required() {
    let seven: Vec<HeldNote> = (1..=7).map(|position| note(2_000_000, position)).collect();
    let notes = all_held(&seven);
    assert_eq!(refusal_code(&notes), Some(Code::TooManyNotesNeeded));
}
