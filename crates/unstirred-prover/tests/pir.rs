//! The private nullifier lookup at h1. Every test of it is an attack or a refusal, so they all
//! live in `tests/adversarial/`; this target holds the synthetic wallet they read from and the
//! fake server they talk to.

#[path = "adversarial"]
mod adversarial {
    mod pir_lookup;
}

mod pir {
    pub mod held_wallet;
}

mod server;
