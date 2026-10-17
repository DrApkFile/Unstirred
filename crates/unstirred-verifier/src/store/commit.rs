#![doc = "The one write an admission is allowed to make: an immediate transaction that inserts the \
five markers, consumes the challenge and records the verdict. Any failure drops the transaction, \
and SQLite rolls it back, so a conflict or a crash leaves the file unchanged."]

use rusqlite::{params, Transaction, TransactionBehavior};

use super::{already_used, challenge_hash, challenge_refused, Store, StoreError};

/// What one successful proof asks the store to remember. `challenge` is the raw value; the
/// transaction hashes it the same way [`super::Store::record`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionWrite {
    pub context: String,
    pub challenge: [u8; 32],
    pub markers: [[u8; 32]; 5],
    pub now: i64,
}

#[derive(Debug)]
pub enum CommitOutcome {
    Admitted,
    Refused(unstirred_core::Refusal),
}

impl Store {
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
