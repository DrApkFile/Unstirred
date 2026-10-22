#![doc = "The demo's forced submission, and nothing else. PRD section 9's act 4 is a holder whose \
coins are newer than the gate's h0: the prover refuses them COINS_TOO_NEW, and the point of the act \
is that forcing the attempt anyway is refused by the gate itself, on the roots, not on their word.\n\n\
So this builds a real proof — real notes, real witnesses, a real private nullifier lookup, a real \
binding signature — against the latest note-commitment root those coins actually appear in, which is \
not the root the gate pinned at h0. Nothing in the package is faked. The gate answers ROOT_MISMATCH \
and writes nothing (A03).\n\n\
It cannot be used by accident: the flag is in no usage text, and this refuses to run at all for a \
wallet the room would admit, where a knock without the flag proves the thing honestly."]

use unstirred_core::{Manifest, Refusal, Window};
use unstirred_prover::{eligible_notes, required_zatoshis, select_held_notes, ProverError};

use super::{open, Prepared, ProofInputs};

/// Deliberately absent from `knock`'s usage text and from the command list: a holder who types
/// what the gate tells them will never reach this, and the demo script names it in full.
pub const DEMO_FLAG: &str = "--force-unqualified-demo";

/// A wallet the room refuses, prepared to prove what it really can prove instead.
#[derive(Debug)]
pub struct ForcedDemo {
    /// What the room's own rule answered this wallet. The forced path exists only for a wallet
    /// the room refuses, so there is always one.
    pub refused_by_the_room: Refusal,
    pub prepared: Prepared,
}

/// Why a forced demo is not available. `TheRoomWouldAdmitThis` is the guard that keeps the flag
/// from being used as a shortcut: there is nothing to demonstrate about a holder who qualifies.
#[derive(Debug)]
pub enum NotForcible {
    TheRoomWouldAdmitThis,
    Local(ProverError),
}

/// Reads the wallet twice: once under the room's rule, which must refuse, and once at the latest
/// height the wallet has scanned, which is where its coins do exist. The second reading is what
/// the proof is built from.
pub fn prepare_forced_demo(inputs: &ProofInputs, manifest: &Manifest) -> Result<ForcedDemo, NotForcible> {
    let (wallet, account, expected_h1_root) = open(inputs, manifest).map_err(NotForcible::Local)?;
    let refused_by_the_room = forcible(eligible_notes(&wallet, account, manifest).map(|_| ()))?;
    let latest = wallet.fully_scanned_height().map_err(NotForcible::Local)?;
    let threshold = required_zatoshis(manifest).map_err(refused)?;
    let held = wallet
        .window_notes(account, Window { h0: latest, h1: latest })
        .map_err(NotForcible::Local)?;
    let selection = select_held_notes(&held, threshold).map_err(refused)?;
    Ok(ForcedDemo {
        refused_by_the_room,
        prepared: Prepared {
            wallet,
            account,
            expected_h1_root,
            selection,
            nc_root_height: latest,
            notes_unspent_at: latest,
        },
    })
}

/// A wallet that cannot reach the threshold even at the latest root has no real package to force,
/// and the demo never invents one.
fn refused(refusal: Refusal) -> NotForcible {
    NotForcible::Local(refusal.into())
}

/// The guard, as one decision over what the room's own rule answered: only a wallet it refused can
/// be forced. A wallet it would admit is not forced through the demo path, because a knock without
/// the flag proves the same thing honestly and there would be nothing to demonstrate.
fn forcible(by_the_rooms_rule: Result<(), ProverError>) -> Result<Refusal, NotForcible> {
    match by_the_rooms_rule {
        Ok(()) => Err(NotForcible::TheRoomWouldAdmitThis),
        Err(ProverError::Refused(refusal)) => Ok(refusal),
        Err(local) => Err(NotForcible::Local(local)),
    }
}

#[cfg(test)]
mod tests {
    use unstirred_core::{Code, Refusal};
    use unstirred_prover::ProverError;

    use super::{forcible, NotForcible};

    /// A46, the guard: the forced submission exists only for a wallet this room refuses. A wallet
    /// it would admit is not forced, and a fault that is not a refusal stays a fault.
    #[test]
    fn a46_the_forced_demo_runs_only_for_a_wallet_the_room_refuses() {
        let refused = forcible(Err(Refusal::new(Code::CoinsTooNew, "no note at h0").into()));
        assert!(
            matches!(&refused, Ok(refusal) if refusal.code == Code::CoinsTooNew),
            "a wallet the room refused is the one the demo is for, got {refused:?}"
        );

        let would_be_admitted = forcible(Ok(()));
        assert!(
            matches!(would_be_admitted, Err(NotForcible::TheRoomWouldAdmitThis)),
            "a wallet the room would admit must not be forced, got {would_be_admitted:?}"
        );

        let faulty = forcible(Err(ProverError::NoAccounts));
        assert!(
            matches!(faulty, Err(NotForcible::Local(ProverError::NoAccounts))),
            "a fault that is not a refusal stays a local fault, got {faulty:?}"
        );
    }
}
