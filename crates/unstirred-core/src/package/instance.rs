#![doc = "The delegation circuit's public instance as the package carries it: fourteen field elements \
in hex, and the one decode that turns them back into the instance a verifier checks."]

use serde::{Deserialize, Serialize};
use voting_circuits::delegation::Instance;
use voting_circuits::ff::{Field, PrimeField};
use voting_crypto_deps::orchard::note::Nullifier;
use voting_crypto_deps::pasta_curves::pallas;

use super::{bytes_from_hex, malformed};
use crate::binding::rk_from_coordinates;
use crate::codes::Refusal;

const FIELD_BYTES: usize = 32;

/// The delegation circuit's fourteen public inputs, each as its 32-byte canonical representation
/// in hex. `rk_y` is carried in full, but only its parity is read: [`PublicInstance::decode`]
/// rebuilds `rk` from `rk_x` plus that parity, so the instance a verifier checks is always the
/// one those two values imply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicInstance {
    /// The signed nullifier the circuit exposes.
    pub nf_signed: String,
    /// The x coordinate of the randomized verification key `rk`.
    pub rk_x: String,
    /// The y coordinate of `rk`. Carried whole, but only its parity is read.
    pub rk_y: String,
    /// The new note commitment the circuit produces.
    pub cmx_new: String,
    /// The value commitment, blinded freshly per proof, so two proofs of the same balance do not
    /// commit to it the same way.
    pub van_comm: String,
    /// The gate's `round_id`, derived from its network and context and from no height. This is
    /// what the verifier compares against its own manifest to catch a proof made for elsewhere.
    pub vote_round_id: String,
    /// The note-commitment root the proof was made against — pinned at `h0`.
    pub nc_root: String,
    /// The nullifier-set IMT root the proof was made against — pinned at `h1`.
    pub nf_imt_root: String,
    /// The five markers. Exactly one per real note repeats across two proofs of the same coins,
    /// which is the collision single use rests on; the rest are the circuit's padding nullifiers.
    pub gov_null: [String; 5],
    /// The circuit's domain separator.
    pub dom: String,
}

impl PublicInstance {
    /// The hex form of a circuit instance, for putting in a package.
    pub fn of(instance: &Instance) -> Self {
        Self {
            nf_signed: hex::encode(instance.nf_signed.to_bytes()),
            rk_x: field_hex(instance.rk_x),
            rk_y: field_hex(instance.rk_y),
            cmx_new: field_hex(instance.cmx_new),
            van_comm: field_hex(instance.van_comm),
            vote_round_id: field_hex(instance.vote_round_id),
            nc_root: field_hex(instance.nc_root),
            nf_imt_root: field_hex(instance.nf_imt_root),
            gov_null: instance.gov_null.map(field_hex),
            dom: field_hex(instance.dom),
        }
    }

    /// Back to a circuit instance, rebuilding `rk` from `rk_x` and `rk_y`'s parity. Any value
    /// that is not a canonical field element is `PROOF_INVALID`.
    pub fn decode(&self) -> Result<Instance, Refusal> {
        let nf_signed = Option::from(Nullifier::from_bytes(&bytes_from_hex::<FIELD_BYTES>(
            "nf_signed",
            &self.nf_signed,
        )?))
        .ok_or_else(|| malformed("nf_signed", "not a canonical field element"))?;
        let rk = rk_from_coordinates(field_from_hex("rk_x", &self.rk_x)?, field_from_hex("rk_y", &self.rk_y)?)?;
        let mut gov_null = [pallas::Base::ZERO; 5];
        for (marker, text) in gov_null.iter_mut().zip(self.gov_null.iter()) {
            *marker = field_from_hex("gov_null", text)?;
        }
        Instance::from_parts(
            nf_signed,
            rk,
            field_from_hex("cmx_new", &self.cmx_new)?,
            field_from_hex("van_comm", &self.van_comm)?,
            field_from_hex("vote_round_id", &self.vote_round_id)?,
            field_from_hex("nc_root", &self.nc_root)?,
            field_from_hex("nf_imt_root", &self.nf_imt_root)?,
            gov_null,
            field_from_hex("dom", &self.dom)?,
        )
        .map_err(|e| malformed("rk_x", &format!("not usable as a public input: {e}")))
    }
}

fn field_hex(value: pallas::Base) -> String {
    hex::encode(value.to_repr())
}

fn field_from_hex(name: &str, text: &str) -> Result<pallas::Base, Refusal> {
    let bytes = bytes_from_hex::<FIELD_BYTES>(name, text)?;
    Option::from(pallas::Base::from_repr(bytes)).ok_or_else(|| malformed(name, "not a canonical field element"))
}
