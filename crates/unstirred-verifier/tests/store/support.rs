//! Shared clock, snapshots and package tampering for the store tests.

use std::error::Error;
use std::path::{Path, PathBuf};

use unstirred_core::{Code, ProofPackage};
use unstirred_verifier::store::{
    AdmissionRow, AdmissionWrite, ChallengeRow, Clock, CommitOutcome, MarkerRow, Store, StoreError,
};
use unstirred_verifier::Verdict;

pub const NOW: i64 = 1_700_000_000;

pub type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub i64);

impl Clock for FixedClock {
    fn unix_seconds(&self) -> i64 {
        self.0
    }
}

pub fn clock_at(now: i64) -> FixedClock {
    FixedClock(now)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub challenges: Vec<ChallengeRow>,
    pub markers: Vec<MarkerRow>,
    pub admissions: Vec<AdmissionRow>,
}

pub fn snapshot(store: &Store) -> Result<Snapshot, StoreError> {
    Ok(Snapshot {
        challenges: store.challenges()?,
        markers: store.markers()?,
        admissions: store.admissions()?,
    })
}

pub fn open_gate() -> Result<(tempfile::TempDir, Store), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let store = Store::open(directory.path().join("gate.sqlite"))?;
    Ok((directory, store))
}

pub fn distinct_markers(first: u8) -> [[u8; 32]; 5] {
    let mut markers = [[0u8; 32]; 5];
    let mut byte = first;
    for marker in &mut markers {
        *marker = [byte; 32];
        byte = byte.wrapping_add(1);
    }
    markers
}

pub fn admission(context: &str, challenge: [u8; 32], markers: [[u8; 32]; 5], now: i64) -> AdmissionWrite {
    AdmissionWrite {
        context: context.to_owned(),
        challenge,
        markers,
        now,
    }
}

pub fn expect_admitted(outcome: CommitOutcome) -> TestResult {
    match outcome {
        CommitOutcome::Admitted => Ok(()),
        CommitOutcome::Refused(refusal) => Err(format!("the store refused with {refusal}").into()),
    }
}

pub fn expect_store_code(outcome: CommitOutcome, code: Code) -> TestResult {
    match outcome {
        CommitOutcome::Refused(refusal) if refusal.code == code => Ok(()),
        CommitOutcome::Refused(refusal) => {
            Err(format!("expected {code}, the store refused with {}", refusal.code).into())
        }
        CommitOutcome::Admitted => Err(format!("expected {code}, the store admitted").into()),
    }
}

pub fn expect_code(verdict: Verdict, code: Code) -> TestResult {
    match verdict {
        Verdict::Refused(refusal) if refusal.code == code => Ok(()),
        Verdict::Refused(refusal) => Err(format!("expected {code}, got {} ({})", refusal.code, refusal.detail).into()),
        other => Err(format!("expected a refusal with {code}, got {other:?}").into()),
    }
}

/// Flips one proof byte. The package still parses, so a later refusal is the check under test.
pub fn tamper_proof(package_json: &str) -> Result<String, Box<dyn Error>> {
    let mut package = ProofPackage::parse(package_json)?;
    let mut bytes = package.proof_bytes()?;
    let index = bytes.len() / 2;
    let byte = bytes
        .get_mut(index)
        .ok_or("the proof was empty, so it could not be tampered")?;
    *byte ^= 0x01;
    package.proof = hex::encode(bytes);
    Ok(package.to_json()?)
}

pub fn file_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = std::fs::read(path)?;
    let wal = wal_path(path);
    if wal.is_file() {
        bytes.extend(std::fs::read(&wal)?);
    }
    Ok(bytes)
}

fn wal_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push("-wal");
    PathBuf::from(name)
}

pub fn require_absent(haystack: &[u8], needle: &[u8], label: &str) -> TestResult {
    if needle.is_empty() {
        return Err(format!("{label} was empty, so its absence proves nothing").into());
    }
    if haystack.windows(needle.len()).any(|window| window == needle) {
        return Err(format!("{label} is present in the gate database").into());
    }
    Ok(())
}
