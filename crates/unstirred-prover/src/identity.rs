#![doc = "Getting the holder's Ironwood spending key into memory, and nowhere else. The mnemonic \
a zcash-devtool wallet keeps in `keys.toml` is age-encrypted; this reads it with the holder's age \
identity, turns it into a seed, and derives the account's spending key. Nothing here prints, \
serialises, logs or returns secret material: the seed and the mnemonic live in zeroizing buffers \
and the spending key is handed straight to the signer. Nothing here can write to the wallet."]

use std::path::Path;

use secrecy::{ExposeSecret, SecretVec, Zeroize};
use serde::Deserialize;
use voting_crypto_deps::orchard;
use zcash_client_backend::keys::UnifiedSpendingKey;
use zcash_protocol::consensus;

use crate::error::ProverError;

/// The file a zcash-devtool wallet directory keeps its encrypted mnemonic in.
pub const WALLET_KEYS_FILE: &str = "keys.toml";

/// The fields of `keys.toml` this crate reads. The network and birthday in that file are the
/// wallet's own business: the gate manifest names the network, and the prover refuses a mismatch
/// through the wallet's own consensus parameters.
#[derive(Debug, Deserialize)]
struct KeysFile {
    mnemonic: Option<String>,
}

/// The holder's seed, decrypted into a zeroizing buffer. Only [`ironwood_spending_key`] and the
/// seed fingerprint ever see it, and its [`std::fmt::Debug`] says nothing.
pub struct WalletSeed(SecretVec<u8>);

impl std::fmt::Debug for WalletSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WalletSeed(<secret>)")
    }
}

impl WalletSeed {
    /// The ZIP 32 seed fingerprint: the public identifier a wallet records next to every account
    /// it derived, and the one thing about a seed that is safe to compare out loud.
    pub fn fingerprint(&self) -> Result<zip32::fingerprint::SeedFingerprint, ProverError> {
        zip32::fingerprint::SeedFingerprint::from_seed(self.expose()).ok_or(ProverError::SeedNotUsableLength)
    }

    pub(crate) fn expose(&self) -> &[u8] {
        self.0.expose_secret()
    }
}

/// Reads `keys.toml` from a wallet directory and decrypts its mnemonic with the age identities in
/// `identity_file`, then stretches the mnemonic into a seed. The mnemonic buffer is zeroized as
/// soon as the seed exists.
pub fn wallet_seed(wallet_dir: &Path, identity_file: &Path) -> Result<WalletSeed, ProverError> {
    let keys_path = wallet_dir.join(WALLET_KEYS_FILE);
    let keys: KeysFile = toml::from_str(&read_file(&keys_path)?).map_err(|e| ProverError::KeysFileUnreadable {
        path: keys_path.clone(),
        detail: format!("{e}"),
    })?;
    let ciphertext = keys
        .mnemonic
        .ok_or_else(|| ProverError::WalletHasNoMnemonic(keys_path))?;
    let identities = read_identities(identity_file)?;
    let mnemonic = decrypt_mnemonic(&ciphertext, &identities)?;
    seed_from_mnemonic(&mnemonic)
}

/// Derives the account's Ironwood spending key. Ironwood reuses Orchard's keys, so this is the
/// Orchard component of the ZIP 32 account key at `account_index`.
pub fn ironwood_spending_key<P: consensus::Parameters>(
    seed: &WalletSeed,
    params: &P,
    account_index: zip32::AccountId,
) -> Result<orchard::keys::SpendingKey, ProverError> {
    let unified = UnifiedSpendingKey::from_seed(params, seed.expose(), account_index)
        .map_err(|e| ProverError::SpendingKeyUnderivable(format!("{e}")))?;
    Ok(*unified.orchard())
}

type Identities = Vec<Box<dyn age::Identity + Send + Sync>>;

fn read_identities(identity_file: &Path) -> Result<Identities, ProverError> {
    let unusable = |detail: String| ProverError::IdentityFileUnusable {
        path: identity_file.to_path_buf(),
        detail,
    };
    age::IdentityFile::from_file(identity_file.display().to_string())
        .map_err(|e| unusable(format!("{e}")))?
        .into_identities()
        .map_err(|e| unusable(format!("{e}")))
}

/// Decrypts the armored age file `keys.toml` holds. The plaintext is taken into a zeroizing
/// buffer before the read result is inspected, so a partial read is still zeroized.
fn decrypt_mnemonic(ciphertext: &str, identities: &Identities) -> Result<SecretVec<u8>, ProverError> {
    let decryptor = age::Decryptor::new(age::armor::ArmoredReader::new(ciphertext.as_bytes()))
        .map_err(|e| ProverError::MnemonicUndecryptable(format!("{e}")))?;
    let mut reader = decryptor
        .decrypt(
            identities
                .iter()
                .map(|identity| identity.as_ref() as &dyn age::Identity),
        )
        .map_err(|e| ProverError::MnemonicUndecryptable(format!("{e}")))?;
    let mut plaintext = Vec::new();
    let read = std::io::Read::read_to_end(&mut reader, &mut plaintext);
    let mnemonic = SecretVec::new(plaintext);
    read.map_err(|e| ProverError::MnemonicUndecryptable(format!("{e}")))?;
    Ok(mnemonic)
}

fn seed_from_mnemonic(mnemonic: &SecretVec<u8>) -> Result<WalletSeed, ProverError> {
    let phrase = std::str::from_utf8(mnemonic.expose_secret())
        .map_err(|_| ProverError::MnemonicNotAPhrase("the decrypted mnemonic is not UTF-8".to_owned()))?;
    let parsed = <bip0039::Mnemonic<bip0039::English>>::from_phrase(phrase)
        .map_err(|e| ProverError::MnemonicNotAPhrase(format!("{e}")))?;
    let mut bytes = parsed.to_seed("");
    let seed = SecretVec::new(bytes.to_vec());
    bytes.zeroize();
    Ok(WalletSeed(seed))
}

fn read_file(path: &Path) -> Result<String, ProverError> {
    std::fs::read_to_string(path).map_err(|e| ProverError::KeysFileUnreadable {
        path: path.to_path_buf(),
        detail: format!("{e}"),
    })
}
