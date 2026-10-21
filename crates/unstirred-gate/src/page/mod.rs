#![doc = "The Holders' Room page: what the gate serves to a browser. One state (`contract`), one \
renderer (`ui/render.js`), and two ways to fill it — live from this gate (`live`) or from a \
recorded act (`replay`). The page is the single source of the demo's visual design: the video is \
rendered from these same documents, which is why the design tokens live in one block and the \
renderer is one pure function of the state."]

mod contract;
mod document;
mod live;
mod replay;

pub use contract::{CodeFacts, GateFacts, MarkerState, Mode, PageState, RoomState, VerdictState, WindowState};
pub use document::{picker_document, room_document};
pub use live::{every_code, live_state, MEMBERS_AT_FIRST_PAINT};
pub use replay::{is_act_name, replay_state, REPLAY_ACTS};
