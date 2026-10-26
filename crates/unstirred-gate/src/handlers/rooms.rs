#![doc = "What a gate publishes about itself: the rooms it hosts, each room's window and \
threshold, the h1 root its operator pinned and the root sources it reads, and the manifest itself. \
A judge can read all of it without proving anything."]

use hyper::StatusCode;
use unstirred_core::Manifest;

use crate::config::{Room, Services};
use crate::reply::{body_with_note, json, Reply};
use crate::state::GateState;

pub(super) fn listing(state: &GateState) -> Reply {
    let note = state.note();
    let mut body = serde_json::json!({
        "rooms": state.rooms().map(describe).collect::<Vec<_>>(),
        "root_sources": state.roots.describe(),
    });
    if let (Some(object), true) = (body.as_object_mut(), state.services.any()) {
        object.insert("services".to_owned(), advertised(&state.services));
    }
    json(StatusCode::OK, body_with_note(body, note), note)
}

/// The two public addresses a holder would otherwise have to be told out of band. They are this
/// operator's own services, identical for everyone who knocks, and a holder who would rather use
/// their own passes `--pir` or `--lightwalletd` instead (LEAKS L18, A49).
fn advertised(services: &Services) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    if let Some(url) = &services.pir_url {
        object.insert("pir_url".to_owned(), serde_json::Value::String(url.clone()));
    }
    if let Some(url) = &services.lightwalletd_url {
        object.insert("lightwalletd_url".to_owned(), serde_json::Value::String(url.clone()));
    }
    serde_json::Value::Object(object)
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
