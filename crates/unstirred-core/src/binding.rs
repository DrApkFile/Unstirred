#![doc = "The binding signature, and the message it is over. A valid Halo2 proof alone is not \
enough to enter a gate: anyone holding a viewing key could produce one. The holder also signs \
the statement, their session key and the gate's challenge under `rk`, which only the spending \
key can do (SPEC.md section 5)."]

use voting_circuits::delegation::Instance;
use voting_circuits::ff::PrimeField;
use voting_crypto_deps::orchard::primitives::redpallas::{self, SpendAuth};
use voting_crypto_deps::pasta_curves::pallas;

use crate::codes::{Code, Refusal};

const BINDING_DOMAIN: &[u8] = b"unstirred/v1/bind";
const SIGN_BIT: u8 = 0x80;

/// The part of a proof's instance that the holder signs: everything that makes this proof *this*
/// proof, for *this* gate. Change any of it and the message changes, so the signature no longer
/// stands (A31).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingStatement {
    /// The gate's `round_id`, derived from its network and context.
    pub round_id: pallas::Base,
    /// The note-commitment root pinned at `h0`.
    pub nc_root: pallas::Base,
    /// The nullifier-set root pinned at `h1`.
    pub nf_imt_root: pallas::Base,
    /// The value commitment for this proof.
    pub van_comm: pallas::Base,
    /// The five markers this proof would write.
    pub markers: [pallas::Base; 5],
}

impl BindingStatement {
    /// The signable part of a circuit instance.
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

/// The exact bytes a holder signs: a domain separator, the statement's fields in a fixed order,
/// the session key and the gate's challenge. Both sides build it with this one function, so the
/// signer and the checker cannot drift apart.
///
/// ```
/// use unstirred_core::{binding_message, BindingStatement};
/// use voting_circuits::ff::Field;
/// use voting_crypto_deps::pasta_curves::pallas;
///
/// let statement = BindingStatement {
///     round_id: pallas::Base::ZERO,
///     nc_root: pallas::Base::ONE,
///     nf_imt_root: pallas::Base::ZERO,
///     van_comm: pallas::Base::ZERO,
///     markers: [pallas::Base::ZERO; 5],
/// };
/// let message = binding_message(&statement, &[7u8; 32], &[9u8; 32]);
///
/// // A different challenge is a different message, so a signature cannot be replayed at
/// // another gate's challenge.
/// assert_ne!(message, binding_message(&statement, &[7u8; 32], &[10u8; 32]));
/// ```
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

/// Rebuilds the randomized verification key from the instance's own coordinates. The verifier
/// never takes an `rk` handed to it: it reconstructs the one the proof commits to, and checks the
/// signature under that. `rk_y` contributes only its parity.
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
