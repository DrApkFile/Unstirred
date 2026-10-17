#![doc = "The gate's memory: one SQLite file of hashed challenges, markers and admission verdicts. \
Nothing else is stored. The raw challenge never touches the file. Refusals write nothing; the \
only committed write is the admission transaction in `commit`."]

mod commit;
mod query;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use unstirred_core::{Code, Refusal};
use voting_circuits::rand::rngs::OsRng;
use voting_circuits::rand::RngCore;

pub use commit::{AdmissionWrite, CommitOutcome};
pub use query::Durability;

/// How long an issued challenge stays open. The check is `expires_at > now`, so a challenge
/// issued at `t` is refused from `t + CHALLENGE_TTL_SECONDS` onward.
pub const CHALLENGE_TTL_SECONDS: i64 = 10 * 60;

const CHALLENGE_PERSONALIZATION: &[u8; 16] = b"UnstirredChal_v1";
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

/// The time the store trusts. The engine and the store never call the operating system; the
/// caller passes a clock, and only [`SystemClock`] reads one.
pub trait Clock: std::fmt::Debug {
    fn unix_seconds(&self) -> i64;
}

/// The clock the command line uses. A value that does not fit in the stored integer, or a clock
/// set before the unix epoch, becomes zero, which expires every challenge: the gate refuses
/// instead of admitting on a clock it cannot read.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn unix_seconds(&self) -> i64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(elapsed) => i64::try_from(elapsed.as_secs()).unwrap_or(0),
            Err(_) => 0,
        }
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("could not open the gate database {path}: {source}")]
    Open { path: PathBuf, source: rusqlite::Error },
    #[error("the gate database at {path} failed: {source}")]
    Sqlite { path: PathBuf, source: rusqlite::Error },
    #[error("the gate database at {path} did not switch to WAL mode (it reported {mode})")]
    NotWal { path: PathBuf, mode: String },
    #[error("the clock cannot represent an expiry, so no challenge was issued")]
    ClockUnusable,
    #[error("the admission transaction was interrupted before commit; nothing was written")]
    InterruptedBeforeCommit,
}

/// A challenge row as the store knows it. The hash is not here: callers address a row by the
/// raw challenge, and the file holds only [`challenge_hash`] of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeRow {
    pub issued_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
}

/// One stored marker. `marker` is the 32-byte canonical encoding of a `gov_null` field element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerRow {
    pub context: String,
    pub marker: [u8; 32],
    pub admitted_at: i64,
}

/// One committed admission. `code` is empty when the verdict is ADMITTED: there is no refusal code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionRow {
    pub context: String,
    pub verdict: String,
    pub code: Option<String>,
    pub at: i64,
}

impl Store {
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Draws a challenge, stores only its hash, and returns the raw bytes for the holder.
    pub fn issue(&self, context: &str, clock: &impl Clock) -> Result<[u8; 32], StoreError> {
        let mut raw = [0u8; 32];
        RngCore::fill_bytes(&mut OsRng, &mut raw);
        self.record(&raw, context, clock)?;
        Ok(raw)
    }

    /// Stores a challenge the caller already chose. The command line uses [`Store::issue`]; tests
    /// record the challenge a package was built against. The raw bytes are hashed first.
    pub fn record(&self, raw: &[u8; 32], context: &str, clock: &impl Clock) -> Result<(), StoreError> {
        let now = clock.unix_seconds();
        let expires_at = now
            .checked_add(CHALLENGE_TTL_SECONDS)
            .ok_or(StoreError::ClockUnusable)?;
        let hash = challenge_hash(raw);
        let inserted = self.conn.execute(
            "INSERT INTO challenges (hash, context, issued_at, expires_at, consumed_at)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![hash.as_slice(), context, now, expires_at],
        );
        self.db_result(inserted)?;
        Ok(())
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

/// Whether the read-only check lets this row through to the signature and the proof.
/// Expired rows are refused here. A consumed row is not: the admission transaction still has to
/// see it, so an already-admitted bundle is reported as already used rather than as an unknown
/// challenge.
pub(crate) fn challenge_is_open(row: &ChallengeRow, now: i64) -> bool {
    row.expires_at > now
}

pub(crate) fn challenge_refused() -> Refusal {
    Refusal::new(
        Code::ChallengeUnknownOrExpired,
        "the package's challenge is not an open challenge for this gate",
    )
}

pub(crate) fn already_used() -> Refusal {
    Refusal::new(
        Code::AlreadyUsedInContext,
        "a marker from this package is already stored for this gate, so the write was rolled back",
    )
}

/// BLAKE2b-256 of the raw challenge under a fixed personalisation. The personalisation keeps
/// this hash from matching any other BLAKE2b the process computes over the same bytes.
pub fn challenge_hash(raw: &[u8; 32]) -> [u8; 32] {
    let digest = blake2b_simd::Params::new()
        .hash_length(32)
        .personal(CHALLENGE_PERSONALIZATION)
        .hash(raw);
    let mut hashed = [0u8; 32];
    hashed.copy_from_slice(digest.as_bytes());
    hashed
}
