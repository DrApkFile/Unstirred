#![doc = "Turning what the first three steps produced into the delegation circuit's note inputs: \
one real Ironwood note per selected position, with its witness at h0 and its non-membership proof \
at h1. Every proof is checked against the pinned h1 root and against the note's own nullifier \
before it is handed to the builder, so a lookup that does not actually exclude a nullifier stops \
the proof instead of going into it."]

use std::fmt;

use voting_circuits::delegation::{ImtProofData, RealNoteInput};
use voting_crypto_deps::orchard::keys::{FullViewingKey, Scope};
use voting_crypto_deps::orchard::note::{Note, NoteVersion};
use voting_crypto_deps::orchard::tree::{MerkleHashOrchard, MerklePath as IronwoodPath};
use zcash_client_sqlite::AccountUuid;
use zcash_protocol::consensus::{self, BlockHeight};

use crate::error::ProverError;
use crate::imt::{proves_absence, root_bytes};
use crate::nullifier::NoteNullifier;
use crate::pir::NullifierSetAtH1;
use crate::treestate::IRONWOOD_TREE_DEPTH;
use crate::wallet::ReadOnlyWallet;
use crate::witness::{NoteWitness, Witnesses};

/// The Ironwood note version the delegation circuit takes. Ironwood notes are V3; an Orchard note
/// of an older version is not provable here, and the builder refuses it too.
const IRONWOOD_NOTE_VERSION: NoteVersion = NoteVersion::V3;

/// The real-note half of one delegation bundle, in the order the notes were selected.
///
/// [`fmt::Debug`] shows the tree positions only. The notes, their witnesses and their leaf bounds
/// are private witnesses of the circuit, and nothing outside it needs to see them.
pub struct NoteInputs {
    pub positions: Vec<u64>,
    pub inputs: Vec<RealNoteInput>,
}

impl fmt::Debug for NoteInputs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoteInputs")
            .field("positions", &self.positions)
            .finish_non_exhaustive()
    }
}

/// Assembles the circuit inputs for the witnessed notes. `h1` is where the nullifiers are read
/// and looked up; the witnesses are at h0, which is what makes the pair a holding statement.
pub fn note_inputs<P: consensus::Parameters>(
    wallet: &ReadOnlyWallet<P>,
    account: AccountUuid,
    viewing_key: &FullViewingKey,
    witnesses: &Witnesses,
    h1: u32,
    nullifier_set: &impl NullifierSetAtH1,
    expected_nf_imt_root: [u8; 32],
) -> Result<NoteInputs, ProverError> {
    let positions: Vec<u64> = witnesses.notes.iter().map(|witness| witness.note.position).collect();
    let nullifiers = wallet.ironwood_nullifiers(account, h1, &positions)?;
    let notes = wallet.ironwood_notes_at(account, witnesses.height, &positions)?;
    let mut inputs = Vec::with_capacity(positions.len());
    for (witness, nullifier) in witnesses.notes.iter().zip(nullifiers.iter()) {
        let position = witness.note.position;
        if nullifier.position() != position {
            return Err(ProverError::NullifierPositionMismatch {
                requested: position,
                returned: nullifier.position(),
            });
        }
        let (note, scope) = notes
            .iter()
            .find(|(held, _, _)| *held == position)
            .map(|(_, note, scope)| (note, *scope))
            .ok_or(ProverError::NoteNotHeldAtThatHeight(position))?;
        require_ironwood(note, position)?;
        inputs.push(RealNoteInput {
            note: *note,
            fvk: viewing_key.clone(),
            merkle_path: ironwood_path(witness)?,
            imt_proof: non_membership(nullifier_set, nullifier, expected_nf_imt_root)?,
            scope,
        });
    }
    Ok(NoteInputs { positions, inputs })
}

impl<P: consensus::Parameters> ReadOnlyWallet<P> {
    /// The real notes behind a set of tree positions, with the scope each was received on. Only
    /// notes the wallet recorded as unspent at `height` are returned, so a position the window
    /// does not allow has nothing to find.
    pub(crate) fn ironwood_notes_at(
        &self,
        account: AccountUuid,
        height: u32,
        positions: &[u64],
    ) -> Result<Vec<(u64, Note, Scope)>, ProverError> {
        let held = self
            .db
            .get_unspent_ironwood_notes_at_historical_height(account, BlockHeight::from_u32(height))?;
        positions
            .iter()
            .map(|position| {
                held.iter()
                    .find(|note| u64::from(note.note_commitment_tree_position()) == *position)
                    .map(|note| (*position, *note.note(), note.spending_key_scope()))
                    .ok_or(ProverError::NoteNotHeldAtThatHeight(*position))
            })
            .collect()
    }
}

/// Asks the h1 nullifier set about one note, then checks the answer twice: it must authenticate
/// to the root the gate pinned, and it must actually exclude this note's nullifier.
fn non_membership(
    nullifier_set: &impl NullifierSetAtH1,
    nullifier: &NoteNullifier,
    expected_nf_imt_root: [u8; 32],
) -> Result<ImtProofData, ProverError> {
    let position = nullifier.position();
    let proof = nullifier_set
        .non_membership_proof(nullifier.value())
        .map_err(|refused| {
            nullifier_set
                .last_failure()
                .unwrap_or(ProverError::NoteNotProvenUnspent {
                    position,
                    detail: refused.0,
                })
        })?;
    let root = root_bytes(&proof);
    if root != expected_nf_imt_root {
        return Err(ProverError::NfImtRootMismatch {
            expected: hex::encode(expected_nf_imt_root),
            provider: hex::encode(root),
        });
    }
    if !proves_absence(&proof, nullifier.value()) {
        return Err(ProverError::NoteNotProvenUnspent {
            position,
            detail: "the returned leaf does not exclude this note's nullifier".to_owned(),
        });
    }
    Ok(proof)
}

fn require_ironwood(note: &Note, position: u64) -> Result<(), ProverError> {
    if note.version() == IRONWOOD_NOTE_VERSION {
        Ok(())
    } else {
        Err(ProverError::NoteNotIronwood {
            position,
            version: format!("{:?}", note.version()),
        })
    }
}

/// Moves one verified witness into the path type the circuit takes. The two types hold the same
/// path; only the position's width and the array's shape differ.
fn ironwood_path(witness: &NoteWitness) -> Result<IronwoodPath, ProverError> {
    let position = u32::try_from(witness.note.position)
        .map_err(|_| ProverError::PositionTooLargeForPath(witness.note.position))?;
    let elements = witness.path.path_elems();
    let auth_path = <[MerkleHashOrchard; IRONWOOD_TREE_DEPTH as usize]>::try_from(elements).map_err(|_| {
        ProverError::WitnessPathWrongLength {
            position: witness.note.position,
            expected: IRONWOOD_TREE_DEPTH as usize,
            returned: elements.len(),
        }
    })?;
    Ok(IronwoodPath::from_parts(position, auth_path))
}
