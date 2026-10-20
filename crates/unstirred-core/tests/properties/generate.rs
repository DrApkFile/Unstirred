//! The cases the properties run on. Biased on purpose towards the awkward ones: uniformly
//! random heights would almost never make a valid window, and a property that only ever
//! exercises refusals proves little.

use rand::rngs::StdRng;
use rand::Rng;
use unstirred_core::{BindingStatement, Network, Window, FINALITY_DEPTH, NULLIFIER_SYNC_STEP};
use voting_crypto_deps::pasta_curves::pallas;

/// The context rule as PRD section 3 and the manifest's own refusal state it.
pub fn context_is_allowed(context: &str) -> bool {
    context.len() <= 64
        && context
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && context
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Windows biased towards the interesting ones: uniformly random triples would almost never put
/// h1 on the nullifier step above h0, and a property that only ever refuses proves little.
pub fn random_window(rng: &mut StdRng) -> Window {
    let h1 = if rng.gen_bool(0.7) {
        rng.gen_range(0..5_000_000u32) / NULLIFIER_SYNC_STEP * NULLIFIER_SYNC_STEP
    } else {
        rng.gen_range(0..5_000_000u32)
    };
    let h0 = if rng.gen_bool(0.8) {
        h1.saturating_sub(rng.gen_range(1..100_000u32))
    } else {
        rng.gen_range(0..5_000_000u32)
    };
    Window { h0, h1 }
}

pub fn random_tip(rng: &mut StdRng, h1: u32) -> u32 {
    match rng.gen_range(0..4u8) {
        0 => h1.saturating_add(FINALITY_DEPTH + rng.gen_range(0..1_000u32)),
        1 => h1.saturating_add(rng.gen_range(0..FINALITY_DEPTH)),
        2 => u32::MAX - rng.gen_range(0..FINALITY_DEPTH),
        _ => rng.gen_range(0..5_200_000u32),
    }
}

pub fn random_statement(rng: &mut StdRng) -> BindingStatement {
    BindingStatement {
        round_id: pallas::Base::from(rng.gen::<u64>()),
        nc_root: pallas::Base::from(rng.gen::<u64>()),
        nf_imt_root: pallas::Base::from(rng.gen::<u64>()),
        van_comm: pallas::Base::from(rng.gen::<u64>()),
        markers: [0u8; 5].map(|_| pallas::Base::from(rng.gen::<u64>())),
    }
}

pub fn random_network(rng: &mut StdRng) -> Network {
    if rng.gen_bool(0.5) {
        Network::Test
    } else {
        Network::Main
    }
}

pub fn other_network(network: Network) -> Network {
    match network {
        Network::Test => Network::Main,
        Network::Main => Network::Test,
    }
}

pub fn manifest_json(context: &str) -> String {
    let context = serde_json::to_string(context).unwrap_or_else(|_| String::from("\"\""));
    format!(
        r#"{{"version":"unstirred/v1","network":"test","context":{context},
             "window":{{"h0":4505790,"h1":4506510}},"min_zatoshis":"12500000"}}"#
    )
}
