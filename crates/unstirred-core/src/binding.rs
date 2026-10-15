use voting_circuits::delegation::Instance;
use voting_circuits::ff::PrimeField;
use voting_crypto_deps::orchard::primitives::redpallas::{self, SpendAuth};
use voting_crypto_deps::pasta_curves::pallas;

use crate::codes::{Code, Refusal};

const BINDING_DOMAIN: &[u8] = b"unstirred/v1/bind";
const SIGN_BIT: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingStatement {
    pub round_id: pallas::Base,
    pub nc_root: pallas::Base,
    pub nf_imt_root: pallas::Base,
    pub van_comm: pallas::Base,
    pub markers: [pallas::Base; 5],
}

impl BindingStatement {
    pub fn of(instance: &Instance) -> Self {
        Self {
            round_id: instance.vote_round_id,
            nc_root: instance.nc_root,
            nf_imt_root: instance.nf_imt_root,
            van_comm: instance.van_comm,
            markers: instance.gov_null,
        }
    }
}

pub fn binding_message(statement: &BindingStatement, session_pubkey: &[u8; 32], challenge: &[u8; 32]) -> Vec<u8> {
    let fields = [
        statement.round_id,
        statement.nc_root,
        statement.nf_imt_root,
        statement.van_comm,
    ];
    let mut message = Vec::with_capacity(BINDING_DOMAIN.len() + 32 * (fields.len() + statement.markers.len() + 2));
    message.extend_from_slice(BINDING_DOMAIN);
    for field in fields.iter().chain(statement.markers.iter()) {
        message.extend_from_slice(&field.to_repr());
    }
    message.extend_from_slice(session_pubkey);
    message.extend_from_slice(challenge);
    message
}

pub fn rk_from_coordinates(
    rk_x: pallas::Base,
    rk_y: pallas::Base,
) -> Result<redpallas::VerificationKey<SpendAuth>, Refusal> {
    let mut compressed = rk_x.to_repr();
    if bool::from(rk_y.is_odd()) {
        compressed[31] |= SIGN_BIT;
    }
    redpallas::VerificationKey::try_from(compressed).map_err(|_| {
        Refusal::new(
            Code::BadSignature,
            "rk in the proof instance is not a valid curve point",
        )
    })
}
