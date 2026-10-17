#![doc = "Reads of the gate file. Nothing here writes, and nothing here returns a challenge hash."]

use std::path::Path;

use rusqlite::{params, OptionalExtension};

use super::{challenge_hash, AdmissionRow, ChallengeRow, MarkerRow, Store, StoreError};

impl Store {
    /// The row for this challenge at this context, if one was issued. A challenge issued for
    /// another context is absent here, which is the same result as one that was never issued.
    pub fn lookup(&self, raw: &[u8; 32], context: &str) -> Result<Option<ChallengeRow>, StoreError> {
        let hash = challenge_hash(raw);
        let found = self
            .conn
            .query_row(
                "SELECT issued_at, expires_at, consumed_at FROM challenges
                 WHERE hash = ?1 AND context = ?2",
                params![hash.as_slice(), context],
                |row| {
                    Ok(ChallengeRow {
                        issued_at: row.get(0)?,
                        expires_at: row.get(1)?,
                        consumed_at: row.get(2)?,
                    })
                },
            )
            .optional();
        self.db_result(found)
    }

    pub fn challenges(&self) -> Result<Vec<ChallengeRow>, StoreError> {
        let prepared = self.conn.prepare(
            "SELECT issued_at, expires_at, consumed_at FROM challenges
             ORDER BY issued_at, context",
        );
        let mut statement = self.db_result(prepared)?;
        let rows = statement.query_map([], |row| {
            Ok(ChallengeRow {
                issued_at: row.get(0)?,
                expires_at: row.get(1)?,
                consumed_at: row.get(2)?,
            })
        });
        collect_rows(&self.path, rows)
    }

    pub fn markers(&self) -> Result<Vec<MarkerRow>, StoreError> {
        let prepared = self
            .conn
            .prepare("SELECT context, marker, admitted_at FROM markers ORDER BY context, marker");
        let mut statement = self.db_result(prepared)?;
        let rows = statement.query_map([], |row| {
            Ok(MarkerRow {
                context: row.get(0)?,
                marker: blob32(row.get(1)?, "marker")?,
                admitted_at: row.get(2)?,
            })
        });
        collect_rows(&self.path, rows)
    }

    pub fn admissions(&self) -> Result<Vec<AdmissionRow>, StoreError> {
        let prepared = self
            .conn
            .prepare("SELECT context, verdict, code, at FROM admissions ORDER BY context, at, rowid");
        let mut statement = self.db_result(prepared)?;
        let rows = statement.query_map([], |row| {
            Ok(AdmissionRow {
                context: row.get(0)?,
                verdict: row.get(1)?,
                code: row.get(2)?,
                at: row.get(3)?,
            })
        });
        collect_rows(&self.path, rows)
    }

    /// The `CREATE` statements SQLite recorded. A dump test reads this to see that the file grew
    /// no extra table.
    pub fn schema_sql(&self) -> Result<String, StoreError> {
        let prepared = self
            .conn
            .prepare("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY sql");
        let mut statement = self.db_result(prepared)?;
        let statements = collect_rows(&self.path, statement.query_map([], |row| row.get::<_, String>(0)))?;
        Ok(statements.join("\n"))
    }

    /// Folds the write-ahead log into the main file so a byte dump of the path sees every row.
    pub fn checkpoint(&self) -> Result<(), StoreError> {
        let flushed = self.conn.execute_batch("PRAGMA wal_checkpoint(FULL)");
        self.db_result(flushed)
    }

    /// Journal mode and this connection's busy timeout. The timeout lives on the connection, so a
    /// second handle opened on the same file does not carry it; the journal mode does.
    pub fn durability(&self) -> Result<Durability, StoreError> {
        let mode = self.conn.query_row("PRAGMA journal_mode", [], |row| row.get(0));
        let journal_mode = self.db_result(mode)?;
        let timeout = self.conn.query_row("PRAGMA busy_timeout", [], |row| row.get(0));
        Ok(Durability {
            journal_mode,
            busy_timeout_ms: self.db_result(timeout)?,
        })
    }
}

/// What [`Store::durability`] reports. `busy_timeout_ms` is this connection's setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Durability {
    pub journal_mode: String,
    pub busy_timeout_ms: i64,
}

fn collect_rows<T>(
    path: &Path,
    rows: rusqlite::Result<impl Iterator<Item = rusqlite::Result<T>>>,
) -> Result<Vec<T>, StoreError> {
    let path = path.to_path_buf();
    match rows {
        Ok(rows) => rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|source| StoreError::Sqlite { path, source }),
        Err(source) => Err(StoreError::Sqlite { path, source }),
    }
}

fn blob32(bytes: Vec<u8>, column: &'static str) -> rusqlite::Result<[u8; 32]> {
    <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| rusqlite::Error::InvalidColumnType(0, column.to_owned(), rusqlite::types::Type::Blob))
}
