#![doc = "Applying one gate's rule to one wallet: whether it has scanned far enough to answer at \
all, which notes it held unmoved across the window, and the fewest of them that reach the \
threshold. Nothing here writes to the wallet or touches the network."]

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
    /// The account that was read.
    pub account: AccountUuid,
    /// The chain tip the wallet reports, which the window's finality check is against.
    pub chain_tip: u32,
    /// The height the wallet has fully scanned to. A wallet short of h1 cannot say what stayed
    /// unmoved, so this is checked before any note rule.
    pub fully_scanned: u32,
    /// The gate's threshold in zatoshis, from its manifest.
    pub min_zatoshis: u64,
    /// What the wallet held at each end of the window.
    pub notes: WindowNotes,
    /// The notes a proof would use, fewest first.
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

/// The gate's threshold as a number. `parse_manifest` has already accepted the text, so this is
/// a refusal only for a `Manifest` that was built without it.
/// The gate's threshold as a number. It is a decimal string in the manifest, and a value that is
/// not a whole number of zatoshis is `MANIFEST_INVALID` rather than a rounded guess.
///
/// ```
/// use unstirred_core::{parse_manifest, REQUIRED_MIN_ZATOSHIS};
/// use unstirred_prover::required_zatoshis;
///
/// let manifest = parse_manifest(r#"{"version":"unstirred/v1","network":"test",
///     "context":"holders-room","window":{"h0":4505790,"h1":4506510},
///     "min_zatoshis":"12500000"}"#)?;
///
/// assert_eq!(required_zatoshis(&manifest)?, 12_500_000);
/// assert_eq!(manifest.min_zatoshis, REQUIRED_MIN_ZATOSHIS);
/// # Ok::<(), unstirred_core::Refusal>(())
/// ```
pub fn required_zatoshis(manifest: &Manifest) -> Result<u64, Refusal> {
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
