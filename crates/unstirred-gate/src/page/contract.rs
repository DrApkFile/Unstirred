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
    /// Polled from this gate, now.
    Live,
    /// A recorded act, rendered with no polling and no clock.
    Replay,
}

/// The whole of what the page is rendered from. `render(state)` in `ui/render.js` reads these
/// fields and no others.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageState {
    /// Whether these numbers were polled from the gate or replayed from a recording.
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
    /// What this gate will say about itself: its root sources, and its single-source note if it
    /// has one.
    pub gate: GateFacts,
    /// The words for every code the page may have to show, from `unstirred-core`.
    pub codes: BTreeMap<String, CodeFacts>,
    /// Every room the page can show, which the picker lists and a room page selects from.
    pub rooms: Vec<RoomState>,
    /// Where these numbers come from, in plain words, printed on the page itself.
    pub provenance: Vec<String>,
}

/// What a gate says about how it establishes its own facts. On the page so a viewer can see
/// whether the roots were independently checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateFacts {
    /// The root sources, as the gate names them.
    pub root_sources: Vec<String>,
    /// The single-source note, when exactly one source was configured.
    pub note: Option<String>,
}

/// One verdict code's words, taken from `unstirred-core` so the page cannot word a code
/// differently from the verifier that issued it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeFacts {
    /// The verdict kind: refused or inconclusive.
    pub kind: String,
    /// What the holder is told.
    pub message: String,
    /// Whether trying again could ever change the answer.
    pub retryable: bool,
}

/// One room as the page shows it: its published rule, the markers it holds, and the verdicts it
/// has answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    /// The room's name, which scopes its markers.
    pub context: String,
    /// The network its rule is about.
    pub network: String,
    /// The two heights the holding is proved across.
    pub window: WindowState,
    /// The threshold, as the manifest's decimal string.
    pub min_zatoshis: String,
    /// The pinned h1 nullifier-set root, hex.
    pub h1_root: String,
    /// Where this room's card opens, when the page is not being served by a gate. A live gate
    /// leaves it unset and the card opens `/room/<context>`; the static export sets it to the
    /// first recorded act of this room, so an exported picker is a file that links to files
    /// beside it. It must be a relative path — a scheme, a host or a leading slash is refused,
    /// because a page the gate serves references no origin (A51, LEAKS L10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Where that root came from, in words. It is operator-pinned in v1, and the page says so
    /// rather than letting a viewer assume it was checked.
    pub h1_root_provenance: String,
    /// The cap the log route answers with, so the page can say what it is showing at most.
    pub limit: u32,
    /// The markers this room holds: its whole memory of who has been through it.
    pub markers: Vec<MarkerState>,
    /// One holder's own line across the window, for an act that shows somebody against the rule
    /// rather than only the rule. Nothing the gate knows, so a live page never has one: it is an
    /// act's own illustration, and the page draws it short of the band when it does not cover it.
    pub holder: Option<HolderLine>,
    /// The admissions the store has committed.
    pub verdicts: Vec<VerdictState>,
    /// The verdicts this gate has just answered, including refusals, which the store never writes.
    pub recent: Vec<VerdictState>,
}

/// The window as the page draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowState {
    /// The height the note-commitment root is pinned at.
    pub h0: u32,
    /// The height the nullifier-set root is pinned at.
    pub h1: u32,
}

/// One holder's coins on the timeline: what to call them, and the blocks they existed unspent
/// between. A line that does not cover the whole window is drawn as not covering it; one the
/// window's axis does not reach at all is marked past a break in the rail, with the distance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HolderLine {
    /// What to call these coins. The page builds a sentence around it, so it is a noun phrase and
    /// not a sentence of its own.
    pub label: String,
    /// The first block these coins existed unspent at.
    pub from: u32,
    /// The last block they are shown unspent at.
    pub to: u32,
    /// Where these two heights were read from, in one line under the rail. It stays on the page in
    /// presentation mode, where the provenance foot comes off, because a holder's own blocks are
    /// the one thing on the rail the gate was never told.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aside: Option<String>,
}

/// One marker on the page. The hex is the whole of it: it links to no address, amount or coin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerState {
    /// The marker, hex.
    pub marker: String,
    /// When the admission that wrote it was committed.
    pub admitted_at: i64,
}

/// One verdict line on the page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictState {
    /// The verdict word, as the wire spells it.
    pub verdict: String,
    /// The refusal code, or nothing for an admission.
    pub code: Option<String>,
    /// When it was answered, in unix seconds.
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
        if let Some(page) = &self.page {
            require_relative_path(page, &self.context)?;
        }
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
            if holder.aside.as_deref() == Some("") {
                return Err(format!(
                    "room {:?} draws a holder whose line says nothing about where it came from",
                    self.context
                ));
            }
        }
        for marker in &self.markers {
            require_hex(&marker.marker, &format!("a marker of room {:?}", self.context))?;
        }
        Ok(())
    }
}

/// A room card may only open something beside the page it is on. Anything with a scheme, an
/// authority or a leading slash would make an exported page reference somewhere else, which is
/// the one thing every page the gate serves is checked not to do.
fn require_relative_path(page: &str, context: &str) -> Result<(), String> {
    if page.is_empty() {
        return Err(format!("room {context:?} says it opens at nothing"));
    }
    if page.starts_with('/') || page.contains("//") || page.contains(':') {
        return Err(format!(
            "room {context:?} opens at {page:?}, which is not a path beside the page it is on"
        ));
    }
    Ok(())
}

fn require_hex(value: &str, what: &str) -> Result<(), String> {
    if value.len() != HEX_ROOT_CHARS || !value.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err(format!(
            "{what} is not {HEX_ROOT_CHARS} characters of lowercase hex: {value:?}"
        ));
    }
    Ok(())
}
