#![doc = "What one lookup at h1 produced: the delegation builder's view of the nullifier set, and \
each selected note's non-membership proof. Neither type shows a path or a leaf bound in Debug: those \
bracket the holder's nullifier."]

use std::fmt;

use voting_circuits::delegation::{ImtProofData, ImtProvider};

use crate::error::ProverError;

/// The h1 nullifier set as the prover is allowed to ask about it: the delegation builder's
/// [`ImtProvider`], plus the typed reason the last question failed.
///
/// The builder's trait can only answer with [`ImtError`], a string. That is enough to stop a
/// proof but not enough to tell a holder what happened: a service that could not be reached is
/// `PIR_UNAVAILABLE` (inconclusive, retryable) and must never be read as "this note has moved".
pub trait NullifierSetAtH1: ImtProvider {
    /// Takes the typed error behind the most recent failed lookup, if there was one.
    fn last_failure(&self) -> Option<ProverError>;
}

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
