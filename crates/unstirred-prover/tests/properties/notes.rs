//! The wallets A32 runs on. The shapes are chosen on purpose: a wallet of dust against the gate's
//! own threshold is the only way to reach the "more than five notes needed" refusal, and uniformly
//! random values never produce one, so the first version of this generator left that path
//! untested until the coverage assertion in A32 said so.

use rand::rngs::StdRng;
use rand::Rng;
use unstirred_core::REQUIRED_MIN_ZATOSHIS;
use unstirred_prover::{HeldNote, WindowNotes};

/// The most notes a case gives a wallet. The oracle is exhaustive, so this bounds its work at
/// 2^MOST_NOTES subsets, and it stays above the five notes one proof can carry.
pub const MOST_NOTES: usize = 8;

/// One wallet across a window, and the threshold it is judged against.
pub fn random_wallet(rng: &mut StdRng) -> (WindowNotes, u64) {
    match rng.gen_range(0..4u8) {
        0 => dust_against_the_gates_threshold(rng),
        shape => {
            let min_zatoshis = random_threshold(rng);
            let count = rng.gen_range(0..=MOST_NOTES);
            let unspent_at_h0: Vec<HeldNote> = (0..count)
                .map(|index| {
                    let value_zatoshis = value_for(rng, shape, min_zatoshis);
                    note(rng, index, value_zatoshis)
                })
                .collect();
            let held_through_h1 = unspent_at_h0.iter().filter(|_| rng.gen_bool(0.75)).copied().collect();
            (
                WindowNotes {
                    unspent_at_h0,
                    held_through_h1,
                },
                min_zatoshis,
            )
        }
    }
}

/// Six to eight notes of about a sixth of the gate's threshold each, all of them held: the total
/// clears the threshold, but no five of them do.
fn dust_against_the_gates_threshold(rng: &mut StdRng) -> (WindowNotes, u64) {
    let min_zatoshis = gate_threshold();
    let sixth = min_zatoshis / 6;
    let count = rng.gen_range(6..=MOST_NOTES);
    let notes: Vec<HeldNote> = (0..count)
        .map(|index| {
            let value_zatoshis = sixth + rng.gen_range(0..=sixth / 20);
            note(rng, index, value_zatoshis)
        })
        .collect();
    (
        WindowNotes {
            unspent_at_h0: notes.clone(),
            held_through_h1: notes,
        },
        min_zatoshis,
    )
}

fn value_for(rng: &mut StdRng, shape: u8, min_zatoshis: u64) -> u64 {
    match shape {
        1 => rng.gen_range(0..min_zatoshis.saturating_mul(2).max(2)),
        2 => rng.gen_range(0..4_000_000u64),
        _ => rng.gen_range(0..u64::from(u32::MAX)),
    }
}

fn note(rng: &mut StdRng, index: usize, value_zatoshis: u64) -> HeldNote {
    HeldNote {
        value_zatoshis,
        mined_height: rng.gen_range(1..4_505_790u32),
        position: u64::try_from(index).unwrap_or(0) + rng.gen_range(0..1_000u64) * 16,
        commitment: rng.gen(),
    }
}

fn random_threshold(rng: &mut StdRng) -> u64 {
    match rng.gen_range(0..3u8) {
        0 => gate_threshold(),
        1 => rng.gen_range(1..30_000_000u64),
        _ => rng.gen_range(1..200_000_000u64),
    }
}

/// The one threshold a v1 manifest may carry, read from the constant the manifest rule enforces.
fn gate_threshold() -> u64 {
    REQUIRED_MIN_ZATOSHIS.parse().unwrap_or(12_500_000)
}
