#![doc = "The gate's memory: one SQLite file of hashed challenges, markers and admission verdicts. \
Nothing else is stored. The raw challenge never touches the file. Refusals write nothing; the \
only committed write is the admission transaction in `commit`."]

mod challenge;
mod clock;
mod commit;
mod query;
mod recent;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

pub use challenge::{challenge_hash, ChallengeRow, Issued, CHALLENGE_TTL_SECONDS};
pub use clock::{Clock, SystemClock};
pub use commit::{AdmissionRow, AdmissionWrite, CommitOutcome, MarkerRow};
pub use query::Durability;

pub(crate) use challenge::{challenge_is_open, challenge_refused};

const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS challenges (
    hash BLOB NOT NULL PRIMARY KEY,
    context TEXT NOT NULL,
    issued_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    consumed_at INTEGER
);
CREATE TABLE IF NOT EXISTS markers (
    context TEXT NOT NULL,
    marker BLOB NOT NULL,
    admitted_at INTEGER NOT NULL,
    UNIQUE (context, marker)
);
CREATE TABLE IF NOT EXISTS admissions (
    context TEXT NOT NULL,
    verdict TEXT NOT NULL,
    code TEXT,
    at INTEGER NOT NULL
);
";

/// One gate database. The file is created on the first open. Each handle is its own connection,
/// so two admissions race through SQLite rather than through a lock in this process.
pub struct Store {
    path: PathBuf,
    conn: Connection,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").field("path", &self.path).finish()
    }
}

/// Why the gate's own file could not be used. None of these is a statement about a holder: a
/// gate that cannot reach its memory reports INCONCLUSIVE.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The file could not be opened or created.
    #[error("could not open the gate database {path}: {source}")]
    Open {
        /// The database path that was tried.
        path: PathBuf,
        /// What SQLite reported.
        source: rusqlite::Error,
    },
    /// A statement against an open database failed.
    #[error("the gate database at {path} failed: {source}")]
    Sqlite {
        /// The database the statement ran against.
        path: PathBuf,
        /// What SQLite reported.
        source: rusqlite::Error,
    },
    /// The file would not switch to write-ahead logging, so the durability the admission
    /// transaction relies on is not there. Opening refuses rather than carrying on.
    #[error("the gate database at {path} did not switch to WAL mode (it reported {mode})")]
    NotWal {
        /// The database path.
        path: PathBuf,
        /// The journal mode SQLite reported instead.
        mode: String,
    },
    /// The clock could not produce an expiry that fits, so no challenge was issued.
    #[error("the clock cannot represent an expiry, so no challenge was issued")]
    ClockUnusable,
    /// Only [`Store::commit_admission_failing_before_commit`] returns this: the deliberate crash
    /// between the writes and the `COMMIT`, used to test that the rollback leaves nothing behind.
    #[error("the admission transaction was interrupted before commit; nothing was written")]
    InterruptedBeforeCommit,
}

impl Store {
    /// Opens the gate file, creating it and its tables if they are not there, and refusing if it
    /// will not run in WAL mode.
    ///
    /// ```
    /// use unstirred_verifier::store::{Store, SystemClock};
    ///
    /// let path = std::env::temp_dir().join(format!("unstirred-doc-{}.sqlite", std::process::id()));
    /// let store = Store::open(&path)?;
    ///
    /// // A challenge the gate hands out can be looked up by its raw bytes, at its own gate only.
    /// let issued = store.issue("holders-room", &SystemClock)?;
    /// assert!(store.lookup(&issued.raw, "holders-room")?.is_some());
    /// assert!(store.lookup(&issued.raw, "second-room")?.is_none());
    ///
    /// // The file holds only the hash, so the raw challenge is not in it.
    /// drop(store);
    /// let bytes = std::fs::read(&path)?;
    /// assert!(!bytes.windows(32).any(|w| w == issued.raw));
    /// # std::fs::remove_file(&path)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path).map_err(|source| StoreError::Open {
            path: path.clone(),
            source,
        })?;
        let store = Self { path, conn };
        store.configure()?;
        let created = store.conn.execute_batch(SCHEMA);
        store.db_result(created)?;
        Ok(store)
    }

    /// The file this handle is open on.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn configure(&self) -> Result<(), StoreError> {
        let queried = self.conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0));
        let mode: String = self.db_result(queried)?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(StoreError::NotWal {
                path: self.path.clone(),
                mode,
            });
        }
        let timeout = self.conn.busy_timeout(BUSY_TIMEOUT);
        self.db_result(timeout)
    }

    pub(crate) fn db_result<T>(&self, result: rusqlite::Result<T>) -> Result<T, StoreError> {
        result.map_err(|source| StoreError::Sqlite {
            path: self.path.clone(),
            source,
        })
    }
}
