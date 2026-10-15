use voting_circuits::ff::FromUniformBytes;
use voting_crypto_deps::pasta_curves::pallas;

use crate::manifest::Network;

const CONTEXT_PERSONALIZATION: &[u8; 16] = b"Unstirred_ctx_v1";

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
