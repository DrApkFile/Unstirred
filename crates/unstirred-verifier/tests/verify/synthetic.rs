//! The synthetic package every test in `verify.rs` checks. **Synthetic:** the wallet, its notes
//! and the nullifier set are built in this process by `unstirred_prover::testing`; the proof
//! itself is a real K=12 Halo2 proof over Valar's unmodified delegation circuit. One proof is
//! made for the whole module, because a real proof costs seconds, and every test is a check on
//! that one package.

use std::sync::OnceLock;

use unstirred_core::Manifest;
use unstirred_prover::testing::{manifest, NoteA, SyntheticNullifierSet, SyntheticWallet, CHAIN_TIP, H0, H1};
use unstirred_prover::{eligible_notes, prove, witnesses_at, OsRng, ProofRequest, ReadOnlyWallet};
use unstirred_verifier::PinnedRoots;

/// Stands in for the challenge the gate issued; the verifier is handed it on the command line.
pub const CHALLENGE: [u8; 32] = [0x5c; 32];

/// Everything one verification needs: the manifest both as rules and as text, the package as
/// text, the pinned roots, the tip and the challenge.
pub struct Fixture {
    pub manifest: Manifest,
    pub manifest_json: String,
    pub package_json: String,
    pub roots: PinnedRoots,
    pub tip: u32,
    pub challenge: [u8; 32],
}

pub fn fixture() -> Result<&'static Fixture, String> {
    static FIXTURE: OnceLock<Result<Fixture, String>> = OnceLock::new();
    FIXTURE.get_or_init(make_fixture).as_ref().map_err(String::clone)
}

fn make_fixture() -> Result<Fixture, String> {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut).map_err(|e| format!("{e}"))?;
    let directory = tempfile::tempdir().map_err(|e| format!("{e}"))?;
    synthetic.write_to(directory.path()).map_err(|e| format!("{e}"))?;
    let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network()).map_err(|e| format!("{e}"))?;
    let account = synthetic.account().map_err(|e| format!("{e}"))?;
    let spend_key = synthetic.ironwood_spending_key().map_err(|e| format!("{e}"))?;
    let authority = wallet
        .spend_authority_for(account, spend_key)
        .map_err(|e| format!("{e}"))?;

    let manifest = manifest(H0, H1).map_err(|e| format!("{e}"))?;
    let selected = eligible_notes(&wallet, account, &manifest)
        .map_err(|e| format!("{e}"))?
        .selection;
    let tree_state = synthetic.tree_state_at(H0).map_err(|e| format!("{e}"))?;
    let witnesses = witnesses_at(&wallet, &tree_state, &selected.notes).map_err(|e| format!("{e}"))?;
    let nullifier_set = SyntheticNullifierSet::with_spent(&[]);

    let proven = prove(
        &wallet,
        account,
        &authority,
        &ProofRequest {
            manifest: &manifest,
            witnesses: &witnesses,
            expected_nf_imt_root: nullifier_set.root_bytes(),
            challenge: CHALLENGE,
        },
        &nullifier_set,
        &mut OsRng,
    )
    .map_err(|e| format!("{e}"))?;

    let manifest_json = serde_json::to_string(&manifest).map_err(|e| format!("{e}"))?;
    let package_json = proven.package.to_json().map_err(|e| format!("{e}"))?;
    Ok(Fixture {
        manifest,
        manifest_json,
        package_json,
        roots: PinnedRoots {
            nc_root_at_h0: proven.nc_root,
            nf_imt_root_at_h1: proven.nf_imt_root,
        },
        tip: CHAIN_TIP,
        challenge: CHALLENGE,
    })
}
