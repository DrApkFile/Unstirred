#![doc = "Rebuilding each selected note's authentication path against the note-commitment tree at \
h0, and checking every path against that height's root before it is used. A path that does not \
hash to the root stops the run here rather than failing inside the circuit."]

use incrementalmerkletree::{MerklePath, Position};
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use zcash_protocol::consensus;

use crate::error::ProverError;
use crate::selection::HeldNote;
use crate::treestate::{IronwoodTreeState, IRONWOOD_TREE_DEPTH};
use crate::wallet::ReadOnlyWallet;

/// One note's authentication path in the Ironwood commitment tree.
pub type IronwoodMerklePath = MerklePath<MerkleHashOrchard, IRONWOOD_TREE_DEPTH>;

/// One note's authentication path at h0, with the root that path hashes to. The path is only
/// kept if that root is the h0 root, so `path_root` always equals [`Witnesses::root`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteWitness {
    /// The note this path is for.
    pub note: HeldNote,
    /// The path itself, a private witness of the circuit.
    pub path: IronwoodMerklePath,
    /// The root the path hashes to.
    pub path_root: [u8; 32],
}

/// Witnesses for a set of notes against one height's note commitment root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witnesses {
    /// The height the tree was read at, which is the gate's h0.
    pub height: u32,
    /// The note-commitment root at that height.
    pub root: [u8; 32],
    /// One witness per note, in selection order.
    pub notes: Vec<NoteWitness>,
}

/// Rebuilds each note's authentication path against the tree at `tree_state`'s height, then checks
/// that every path hashes to that height's root. One failure returns an error and no witness: a
/// path that does not reproduce the root is not evidence of anything.
///
/// This checks that the paths and the given tree agree, not that the tree is the real one. A
/// frontier that contradicts the wallet's own shards is rejected while the tree is rebuilt, but
/// the authenticity of the h0 root comes from the root authority (PRD section 4), never from here.
pub fn witnesses_at(
    wallet: &ReadOnlyWallet<impl consensus::Parameters>,
    tree_state: &IronwoodTreeState,
    notes: &[HeldNote],
) -> Result<Witnesses, ProverError> {
    if notes.is_empty() {
        return Err(ProverError::NoNotesToWitness);
    }
    let positions: Vec<Position> = notes.iter().map(|note| Position::from(note.position)).collect();
    let paths = wallet.ironwood_witnesses(&positions, tree_state)?;
    if paths.len() != notes.len() {
        return Err(ProverError::WitnessCountMismatch {
            expected: notes.len(),
            returned: paths.len(),
        });
    }
    let root = tree_state.root();
    let mut witnessed = Vec::with_capacity(notes.len());
    for (note, path) in notes.iter().zip(paths) {
        witnessed.push(verified_witness(*note, path, root)?);
    }
    Ok(Witnesses {
        height: tree_state.height(),
        root: root.to_bytes(),
        notes: witnessed,
    })
}

fn verified_witness(
    note: HeldNote,
    path: IronwoodMerklePath,
    root: MerkleHashOrchard,
) -> Result<NoteWitness, ProverError> {
    let position = Position::from(note.position);
    if path.position() != position {
        return Err(ProverError::WitnessPositionMismatch {
            requested: note.position,
            returned: u64::from(path.position()),
        });
    }
    let path_root = path.root(leaf_of(&note)?);
    if path_root != root {
        return Err(ProverError::WitnessRootMismatch {
            position: note.position,
            path_root: hex::encode(path_root.to_bytes()),
            tree_root: hex::encode(root.to_bytes()),
        });
    }
    Ok(NoteWitness {
        note,
        path,
        path_root: path_root.to_bytes(),
    })
}

fn leaf_of(note: &HeldNote) -> Result<MerkleHashOrchard, ProverError> {
    Option::from(MerkleHashOrchard::from_bytes(&note.commitment))
        .ok_or(ProverError::NoteCommitmentNotCanonical(note.position))
}
