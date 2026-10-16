#![doc = "Unstirred prover: reads a zcash-devtool wallet database read-only, decides which Ironwood \
notes a gate's window allows, and rebuilds their note commitment witnesses at h0. Building, \
signing and broadcasting are not this crate's business: nothing here can create a transaction."]

pub mod error;
pub mod notes;
mod open;
pub mod selection;
pub mod treestate;
pub mod wallet;
pub mod witness;

pub use error::ProverError;
pub use notes::{eligible_notes, EligibleNotes};
pub use open::WALLET_DATABASE_FILE;
pub use selection::{select_held_notes, HeldNote, Selection, WindowNotes, MAX_NOTES_PER_PROOF};
pub use treestate::{fetch_ironwood_tree_state, IronwoodFrontier, IronwoodTreeState, IRONWOOD_TREE_DEPTH};
pub use wallet::ReadOnlyWallet;
pub use witness::{witnesses_at, IronwoodMerklePath, NoteWitness, Witnesses};
