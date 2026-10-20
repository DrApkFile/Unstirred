#![doc = "Making one proof, for whichever command asked. `prove` writes the package to a file and \
`knock` posts it to a gate, and both of them come through here, so a holder's wallet is read, a \
window is checked and a key is derived in exactly one way."]

use std::path::PathBuf;

use unstirred_core::Manifest;
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, prove, root_from_hex, witnesses_at, OsRng, PirNullifierOracle,
    ProofRequest, Proven, ProverError, ReadOnlyWallet,
};

use crate::args::{optional, required, Flags};

/// Everything a proof needs except the challenge, which the gate chooses.
#[derive(Debug)]
pub struct ProofInputs {
    pub wallet: PathBuf,
    pub lightwalletd: String,
    pub pir: String,
    pub identity: PathBuf,
    pub expected_h1_root: Option<String>,
    pub account: Option<String>,
}

/// The flags every proving command shares.
pub const PROOF_FLAGS: [&str; 6] = [
    "--wallet",
    "--lightwalletd",
    "--pir",
    "--identity",
    "--expected-h1-root",
    "--account",
];

impl ProofInputs {
    pub fn from_flags(flags: &Flags) -> Result<Self, String> {
        Ok(Self {
            wallet: PathBuf::from(required(flags, "--wallet")?),
            lightwalletd: required(flags, "--lightwalletd")?.to_owned(),
            pir: required(flags, "--pir")?.to_owned(),
            identity: PathBuf::from(required(flags, "--identity")?),
            expected_h1_root: optional(flags, "--expected-h1-root").map(str::to_owned),
            account: optional(flags, "--account").map(str::to_owned),
        })
    }
}

/// One proof, plus the wallet it came from. The wallet is opened read-only and no transaction is
/// created; the spending key lives in memory inside this call and is never written or printed.
#[derive(Debug)]
pub struct Made {
    pub wallet_path: PathBuf,
    pub proven: Proven,
}

pub fn make_proof(inputs: &ProofInputs, manifest: &Manifest, challenge: [u8; 32]) -> Result<Made, ProverError> {
    let expected_root = match inputs.expected_h1_root.as_deref() {
        Some(hex_root) => Some(root_from_hex(hex_root)?),
        None => None,
    };
    let wallet = ReadOnlyWallet::open(&inputs.wallet, manifest.network)?;
    let account = wallet.account(inputs.account.as_deref())?;
    let selected = eligible_notes(&wallet, account, manifest)?.selection;
    let tree_state = fetch_ironwood_tree_state(&inputs.lightwalletd, manifest.network, manifest.window.h0)?;
    let witnesses = witnesses_at(&wallet, &tree_state, &selected.notes)?;
    let nullifier_set = PirNullifierOracle::connect(&inputs.pir, manifest.network, manifest.window.h1, expected_root)?;
    let authority = wallet.ironwood_spend_authority(account, &inputs.identity)?;
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
        &mut OsRng,
    )?;
    Ok(Made {
        wallet_path: wallet.path().to_path_buf(),
        proven,
    })
}
