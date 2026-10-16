use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use unstirred_core::Network;
use zcash_protocol::consensus::Network as ConsensusNetwork;

use crate::error::ProverError;

/// The wallet database inside a zcash-devtool wallet directory.
pub const WALLET_DATABASE_FILE: &str = "data.sqlite";

const MAIN_DATABASE: &str = "main";

pub(crate) const fn consensus_network(network: Network) -> ConsensusNetwork {
    match network {
        Network::Test => ConsensusNetwork::TestNetwork,
        Network::Main => ConsensusNetwork::MainNetwork,
    }
}

/// Resolves a wallet directory, or the database file itself, to an existing database. A missing
/// database is an error: nothing here may bring one into existence.
pub(crate) fn database_path(wallet: &Path) -> Result<PathBuf, ProverError> {
    let candidate = if wallet.is_dir() {
        wallet.join(WALLET_DATABASE_FILE)
    } else {
        wallet.to_path_buf()
    };
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(ProverError::WalletDatabaseMissing(candidate))
    }
}

/// Asks SQLite for a read-only handle, tells the connection to reject writes, and then asks
/// SQLite whether the handle it returned really is read-only.
pub(crate) fn open_read_only(path: &Path) -> Result<Connection, ProverError> {
    let connection =
        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
            .map_err(|source| unopenable(path, source))?;
    connection
        .pragma_update(None, "query_only", true)
        .map_err(|source| unopenable(path, source))?;
    rusqlite::vtab::array::load_module(&connection).map_err(|source| unopenable(path, source))?;
    if connection
        .is_readonly(MAIN_DATABASE)
        .map_err(|source| unopenable(path, source))?
    {
        Ok(connection)
    } else {
        Err(ProverError::WalletNotReadOnly(path.to_path_buf()))
    }
}

fn unopenable(path: &Path, source: rusqlite::Error) -> ProverError {
    ProverError::WalletUnopenable {
        path: path.to_path_buf(),
        source,
    }
}
