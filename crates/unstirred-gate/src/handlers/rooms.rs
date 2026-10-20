#![doc = "What a gate publishes about itself: the rooms it hosts, each room's window and \
threshold, the h1 root its operator pinned and the root sources it reads, and the manifest itself. \
A judge can read all of it without proving anything."]

use hyper::StatusCode;
use unstirred_core::Manifest;

use crate::config::Room;
use crate::reply::{body_with_note, json, Reply};
use crate::state::GateState;

pub(super) fn listing(state: &GateState) -> Reply {
    let note = state.note();
    let body = serde_json::json!({
        "rooms": state.rooms().map(describe).collect::<Vec<_>>(),
        "root_sources": state.roots.describe(),
    });
    json(StatusCode::OK, body_with_note(body, note), note)
}

/// The room's manifest, exactly as this gate parsed it. It carries no field of ours: a manifest
/// with an unknown field is one `unstirred-core` refuses, so the single-source label travels in
/// the response header instead (A27).
pub(super) fn manifest(room: &Room, note: Option<&str>) -> Reply {
    json(StatusCode::OK, room.manifest_json.clone(), note)
}

fn describe(room: &Room) -> serde_json::Value {
    let Manifest {
        network,
        context,
        window,
        min_zatoshis,
        ..
    } = &room.manifest;
    serde_json::json!({
        "context": context,
        "network": network.to_string(),
        "window": { "h0": window.h0, "h1": window.h1 },
        "min_zatoshis": min_zatoshis,
        "h1_root": hex::encode(room.h1_root),
        "h1_root_provenance": "operator-pinned",
    })
}
