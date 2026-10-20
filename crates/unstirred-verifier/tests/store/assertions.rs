//! What the store tests assert about one admission: the admit call itself, the marker sets a
//! package carries, and whether a challenge is still unconsumed in the file.

use std::collections::BTreeSet;
use std::error::Error;

use unstirred_verifier::store::Store;
use unstirred_verifier::{admit, Verdict};

use super::fixture::Made;
use super::support::{clock_at, TestResult};

pub fn admit_made(store: &mut Store, made: &Made, now: i64) -> Result<Verdict, Box<dyn Error>> {
    Ok(admit(
        &made.manifest,
        &made.package_json,
        &made.roots,
        made.tip,
        store,
        &clock_at(now),
    )?)
}

pub fn shared_markers(left: &[[u8; 32]; 5], right: &[[u8; 32]; 5]) -> BTreeSet<[u8; 32]> {
    let right = marker_set(right);
    marker_set(left).intersection(&right).copied().collect()
}

pub fn marker_set(markers: &[[u8; 32]; 5]) -> BTreeSet<[u8; 32]> {
    markers.iter().copied().collect()
}

pub fn assert_distinct(markers: &[[u8; 32]; 5]) -> TestResult {
    if marker_set(markers).len() == markers.len() {
        Ok(())
    } else {
        Err("one proof produced a repeated marker, so a unique conflict would not mean already used".into())
    }
}

pub fn assert_unconsumed(store: &Store, challenge: &[u8; 32], context: &str) -> TestResult {
    match store.lookup(challenge, context)? {
        Some(row) if row.consumed_at.is_none() => Ok(()),
        Some(row) => Err(format!("the challenge was consumed at {:?}", row.consumed_at).into()),
        None => Err("the challenge is not stored for this context".into()),
    }
}
