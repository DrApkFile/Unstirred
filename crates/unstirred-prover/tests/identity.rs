//! Getting the spending key out of an age-encrypted mnemonic, and refusing every key that is not
//! the account's. **Synthetic:** the mnemonic is the published all-zero-entropy BIP 39 test
//! vector, the age identity is generated here, and the wallet is the one `unstirred_prover::testing` builds.
//! No real wallet, seed or identity is touched.

use std::error::Error;
use std::path::Path;

use age::secrecy::ExposeSecret;
use unstirred_prover::testing::{NoteA, SyntheticWallet, TestResult};
use unstirred_prover::{ironwood_spending_key, wallet_seed, ProverError, ReadOnlyWallet, WALLET_KEYS_FILE};
use voting_crypto_deps::orchard::keys::SpendingKey;
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_protocol::consensus::Network;

/// BIP 39 test vector: twelve `abandon`s and `about`, the phrase for all-zero entropy.
const VECTOR_PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

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

/// The synthetic wallet on disk, with the spending key the test wallet was built with.
struct HeldWallet {
    directory: tempfile::TempDir,
    wallet: ReadOnlyWallet<zcash_protocol::local_consensus::LocalNetwork>,
    account: zcash_client_sqlite::AccountUuid,
    spend_key: SpendingKey,
}

impl HeldWallet {
    fn build() -> Result<Self, Box<dyn Error>> {
        let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
        let directory = tempfile::tempdir()?;
        synthetic.write_to(directory.path())?;
        let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network())?;
        Ok(Self {
            directory,
            wallet,
            account: synthetic.account()?,
            spend_key: synthetic.ironwood_spending_key()?,
        })
    }
}

/// Writes a `keys.toml` holding `phrase` encrypted to a fresh age identity, and that identity's
/// file next to it, the way a zcash-devtool wallet directory is laid out.
fn write_wallet_keys(directory: &Path, phrase: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public();
    let encryptor = age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))?;
    let mut armored = Vec::new();
    let mut writer = encryptor.wrap_output(age::armor::ArmoredWriter::wrap_output(
        &mut armored,
        age::armor::Format::AsciiArmor,
    )?)?;
    std::io::Write::write_all(&mut writer, phrase.as_bytes())?;
    writer.finish().and_then(|armor| armor.finish())?;
    let keys = format!(
        "mnemonic = \"\"\"\n{}\n\"\"\"\nnetwork = \"test\"\nbirthday = 1\n",
        String::from_utf8(armored)?.trim()
    );
    std::fs::write(directory.join(WALLET_KEYS_FILE), keys)?;
    write_identity(directory, "identity.age", &identity)
}

fn write_identity(
    directory: &Path,
    name: &str,
    identity: &age::x25519::Identity,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let path = directory.join(name);
    std::fs::write(&path, format!("{}\n", identity.to_string().expose_secret()))?;
    Ok(path)
}
