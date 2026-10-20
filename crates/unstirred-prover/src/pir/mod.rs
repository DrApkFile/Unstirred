#![doc = "The private nullifier lookup at h1. Every nullifier the prover asks about is queried \
through Valar's PIR client, which only ever puts it on the wire inside a YPIR ciphertext. There \
is no plaintext path: if the server cannot be reached the lookup refuses with PIR_UNAVAILABLE and \
the prover stops. The same oracle answers the delegation builder's questions about its padding \
slots, so one connection serves a whole proof and no nullifier is asked about twice."]

mod lookup;
mod oracle;

pub use lookup::{NoteNonMembership, NullifierLookup, NullifierSetAtH1};
pub use oracle::PirNullifierOracle;

use unstirred_core::Network;

use crate::error::ProverError;
use crate::nullifier::NoteNullifier;

/// Fetches and checks a non-membership proof for each nullifier at h1.
///
/// The server's advertised root is checked against `expected_root` before a single query is sent,
/// so a server holding the wrong snapshot is refused rather than queried. Every returned proof
/// must then authenticate to that same root. This blocks, and must not be called from inside an
/// async runtime.
pub fn look_up_nullifiers(
    pir_url: &str,
    network: Network,
    h1: u32,
    expected_root: Option<[u8; 32]>,
    nullifiers: &[NoteNullifier],
) -> Result<NullifierLookup, ProverError> {
    if nullifiers.is_empty() {
        return Err(ProverError::NoNullifiersToLookUp);
    }
    let oracle = PirNullifierOracle::connect(pir_url, network, h1, expected_root)?;
    let mut notes = Vec::with_capacity(nullifiers.len());
    for nullifier in nullifiers {
        notes.push(oracle.look_up(nullifier)?);
    }
    Ok(NullifierLookup {
        height: oracle.height(),
        root: oracle.root_bytes(),
        notes,
    })
}

/// A root the gate pinned at h1 and a root a server serves must be the same root, or nothing a
/// lookup returns means anything.
pub(crate) fn require_root(expected: [u8; 32], serving: [u8; 32]) -> Result<(), ProverError> {
    if expected == serving {
        Ok(())
    } else {
        Err(ProverError::PirRootMismatch {
            expected: hex::encode(expected),
            returned: hex::encode(serving),
        })
    }
}
