//! The one proof this file's tests share. **Synthetic:** the wallet, its notes and the nullifier
//! set at h1 are built in this process; the proof is a real K=12 Halo2 proof. It is made once,
//! because a real proof costs seconds, and a failure is reported to every test that needed it.

use std::error::Error;
use std::sync::OnceLock;

use unstirred_prover::testing::{manifest, NoteA, SyntheticNullifierSet, SyntheticWallet, H0, H1};
use unstirred_prover::{eligible_notes, prove, witnesses_at, HeldNote, OsRng, ProofRequest, Proven, ReadOnlyWallet};
use voting_crypto_deps::orchard::keys::{FullViewingKey, Scope};
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;

/// Stands in for the gate's challenge, which the holder passes by hand until the gate issues one.
const CHALLENGE: [u8; 32] = [0x5c; 32];

/// One proof, made once for the whole file. A failure is reported to every test that needed it.
pub fn proved() -> Result<&'static Proved, String> {
    static PROVED: OnceLock<Result<Proved, String>> = OnceLock::new();

    PROVED
        .get_or_init(|| make_one_proof().map_err(|e| format!("{e}")))
        .as_ref()
        .map_err(String::clone)
}

/// A package, plus what the wallet knows about the note behind it, so the leak scan has
/// something to look for.
#[derive(Debug)]
pub struct Proved {
    pub proven: Proven,
    pub note: HeldNote,
    pub nullifier: [u8; 32],
    pub address: [u8; 43],
}

fn make_one_proof() -> Result<Proved, Box<dyn Error>> {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let directory = tempfile::tempdir()?;
    synthetic.write_to(directory.path())?;
    let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network())?;
    let account = synthetic.account()?;
    let spend_key = synthetic.ironwood_spending_key()?;
    let authority = wallet.spend_authority_for(account, spend_key)?;

    let manifest = manifest(H0, H1)?;
    let selected = eligible_notes(&wallet, account, &manifest)?.selection;
    let note = *selected
        .notes
        .first()
        .ok_or("the selection must hold at least one note")?;
    let witnesses = witnesses_at(&wallet, &synthetic.tree_state_at(H0)?, &selected.notes)?;
    let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
    let nullifiers = wallet.ironwood_nullifiers(account, H1, &positions)?;
    let nullifier = nullifiers
        .first()
        .ok_or("the wallet must derive a nullifier for the selected note")?
        .value()
        .to_repr();
    let nullifier_set = SyntheticNullifierSet::with_spent(&[]);

    let proven = prove(
        &wallet,
        account,
        &authority,
        &ProofRequest {
            manifest: &manifest,
            witnesses: &witnesses,
            notes_unspent_at: H1,
            expected_nf_imt_root: nullifier_set.root_bytes(),
            challenge: CHALLENGE,
        },
        &nullifier_set,
        &mut OsRng,
    )?;

    Ok(Proved {
        proven,
        note,
        nullifier,
        address: FullViewingKey::from(&spend_key)
            .address_at(0u32, Scope::External)
            .to_raw_address_bytes(),
    })
}
