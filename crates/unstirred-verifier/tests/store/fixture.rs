//! Four real K=12 proofs over one synthetic wallet. **Synthetic:** the notes, the tree and the
//! nullifier set are built in this process. The proofs are real. One corpus is shared because
//! each proof costs seconds, and the store tests are checks on those packages.

use std::sync::OnceLock;

use unstirred_core::{Manifest, Window};
use unstirred_prover::testing::{NoteA, SyntheticNullifierSet, SyntheticWallet, CHAIN_TIP, H0, H1, NOTE_A_HEIGHT};
use unstirred_prover::{eligible_notes, prove, witnesses_at, HeldNote, OsRng, ProofRequest, ReadOnlyWallet, Witnesses};
use unstirred_verifier::PinnedRoots;
use voting_circuits::ff::PrimeField;

pub const CHALLENGE_A: [u8; 32] = [0x11; 32];
pub const CHALLENGE_B: [u8; 32] = [0x22; 32];
pub const CHALLENGE_C: [u8; 32] = [0x33; 32];
pub const CHALLENGE_D: [u8; 32] = [0x44; 32];

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
    /// A newcomer's forced submission: a real proof of a note that did not exist at the room's own
    /// h0, built against the later root it does appear in, which is what
    /// `knock --force-unqualified-demo` builds (A03, verifier half).
    pub newcomer: Made,
}

/// What one package is about. `pinned_nc_root` is the root the gate pins at its own h0: the
/// proof's own root for an honest package, an earlier one for the newcomer's forced proof.
struct About<'a> {
    manifest: &'a Manifest,
    challenge: [u8; 32],
    witnesses: &'a Witnesses,
    notes_unspent_at: u32,
    pinned_nc_root: [u8; 32],
    notes: &'a [HeldNote],
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
    let tree_at_h0 = synthetic.tree_state_at(H0).map_err(|e| e.to_string())?;
    let holders = manifest_over("holders-room", H0, H1)?;
    let selected = eligible_notes(&wallet, account, &holders)
        .map_err(|e| e.to_string())?
        .selection;
    let witnesses = witnesses_at(&wallet, &tree_at_h0, &selected.notes).map_err(|e| e.to_string())?;
    let nullifier_set = SyntheticNullifierSet::with_spent(&[]);
    let mut rng = OsRng;
    let second = manifest_over("second-room", H0, H1)?;

    let mut prove_one = |about: About<'_>| -> Result<Made, String> {
        let proven = prove(
            &wallet,
            account,
            &authority,
            &ProofRequest {
                manifest: about.manifest,
                witnesses: about.witnesses,
                notes_unspent_at: about.notes_unspent_at,
                expected_nf_imt_root: nullifier_set.root_bytes(),
                challenge: about.challenge,
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
            manifest: about.manifest.clone(),
            package_json,
            roots: PinnedRoots {
                nc_root_at_h0: about.pinned_nc_root,
                nf_imt_root_at_h1: proven.nf_imt_root,
            },
            tip: CHAIN_TIP,
            challenge: about.challenge,
            markers: instance.gov_null.map(|marker| marker.to_repr()),
            positions: about.notes.iter().map(|note| note.position).collect(),
            values: about.notes.iter().map(|note| note.value_zatoshis).collect(),
            session_pubkey: proven.package.session_pubkey.clone(),
            proof_slice: proof.chars().skip(start).take(64).collect(),
        })
    };

    let newcomer_room = manifest_over("holders-room", NOTE_A_HEIGHT, H1)?;
    let newcomer_notes = notes_mined_after(
        wallet
            .window_notes(account, Window { h0: H0, h1: H1 })
            .map_err(|e| e.to_string())?
            .held_through_h1,
        NOTE_A_HEIGHT,
    )?;
    let newcomer_witnesses = witnesses_at(&wallet, &tree_at_h0, &newcomer_notes).map_err(|e| e.to_string())?;
    let root_before_those_coins = synthetic
        .tree_state_at(NOTE_A_HEIGHT)
        .map_err(|e| e.to_string())?
        .root_bytes();

    Ok(Corpus {
        primary: prove_one(honest(&holders, CHALLENGE_A, &witnesses, &selected.notes))?,
        same_coins: prove_one(honest(&holders, CHALLENGE_B, &witnesses, &selected.notes))?,
        other_room: prove_one(honest(&second, CHALLENGE_C, &witnesses, &selected.notes))?,
        newcomer: prove_one(About {
            manifest: &newcomer_room,
            challenge: CHALLENGE_D,
            witnesses: &newcomer_witnesses,
            notes_unspent_at: newcomer_witnesses.height,
            pinned_nc_root: root_before_those_coins,
            notes: &newcomer_notes,
        })?,
    })
}

/// An honest package: the proof's own root is the one the gate pinned at h0, and the wallet still
/// showed the notes unspent at h1.
fn honest<'a>(
    manifest: &'a Manifest,
    challenge: [u8; 32],
    witnesses: &'a Witnesses,
    notes: &'a [HeldNote],
) -> About<'a> {
    About {
        manifest,
        challenge,
        witnesses,
        notes_unspent_at: H1,
        pinned_nc_root: witnesses.root,
        notes,
    }
}

/// The notes that did not exist at `h0` yet — the newcomer's coins. They are held at `H0`, which
/// is where their witnesses come from, and nowhere in the room's own window.
fn notes_mined_after(held: Vec<HeldNote>, h0: u32) -> Result<Vec<HeldNote>, String> {
    let newer: Vec<HeldNote> = held.into_iter().filter(|note| note.mined_height > h0).collect();
    if newer.is_empty() {
        return Err(format!("the synthetic wallet holds no note mined after {h0}"));
    }
    Ok(newer)
}

fn manifest_over(context: &str, h0: u32, h1: u32) -> Result<Manifest, String> {
    let json = format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"{context}","window":{{"h0":{h0},"h1":{h1}}},"min_zatoshis":"12500000"}}"#
    );
    unstirred_core::parse_manifest(&json).map_err(|e| e.to_string())
}
