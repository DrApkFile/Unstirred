//! Getting the spending key out of an age-encrypted mnemonic. **Synthetic:** the mnemonic is
//! the published all-zero-entropy BIP 39 test vector, the age identity is generated here, and
//! the wallet is the one `unstirred_prover::testing` builds. No real wallet, seed or identity
//! is touched. Every key that is not this account's is refused in `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod wrong_identity;
}

mod identity {
    pub mod keys_file;
}

use identity::keys_file::{write_identity, write_wallet_keys, VECTOR_PHRASE};
use unstirred_prover::testing::TestResult;
use unstirred_prover::{ironwood_spending_key, wallet_seed, ProverError, WALLET_KEYS_FILE};
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_protocol::consensus::Network;

/// The seed that phrase stretches to with an empty passphrase, from the same published vector.
const VECTOR_SEED: &str = "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4";

#[test]
fn a24_the_age_encrypted_mnemonic_yields_the_wallets_seed_and_its_ironwood_spending_key() -> TestResult {
    let directory = tempfile::tempdir()?;
    let identity = write_wallet_keys(directory.path(), VECTOR_PHRASE)?;

    let seed = wallet_seed(directory.path(), &identity)?;

    let vector = hex::decode(VECTOR_SEED)?;
    assert_eq!(
        seed.fingerprint()?,
        zip32::fingerprint::SeedFingerprint::from_seed(&vector)
            .ok_or("the published vector seed must have a ZIP 32 fingerprint")?,
        "the decrypted seed must be the one the published vector stretches to"
    );
    let derived = ironwood_spending_key(&seed, &Network::TestNetwork, zip32::AccountId::ZERO)?;
    let expected = UnifiedSpendingKey::from_seed(&Network::TestNetwork, &vector, zip32::AccountId::ZERO)?;
    assert_eq!(
        derived.to_bytes(),
        expected.orchard().to_bytes(),
        "the Ironwood spending key must be the Orchard component of the ZIP 32 account key"
    );
    Ok(())
}

#[test]
fn a24_a_wallet_that_keeps_no_mnemonic_cannot_sign_anything() -> TestResult {
    let directory = tempfile::tempdir()?;
    let identity = write_identity(directory.path(), "identity.age", &age::x25519::Identity::generate())?;
    std::fs::write(
        directory.path().join(WALLET_KEYS_FILE),
        "network = \"test\"\nbirthday = 1\n",
    )?;

    let refused = wallet_seed(directory.path(), &identity);

    assert!(
        matches!(refused, Err(ProverError::WalletHasNoMnemonic(_))),
        "{refused:?}"
    );
    Ok(())
}
