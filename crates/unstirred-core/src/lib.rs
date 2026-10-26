#![doc = "Pure rules shared by the Unstirred prover, verifier and gate. No network, clock or ambient randomness."]
//!
//! Everything three crates must agree on lives here and nowhere else: what a gate's rule looks
//! like, what a holder hands over, what is signed, and the closed set of words a verdict may use.
//! A rule that existed in two places would be a rule that could disagree with itself.
//!
//! ```
//! use unstirred_core::{build_manifest, parse_manifest, Network, Window};
//!
//! // What a gate publishes: four numbers, and nothing a holder has to take on trust.
//! let json = build_manifest(Network::Test, "holders-room", Window { h0: 4_505_790, h1: 4_506_510 })?;
//! let manifest = parse_manifest(&json)?;
//!
//! assert_eq!(manifest.context, "holders-room");
//! assert_eq!(manifest.window.h0, 4_505_790);
//! // The threshold is one ballot in the circuit and is not the gate's to choose.
//! assert_eq!(manifest.min_zatoshis, unstirred_core::REQUIRED_MIN_ZATOSHIS);
//! # Ok::<(), unstirred_core::Refusal>(())
//! ```

#![warn(missing_docs)]

/// The message a holder signs under their randomized key, and rebuilding that key to check it.
pub mod binding;
/// The closed set of verdict codes, each with its own message, kind and retryability.
pub mod codes;
/// Deriving a gate's `round_id` from its network and context, and from nothing else.
pub mod context;
/// The gate manifest: the rule a gate publishes and a holder proves against.
pub mod manifest;
/// The proof package: the nine fields a holder hands a gate.
pub mod package;
/// The window rules — what makes an `h0` and an `h1` usable at all.
pub mod window;

pub use binding::{binding_message, rk_from_coordinates, BindingStatement};
pub use codes::{Code, CodeSpec, Refusal, VerdictKind};
pub use context::derive_round_id;
pub use manifest::{
    build_manifest, parse_manifest, Manifest, Network, Window, PROTOCOL_VERSION, REQUIRED_MIN_ZATOSHIS,
};
pub use package::{ProofPackage, PublicInstance};
pub use window::{check_window, check_window_shape, FINALITY_DEPTH, NULLIFIER_SYNC_STEP};
