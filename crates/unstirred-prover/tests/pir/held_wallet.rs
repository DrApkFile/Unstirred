//! A synthetic wallet on disk, opened the way the CLI opens the holder's wallet, and the
//! nullifiers of the notes a gate's window selects from it.

use unstirred_prover::testing::{manifest, NoteA, SyntheticWallet, H0, H1};
use unstirred_prover::{eligible_notes, NoteNullifier, ReadOnlyWallet};

/// A synthetic wallet on disk, opened the way the CLI opens the holder's wallet.
pub struct HeldWallet {
    _directory: tempfile::TempDir,
    wallet: ReadOnlyWallet<zcash_protocol::local_consensus::LocalNetwork>,
    account: zcash_client_sqlite::AccountUuid,
}

impl HeldWallet {
    pub fn build(note_a: NoteA) -> Result<Self, Box<dyn std::error::Error>> {
        let synthetic = SyntheticWallet::build(note_a)?;
        let directory = tempfile::tempdir()?;
        synthetic.write_to(directory.path())?;
        let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network())?;
        let account = synthetic.account()?;
        Ok(Self {
            _directory: directory,
            wallet,
            account,
        })
    }

    pub fn nullifiers_of_selected_notes(&self) -> Result<Vec<NoteNullifier>, Box<dyn std::error::Error>> {
        let selected = eligible_notes(&self.wallet, self.account, &manifest(H0, H1)?)?.selection;
        let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
        Ok(self.wallet.ironwood_nullifiers(self.account, H0, &positions)?)
    }

    /// Notes are named by value here because that is what the testing wallet fixes; the nullifier is
    /// derived from the wallet's own viewing key, as the prover does it.
    pub fn nullifier_of_note_worth(&self, value_zatoshis: u64) -> Result<NoteNullifier, Box<dyn std::error::Error>> {
        let notes = self
            .wallet
            .window_notes(self.account, manifest(H0, H1)?.window)?
            .unspent_at_h0;
        let note = notes
            .iter()
            .find(|note| note.value_zatoshis == value_zatoshis)
            .ok_or_else(|| format!("the wallet holds no note worth {value_zatoshis} zatoshis at h0"))?;
        first(&self.wallet.ironwood_nullifiers(self.account, H0, &[note.position])?)
    }
}

pub fn first(nullifiers: &[NoteNullifier]) -> Result<NoteNullifier, Box<dyn std::error::Error>> {
    nullifiers
        .first()
        .copied()
        .ok_or_else(|| "no nullifier was derived".into())
}
