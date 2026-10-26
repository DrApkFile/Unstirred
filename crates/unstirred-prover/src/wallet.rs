#![doc = "Opening a zcash-devtool wallet read-only, and the few questions the prover asks it. \
The handle is checked to be read-only after opening, so a wallet this crate holds cannot be \
written to even by mistake."]

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use incrementalmerkletree::Position;
use rusqlite::Connection;
use unstirred_core::{Code, Network, Refusal, Window};
use voting_crypto_deps::orchard;
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use zcash_client_backend::data_api::WalletRead;
use zcash_client_sqlite::{AccountUuid, WalletDb};
use zcash_protocol::consensus::{self, BlockHeight, Network as ConsensusNetwork};

use crate::error::ProverError;
use crate::open::{consensus_network, database_path, open_read_only};
use crate::selection::{HeldNote, WindowNotes};
use crate::treestate::IronwoodTreeState;
use crate::witness::IronwoodMerklePath;

/// A wallet this process cannot write to: the value only exists once SQLite has confirmed the
/// handle is read-only. No method on it can create a transaction.
pub struct ReadOnlyWallet<P = ConsensusNetwork> {
    pub(crate) db: WalletDb<Connection, P, (), ()>,
    path: PathBuf,
}

impl<P> fmt::Debug for ReadOnlyWallet<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadOnlyWallet")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl ReadOnlyWallet<ConsensusNetwork> {
    /// Opens `wallet`, either a zcash-devtool wallet directory or the database file itself, under
    /// the network the gate manifest names.
    pub fn open(wallet: &Path, network: Network) -> Result<Self, ProverError> {
        Self::open_with_params(wallet, consensus_network(network))
    }
}

impl<P: consensus::Parameters> ReadOnlyWallet<P> {
    /// Opens a wallet under consensus parameters given directly, for networks a gate manifest
    /// cannot name, such as a local test network.
    pub fn open_with_params(wallet: &Path, params: P) -> Result<Self, ProverError> {
        let path = database_path(wallet)?;
        let connection = open_read_only(&path)?;
        Ok(Self {
            db: WalletDb::from_connection(connection, params, (), ()),
            path,
        })
    }

    /// The database file this handle is open on.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every account the wallet holds.
    pub fn accounts(&self) -> Result<Vec<AccountUuid>, ProverError> {
        Ok(self.db.get_account_ids()?)
    }

    /// Resolves the account to prove from: the one named, or the only one the wallet has.
    pub fn account(&self, requested: Option<&str>) -> Result<AccountUuid, ProverError> {
        let accounts = self.accounts()?;
        match requested {
            Some(uuid) => accounts
                .into_iter()
                .find(|account| account.expose_uuid().to_string() == uuid)
                .ok_or_else(|| ProverError::AccountUnknown(uuid.to_owned())),
            None => match accounts.as_slice() {
                [] => Err(ProverError::NoAccounts),
                [only] => Ok(*only),
                many => Err(ProverError::AccountAmbiguous { count: many.len() }),
            },
        }
    }

    /// The chain tip as the wallet last recorded it. This is the wallet's own view; the verifier
    /// takes heights and roots from its own sources instead.
    pub fn chain_tip(&self) -> Result<u32, ProverError> {
        let tip = self
            .db
            .chain_height()?
            .ok_or_else(|| Refusal::new(Code::WalletNotSyncedPastH1, "the wallet does not know a chain tip yet"))?;
        Ok(u32::from(tip))
    }

    /// The height through which every block has been scanned, so that spends below it are known.
    pub fn fully_scanned_height(&self) -> Result<u32, ProverError> {
        let scanned = self.db.block_fully_scanned()?.ok_or_else(|| {
            Refusal::new(
                Code::WalletNotSyncedPastH1,
                "the wallet has not fully scanned any block yet",
            )
        })?;
        Ok(u32::from(scanned.block_height()))
    }

    /// The account's Ironwood notes as the window sees them: those unspent at h0, and which of
    /// those were still unspent at h1.
    pub fn window_notes(&self, account: AccountUuid, window: Window) -> Result<WindowNotes, ProverError> {
        let unspent_at_h0 = self.unspent_ironwood_notes(account, window.h0)?;
        let unspent_at_h1: BTreeSet<u64> = self
            .unspent_ironwood_notes(account, window.h1)?
            .iter()
            .map(|note| note.position)
            .collect();
        let held_through_h1 = unspent_at_h0
            .iter()
            .copied()
            .filter(|note| unspent_at_h1.contains(&note.position))
            .collect();
        Ok(WindowNotes {
            unspent_at_h0,
            held_through_h1,
        })
    }

    /// Rebuilds the authentication path of each position against the tree at `tree_state`'s
    /// height, from the wallet's stored shards plus that height's frontier. This reads the wallet
    /// and builds the tree in memory; the database is not written to.
    pub fn ironwood_witnesses(
        &self,
        positions: &[Position],
        tree_state: &IronwoodTreeState,
    ) -> Result<Vec<IronwoodMerklePath>, ProverError> {
        Ok(self.db.generate_ironwood_witnesses_at_historical_height(
            positions,
            tree_state.frontier()?,
            BlockHeight::from_u32(tree_state.height()),
        )?)
    }

    fn unspent_ironwood_notes(&self, account: AccountUuid, height: u32) -> Result<Vec<HeldNote>, ProverError> {
        self.db
            .get_unspent_ironwood_notes_at_historical_height(account, BlockHeight::from_u32(height))?
            .into_iter()
            .map(|note| {
                let position = u64::from(note.note_commitment_tree_position());
                let mined = note
                    .mined_height()
                    .ok_or(ProverError::NoteWithoutMinedHeight(position))?;
                Ok(HeldNote {
                    value_zatoshis: note.note().value().inner(),
                    mined_height: u32::from(mined),
                    position,
                    commitment: leaf_digest(note.note()),
                })
            })
            .collect()
    }
}

/// The note commitment as it appears as a leaf of the Ironwood note commitment tree.
fn leaf_digest(note: &orchard::note::Note) -> [u8; 32] {
    let cmx = orchard::note::ExtractedNoteCommitment::from(note.commitment());
    MerkleHashOrchard::from_cmx(&cmx).to_bytes()
}
