//! A zcash-devtool wallet directory as these tests write it: a `keys.toml` holding an
//! age-encrypted mnemonic, the identity file next to it, and the synthetic wallet the
//! spending-key checks read. **Synthetic:** the mnemonic is the published all-zero-entropy
//! BIP 39 test vector and every identity is generated here. No real wallet, seed or identity
//! is touched.

use std::error::Error;
use std::path::Path;

use age::secrecy::ExposeSecret;
use unstirred_prover::testing::{NoteA, SyntheticWallet};
use unstirred_prover::{ReadOnlyWallet, WALLET_KEYS_FILE};
use voting_crypto_deps::orchard::keys::SpendingKey;

/// BIP 39 test vector: twelve `abandon`s and `about`, the phrase for all-zero entropy.
pub const VECTOR_PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The synthetic wallet on disk, with the spending key the test wallet was built with.
pub struct HeldWallet {
    pub directory: tempfile::TempDir,
    pub wallet: ReadOnlyWallet<zcash_protocol::local_consensus::LocalNetwork>,
    pub account: zcash_client_sqlite::AccountUuid,
    pub spend_key: SpendingKey,
}

impl HeldWallet {
    pub fn build() -> Result<Self, Box<dyn Error>> {
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
pub fn write_wallet_keys(directory: &Path, phrase: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
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

pub fn write_identity(
    directory: &Path,
    name: &str,
    identity: &age::x25519::Identity,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let path = directory.join(name);
    std::fs::write(&path, format!("{}\n", identity.to_string().expose_secret()))?;
    Ok(path)
}
