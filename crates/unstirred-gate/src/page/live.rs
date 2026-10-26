#![doc = "The live page state: this gate's own rooms, this gate's own database and the verdicts \
this process has just answered, assembled into the one state the renderer reads. Every value comes \
from the gate — the rule from the manifest, the markers from the store, the words for a code from \
`unstirred-core` — so the page has nothing of its own to be wrong about."]

use std::collections::BTreeMap;

use unstirred_core::Code;
use unstirred_verifier::store::Store;

use crate::page::contract::{CodeFacts, GateFacts, MarkerState, Mode, PageState, RoomState, VerdictState, WindowState};
use crate::state::{GateState, ROOM_LOG_LIMIT};

/// What the bot-cost counter starts at, which is the number the demo script prices (PRD section
/// 9). The viewer types over it and the page recomputes from whatever they type.
pub const MEMBERS_AT_FIRST_PAINT: u64 = 1_000;

/// Reads this gate's own state for one page. Opens the database once, like every other route, so
/// it must be called from a blocking task.
pub fn live_state(state: &GateState, room: Option<&str>, present: bool) -> Result<PageState, String> {
    let store = Store::open(&state.database).map_err(|e| e.to_string())?;
    let mut rooms = Vec::new();
    for hosted in state.rooms() {
        let context = hosted.context();
        let markers = store
            .recent_markers(context, ROOM_LOG_LIMIT)
            .map_err(|e| e.to_string())?;
        let verdicts = store
            .recent_admissions(context, ROOM_LOG_LIMIT)
            .map_err(|e| e.to_string())?;
        rooms.push(RoomState {
            context: context.to_owned(),
            network: hosted.manifest.network.to_string(),
            window: WindowState {
                h0: hosted.manifest.window.h0,
                h1: hosted.manifest.window.h1,
            },
            min_zatoshis: hosted.manifest.min_zatoshis.to_string(),
            h1_root: hex::encode(hosted.h1_root),
            h1_root_provenance: "operator-pinned".to_owned(),
            // A gate serves its own routes, so a card opens /room/<context>. Only a static
            // export, where that route is no file, says where it opens (A51).
            page: None,
            limit: ROOM_LOG_LIMIT,
            // A holder line illustrates one act; it is not something a gate knows about anyone.
            holder: None,
            markers: markers
                .iter()
                .map(|row| MarkerState {
                    marker: hex::encode(row.marker),
                    admitted_at: row.admitted_at,
                })
                .collect(),
            verdicts: verdicts
                .iter()
                .map(|row| VerdictState {
                    verdict: row.verdict.clone(),
                    code: row.code.clone(),
                    at: row.at,
                    at_note: None,
                })
                .collect(),
            recent: state
                .recent
                .newest(context)
                .into_iter()
                .map(|entry| VerdictState {
                    verdict: entry.verdict.to_owned(),
                    code: entry.code.map(str::to_owned),
                    at: entry.at,
                    at_note: None,
                })
                .collect(),
        });
    }
    let page = PageState {
        mode: Mode::Live,
        source: "gate".to_owned(),
        replay: None,
        room: room.map(str::to_owned),
        present,
        members: MEMBERS_AT_FIRST_PAINT,
        gate: GateFacts {
            root_sources: state.roots.describe(),
            note: state.note().map(str::to_owned),
        },
        codes: every_code(),
        rooms,
        provenance: provenance(state),
    };
    page.check()?;
    Ok(page)
}

/// The words for the whole closed code set, so the page can name any verdict this gate can give
/// without holding a copy of the words.
pub fn every_code() -> BTreeMap<String, CodeFacts> {
    Code::ALL
        .into_iter()
        .map(|code| {
            let spec = code.spec();
            (
                spec.wire_name.to_owned(),
                CodeFacts {
                    kind: spec.kind.to_string(),
                    message: spec.message.to_owned(),
                    retryable: spec.retryable,
                },
            )
        })
        .collect()
}

fn provenance(state: &GateState) -> Vec<String> {
    let mut lines = vec!["Live from this gate: every number on this page was read from it.".to_owned()];
    lines.push(format!("Root sources: {}.", state.roots.describe().join(", ")));
    if let Some(note) = state.note() {
        lines.push(note.to_owned());
    }
    lines
}
