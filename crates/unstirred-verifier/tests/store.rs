//! The gate database. **Synthetic** where a package is involved: the notes come from
//! `store/fixture.rs`. The SQLite file, the immediate transaction and the admission order are
//! the real ones. Everything that attacks the store is in `tests/adversarial/`; what is left
//! here is what an honest admission writes, and what a crash before the commit does not.

#[path = "adversarial"]
mod adversarial {
    mod challenge_misuse;
    mod forced_newcomer;
    mod replayed_coins;
}

mod store {
    pub mod assertions;
    mod dump;
    pub mod fixture;
    pub mod support;
    mod two_contexts;
}

use store::support::{admission, clock_at, distinct_markers, expect_admitted, open_gate, TestResult, NOW};
use unstirred_verifier::store::{Store, StoreError, CHALLENGE_TTL_SECONDS};

const KEPT: [u8; 32] = [0xa1; 32];
const INTERRUPTED: [u8; 32] = [0xb1; 32];

/// A05d: the writes of an admission that dies before `COMMIT` are gone, including when an
/// earlier admission had already committed. Also the file is WAL and this connection waits.
#[test]
fn a05d_a_failure_before_commit_persists_nothing() -> TestResult {
    let (_directory, mut store) = open_gate()?;
    let clock = clock_at(NOW);
    let durability = store.durability()?;
    assert!(
        durability.journal_mode.eq_ignore_ascii_case("wal"),
        "the gate file must be WAL, it reported {}",
        durability.journal_mode
    );
    assert!(
        durability.busy_timeout_ms >= 10_000,
        "the busy timeout must be at least 10s, it was {}ms",
        durability.busy_timeout_ms
    );
    let reopened = rusqlite::Connection::open(store.path())?;
    let persisted: String = reopened.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    assert!(
        persisted.eq_ignore_ascii_case("wal"),
        "a second connection must still see WAL, it reported {persisted}"
    );

    store.record(&KEPT, "holders-room", &clock)?;
    expect_admitted(store.commit_admission(&admission("holders-room", KEPT, distinct_markers(0xa1), NOW))?)?;
    store.record(&INTERRUPTED, "holders-room", &clock)?;
    let interrupted = store.commit_admission_failing_before_commit(&admission(
        "holders-room",
        INTERRUPTED,
        distinct_markers(0xb1),
        NOW,
    ));
    match interrupted {
        Err(StoreError::InterruptedBeforeCommit) => {}
        other => return Err(format!("the injected failure must be the one before commit, got {other:?}").into()),
    }

    let path = store.path().to_path_buf();
    drop(store);
    let store = Store::open(&path)?;
    let rows = store.challenges()?;
    assert_eq!(rows.len(), 2, "both challenges were recorded before the crash");
    assert_eq!(
        rows.iter().filter(|row| row.consumed_at == Some(NOW)).count(),
        1,
        "only the committed admission consumes its challenge"
    );
    assert_eq!(
        rows.iter().filter(|row| row.consumed_at.is_none()).count(),
        1,
        "the interrupted admission must leave its challenge unconsumed"
    );
    for row in &rows {
        assert_eq!(
            row.expires_at - row.issued_at,
            CHALLENGE_TTL_SECONDS,
            "a challenge expires ten minutes after it is issued"
        );
    }
    let markers = store.markers()?;
    let kept = distinct_markers(0xa1);
    assert_eq!(markers.len(), 5, "the interrupted markers must not be stored");
    assert!(
        markers.iter().all(|row| kept.contains(&row.marker)),
        "only the committed markers remain"
    );
    assert_eq!(
        store.admissions()?.len(),
        1,
        "the interrupted admission row must not be stored"
    );
    Ok(())
}
