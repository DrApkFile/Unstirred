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
    pub nf_signed: String,
    pub rk_x: String,
    pub rk_y: String,
    pub cmx_new: String,
    pub van_comm: String,
    pub vote_round_id: String,
    pub nc_root: String,
    pub nf_imt_root: String,
    pub gov_null: [String; 5],
    pub dom: String,
}

impl PublicInstance {
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
