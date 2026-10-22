#![doc = "The recorded acts of the demo, each a page state the video is rendered from. They are \
compiled in rather than read from disk, so a replay page is the same bytes on any machine and a \
published crate carries its own demo. Their values come from the recorded runs in `evidence/` — \
the two-room knock for acts 1, 2, 3 and 5, the newcomer's forced knock for act 4 — and each act \
says which one on the page itself, in its own `provenance` lines."]

use crate::page::contract::PageState;

// The acts and the list of them are one output of `scripts/replay-from-evidence.py`: it writes
// `ui/replay/acts.rs` beside the files it generates, so an act that has been recorded is served
// without a hand edit here, and a name in that list always points at a file the script wrote.
include!("../../ui/replay/acts.rs");

/// One act by name, already checked. An act whose own file is malformed is served by nobody: it
/// fails here, in the test that renders every act, and at the gate.
pub fn replay_state(name: &str) -> Option<Result<PageState, String>> {
    let (_, json) = REPLAY_ACTS.iter().find(|(act, _)| *act == name)?;
    Some(parse(json))
}

fn parse(json: &str) -> Result<PageState, String> {
    let state: PageState = serde_json::from_str(json).map_err(|e| format!("{e}"))?;
    state.check()?;
    Ok(state)
}

/// Whether a name could be an act at all. A URL carrying anything else is answered with the same
/// 404 as an unknown room, before any lookup.
pub fn is_act_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 40
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
