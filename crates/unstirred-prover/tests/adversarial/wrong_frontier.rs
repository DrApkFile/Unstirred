//! A frontier that is the right shape and height but not the wallet's own tree at h0. The
//! witness rebuild has to reject it instead of producing a path that hashes to a root nobody
//! pinned. **Synthetic:** the wallet is the one `unstirred_prover::testing` builds.

use incrementalmerkletree::frontier::{Frontier, NonEmptyFrontier};
use unstirred_prover::testing::{manifest, NoteA, SyntheticWallet, TestResult, H0, H1};
use unstirred_prover::{eligible_notes, witnesses_at, IronwoodTreeState, ProverError, ReadOnlyWallet, Witnesses};
use voting_crypto_deps::orchard::tree::MerkleHashOrchard;
use zcash_client_sqlite::error::SqliteClientError;

#[test]
fn a20_a_frontier_that_is_not_the_wallets_tree_at_h0_is_rejected() -> TestResult {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let dir = tempfile::tempdir()?;
    synthetic.write_to(dir.path())?;
    let wallet = ReadOnlyWallet::open_with_params(dir.path(), synthetic.network())?;
    let selected = eligible_notes(&wallet, synthetic.account()?, &manifest(H0, H1)?)?.selection;
    let honest = synthetic.tree_state_at(H0)?;

    let forged = tampered_leaf(&honest)?;
    assert_ne!(forged.root_bytes(), honest.root_bytes());

    let refused: Result<Witnesses, ProverError> = witnesses_at(&wallet, &forged, &selected.notes);

    assert!(
        matches!(
            refused,
            Err(ProverError::WalletUnreadable(ref error))
                if matches!(**error, SqliteClientError::HistoricalFrontierInvalid(_))
        ),
        "a forged frontier must be rejected as inconsistent with the wallet's own tree: {refused:?}"
    );
    Ok(())
}

/// The same frontier with a different leaf: the shape and height are right, the contents are not.
fn tampered_leaf(honest: &IronwoodTreeState) -> Result<IronwoodTreeState, Box<dyn std::error::Error>> {
    let real = honest.frontier()?;
    let (position, leaf, ommers) = real.into_parts();
    let forged_leaf = MerkleHashOrchard::from_bytes(&[1u8; 32])
        .into_option()
        .ok_or("the test's stand-in leaf is not a canonical field element")?;
    assert_ne!(forged_leaf.to_bytes(), leaf.to_bytes());
    let forged = NonEmptyFrontier::from_parts(position, forged_leaf, ommers)
        .map_err(|e| format!("the test's forged frontier is malformed: {e:?}"))?;
    let tree = Frontier::try_from(forged).map_err(|e| format!("the test's forged frontier is too deep: {e:?}"))?;
    Ok(IronwoodTreeState::from_parts(honest.height(), tree))
}
