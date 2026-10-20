//! A29: `round_id` is a function of network and context and nothing else, and no two gates in
//! these cases collide on one.

use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use unstirred_core::{derive_round_id, Network};
use voting_circuits::ff::PrimeField;

use super::generate::{other_network, random_network};
use super::seeded::{mixed_text, over_seeds, CASES};

/// A29: `round_id` is a function of network and context and nothing else. The same pair always
/// gives the same id, the other network always gives a different one, and no two gates collide.
#[test]
fn a29_round_id_is_derived_from_network_and_context_only() {
    let failures = over_seeds(CASES, |rng| {
        let context = mixed_text(rng, 40);
        let network = random_network(rng);
        let id = derive_round_id(network, &context);
        if id != derive_round_id(network, &context) {
            return Err(format!("{network} {context:?} derived two different ids"));
        }
        if id == derive_round_id(other_network(network), &context) {
            return Err(format!("{context:?} derives one id on both networks"));
        }
        Ok(())
    });
    assert!(
        failures.is_empty(),
        "round ids are not a function of the pair: {failures:#?}"
    );

    let mut seen: HashMap<[u8; 32], (Network, String)> = HashMap::new();
    let mut collisions = Vec::new();
    for seed in 0..CASES {
        let mut rng = StdRng::seed_from_u64(seed);
        let context = mixed_text(&mut rng, 40);
        let network = random_network(&mut rng);
        let id = derive_round_id(network, &context).to_repr();
        match seen.get(&id) {
            Some(first) if *first != (network, context.clone()) => {
                collisions.push(format!("{first:?} and {:?} share a round id", (network, context)));
            }
            _ => {
                seen.insert(id, (network, context));
            }
        }
    }
    assert!(collisions.is_empty(), "two gates share a round id: {collisions:#?}");
}
