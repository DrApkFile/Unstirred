#![doc = "The one write an admission is allowed to make: an immediate transaction that inserts the \
five markers, consumes the challenge and records the verdict. Any failure drops the transaction, \
and SQLite rolls it back, so a conflict or a crash leaves the file unchanged."]

use rusqlite::{params, Transaction, TransactionBehavior};

use unstirred_core::{Code, Refusal};

use super::{challenge_hash, challenge_refused, Store, StoreError};

/// What one successful proof asks the store to remember. `challenge` is the raw value; the
/// transaction hashes it the same way [`super::Store::record`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionWrite {
    /// The gate this admission belongs to. Markers are unique per context, not globally.
    pub context: String,
    /// The raw challenge the package answered. The transaction hashes it.
    pub challenge: [u8; 32],
    /// The five markers to insert. A single one already present rolls the whole write back.
    pub markers: [[u8; 32]; 5],
    /// The time to record, from the caller's clock.
    pub now: i64,
}

/// What the admission transaction did. A `Refused` outcome has written nothing: the transaction
/// was rolled back.
#[derive(Debug)]
pub enum CommitOutcome {
    /// The markers, the consumed challenge and the verdict row are committed.
    Admitted,
    /// A marker was already held, or the challenge was not open any more. Nothing was written.
    Refused(unstirred_core::Refusal),
}

impl Store {
    /// The one write an admission may make: an immediate transaction that inserts the five
    /// markers, consumes the challenge and records the verdict, or rolls all of it back.
    pub fn commit_admission(&mut self, write: &AdmissionWrite) -> Result<CommitOutcome, StoreError> {
        self.write_admission(write, false)
    }

    /// Runs the admission writes and then returns before `COMMIT`. The transaction is dropped,
    /// which rolls it back. This is the crash between the writes and the commit (A05d).
    pub fn commit_admission_failing_before_commit(
        &mut self,
        write: &AdmissionWrite,
    ) -> Result<CommitOutcome, StoreError> {
        self.write_admission(write, true)
    }

    fn write_admission(
        &mut self,
        write: &AdmissionWrite,
        fail_before_commit: bool,
    ) -> Result<CommitOutcome, StoreError> {
        let path = self.path.clone();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StoreError::Sqlite { path, source })?;
        match apply(&tx, write) {
            Ok(Applied::Ready) if fail_before_commit => Err(StoreError::InterruptedBeforeCommit),
            Ok(Applied::Ready) => {
                tx.commit().map_err(|source| StoreError::Sqlite {
                    path: self.path.clone(),
                    source,
                })?;
                Ok(CommitOutcome::Admitted)
            }
            Ok(Applied::Refused(refusal)) => Ok(CommitOutcome::Refused(refusal)),
            Err(source) => Err(StoreError::Sqlite {
                path: self.path.clone(),
                source,
            }),
        }
    }
}

/// One stored marker. `marker` is the 32-byte canonical encoding of a `gov_null` field element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerRow {
    /// The gate the marker was written for.
    pub context: String,
    /// The marker bytes. They identify these coins at this gate and nowhere else.
    pub marker: [u8; 32],
    /// When the admission that wrote it was committed.
    pub admitted_at: i64,
}

/// One committed admission. `code` is empty when the verdict is ADMITTED: there is no refusal code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionRow {
    /// The gate this verdict was recorded at.
    pub context: String,
    /// The verdict word, as the wire spells it.
    pub verdict: String,
    /// The refusal code, or `None` for an admission.
    pub code: Option<String>,
    /// When the row was committed.
    pub at: i64,
}

/// A marker this gate already holds means these coins were admitted here before. The conflict is
/// the UNIQUE constraint, so the refusal is a rolled-back transaction, never a read-then-write.
fn already_used() -> Refusal {
    Refusal::new(
        Code::AlreadyUsedInContext,
        "a marker from this package is already stored for this gate, so the write was rolled back",
    )
}

enum Applied {
    Ready,
    Refused(unstirred_core::Refusal),
}

fn apply(tx: &Transaction, write: &AdmissionWrite) -> rusqlite::Result<Applied> {
    for marker in &write.markers {
        if let Err(error) = tx.execute(
            "INSERT INTO markers (context, marker, admitted_at) VALUES (?1, ?2, ?3)",
            params![write.context, marker.as_slice(), write.now],
        ) {
            return if is_unique(&error) {
                Ok(Applied::Refused(already_used()))
            } else {
                Err(error)
            };
        }
    }
    let hash = challenge_hash(&write.challenge);
    let consumed = tx.execute(
        "UPDATE challenges SET consumed_at = ?1
         WHERE hash = ?2 AND context = ?3 AND consumed_at IS NULL AND expires_at > ?1",
        params![write.now, hash.as_slice(), write.context],
    )?;
    if consumed == 0 {
        return Ok(Applied::Refused(challenge_refused()));
    }
    tx.execute(
        "INSERT INTO admissions (context, verdict, code, at) VALUES (?1, 'ADMITTED', NULL, ?2)",
        params![write.context, write.now],
    )?;
    Ok(Applied::Ready)
}

fn is_unique(error: &rusqlite::Error) -> bool {
    match error {
        rusqlite::Error::SqliteFailure(code, message) => {
            code.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
                || (code.code == rusqlite::ErrorCode::ConstraintViolation
                    && message.as_deref().is_some_and(|text| text.contains("UNIQUE")))
        }
        _ => false,
    }
}
