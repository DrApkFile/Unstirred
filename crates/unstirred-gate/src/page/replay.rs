#![doc = "The recorded acts of the demo, each a page state the video is rendered from. They are \
compiled in rather than read from disk, so a replay page is the same bytes on any machine and a \
published crate carries its own demo. Their values come from `evidence/2026-10-20-two-room-knock.txt` \
and its header file; each act says so on the page itself, in its own `provenance` lines."]

use crate::page::contract::PageState;

/// Every act, in demo order: the name a URL asks for, and the state it renders.
pub const REPLAY_ACTS: [(&str, &str); 4] = [
    ("act1-rule", include_str!("../../ui/replay/act1-rule.json")),
    ("act2-admitted", include_str!("../../ui/replay/act2-admitted.json")),
    ("act3-refused", include_str!("../../ui/replay/act3-refused.json")),
    (
        "act5-second-room",
        include_str!("../../ui/replay/act5-second-room.json"),
    ),
];

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
