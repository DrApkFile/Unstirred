#![doc = "Unstirred prover: reads a zcash-devtool wallet database read-only, decides which Ironwood \
notes a gate's window allows, rebuilds their note commitment witnesses at h0, and fetches a \
private non-membership proof for each of their nullifiers at h1. Building, signing and \
broadcasting are not this crate's business: nothing here can create a transaction."]
//!
//! The holder's path, in the order the crate runs it. Compile-only: every step here needs a real
//! wallet, a lightwalletd and an nf-server, and the last one builds a real Halo2 proof.
//!
//! ```no_run
//! use std::path::Path;
//!
//! use unstirred_core::parse_manifest;
//! use unstirred_prover::{
//!     eligible_notes, fetch_ironwood_tree_state, prove, witnesses_at, OsRng, PirNullifierOracle,
//!     ProofRequest, ReadOnlyWallet,
//! };
//!
//! // The room's published rule, and the wallet, opened read-only.
//! let manifest = parse_manifest(&std::fs::read_to_string("holders-room.json")?)?;
//! let wallet = ReadOnlyWallet::open(Path::new("wallet"), manifest.network)?;
//! let account = wallet.account(None)?;
//!
//! // Does this wallet qualify at all? Nothing has touched the network or the gate yet, so a
//! // holder who does not qualify never appears in a gate's records.
//! let selection = eligible_notes(&wallet, account, &manifest)?.selection;
//!
//! // Only now is the gate's challenge worth asking for.
//! let challenge = [0u8; 32];
//!
//! let tree_state = fetch_ironwood_tree_state("https://testnet.zec.rocks:443", manifest.network, manifest.window.h0)?;
//! let witnesses = witnesses_at(&wallet, &tree_state, &selection.notes)?;
//! let nullifiers = PirNullifierOracle::connect("http://127.0.0.1:8080", manifest.network, manifest.window.h1, None)?;
//! let authority = wallet.ironwood_spend_authority(account, Path::new("identity.txt"))?;
//!
//! let proven = prove(
//!     &wallet,
//!     account,
//!     &authority,
//!     &ProofRequest {
//!         manifest: &manifest,
//!         witnesses: &witnesses,
//!         notes_unspent_at: manifest.window.h1,
//!         expected_nf_imt_root: nullifiers.root_bytes(),
//!         challenge,
//!     },
//!     &nullifiers,
//!     &mut OsRng,
//! )?;
//!
//! // The package is the only thing that leaves: no address, amount, position or nullifier.
//! println!("{}", proven.package.to_json()?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs)]

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
pub use notes::{eligible_notes, required_zatoshis, EligibleNotes};
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
/// Which account of a wallet a proof is about. Re-exported so a caller that holds a prepared
/// proof can name the type without depending on `zcash_client_sqlite` itself.
pub use zcash_client_sqlite::AccountUuid;
