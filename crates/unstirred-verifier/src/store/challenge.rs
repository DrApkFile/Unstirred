#![doc = "The challenge half of the file: issuing one, storing only its hash, and the rule that \
decides whether a stored challenge may still be presented. The raw challenge is never written."]

use rusqlite::params;
use unstirred_core::{Code, Refusal};
use voting_circuits::rand::rngs::OsRng;
use voting_circuits::rand::RngCore;

use super::{Clock, Store, StoreError};

/// How long an issued challenge stays open. The check is `expires_at > now`, so a challenge
/// issued at `t` is refused from `t + CHALLENGE_TTL_SECONDS` onward.
pub const CHALLENGE_TTL_SECONDS: i64 = 10 * 60;

const CHALLENGE_PERSONALIZATION: &[u8; 16] = b"UnstirredChal_v1";

/// A challenge row as the store knows it. The hash is not here: callers address a row by the
/// raw challenge, and the file holds only [`challenge_hash`] of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChallengeRow {
    pub issued_at: i64,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
}

impl Store {
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
