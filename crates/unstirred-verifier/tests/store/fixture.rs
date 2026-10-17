//! Three real K=12 proofs over one synthetic wallet. **Synthetic:** the notes, the tree and the
//! nullifier set are built in this process. The proofs are real. One corpus is shared because
//! each proof costs seconds, and the store tests are checks on those packages.

use std::sync::OnceLock;

use unstirred_core::Manifest;
use unstirred_prover::testing::{NoteA, SyntheticNullifierSet, SyntheticWallet, CHAIN_TIP, H0, H1};
use unstirred_prover::{eligible_notes, prove, witnesses_at, OsRng, ProofRequest, ReadOnlyWallet};
use unstirred_verifier::PinnedRoots;
use voting_circuits::ff::PrimeField;

pub const CHALLENGE_A: [u8; 32] = [0x11; 32];
pub const CHALLENGE_B: [u8; 32] = [0x22; 32];
pub const CHALLENGE_C: [u8; 32] = [0x33; 32];

/// One proved package and the facts a leak test has to show are absent from the database.
pub struct Made {
    pub manifest: Manifest,
    pub package_json: String,
    pub roots: PinnedRoots,
    pub tip: u32,
    pub challenge: [u8; 32],
    pub markers: [[u8; 32]; 5],
    pub positions: Vec<u64>,
    pub values: Vec<u64>,
    pub session_pubkey: String,
    pub proof_slice: String,
}

pub struct Corpus {
    pub primary: Made,
    pub same_coins: Made,
    pub other_room: Made,
}

pub fn corpus() -> Result<&'static Corpus, String> {
    static CELL: OnceLock<Result<Corpus, String>> = OnceLock::new();
    CELL.get_or_init(build).as_ref().map_err(String::clone)
}

fn build() -> Result<Corpus, String> {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut).map_err(|e| e.to_string())?;
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    synthetic.write_to(directory.path()).map_err(|e| e.to_string())?;
    let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network()).map_err(|e| e.to_string())?;
    let account = synthetic.account().map_err(|e| e.to_string())?;
    let spend_key = synthetic.ironwood_spending_key().map_err(|e| e.to_string())?;
    let authority = wallet
        .spend_authority_for(account, spend_key)
        .map_err(|e| e.to_string())?;
    let holders = manifest_for("holders-room")?;
    let selected = eligible_notes(&wallet, account, &holders)
        .map_err(|e| e.to_string())?
        .selection;
    let witnesses = witnesses_at(
        &wallet,
        &synthetic.tree_state_at(H0).map_err(|e| e.to_string())?,
        &selected.notes,
    )
    .map_err(|e| e.to_string())?;
    let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
    let values: Vec<u64> = selected.notes.iter().map(|note| note.value_zatoshis).collect();
    let nullifier_set = SyntheticNullifierSet::with_spent(&[]);
    let mut rng = OsRng;
    let second = manifest_for("second-room")?;

    let mut prove_one = |manifest: &Manifest, challenge: [u8; 32]| -> Result<Made, String> {
        let proven = prove(
            &wallet,
            account,
            &authority,
            &ProofRequest {
                manifest,
                witnesses: &witnesses,
                expected_nf_imt_root: nullifier_set.root_bytes(),
                challenge,
            },
            &nullifier_set,
            &mut rng,
        )
        .map_err(|e| e.to_string())?;
        let instance = proven.package.instance().map_err(|e| e.to_string())?;
        let package_json = proven.package.to_json().map_err(|e| e.to_string())?;
        let proof = &proven.package.proof;
        let start = proof.len() / 2;
        Ok(Made {
            manifest: manifest.clone(),
            package_json,
            roots: PinnedRoots {
                nc_root_at_h0: proven.nc_root,
                nf_imt_root_at_h1: proven.nf_imt_root,
            },
            tip: CHAIN_TIP,
            challenge,
            markers: instance.gov_null.map(|marker| marker.to_repr()),
            positions: positions.clone(),
            values: values.clone(),
            session_pubkey: proven.package.session_pubkey.clone(),
            proof_slice: proof.chars().skip(start).take(64).collect(),
        })
    };

    Ok(Corpus {
        primary: prove_one(&holders, CHALLENGE_A)?,
        same_coins: prove_one(&holders, CHALLENGE_B)?,
        other_room: prove_one(&second, CHALLENGE_C)?,
    })
}

fn manifest_for(context: &str) -> Result<Manifest, String> {
    let json = format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"{context}","window":{{"h0":{H0},"h1":{H1}}},"min_zatoshis":"12500000"}}"#
    );
    unstirred_core::parse_manifest(&json).map_err(|e| e.to_string())
}
