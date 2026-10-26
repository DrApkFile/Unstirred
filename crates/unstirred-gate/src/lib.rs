#![doc = "unstirred-gate: the gate as a small web service. It hosts one room per manifest and \
answers, per room, the manifest, a challenge, an admission, that room's log and the verdicts this \
process has just answered, with the room list beside them. It adds no verification of its own — \
`handlers::admit` calls the same `unstirred_verifier::admit` that `unstirred admit` calls, against \
this gate's own root authority and its own database — and the only thing it keeps beyond what the \
store already holds (hashed challenges, markers and admission verdicts) is a bounded in-memory \
feed of verdicts, which is forgotten when the process ends (LEAKS L16). It also serves the \
Holders' Room page itself, from its own data, with no third-party request of any kind, no request \
logging that records an address and no request body in any log (LEAKS L9, L10)."]
#![warn(missing_docs)]

pub mod client;
pub mod config;
pub mod events;
mod handlers;
pub mod page;
pub mod recent;
mod reply;
pub mod rfc3339;
pub mod roots;
pub mod router;
pub mod serve;
pub mod state;

pub use client::{AdvertisedServices, Answer, GateClient, GateClientError, GateVerdict, IssuedChallenge};
pub use config::{load_config, ConfigError, GateConfig, Room, Services};
pub use events::{CollectedLog, GateLog, StdoutLog};
pub use page::{is_act_name, live_state, replay_state, Mode, PageState, REPLAY_ACTS};
pub use recent::{RecentVerdict, RecentVerdicts, RECENT_PER_ROOM};
pub use reply::ROOT_NOTE_HEADER;
pub use roots::{RootAuthority, SpecRootAuthority};
pub use serve::{serve, start, RunningGate, ServeError};
pub use state::{GateState, MAX_ADMIT_BODY_BYTES, ROOM_LOG_LIMIT};
