#![doc = "Making one proof, for whichever command asked. `prove` writes the package to a file and \
`knock` posts it to a gate, and both of them come through here, so a holder's wallet is read, a \
window is checked and a key is derived in exactly one way.\n\n\
It is two steps on purpose. [`prepare`] reads the wallet and applies the room's rule, touching no \
network and no gate; only then is a challenge worth asking for, so a holder the room cannot admit \
leaves nothing behind at the gate. [`Prepared::prove`] is the part that needs the challenge, the \
tree state and the nullifier set."]

mod demo;

use std::path::PathBuf;

use unstirred_core::Manifest;
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, prove, root_from_hex, witnesses_at, AccountUuid, HeldNote, OsRng,
    PirNullifierOracle, ProofRequest, Proven, ProverError, ReadOnlyWallet, Selection,
};

pub use demo::{prepare_forced_demo, ForcedDemo, NotForcible, DEMO_FLAG};

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

/// A wallet that has been read and found to have something to prove: which account, which notes,
/// and the height whose note-commitment root they will be witnessed against. Nothing here has
/// touched the network or the gate.
#[derive(Debug)]
pub struct Prepared {
    wallet: ReadOnlyWallet,
    account: AccountUuid,
    expected_h1_root: Option<[u8; 32]>,
    selection: Selection,
    nc_root_height: u32,
    notes_unspent_at: u32,
}

/// Reads the wallet and applies the room's own rule: the notes it held unmoved from h0 through h1,
/// and the fewest of them that reach the threshold. The witnesses will come from h0, which is what
/// the gate pins.
pub fn prepare(inputs: &ProofInputs, manifest: &Manifest) -> Result<Prepared, ProverError> {
    let (wallet, account, expected_h1_root) = open(inputs, manifest)?;
    let selection = eligible_notes(&wallet, account, manifest)?.selection;
    Ok(Prepared {
        wallet,
        account,
        expected_h1_root,
        selection,
        nc_root_height: manifest.window.h0,
        notes_unspent_at: manifest.window.h1,
    })
}

fn open(
    inputs: &ProofInputs,
    manifest: &Manifest,
) -> Result<(ReadOnlyWallet, AccountUuid, Option<[u8; 32]>), ProverError> {
    let expected_h1_root = match inputs.expected_h1_root.as_deref() {
        Some(hex_root) => Some(root_from_hex(hex_root)?),
        None => None,
    };
    let wallet = ReadOnlyWallet::open(&inputs.wallet, manifest.network)?;
    let account = wallet.account(inputs.account.as_deref())?;
    Ok((wallet, account, expected_h1_root))
}

impl Prepared {
    /// The notes this proof will be about, as the holder's own wallet knows them. They stay on
    /// this side: no value, height or position of theirs is in the package.
    pub fn notes(&self) -> &[HeldNote] {
        &self.selection.notes
    }

    /// The height whose note-commitment root this proof will be about. A gate admits a proof only
    /// against the root it pinned at its own h0, so anything else is refused ROOT_MISMATCH.
    pub fn nc_root_height(&self) -> u32 {
        self.nc_root_height
    }

    /// The height at which this wallet still shows the selected notes unspent.
    pub fn notes_unspent_at(&self) -> u32 {
        self.notes_unspent_at
    }

    /// The part that needs the gate's challenge: the tree state, the private nullifier lookup,
    /// the spending key and the proof. The wallet stays read-only and no transaction is created;
    /// the spending key lives in memory inside this call and is never written or printed.
    pub fn prove(&self, inputs: &ProofInputs, manifest: &Manifest, challenge: [u8; 32]) -> Result<Made, ProverError> {
        let tree_state = fetch_ironwood_tree_state(&inputs.lightwalletd, manifest.network, self.nc_root_height)?;
        let witnesses = witnesses_at(&self.wallet, &tree_state, &self.selection.notes)?;
        let nullifier_set =
            PirNullifierOracle::connect(&inputs.pir, manifest.network, manifest.window.h1, self.expected_h1_root)?;
        let authority = self.wallet.ironwood_spend_authority(self.account, &inputs.identity)?;
        let proven = prove(
            &self.wallet,
            self.account,
            &authority,
            &ProofRequest {
                manifest,
                witnesses: &witnesses,
                notes_unspent_at: self.notes_unspent_at,
                expected_nf_imt_root: nullifier_set.root_bytes(),
                challenge,
            },
            &nullifier_set,
            &mut OsRng,
        )?;
        Ok(Made {
            wallet_path: self.wallet.path().to_path_buf(),
            proven,
        })
    }
}

/// One proof, plus the wallet it came from.
#[derive(Debug)]
pub struct Made {
    pub wallet_path: PathBuf,
    pub proven: Proven,
}
