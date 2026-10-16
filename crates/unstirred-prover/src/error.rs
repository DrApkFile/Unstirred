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

    #[error("no keys file could be read at {path}: {detail}")]
    KeysFileUnreadable { path: PathBuf, detail: String },

    #[error("{0} holds no mnemonic, so this wallet cannot sign anything")]
    WalletHasNoMnemonic(PathBuf),

    #[error("{0} has no parent directory, so the wallet's keys file cannot be found")]
    WalletDirectoryUnknown(PathBuf),

    #[error("{path} is not a usable age identity file: {detail}")]
    IdentityFileUnusable { path: PathBuf, detail: String },

    #[error("the wallet's mnemonic could not be decrypted with that identity: {0}")]
    MnemonicUndecryptable(String),

    #[error("what the identity decrypted is not a BIP 39 mnemonic phrase: {0}")]
    MnemonicNotAPhrase(String),

    #[error("the decrypted seed is not a length ZIP 32 can fingerprint")]
    SeedNotUsableLength,

    #[error("that identity decrypts a mnemonic for another wallet: its seed fingerprint is not the one this account was derived from")]
    SeedIsForAnotherWallet,

    #[error("the account was imported from a viewing key, so the wallet records no derivation path and no spending key can be derived")]
    AccountIsViewOnly,

    #[error("no spending key could be derived from the seed: {0}")]
    SpendingKeyUnderivable(String),

    #[error("the spending key derived from that seed does not produce this account's Ironwood viewing key, so it is not the key that controls these notes. Ironwood accounts may use a key path this version does not derive; no other derivation is attempted")]
    SpendingKeyIsNotThisAccounts,

    #[error("the nullifier-set root the lookup service serves, {provider}, is not the expected h1 root {expected}")]
    NfImtRootMismatch { expected: String, provider: String },

    #[error("the note at tree position {position} cannot be proven unmoved at h1: {detail}")]
    NoteNotProvenUnspent { position: u64, detail: String },

    #[error("the note commitment root at h0, {0}, is not a canonical field element")]
    NcRootNotCanonical(String),

    #[error("the note at tree position {position} is {version:?}, not the Ironwood note version the delegation circuit takes")]
    NoteNotIronwood { position: u64, version: String },

    #[error("tree position {0} does not fit the 32-bit position an Ironwood Merkle path carries")]
    PositionTooLargeForPath(u64),

    #[error("the witness at tree position {position} has {returned} path elements, not the {expected} the Ironwood tree has")]
    WitnessPathWrongLength {
        position: u64,
        expected: usize,
        returned: usize,
    },

    #[error("the delegation bundle could not be built: {0}")]
    BundleUnbuildable(String),

    #[error("proving failed: {0}")]
    ProvingFailed(String),

    #[error("the proof this prover just made does not verify against its own instance ({0}), so nothing was written")]
    ProofDidNotVerifyHere(String),

    #[error("the binding signature this prover just made does not verify under rk from the instance, so nothing was written")]
    BindingSignatureDidNotVerifyHere,

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

    #[error("the nullifier offered for tree position {requested} belongs to position {returned}")]
    NullifierPositionMismatch { requested: u64, returned: u64 },

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

    #[error("no non-membership proof came back for a nullifier: {0}. Either the set contains it, or the server would not answer; a PIR answer cannot tell those apart")]
    NullifierProofUnobtainable(String),

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
