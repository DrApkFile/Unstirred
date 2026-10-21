#![doc = "The verdicts this gate has just answered, per room. A refusal is never written to the \
store, so this is the only way the Holders' Room page can show one — and it is in memory only, \
bounded, and gone when the process ends. Three fields per entry: the verdict, its code, and the \
second it was answered (LEAKS L16)."]

use hyper::StatusCode;

use crate::recent::RECENT_PER_ROOM;
use crate::reply::{body_with_note, json, Reply};
use crate::state::GateState;

pub(super) fn feed(state: &GateState, context: &str) -> Reply {
    let note = state.note();
    let verdicts: Vec<serde_json::Value> = state
        .recent
        .newest(context)
        .into_iter()
        .map(|entry| serde_json::json!({ "verdict": entry.verdict, "code": entry.code, "at": entry.at }))
        .collect();
    let body = serde_json::json!({
        "context": context,
        "limit": RECENT_PER_ROOM,
        "verdicts": verdicts,
        "kept_in_memory_only":
            "a verdict, its code and the second it was answered; never written to disk, and forgotten when this gate restarts",
    });
    json(StatusCode::OK, body_with_note(body, note), note)
}
