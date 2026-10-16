use unstirred_core::{check_window, Code, Manifest, Refusal};
use zcash_client_sqlite::AccountUuid;
use zcash_protocol::consensus;

use crate::error::ProverError;
use crate::selection::{select_held_notes, Selection, WindowNotes};
use crate::wallet::ReadOnlyWallet;

/// What the wallet holds across one gate's window, and the fewest of those notes that reach the
/// gate's threshold.
#[derive(Debug)]
pub struct EligibleNotes {
    pub account: AccountUuid,
    pub chain_tip: u32,
    pub fully_scanned: u32,
    pub min_zatoshis: u64,
    pub notes: WindowNotes,
    pub selection: Selection,
}

/// Reads the wallet and applies the prover's rules in the only order that makes them meaningful:
/// a wallet that has not scanned through h1 cannot say which notes stayed unmoved, so the sync
/// checks come before the note rules. Nothing here writes to the wallet or builds a transaction.
pub fn eligible_notes<P: consensus::Parameters>(
    wallet: &ReadOnlyWallet<P>,
    account: AccountUuid,
    manifest: &Manifest,
) -> Result<EligibleNotes, ProverError> {
    let chain_tip = wallet.chain_tip()?;
    check_window(manifest.window, chain_tip)?;
    let fully_scanned = wallet.fully_scanned_height()?;
    if fully_scanned < manifest.window.h1 {
        return Err(Refusal::new(
            Code::WalletNotSyncedPastH1,
            format!(
                "the wallet is fully scanned through {fully_scanned}, h1 is {}",
                manifest.window.h1
            ),
        )
        .into());
    }
    let min_zatoshis = required_zatoshis(manifest)?;
    let notes = wallet.window_notes(account, manifest.window)?;
    let selection = select_held_notes(&notes, min_zatoshis)?;
    Ok(EligibleNotes {
        account,
        chain_tip,
        fully_scanned,
        min_zatoshis,
        notes,
        selection,
    })
}

fn required_zatoshis(manifest: &Manifest) -> Result<u64, Refusal> {
    manifest.min_zatoshis.parse().map_err(|_| {
        Refusal::new(
            Code::ManifestInvalid,
            format!(
                "min_zatoshis {:?} is not a whole number of zatoshis",
                manifest.min_zatoshis
            ),
        )
    })
}
