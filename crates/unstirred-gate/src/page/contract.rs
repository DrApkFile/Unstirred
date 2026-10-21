#![doc = "The one state the Holders' Room page is rendered from, as a type. `render(state)` in \
`ui/render.js` reads exactly these fields and no others, so this is the contract between the gate, \
the page and the replay files the demo video is rendered from: a live page serialises this type, a \
replay file is deserialised into it, and the same `check` runs over both. Unknown fields are \
refused, so a field can never be added to a replay file without being added to the renderer."]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const HEX_ROOT_CHARS: usize = 64;

/// Where the state on screen came from. Live is polled from this gate; replay is a recorded act,
/// rendered with no polling and no clock, which is what the demo video is captured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Live,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageState {
    pub mode: Mode,
    /// What the viewer is told the numbers are: `gate` for a live read, `real-testnet` for an act
    /// replayed from a recorded run.
    pub source: String,
    /// The act's name in replay mode, and nothing in live mode.
    pub replay: Option<String>,
    /// The room this page is about, or nothing for the room picker.
    pub room: Option<String>,
    /// The bot-cost counter's input: how many fake members to price.
    pub members: u64,
    /// Presentation mode: the smallest type on the page gets larger and the gate's own technical
    /// lines come off the provenance footer. Nothing else changes, so a presented act and a read
    /// act are the same state.
    pub present: bool,
    pub gate: GateFacts,
    /// The words for every code the page may have to show, from `unstirred-core`.
    pub codes: BTreeMap<String, CodeFacts>,
    pub rooms: Vec<RoomState>,
    /// Where these numbers come from, in plain words, printed on the page itself.
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateFacts {
    pub root_sources: Vec<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeFacts {
    pub kind: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    pub context: String,
    pub network: String,
    pub window: WindowState,
    pub min_zatoshis: String,
    pub h1_root: String,
    pub h1_root_provenance: String,
    /// The cap the log route answers with, so the page can say what it is showing at most.
    pub limit: u32,
    pub markers: Vec<MarkerState>,
    /// One holder's own line across the window, for an act that shows somebody against the rule
    /// rather than only the rule. Nothing the gate knows, so a live page never has one: it is an
    /// act's own illustration, and the page draws it short of the band when it does not cover it.
    pub holder: Option<HolderLine>,
    pub verdicts: Vec<VerdictState>,
    /// The verdicts this gate has just answered, including refusals, which the store never writes.
    pub recent: Vec<VerdictState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowState {
    pub h0: u32,
    pub h1: u32,
}

/// One holder's coins on the timeline: what to call them, and the blocks they existed unspent
/// between. A line that does not cover the whole window is drawn as not covering it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HolderLine {
    pub label: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerState {
    pub marker: String,
    pub admitted_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictState {
    pub verdict: String,
    pub code: Option<String>,
    pub at: i64,
    /// Where `at` came from, when it was not read from a clock at the moment of the verdict. A
    /// recorded act whose time had to be derived says so here, and the page prints it beside the
    /// time rather than in a footnote.
    pub at_note: Option<String>,
}

impl PageState {
    /// Everything the renderer assumes and the type cannot say. A state that fails this is never
    /// served and never shipped as a replay file: the page would render something misleading.
    pub fn check(&self) -> Result<(), String> {
        if self.rooms.is_empty() {
            return Err("a page state names no rooms".to_owned());
        }
        match (self.mode, self.replay.as_deref()) {
            (Mode::Replay, None) => return Err("a replay state does not name its act".to_owned()),
            (Mode::Live, Some(act)) => return Err(format!("a live state names the replay act {act:?}")),
            _ => {}
        }
        for (index, room) in self.rooms.iter().enumerate() {
            if self.rooms[..index]
                .iter()
                .any(|earlier| earlier.context == room.context)
            {
                return Err(format!("two rooms share the context {:?}", room.context));
            }
            room.check()?;
        }
        if let Some(room) = &self.room {
            if !self.rooms.iter().any(|hosted| &hosted.context == room) {
                return Err(format!("the page is about {room:?}, which is not one of its rooms"));
            }
        }
        for room in &self.rooms {
            for verdict in room.verdicts.iter().chain(&room.recent) {
                if let Some(code) = &verdict.code {
                    if !self.codes.contains_key(code) {
                        return Err(format!("{code:?} is shown but its words are not in the state"));
                    }
                }
            }
        }
        Ok(())
    }
}

impl RoomState {
    fn check(&self) -> Result<(), String> {
        if self.context.is_empty() {
            return Err("a room has no context".to_owned());
        }
        if self.window.h0 >= self.window.h1 {
            return Err(format!(
                "room {:?} has window h0 {} h1 {}, which is not a window",
                self.context, self.window.h0, self.window.h1
            ));
        }
        self.min_zatoshis
            .parse::<u64>()
            .map_err(|e| format!("room {:?} has min_zatoshis {:?}: {e}", self.context, self.min_zatoshis))?;
        require_hex(&self.h1_root, &format!("room {:?}'s h1 root", self.context))?;
        if let Some(holder) = &self.holder {
            if holder.from > holder.to {
                return Err(format!(
                    "room {:?} draws a holder from block {} to {}, which is backwards",
                    self.context, holder.from, holder.to
                ));
            }
            if holder.label.is_empty() {
                return Err(format!("room {:?} draws a holder with no name", self.context));
            }
        }
        for marker in &self.markers {
            require_hex(&marker.marker, &format!("a marker of room {:?}", self.context))?;
        }
        Ok(())
    }
}

fn require_hex(value: &str, what: &str) -> Result<(), String> {
    if value.len() != HEX_ROOT_CHARS || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!(
            "{what} is not {HEX_ROOT_CHARS} characters of lowercase hex: {value:?}"
        ));
    }
    Ok(())
}
