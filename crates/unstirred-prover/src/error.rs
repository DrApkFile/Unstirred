use std::path::PathBuf;

use unstirred_core::{Network, Refusal};
use zcash_client_sqlite::error::SqliteClientError;

/// Everything that can stop the prover before it proves. A [`Refusal`] carries a code from the
/// closed set in `unstirred-core`; the other variants are local faults with no verdict meaning.
#[derive(Debug, thiserror::Error)]
pub enum ProverError {
    #[error(transparent)]
    Refused(#[from] Refusal),

    #[error("no wallet database at {0}: expected a zcash-devtool wallet directory holding data.sqlite")]
    WalletDatabaseMissing(PathBuf),

    #[error("could not open {path} read-only: {source}")]
    WalletUnopenable { path: PathBuf, source: rusqlite::Error },

    #[error("SQLite reports {0} as writable, so this handle is not the read-only one the prover asked for")]
    WalletNotReadOnly(PathBuf),

    #[error("reading the wallet database failed: {0}")]
    WalletUnreadable(Box<SqliteClientError>),

    #[error("the wallet has no accounts")]
    NoAccounts,

    #[error("the wallet holds {count} accounts; name the one to use with --account <uuid>")]
    AccountAmbiguous { count: usize },

    #[error("account {0} is not in this wallet")]
    AccountUnknown(String),

    #[error("the account has no Ironwood viewing key, so no nullifier can be derived from it")]
    AccountWithoutIronwoodViewingKey,

    #[error("the wallet recorded no held note at tree position {0} at that height")]
    NoteNotHeldAtThatHeight(u64),

    #[error("the note at tree position {0} has a nullifier that is not a canonical field element")]
    NullifierNotCanonical(u64),

    #[error("the note at tree position {0} has no mined height in the wallet database")]
    NoteWithoutMinedHeight(u64),

    #[error("the note at tree position {0} has a note commitment that is not a canonical field element")]
    NoteCommitmentNotCanonical(u64),

    #[error("{url} is not a usable lightwalletd URL: {detail}")]
    LightwalletdUrlInvalid { url: String, detail: String },

    #[error("the tree state at height {height} could not be read: {detail}")]
    TreeStateUnreadable { height: u32, detail: String },

    #[error("asked for the tree state at height {requested} and got height {returned}")]
    TreeStateHeightMismatch { requested: u32, returned: u64 },

    #[error("the tree state is for network {reported:?}, but this gate is on {expected}")]
    TreeStateWrongNetwork { expected: Network, reported: String },

    #[error("the Ironwood pool held no note at height {0}, so there is no tree to witness against")]
    IronwoodTreeEmpty(u32),

    #[error("no notes were given to witness")]
    NoNotesToWitness,

    #[error("asked for {expected} witnesses and got {returned}")]
    WitnessCountMismatch { expected: usize, returned: usize },

    #[error("asked for a witness at tree position {requested} and got one for position {returned}")]
    WitnessPositionMismatch { requested: u64, returned: u64 },

    #[error("{url} is not a usable PIR server URL: {detail}")]
    PirUrlInvalid { url: String, detail: String },

    #[error("the PIR server at {url} did not describe its dataset: {detail}")]
    PirDatasetUnreadable { url: String, detail: String },

    #[error("the PIR server serves the {pool:?} nullifier set at dataset version {version}, not ironwood version 2")]
    PirDatasetWrongPool { pool: String, version: u32 },

    #[error("the PIR server serves network {reported}, but this gate is on {expected}")]
    PirDatasetWrongNetwork { expected: Network, reported: String },

    #[error("the PIR server is synced to height {reported}, but this gate's h1 is {expected}")]
    PirDatasetWrongHeight { expected: u32, reported: String },

    #[error("the PIR server advertises tree layout {reported}, which this build does not speak: {expected}")]
    PirLayoutMismatch { expected: String, reported: String },

    #[error("the PIR server's h1 root {root:?} is not a 32-byte canonical field element")]
    PirRootNotCanonical { root: String },

    #[error("{0:?} is not a 32-byte hex-encoded root")]
    ExpectedRootMalformed(String),

    #[error("the lookup for tree position {position} returned no proof: {detail}")]
    NullifierLookupFailed { position: u64, detail: String },

    #[error("the lookup returned proofs against two roots, {first} and {other}")]
    PirRootsDisagree { first: String, other: String },

    #[error("the lookup returned proofs against root {returned}, not the expected h1 root {expected}")]
    PirRootMismatch { expected: String, returned: String },

    #[error("no nullifiers were given to look up")]
    NoNullifiersToLookUp,

    #[error("the witness for tree position {position} hashes to {path_root}, not to the root {tree_root}")]
    WitnessRootMismatch {
        position: u64,
        path_root: String,
        tree_root: String,
    },
}

impl From<SqliteClientError> for ProverError {
    fn from(error: SqliteClientError) -> Self {
        ProverError::WalletUnreadable(Box::new(error))
    }
}
