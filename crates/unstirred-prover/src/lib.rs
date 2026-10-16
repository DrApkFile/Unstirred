#![doc = "Unstirred prover: reads a zcash-devtool wallet database read-only and decides which Ironwood \
notes a gate's window allows. Building, signing and broadcasting are not this crate's business: \
nothing here can create a transaction."]

pub mod error;
pub mod notes;
mod open;
pub mod selection;
pub mod wallet;

pub use error::ProverError;
pub use notes::{eligible_notes, EligibleNotes};
pub use open::WALLET_DATABASE_FILE;
pub use selection::{select_held_notes, HeldNote, Selection, WindowNotes, MAX_NOTES_PER_PROOF};
pub use wallet::ReadOnlyWallet;
