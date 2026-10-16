use std::path::PathBuf;

use unstirred_core::Refusal;
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

    #[error("the note at tree position {0} has no mined height in the wallet database")]
    NoteWithoutMinedHeight(u64),
}

impl From<SqliteClientError> for ProverError {
    fn from(error: SqliteClientError) -> Self {
        ProverError::WalletUnreadable(Box::new(error))
    }
}
