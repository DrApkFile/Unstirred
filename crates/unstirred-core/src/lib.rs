#![doc = "Pure rules shared by the Unstirred prover, verifier and gate. No network, clock or ambient randomness."]

pub mod binding;
pub mod codes;
pub mod context;
pub mod manifest;
pub mod package;
pub mod window;

pub use binding::{binding_message, rk_from_coordinates, BindingStatement};
pub use codes::{Code, CodeSpec, Refusal, VerdictKind};
pub use context::derive_round_id;
pub use manifest::{
    build_manifest, parse_manifest, Manifest, Network, Window, PROTOCOL_VERSION, REQUIRED_MIN_ZATOSHIS,
};
pub use package::{ProofPackage, PublicInstance};
pub use window::{check_window, check_window_shape, FINALITY_DEPTH, NULLIFIER_SYNC_STEP};
