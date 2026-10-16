#![doc = "Binding the holder's spending key to the account the wallet recorded, before anything \
is signed. Two checks stand between an age identity and a signature: the seed's ZIP 32 \
fingerprint must be the one this account was derived from, and the full viewing key the spending \
key produces must be the account's own Ironwood viewing key. A mismatch stops the prover; it \
never tries another derivation path."]

use std::path::Path;

use voting_crypto_deps::orchard::keys::{FullViewingKey, SpendAuthorizingKey, SpendingKey};
use voting_crypto_deps::orchard::primitives::redpallas::{SigningKey, SpendAuth};
use voting_crypto_deps::pasta_curves::pallas;
use zcash_client_backend::data_api::{Account as _, WalletRead, Zip32Derivation};
use zcash_client_sqlite::AccountUuid;
use zcash_protocol::consensus;

use crate::error::ProverError;
use crate::identity::{ironwood_spending_key, wallet_seed};
use crate::wallet::ReadOnlyWallet;

/// The authority to sign for one account's Ironwood notes. Holds the spending key, so it is kept
/// for as long as one proof takes and no longer, and its [`std::fmt::Debug`] says nothing.
pub struct SpendAuthority {
    spend_key: SpendingKey,
    viewing_key: FullViewingKey,
}

impl std::fmt::Debug for SpendAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpendAuthority(<secret>)")
    }
}

impl SpendAuthority {
    /// The account's Ironwood viewing key, which the bundle builder needs and which reveals
    /// nothing a gate could not already see in a proof.
    pub fn viewing_key(&self) -> &FullViewingKey {
        &self.viewing_key
    }

    /// The signing key for one bundle, randomized by that bundle's `alpha`. This is the key whose
    /// public half the circuit publishes as `rk`, so a signature under it is what separates the
    /// holder from someone who only has the viewing key.
    pub fn randomized_signer(&self, alpha: &pallas::Scalar) -> SigningKey<SpendAuth> {
        SpendAuthorizingKey::from(&self.spend_key).randomize(alpha)
    }
}

impl<P: consensus::Parameters> ReadOnlyWallet<P> {
    /// Decrypts the wallet's mnemonic with `identity_file` and derives the spending authority for
    /// `account`, checking it against what the wallet already knows about that account.
    pub fn ironwood_spend_authority(
        &self,
        account: AccountUuid,
        identity_file: &Path,
    ) -> Result<SpendAuthority, ProverError> {
        let derivation = self.account_derivation(account)?;
        let wallet_dir = self
            .path()
            .parent()
            .ok_or_else(|| ProverError::WalletDirectoryUnknown(self.path().to_path_buf()))?;
        let seed = wallet_seed(wallet_dir, identity_file)?;
        if &seed.fingerprint()? != derivation.seed_fingerprint() {
            return Err(ProverError::SeedIsForAnotherWallet);
        }
        let spend_key = ironwood_spending_key(&seed, self.db.params(), derivation.account_index())?;
        self.spend_authority_for(account, spend_key)
    }

    /// Keeps a spending key only if it is this account's: the full viewing key it produces must
    /// be the Ironwood viewing key the wallet already records for the account. Ironwood accounts
    /// may use a key path this version does not derive, and a mismatch stops here rather than
    /// sending the prover looking for another one.
    pub fn spend_authority_for(
        &self,
        account: AccountUuid,
        spend_key: SpendingKey,
    ) -> Result<SpendAuthority, ProverError> {
        let viewing_key = FullViewingKey::from(&spend_key);
        if viewing_key != self.ironwood_viewing_key(account)? {
            return Err(ProverError::SpendingKeyIsNotThisAccounts);
        }
        Ok(SpendAuthority { spend_key, viewing_key })
    }

    /// The ZIP 32 path the wallet recorded for this account. An account imported from a viewing
    /// key has none, and no spending key can be derived for it.
    pub fn account_derivation(&self, account: AccountUuid) -> Result<Zip32Derivation, ProverError> {
        let stored = self
            .db
            .get_account(account)?
            .ok_or_else(|| ProverError::AccountUnknown(account.expose_uuid().to_string()))?;
        stored
            .source()
            .key_derivation()
            .cloned()
            .ok_or(ProverError::AccountIsViewOnly)
    }
}
