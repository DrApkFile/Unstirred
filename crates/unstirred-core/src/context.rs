#![doc = "A gate's `round_id`, derived from its network and its context and from nothing else. No \
height goes in, which is what lets a gate republish a window without invalidating the markers it \
already holds, and what stops two gates' marker sets from ever matching (SPEC.md section 4)."]

use voting_circuits::ff::FromUniformBytes;
use voting_crypto_deps::pasta_curves::pallas;

use crate::manifest::Network;

const CONTEXT_PERSONALIZATION: &[u8; 16] = b"Unstirred_ctx_v1";

/// The field element that identifies one gate. The same `(network, context)` always gives the
/// same id; a different context or network always gives a different one; no height can change it
/// (A29).
///
/// ```
/// use unstirred_core::{derive_round_id, Network};
///
/// let holders = derive_round_id(Network::Test, "holders-room");
/// assert_eq!(holders, derive_round_id(Network::Test, "holders-room"));
/// assert_ne!(holders, derive_round_id(Network::Test, "second-room"));
/// assert_ne!(holders, derive_round_id(Network::Main, "holders-room"));
/// ```
pub fn derive_round_id(network: Network, context: &str) -> pallas::Base {
    let digest = blake2b_simd::Params::new()
        .hash_length(64)
        .personal(CONTEXT_PERSONALIZATION)
        .to_state()
        .update(&[network.domain_byte(), 0x00])
        .update(context.as_bytes())
        .finalize();
    let mut wide = [0u8; 64];
    wide.copy_from_slice(digest.as_bytes());
    pallas::Base::from_uniform_bytes(&wide)
}
