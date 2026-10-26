#![doc = "Everything that can stop the prover, as one enum. A variant either carries a verdict \
code from the closed set in `unstirred-core` or is a local fault with no verdict meaning at all; \
the two are never mixed, so a local problem can never be reported to a holder as a refusal."]

use std::path::PathBuf;

use unstirred_core::{Network, Refusal};
use zcash_client_sqlite::error::SqliteClientError;

/// Everything that can stop the prover before it proves. A [`Refusal`] carries a code from the
/// closed set in `unstirred-core`; the other variants are local faults with no verdict meaning.
#[derive(Debug, thiserror::Error)]
pub enum ProverError {
    /// A verdict-bearing refusal from the closed code set. Everything else here is a local fault with no verdict meaning.
    #[error(transparent)]
    Refused(#[from] Refusal),

    /// The wallet path holds no `data.sqlite`, so there is no wallet to read.
    #[error("no wallet database at {0}: expected a zcash-devtool wallet directory holding data.sqlite")]
    WalletDatabaseMissing(PathBuf),

    /// SQLite would not open the wallet at all.
    #[error("could not open {path} read-only: {source}")]
    WalletUnopenable {
        /// The wallet database that was tried.
        path: PathBuf,
        /// What SQLite reported.
        source: rusqlite::Error,
    },

    /// The handle came back writable. The prover refuses rather than hold a wallet it could change.
    #[error("SQLite reports {0} as writable, so this handle is not the read-only one the prover asked for")]
    WalletNotReadOnly(PathBuf),

    /// A read against an open wallet failed.
    #[error("reading the wallet database failed: {0}")]
    WalletUnreadable(Box<SqliteClientError>),

    /// The wallet's `keys.toml` could not be read.
    #[error("no keys file could be read at {path}: {detail}")]
    KeysFileUnreadable {
        /// The keys file that was tried.
        path: PathBuf,
        /// Why it could not be read.
        detail: String,
    },

    /// The keys file records no mnemonic, so this wallet can prove nothing: a proof needs the spending key.
    #[error("{0} holds no mnemonic, so this wallet cannot sign anything")]
    WalletHasNoMnemonic(PathBuf),

    /// The wallet path has no parent, so the keys file beside it cannot be located.
    #[error("{0} has no parent directory, so the wallet's keys file cannot be found")]
    WalletDirectoryUnknown(PathBuf),

    /// The age identity file could not be read as an identity.
    #[error("{path} is not a usable age identity file: {detail}")]
    IdentityFileUnusable {
        /// The identity file that was tried.
        path: PathBuf,
        /// Why it is not usable as an age identity.
        detail: String,
    },

    /// That identity does not decrypt this wallet's mnemonic.
    #[error("the wallet's mnemonic could not be decrypted with that identity: {0}")]
    MnemonicUndecryptable(String),

    /// What the identity decrypted is not a BIP 39 phrase.
    #[error("what the identity decrypted is not a BIP 39 mnemonic phrase: {0}")]
    MnemonicNotAPhrase(String),

    /// The decrypted seed is not a length ZIP 32 can fingerprint.
    #[error("the decrypted seed is not a length ZIP 32 can fingerprint")]
    SeedNotUsableLength,

    /// The seed fingerprint does not match the account's, so this is a mnemonic for a different wallet.
    #[error("that identity decrypts a mnemonic for another wallet: its seed fingerprint is not the one this account was derived from")]
    SeedIsForAnotherWallet,

    /// The account was imported from a viewing key. A viewing key can see the notes but cannot sign, so it cannot prove.
    #[error("the account was imported from a viewing key, so the wallet records no derivation path and no spending key can be derived")]
    AccountIsViewOnly,

    /// ZIP 32 derivation from the seed failed.
    #[error("no spending key could be derived from the seed: {0}")]
    SpendingKeyUnderivable(String),

    /// The derived key does not control this account's notes. No other derivation path is tried.
    #[error("the spending key derived from that seed does not produce this account's Ironwood viewing key, so it is not the key that controls these notes. Ironwood accounts may use a key path this version does not derive; no other derivation is attempted")]
    SpendingKeyIsNotThisAccounts,

    /// The lookup service serves a different nullifier-set root than the gate's h1, so its answers cannot be used.
    #[error("the nullifier-set root the lookup service serves, {provider}, is not the expected h1 root {expected}")]
    NfImtRootMismatch {
        /// The gate's h1 nullifier-set root, hex.
        expected: String,
        /// The root the lookup service serves instead, hex.
        provider: String,
    },

    /// No non-membership proof could be had for one note's nullifier at h1.
    #[error("the note at tree position {position} cannot be proven unmoved at h1: {detail}")]
    NoteNotProvenUnspent {
        /// The note's position in the Ironwood commitment tree.
        position: u64,
        /// What the lookup reported.
        detail: String,
    },

    /// The h0 note-commitment root is not a canonical field element.
    #[error("the note commitment root at h0, {0}, is not a canonical field element")]
    NcRootNotCanonical(String),

    /// The note is from another pool version. The delegation circuit takes Ironwood notes only.
    #[error("the note at tree position {position} is {version:?}, not the Ironwood note version the delegation circuit takes")]
    NoteNotIronwood {
        /// The note's position in the commitment tree.
        position: u64,
        /// The note version the wallet records for it.
        version: String,
    },

    /// The tree position does not fit the 32 bits an Ironwood Merkle path carries.
    #[error("tree position {0} does not fit the 32-bit position an Ironwood Merkle path carries")]
    PositionTooLargeForPath(u64),

    /// A witness came back with the wrong number of path elements for the Ironwood tree.
    #[error("the witness at tree position {position} has {returned} path elements, not the {expected} the Ironwood tree has")]
    WitnessPathWrongLength {
        /// The position the witness was asked for.
        position: u64,
        /// The Ironwood tree's depth.
        expected: usize,
        /// How many path elements came back.
        returned: usize,
    },

    /// The delegation bundle could not be assembled from the witnesses.
    #[error("the delegation bundle could not be built: {0}")]
    BundleUnbuildable(String),

    /// Halo2 proving itself failed.
    #[error("proving failed: {0}")]
    ProvingFailed(String),

    /// The prover verified its own fresh proof and it did not pass, so nothing was written. A proof that fails here would fail at the gate.
    #[error("the proof this prover just made does not verify against its own instance ({0}), so nothing was written")]
    ProofDidNotVerifyHere(String),

    /// The prover checked its own fresh binding signature under rk and it did not pass, so nothing was written.
    #[error("the binding signature this prover just made does not verify under rk from the instance, so nothing was written")]
    BindingSignatureDidNotVerifyHere,

    /// The wallet holds no account to prove for.
    #[error("the wallet has no accounts")]
    NoAccounts,

    /// The wallet holds more than one account and none was named. The prover never picks for the holder.
    #[error("the wallet holds {count} accounts; name the one to use with --account <uuid>")]
    AccountAmbiguous {
        /// How many accounts the wallet holds.
        count: usize,
    },

    /// The named account is not in this wallet.
    #[error("account {0} is not in this wallet")]
    AccountUnknown(String),

    /// The account has no Ironwood viewing key, so no nullifier can be derived.
    #[error("the account has no Ironwood viewing key, so no nullifier can be derived from it")]
    AccountWithoutIronwoodViewingKey,

    /// The wallet records no held note at that position at that height.
    #[error("the wallet recorded no held note at tree position {0} at that height")]
    NoteNotHeldAtThatHeight(u64),

    /// A note's nullifier is not a canonical field element.
    #[error("the note at tree position {0} has a nullifier that is not a canonical field element")]
    NullifierNotCanonical(u64),

    /// A note has no mined height in the wallet, so it cannot be placed inside the window.
    #[error("the note at tree position {0} has no mined height in the wallet database")]
    NoteWithoutMinedHeight(u64),

    /// A note's commitment is not a canonical field element.
    #[error("the note at tree position {0} has a note commitment that is not a canonical field element")]
    NoteCommitmentNotCanonical(u64),

    /// The lightwalletd URL does not parse, or names a transport this build does not speak.
    #[error("{url} is not a usable lightwalletd URL: {detail}")]
    LightwalletdUrlInvalid {
        /// The URL as given.
        url: String,
        /// What is wrong with it.
        detail: String,
    },

    /// `GetTreeState` for h0 failed.
    #[error("the tree state at height {height} could not be read: {detail}")]
    TreeStateUnreadable {
        /// The height the tree state was asked for.
        height: u32,
        /// What the server reported.
        detail: String,
    },

    /// The server answered for a height other than the one asked for.
    #[error("asked for the tree state at height {requested} and got height {returned}")]
    TreeStateHeightMismatch {
        /// The height that was asked for.
        requested: u32,
        /// The height the server answered for.
        returned: u64,
    },

    /// The server is on another network, so its tree is not this gate's.
    #[error("the tree state is for network {reported:?}, but this gate is on {expected}")]
    TreeStateWrongNetwork {
        /// The gate's network.
        expected: Network,
        /// The chain name the server reported.
        reported: String,
    },

    /// The Ironwood pool was empty at h0, so there is no tree to witness against.
    #[error("the Ironwood pool held no note at height {0}, so there is no tree to witness against")]
    IronwoodTreeEmpty(u32),

    /// Witnessing was asked for with no notes.
    #[error("no notes were given to witness")]
    NoNotesToWitness,

    /// Fewer or more witnesses came back than notes were given.
    #[error("asked for {expected} witnesses and got {returned}")]
    WitnessCountMismatch {
        /// How many notes were given.
        expected: usize,
        /// How many witnesses came back.
        returned: usize,
    },

    /// A witness came back for a position other than the one asked for.
    #[error("asked for a witness at tree position {requested} and got one for position {returned}")]
    WitnessPositionMismatch {
        /// The position that was asked for.
        requested: u64,
        /// The position the witness is for.
        returned: u64,
    },

    /// A nullifier was offered for a position other than the one asked for.
    #[error("the nullifier offered for tree position {requested} belongs to position {returned}")]
    NullifierPositionMismatch {
        /// The position that was asked for.
        requested: u64,
        /// The position the nullifier belongs to.
        returned: u64,
    },

    /// The PIR server URL does not parse, or names a transport this build does not speak.
    #[error("{url} is not a usable PIR server URL: {detail}")]
    PirUrlInvalid {
        /// The URL as given.
        url: String,
        /// What is wrong with it.
        detail: String,
    },

    /// The PIR server would not describe the dataset it serves, so none of its answers can be placed.
    #[error("the PIR server at {url} did not describe its dataset: {detail}")]
    PirDatasetUnreadable {
        /// The PIR server that was asked.
        url: String,
        /// What it reported instead.
        detail: String,
    },

    /// The PIR server serves another pool or dataset version.
    #[error("the PIR server serves the {pool:?} nullifier set at dataset version {version}, not ironwood version 2")]
    PirDatasetWrongPool {
        /// The pool the server serves.
        pool: String,
        /// The dataset version it serves.
        version: u32,
    },

    /// The PIR server serves another network.
    #[error("the PIR server serves network {reported}, but this gate is on {expected}")]
    PirDatasetWrongNetwork {
        /// The gate's network.
        expected: Network,
        /// The network the server serves.
        reported: String,
    },

    /// The PIR server is synced to another height, so it cannot answer at this gate's h1.
    #[error("the PIR server is synced to height {reported}, but this gate's h1 is {expected}")]
    PirDatasetWrongHeight {
        /// The gate's h1.
        expected: u32,
        /// The height the server is synced to.
        reported: String,
    },

    /// The PIR server's tree layout is not the one this build's non-membership proofs assume.
    #[error("the PIR server advertises tree layout {reported}, which this build does not speak: {expected}")]
    PirLayoutMismatch {
        /// The layout this build speaks.
        expected: String,
        /// The layout the server advertises.
        reported: String,
    },

    /// The PIR server's h1 root is not a 32-byte canonical field element.
    #[error("the PIR server's h1 root {root:?} is not a 32-byte canonical field element")]
    PirRootNotCanonical {
        /// The root the server served, as it was given.
        root: String,
    },

    /// The root the caller supplied is not 32 bytes of hex.
    #[error("{0:?} is not a 32-byte hex-encoded root")]
    ExpectedRootMalformed(String),

    /// No non-membership proof came back. A PIR answer cannot distinguish a spent note from a server that would not answer, so the prover says only that it could not obtain one.
    #[error("no non-membership proof came back for a nullifier: {0}. Either the set contains it, or the server would not answer; a PIR answer cannot tell those apart")]
    NullifierProofUnobtainable(String),

    /// One note's lookup returned no proof.
    #[error("the lookup for tree position {position} returned no proof: {detail}")]
    NullifierLookupFailed {
        /// The note's position in the commitment tree.
        position: u64,
        /// What the lookup reported.
        detail: String,
    },

    /// The lookup's proofs are against two different roots, so they cannot all be about h1.
    #[error("the lookup returned proofs against two roots, {first} and {other}")]
    PirRootsDisagree {
        /// The first root seen, hex.
        first: String,
        /// The differing root, hex.
        other: String,
    },

    /// The lookup's proofs are against a root other than the gate's h1.
    #[error("the lookup returned proofs against root {returned}, not the expected h1 root {expected}")]
    PirRootMismatch {
        /// The gate's h1 root, hex.
        expected: String,
        /// The root the proofs are against, hex.
        returned: String,
    },

    /// A lookup was asked for with no nullifiers.
    #[error("no nullifiers were given to look up")]
    NoNullifiersToLookUp,

    /// A witness does not hash up to the tree's own root. The prover checks this itself rather than letting the circuit fail later.
    #[error("the witness for tree position {position} hashes to {path_root}, not to the root {tree_root}")]
    WitnessRootMismatch {
        /// The note's position in the commitment tree.
        position: u64,
        /// What the witness hashes up to, hex.
        path_root: String,
        /// The tree's own root, hex.
        tree_root: String,
    },
}

impl From<SqliteClientError> for ProverError {
    fn from(error: SqliteClientError) -> Self {
        ProverError::WalletUnreadable(Box::new(error))
    }
}
