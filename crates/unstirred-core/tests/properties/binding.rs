//! A31: the binding message covers everything it claims to bind, so no field the gate relies
//! on can be changed without changing what the holder signed.

use rand::Rng;
use unstirred_core::binding_message;
use voting_crypto_deps::pasta_curves::pallas;

use super::generate::random_statement;
use super::seeded::{over_seeds, CASES};

/// A31: the binding message covers everything it claims to bind. Changing any statement field,
/// the session key or the challenge changes the message, and its length never moves.
#[test]
fn a31_the_binding_message_changes_with_every_field_it_binds() {
    let failures = over_seeds(CASES, |rng| {
        let statement = random_statement(rng);
        let session: [u8; 32] = rng.gen();
        let challenge: [u8; 32] = rng.gen();
        let reference = binding_message(&statement, &session, &challenge);
        let replacement = pallas::Base::from(rng.gen::<u64>()) + pallas::Base::from(1u64);

        let mut variants = vec![
            binding_message(&statement, &rng.gen::<[u8; 32]>(), &challenge),
            binding_message(&statement, &session, &rng.gen::<[u8; 32]>()),
        ];
        for field in 0..9usize {
            let mut changed = statement;
            match field {
                0 => changed.round_id += replacement,
                1 => changed.nc_root += replacement,
                2 => changed.nf_imt_root += replacement,
                3 => changed.van_comm += replacement,
                marker => changed.markers[marker - 4] += replacement,
            }
            variants.push(binding_message(&changed, &session, &challenge));
        }
        for (index, variant) in variants.iter().enumerate() {
            if variant.len() != reference.len() {
                return Err(format!("variant {index} changed the message length"));
            }
            if *variant == reference {
                return Err(format!("variant {index} did not change the message"));
            }
        }
        Ok(())
    });
    assert!(failures.is_empty(), "the binding message misses a field: {failures:#?}");
}
