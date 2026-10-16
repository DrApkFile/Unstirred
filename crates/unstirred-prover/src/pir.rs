#![doc = "The private nullifier lookup at h1. Every selected note's real nullifier is queried \
through Valar's PIR client, which only ever puts it on the wire inside a YPIR ciphertext. There \
is no plaintext path: if the server cannot be reached the lookup refuses with PIR_UNAVAILABLE and \
the prover stops."]

use std::fmt;
use std::sync::Arc;

use pir_client::{PirClientBlocking, COMPILED_PIR_LAYOUT};
use unstirred_core::Network;
use voting_circuits::delegation::ImtProofData;

use crate::error::ProverError;
use crate::imt::{circuit_proof, proves_absence, root_bytes};
use crate::nullifier::NoteNullifier;
use crate::snapshot::{describe, unavailable};
use crate::transport::HttpTransport;

/// One selected note's non-membership proof at h1, in the shape the delegation circuit takes.
/// `proven_unspent` is the out-of-circuit check of that same proof against the note's nullifier.
///
/// [`fmt::Debug`] shows neither the path nor the leaf bounds: those bracket the holder's
/// nullifier, and the circuit keeps them as private witnesses.
#[derive(Clone)]
pub struct NoteNonMembership {
    pub position: u64,
    pub proof: ImtProofData,
    pub proven_unspent: bool,
}

impl fmt::Debug for NoteNonMembership {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoteNonMembership")
            .field("position", &self.position)
            .field("proven_unspent", &self.proven_unspent)
            .finish_non_exhaustive()
    }
}

/// Every selected note's lookup, against one h1 nullifier-set root.
#[derive(Debug, Clone)]
pub struct NullifierLookup {
    pub height: u32,
    pub root: [u8; 32],
    pub notes: Vec<NoteNonMembership>,
}

impl NullifierLookup {
    pub fn all_proven_unspent(&self) -> bool {
        self.notes.iter().all(|note| note.proven_unspent)
    }
}

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
    let base = pir_url.trim_end_matches('/');
    let transport = Arc::new(HttpTransport::for_url(base)?);
    let dataset = describe(&transport, base, network, h1)?;
    if let Some(expected) = expected_root {
        require_root(expected, dataset.root)?;
    }
    let client = PirClientBlocking::with_transport(base, COMPILED_PIR_LAYOUT, transport)
        .map_err(|e| unavailable(format!("the PIR client could not start against {base}: {e}")))?;
    let mut notes = Vec::with_capacity(nullifiers.len());
    for nullifier in nullifiers {
        notes.push(looked_up(&client, nullifier, dataset.root)?);
    }
    Ok(NullifierLookup {
        height: dataset.height,
        root: dataset.root,
        notes,
    })
}

fn looked_up(
    client: &PirClientBlocking,
    nullifier: &NoteNullifier,
    root: [u8; 32],
) -> Result<NoteNonMembership, ProverError> {
    let returned = client
        .fetch_proof(nullifier.value())
        .map_err(|e| ProverError::NullifierLookupFailed {
            position: nullifier.position(),
            detail: format!("{e}"),
        })?;
    let proof = circuit_proof(&returned);
    let returned_root = root_bytes(&proof);
    if returned_root != root {
        return Err(ProverError::PirRootsDisagree {
            first: hex::encode(root),
            other: hex::encode(returned_root),
        });
    }
    Ok(NoteNonMembership {
        position: nullifier.position(),
        proven_unspent: proves_absence(&proof, nullifier.value()),
        proof,
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
