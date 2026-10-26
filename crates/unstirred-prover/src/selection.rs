#![doc = "Choosing which notes a proof is about. The circuit takes five inputs, so a proof covers \
at most five notes, and the rule is the fewest notes that reach the gate's threshold. A wallet \
that cannot reach it is refused here, before any network request or gate contact."]

use unstirred_core::{Code, Refusal};

/// One proof covers at most five notes, because the delegation circuit takes five inputs.
pub const MAX_NOTES_PER_PROOF: usize = 5;

/// One Ironwood note as the prover's rules need to see it. `commitment` is the note commitment
/// as a tree leaf digest, which is what a witness path has to hash from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeldNote {
    /// The note's value. A private witness of the circuit; it is in no package.
    pub value_zatoshis: u64,
    /// The height the note was mined at, which places it against the window's h0.
    pub mined_height: u32,
    /// The note's position in the Ironwood commitment tree.
    pub position: u64,
    /// The note commitment as a tree leaf digest, which is what a witness path hashes from.
    pub commitment: [u8; 32],
}

/// What one account's notes look like across one window: read once at h0, then again at h1.
/// `held_through_h1` is the subset of `unspent_at_h0` that the wallet still had unspent at h1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowNotes {
    /// What the wallet had unspent at h0.
    pub unspent_at_h0: Vec<HeldNote>,
    /// Those of them it still had unspent at h1. Only these can be part of a holding proof.
    pub held_through_h1: Vec<HeldNote>,
}

/// The notes a proof would spend as inputs, fewest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// The chosen notes, largest first, so the count stays as low as it can be.
    pub notes: Vec<HeldNote>,
    /// Their total. Wider than a note's own value, because five notes can sum past `u64`.
    pub total_zatoshis: u128,
}

/// Picks the fewest notes that reach `min_zatoshis`, or refuses with the code that says why no
/// proof can be made.
///
/// The refusal codes are the whole point of doing this before anything else: they tell a holder
/// which rule they miss, without a gate ever hearing from them.
///
/// ```
/// use unstirred_core::Code;
/// use unstirred_prover::{select_held_notes, HeldNote, WindowNotes, MAX_NOTES_PER_PROOF};
///
/// let note = |value, position| HeldNote {
///     value_zatoshis: value,
///     mined_height: 4_505_000,
///     position,
///     commitment: [0u8; 32],
/// };
///
/// // One note over the threshold is one input, not five.
/// let held = vec![note(10_000_000, 1), note(6_000_000, 2), note(200_000_000, 3)];
/// let selection = select_held_notes(
///     &WindowNotes { unspent_at_h0: held.clone(), held_through_h1: held },
///     12_500_000,
/// )?;
/// assert_eq!(selection.notes.len(), 1);
/// assert_eq!(selection.total_zatoshis, 200_000_000);
///
/// // Coins that moved inside the window count for nothing: only what survived to h1 is
/// // weighed against the threshold.
/// let moved = WindowNotes {
///     unspent_at_h0: vec![note(200_000_000, 1)],
///     held_through_h1: Vec::new(),
/// };
/// assert_eq!(
///     select_held_notes(&moved, 12_500_000).unwrap_err().code,
///     Code::InsufficientHeldBalance,
/// );
///
/// // Nothing at h0 at all is a different thing to say, and says it.
/// let newcomer = WindowNotes { unspent_at_h0: Vec::new(), held_through_h1: Vec::new() };
/// assert_eq!(select_held_notes(&newcomer, 12_500_000).unwrap_err().code, Code::CoinsTooNew);
///
/// // A balance that only adds up across more than five notes cannot be proved in one proof.
/// let dust: Vec<HeldNote> = (0..=MAX_NOTES_PER_PROOF as u64)
///     .map(|i| note(3_000_000, i))
///     .collect();
/// assert_eq!(
///     select_held_notes(&WindowNotes { unspent_at_h0: dust.clone(), held_through_h1: dust }, 18_000_000)
///         .unwrap_err()
///         .code,
///     Code::TooManyNotesNeeded,
/// );
/// # Ok::<(), unstirred_core::Refusal>(())
/// ```
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
