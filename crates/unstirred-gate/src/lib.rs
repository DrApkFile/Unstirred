#![doc = "unstirred-gate: the gate as a small web service. It hosts one room per manifest and \
answers five routes per room: the manifest, a challenge, an admission, and that room's log, with \
the room list beside them. It adds no verification of its own — `handlers::admit` calls the same \
`unstirred_verifier::admit` that `unstirred admit` calls, against this gate's own root authority \
and its own database — and it remembers nothing but what the store already keeps: hashed \
challenges, markers and admission verdicts. No request logging that records an address, no \
request body in any log, and no third-party request from the service or the page (LEAKS L9, L10)."]

pub mod client;
pub mod config;
pub mod events;
mod handlers;
mod reply;
pub mod rfc3339;
pub mod roots;
pub mod router;
pub mod serve;
pub mod state;

pub use client::{Answer, GateClient, GateClientError, GateVerdict, IssuedChallenge};
pub use config::{load_config, ConfigError, GateConfig, Room};
pub use events::{CollectedLog, GateLog, StdoutLog};
pub use reply::ROOT_NOTE_HEADER;
pub use roots::{RootAuthority, SpecRootAuthority};
pub use serve::{serve, start, RunningGate, ServeError};
pub use state::{GateState, MAX_ADMIT_BODY_BYTES, ROOM_LOG_ROWS};
