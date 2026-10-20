//! Keys and identities that are not this account's: a stranger's age identity against the
//! wallet's `keys.toml`, a mnemonic for another seed, and a spending key the account was not
//! derived from. Each one has to be refused before anything is signed. **Synthetic:** see
//! `identity/keys_file.rs`.

use unstirred_prover::testing::TestResult;
use unstirred_prover::{wallet_seed, ProverError};
use voting_crypto_deps::orchard::keys::SpendingKey;

use crate::identity::keys_file::{write_identity, write_wallet_keys, HeldWallet, VECTOR_PHRASE};

#[test]
fn a24_another_identity_cannot_decrypt_the_wallets_mnemonic() -> TestResult {
    let directory = tempfile::tempdir()?;
    write_wallet_keys(directory.path(), VECTOR_PHRASE)?;
    let stranger = write_identity(directory.path(), "stranger.age", &age::x25519::Identity::generate())?;

    let refused = wallet_seed(directory.path(), &stranger);

    assert!(
        matches!(refused, Err(ProverError::MnemonicUndecryptable(_))),
        "{refused:?}"
    );
    Ok(())
}

#[test]
fn a23_a_seed_the_account_was_not_derived_from_is_refused_before_anything_is_signed() -> TestResult {
    let held = HeldWallet::build()?;
    let identity = write_wallet_keys(held.directory.path(), VECTOR_PHRASE)?;

    let refused = held.wallet.ironwood_spend_authority(held.account, &identity);

    assert!(
        matches!(refused, Err(ProverError::SeedIsForAnotherWallet)),
        "a mnemonic for another seed must be refused at the fingerprint, before any key is derived: {refused:?}"
    );
    Ok(())
}

#[test]
fn a23_a_spending_key_that_is_not_the_accounts_is_refused() -> TestResult {
    let held = HeldWallet::build()?;
    let stranger = SpendingKey::from_bytes([0x2a; 32])
        .into_option()
        .ok_or("that byte string must be a usable spending key")?;

    let refused = held.wallet.spend_authority_for(held.account, stranger);

    assert!(
        matches!(refused, Err(ProverError::SpendingKeyIsNotThisAccounts)),
        "{refused:?}"
    );
    held.wallet
        .spend_authority_for(held.account, held.spend_key)
        .map_err(|e| format!("the account's own spending key must be accepted: {e}"))?;
    Ok(())
}
