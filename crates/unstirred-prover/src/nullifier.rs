#![doc = "The real Ironwood nullifier of a held note, derived from the wallet's own viewing key. \
A nullifier is the one value in this crate that must never reach a server in the clear: the \
non-membership lookup at h1 is private (PRD section 4), so nothing here prints or serialises one."]

use std::fmt;

use voting_crypto_deps::orchard;
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::Fp;
use zcash_client_backend::data_api::{Account as _, WalletRead};
use zcash_client_backend::keys::UnifiedFullViewingKey;
use zcash_client_backend::wallet::ReceivedNote;
use zcash_client_sqlite::{AccountUuid, ReceivedNoteId};
use zcash_protocol::consensus::{self, BlockHeight};

use crate::error::ProverError;
use crate::wallet::ReadOnlyWallet;

/// One held note's nullifier, kept next to the tree position that names the note. The tree
/// position is public; the nullifier is not, so [`fmt::Debug`] shows the position only.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NoteNullifier {
    position: u64,
    value: Fp,
}

impl NoteNullifier {
    pub fn position(&self) -> u64 {
        self.position
    }

    /// The field element the private lookup queries for. Every caller inside this crate hands it
    /// straight to the PIR client, which only ever sends it encrypted.
    pub fn value(&self) -> Fp {
        self.value
    }
}

impl fmt::Debug for NoteNullifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoteNullifier")
            .field("position", &self.position)
            .finish_non_exhaustive()
    }
}

impl<P: consensus::Parameters> ReadOnlyWallet<P> {
    /// Derives the nullifier of each note named by tree position, from the account's Ironwood
    /// viewing key and the notes the wallet recorded as unspent at `height`. Deriving a nullifier
    /// needs no spending key and writes nothing.
    pub fn ironwood_nullifiers(
        &self,
        account: AccountUuid,
        height: u32,
        positions: &[u64],
    ) -> Result<Vec<NoteNullifier>, ProverError> {
        let viewing_key = self.ironwood_viewing_key(account)?;
        let notes = self
            .db
            .get_unspent_ironwood_notes_at_historical_height(account, BlockHeight::from_u32(height))?;
        positions
            .iter()
            .map(|position| nullifier_at(&viewing_key, &notes, *position))
            .collect()
    }

    /// Ironwood reuses Orchard's receivers and keys, so the account's Orchard full viewing key is
    /// what derives an Ironwood nullifier.
    fn ironwood_viewing_key(&self, account: AccountUuid) -> Result<orchard::keys::FullViewingKey, ProverError> {
        let stored = self
            .db
            .get_account(account)?
            .ok_or_else(|| ProverError::AccountUnknown(account.expose_uuid().to_string()))?;
        stored
            .ufvk()
            .and_then(UnifiedFullViewingKey::orchard)
            .cloned()
            .ok_or(ProverError::AccountWithoutIronwoodViewingKey)
    }
}

fn nullifier_at(
    viewing_key: &orchard::keys::FullViewingKey,
    notes: &[ReceivedNote<ReceivedNoteId, orchard::note::Note>],
    position: u64,
) -> Result<NoteNullifier, ProverError> {
    let note = notes
        .iter()
        .find(|note| u64::from(note.note_commitment_tree_position()) == position)
        .ok_or(ProverError::NoteNotHeldAtThatHeight(position))?;
    let bytes = note.note().nullifier(viewing_key).to_bytes();
    let value = Option::from(Fp::from_repr(bytes)).ok_or(ProverError::NullifierNotCanonical(position))?;
    Ok(NoteNullifier { position, value })
}
