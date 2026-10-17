#![doc = "Unstirred prover: reads a zcash-devtool wallet database read-only, decides which Ironwood \
notes a gate's window allows, rebuilds their note commitment witnesses at h0, and fetches a \
private non-membership proof for each of their nullifiers at h1. Building, signing and \
broadcasting are not this crate's business: nothing here can create a transaction."]

pub mod bundle;
pub mod error;
pub mod identity;
pub mod imt;
pub mod notes;
pub mod nullifier;
mod open;
pub mod pir;
pub mod prove;
pub mod selection;
pub mod snapshot;
pub mod spend_authority;
#[cfg(feature = "test-harness")]
pub mod testing;
pub mod transport;
pub mod treestate;
pub mod wallet;
pub mod witness;

pub use bundle::{note_inputs, NoteInputs};
pub use error::ProverError;
pub use identity::{ironwood_spending_key, wallet_seed, WalletSeed, WALLET_KEYS_FILE};
pub use imt::{circuit_proof, proves_absence};
pub use notes::{eligible_notes, EligibleNotes};
pub use nullifier::NoteNullifier;
pub use open::WALLET_DATABASE_FILE;
pub use pir::{look_up_nullifiers, NoteNonMembership, NullifierLookup, PirNullifierOracle};
pub use prove::{prove, OsRng, ProofRequest, Proven};
pub use selection::{select_held_notes, HeldNote, Selection, WindowNotes, MAX_NOTES_PER_PROOF};
pub use snapshot::{root_from_hex, PirDataset};
pub use spend_authority::SpendAuthority;
pub use transport::HttpTransport;
pub use treestate::{fetch_ironwood_tree_state, IronwoodFrontier, IronwoodTreeState, IRONWOOD_TREE_DEPTH};
pub use wallet::ReadOnlyWallet;
pub use witness::{witnesses_at, IronwoodMerklePath, NoteWitness, Witnesses};
