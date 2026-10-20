#![doc = "What one room of a gate shows: its newest markers and admission verdicts, newest first. \
Nothing here returns a challenge hash, and nothing here writes."]

use rusqlite::params;

use super::{AdmissionRow, MarkerRow, Store, StoreError};
use crate::store::query::{blob32, collect_rows};

impl Store {
    /// The newest markers of one context, at most `limit` of them. This is what a gate's room log
    /// shows: a marker and when it was admitted, never a challenge hash.
    pub fn recent_markers(&self, context: &str, limit: u32) -> Result<Vec<MarkerRow>, StoreError> {
        let prepared = self.conn.prepare(
            "SELECT context, marker, admitted_at FROM markers
             WHERE context = ?1 ORDER BY admitted_at DESC, rowid DESC LIMIT ?2",
        );
        let mut statement = self.db_result(prepared)?;
        let rows = statement.query_map(params![context, limit], |row| {
            Ok(MarkerRow {
                context: row.get(0)?,
                marker: blob32(row.get(1)?, "marker")?,
                admitted_at: row.get(2)?,
            })
        });
        collect_rows(&self.path, rows)
    }

    /// The newest admission verdicts of one context, at most `limit` of them.
    pub fn recent_admissions(&self, context: &str, limit: u32) -> Result<Vec<AdmissionRow>, StoreError> {
        let prepared = self.conn.prepare(
            "SELECT context, verdict, code, at FROM admissions
             WHERE context = ?1 ORDER BY at DESC, rowid DESC LIMIT ?2",
        );
        let mut statement = self.db_result(prepared)?;
        let rows = statement.query_map(params![context, limit], |row| {
            Ok(AdmissionRow {
                context: row.get(0)?,
                verdict: row.get(1)?,
                code: row.get(2)?,
                at: row.get(3)?,
            })
        });
        collect_rows(&self.path, rows)
    }
}
