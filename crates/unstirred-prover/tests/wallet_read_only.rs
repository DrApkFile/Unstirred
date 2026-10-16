use std::error::Error;
use std::fs;
use std::path::Path;

use rand::rngs::OsRng;
use unstirred_core::{Code, Network};
use unstirred_prover::{ProverError, ReadOnlyWallet, WALLET_DATABASE_FILE};
use zcash_client_sqlite::util::SystemClock;
use zcash_client_sqlite::wallet::init::WalletMigrator;
use zcash_client_sqlite::WalletDb;
use zcash_protocol::consensus::Network as ConsensusNetwork;

type TestResult = Result<(), Box<dyn Error>>;

fn create_wallet(database: &Path) -> TestResult {
    let mut db = WalletDb::for_path(database, ConsensusNetwork::TestNetwork, SystemClock, OsRng)?;
    WalletMigrator::new()
        .init_or_migrate(&mut db)
        .map_err(|e| format!("could not create a test wallet: {e}"))?;
    Ok(())
}

fn refusal_code(error: ProverError) -> Option<Code> {
    match error {
        ProverError::Refused(refusal) => Some(refusal.code),
        _ => None,
    }
}

#[test]
fn a16_reads_the_wallet_without_changing_a_byte_of_it() -> TestResult {
    let wallet_dir = tempfile::tempdir()?;
    let database = wallet_dir.path().join(WALLET_DATABASE_FILE);
    create_wallet(&database)?;
    let before = fs::read(&database)?;

    let wallet = ReadOnlyWallet::open(wallet_dir.path(), Network::Test)?;
    assert_eq!(wallet.accounts()?, vec![], "a fresh wallet has no accounts");
    drop(wallet);

    assert_eq!(fs::read(&database)?, before, "the wallet database changed");
    Ok(())
}

#[test]
fn a16_a_directory_without_a_wallet_database_is_refused_not_created() -> TestResult {
    let empty_dir = tempfile::tempdir()?;
    let opened = ReadOnlyWallet::open(empty_dir.path(), Network::Test);
    assert!(
        matches!(opened, Err(ProverError::WalletDatabaseMissing(_))),
        "{opened:?}"
    );
    assert_eq!(
        fs::read_dir(empty_dir.path())?.count(),
        0,
        "opening must not create a wallet"
    );
    Ok(())
}

#[test]
fn a18_a_wallet_that_knows_no_chain_tip_refuses_with_wallet_not_synced_past_h1() -> TestResult {
    let wallet_dir = tempfile::tempdir()?;
    create_wallet(&wallet_dir.path().join(WALLET_DATABASE_FILE))?;
    let wallet = ReadOnlyWallet::open(wallet_dir.path(), Network::Test)?;
    assert_eq!(
        wallet.chain_tip().err().and_then(refusal_code),
        Some(Code::WalletNotSyncedPastH1)
    );
    assert_eq!(
        wallet.fully_scanned_height().err().and_then(refusal_code),
        Some(Code::WalletNotSyncedPastH1)
    );
    Ok(())
}
