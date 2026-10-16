use unstirred_core::{Code, Refusal};

/// One proof covers at most five notes, because the delegation circuit takes five inputs.
pub const MAX_NOTES_PER_PROOF: usize = 5;

/// One Ironwood note as the prover's rules need to see it. `commitment` is the note commitment
/// as a tree leaf digest, which is what a witness path has to hash from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldNote {
    pub value_zatoshis: u64,
    pub mined_height: u32,
    pub position: u64,
    pub commitment: [u8; 32],
}

/// What one account's notes look like across one window: read once at h0, then again at h1.
/// `held_through_h1` is the subset of `unspent_at_h0` that the wallet still had unspent at h1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowNotes {
    pub unspent_at_h0: Vec<HeldNote>,
    pub held_through_h1: Vec<HeldNote>,
}

/// The notes a proof would spend as inputs, fewest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub notes: Vec<HeldNote>,
    pub total_zatoshis: u128,
}

/// Picks the fewest notes that reach `min_zatoshis`, or refuses with the code that says why no
/// proof can be made.
pub fn select_held_notes(notes: &WindowNotes, min_zatoshis: u64) -> Result<Selection, Refusal> {
    if notes.unspent_at_h0.is_empty() {
        return Err(Refusal::new(
            Code::CoinsTooNew,
            "the account held no unspent Ironwood note at h0",
        ));
    }
    let held = total_zatoshis(&notes.held_through_h1);
    if held < u128::from(min_zatoshis) {
        return Err(Refusal::new(
            Code::InsufficientHeldBalance,
            format!(
                "{held} zatoshis stayed unmoved from h0 through h1 across {} notes, {min_zatoshis} needed",
                notes.held_through_h1.len()
            ),
        ));
    }
    let notes = fewest_notes_reaching(&notes.held_through_h1, min_zatoshis);
    if notes.len() > MAX_NOTES_PER_PROOF {
        return Err(Refusal::new(
            Code::TooManyNotesNeeded,
            format!(
                "{} notes are needed to reach {min_zatoshis} zatoshis; one proof covers {MAX_NOTES_PER_PROOF}",
                notes.len()
            ),
        ));
    }
    Ok(Selection {
        total_zatoshis: total_zatoshis(&notes),
        notes,
    })
}

/// Largest notes first, so the count is the smallest that can reach the threshold. Equal values
/// are ordered by tree position, so the same wallet always proposes the same bundle.
fn fewest_notes_reaching(held: &[HeldNote], min_zatoshis: u64) -> Vec<HeldNote> {
    let mut ranked = held.to_vec();
    ranked.sort_unstable_by(|a, b| {
        b.value_zatoshis
            .cmp(&a.value_zatoshis)
            .then(a.position.cmp(&b.position))
    });
    let mut chosen = Vec::new();
    let mut reached = 0u128;
    for note in ranked {
        if reached >= u128::from(min_zatoshis) {
            break;
        }
        reached += u128::from(note.value_zatoshis);
        chosen.push(note);
    }
    chosen
}

fn total_zatoshis(notes: &[HeldNote]) -> u128 {
    notes.iter().map(|note| u128::from(note.value_zatoshis)).sum()
}
