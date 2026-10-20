//! What the gate file holds after an admission, read as bytes and as schema. **Synthetic:** the
//! package comes from `store/fixture.rs`, and what a leak would look like is what that fixture
//! knows about the note behind it.

use unstirred_verifier::store::Store;
use unstirred_verifier::Verdict;

use super::assertions::admit_made;
use super::fixture::{corpus, Made};
use super::support::{clock_at, file_bytes, require_absent, TestResult, NOW};

/// A02b: the file holds the challenge hash, the markers and the verdict. It does not hold the
/// raw challenge, the package, a note position or a note value.
#[test]
fn a02b_a_database_dump_contains_no_ip_package_body_position_or_value() -> TestResult {
    let made = corpus()?;
    let primary = &made.primary;
    let (_directory, mut store) = super::support::open_gate()?;
    store.record(&primary.challenge, &primary.manifest.context, &clock_at(NOW))?;
    let verdict = admit_made(&mut store, primary, NOW)?;
    assert_eq!(
        verdict,
        Verdict::Admitted,
        "the dump is of an admitted package, got {verdict:?}"
    );
    store.checkpoint()?;

    let schema = store.schema_sql()?;
    let tables = schema.matches("CREATE TABLE").count();
    assert_eq!(
        tables, 3,
        "the file has challenges, markers and admissions, got:\n{schema}"
    );
    let lowered = schema.to_ascii_lowercase();
    for word in ["ip", "package", "proof", "position", "value", "nullifier"] {
        assert!(!lowered.contains(word), "the schema must not mention {word}:\n{schema}");
    }
    assert_no_leaked_integers(&store, primary)?;

    let bytes = file_bytes(store.path())?;
    require_absent(&bytes, &primary.challenge, "the raw challenge")?;
    require_absent(
        &bytes,
        hex::encode(primary.challenge).as_bytes(),
        "the challenge in hex",
    )?;
    require_absent(&bytes, primary.proof_slice.as_bytes(), "a slice of the proof body")?;
    require_absent(&bytes, primary.session_pubkey.as_bytes(), "the session pubkey")?;
    for value in &primary.values {
        let text = value.to_string();
        if text.len() >= 6 {
            require_absent(&bytes, text.as_bytes(), "a note value")?;
        }
    }
    for position in &primary.positions {
        let text = position.to_string();
        if text.len() >= 6 {
            require_absent(&bytes, text.as_bytes(), "a note position")?;
        }
    }
    if primary.proof_slice.len() < 32 {
        return Err("the proof slice was too short to be a meaningful canary".into());
    }
    Ok(())
}

fn assert_no_leaked_integers(store: &Store, made: &Made) -> TestResult {
    let forbidden: Vec<i64> = made
        .positions
        .iter()
        .copied()
        .chain(made.values.iter().copied())
        .filter_map(|value| i64::try_from(value).ok())
        .collect();
    let mut stored = Vec::new();
    for row in store.challenges()? {
        stored.push(row.issued_at);
        stored.push(row.expires_at);
        if let Some(consumed) = row.consumed_at {
            stored.push(consumed);
        }
    }
    for row in store.markers()? {
        stored.push(row.admitted_at);
    }
    for row in store.admissions()? {
        stored.push(row.at);
    }
    for value in stored {
        if forbidden.contains(&value) {
            return Err(format!("a stored integer equals a note position or value ({value})").into());
        }
    }
    Ok(())
}
