//! The synthetic nullifier set: the delegation circuit's own reference Indexed Merkle Tree.

use voting_circuits::delegation::{ImtError, ImtProofData, ImtProvider, SpacedLeafImtProvider};
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::{pallas, Fp};

use crate::pir::NullifierSetAtH1;
use crate::ProverError;

/// The delegation circuit's own reference Indexed Merkle Tree, wrapped so the prover can ask it
/// what it would otherwise ask a real `nf-server`. **Synthetic:** this tree is built in this
/// process from the sentinel skeleton plus whatever nullifiers a test says are spent. It is not
/// chain data, and a root from it is not the root of any real snapshot.
#[derive(Debug)]
pub struct SyntheticNullifierSet(SpacedLeafImtProvider);

impl SyntheticNullifierSet {
    /// A set in which `spent` are the only nullifiers present.
    pub fn with_spent(spent: &[Fp]) -> Self {
        Self(SpacedLeafImtProvider::with_extra_nullifiers(spent))
    }

    /// The root a gate would have to pin for proofs against this set to mean anything.
    pub fn root_bytes(&self) -> [u8; 32] {
        self.0.root().to_repr()
    }
}

impl ImtProvider for SyntheticNullifierSet {
    fn root(&self) -> pallas::Base {
        self.0.root()
    }

    fn non_membership_proof(&self, nf: pallas::Base) -> Result<ImtProofData, ImtError> {
        self.0.non_membership_proof(nf)
    }
}

impl NullifierSetAtH1 for SyntheticNullifierSet {
    /// An in-memory tree is never unavailable, so a failed question here always means the set
    /// contains the nullifier.
    fn last_failure(&self) -> Option<ProverError> {
        None
    }
}
