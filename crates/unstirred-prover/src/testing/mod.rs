//! Synthetic, not real chain data: a local test network, fake compact blocks and Ironwood notes
//! created by the `zcash_client_sqlite` test harness. These build a wallet the prover and the
//! verifier can be pointed at in tests; they say nothing about Zcash testnet. No real-testnet run
//! is recorded here.
//!
//! This module exists behind the `test-harness` feature so integration tests of sibling crates
//! (the verifier above all) share one synthetic wallet instead of growing a second copy.
#![allow(dead_code)]

mod nullifier_set;
mod wallet;

pub use nullifier_set::SyntheticNullifierSet;
pub use wallet::{
    manifest, NoteA, SyntheticWallet, ACTIVATION, CHAIN_TIP, H0, H1, NOTE_A_HEIGHT, NOTE_A_ZATOSHIS, NOTE_B_HEIGHT,
    NOTE_B_ZATOSHIS, SPEND_HEIGHT,
};

use std::error::Error;

use unstirred_core::Code;

use crate::ProverError;

/// What a test in this workspace returns: anything that went wrong, boxed, since a synthetic
/// wallet can fail in as many ways as the harness it is built from.
pub type TestResult = Result<(), Box<dyn Error>>;

/// The verdict code behind a prover error, or `None` when the error is a local fault with no
/// verdict meaning. A test asserting on a code should never pass because of a broken wallet.
pub fn refusal_code(error: ProverError) -> Option<Code> {
    match error {
        ProverError::Refused(refusal) => Some(refusal.code),
        _ => None,
    }
}
