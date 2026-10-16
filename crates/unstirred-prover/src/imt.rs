#![doc = "Moving one returned non-membership proof into the shape the delegation circuit takes, \
and checking it out of circuit before anything is proved. The two types hold the same four \
fields; the copy exists because the PIR client and the circuit are separate crates."]

use pir_client::ImtProofData as ReturnedProof;
use voting_circuits::delegation::ImtProofData;
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::Fp;

/// Copies a returned proof field for field into the circuit's IMT proof type.
pub fn circuit_proof(returned: &ReturnedProof) -> ImtProofData {
    ImtProofData {
        root: returned.root,
        nf_bounds: returned.nf_bounds,
        leaf_pos: returned.leaf_pos,
        path: returned.path,
    }
}

/// Checks out of circuit that `proof` places `nullifier` strictly inside the punctured interval
/// its leaf covers, and that the path hashes to the proof's root. This is the same arithmetic the
/// circuit constrains, run on the converted data, so a lookup that does not actually exclude the
/// nullifier is caught before any proving starts.
pub fn proves_absence(proof: &ImtProofData, nullifier: Fp) -> bool {
    reference_proof(proof).verify(nullifier)
}

/// The root the proof authenticates to, as the hex the PIR server and `nf-server verify-root`
/// both print.
pub fn root_bytes(proof: &ImtProofData) -> [u8; 32] {
    proof.root.to_repr()
}

/// The same four fields again, in the reference tree's type, which is the one that can walk a
/// path out of circuit.
fn reference_proof(proof: &ImtProofData) -> ReturnedProof {
    ReturnedProof {
        root: proof.root,
        nf_bounds: proof.nf_bounds,
        leaf_pos: proof.leaf_pos,
        path: proof.path,
    }
}
