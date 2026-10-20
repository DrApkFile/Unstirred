#![doc = "What one room remembers, for anyone who asks: its newest markers and admission verdicts. \
No challenge hashes, no package bodies, no addresses — the whole of LEAKS L9 is that this is all \
there is to show."]

use hyper::StatusCode;
use unstirred_verifier::store::{AdmissionRow, MarkerRow, Store};

use crate::handlers::blocking;
use crate::reply::{body_with_note, error, json, Reply};
use crate::state::{GateState, ROOM_LOG_ROWS};

pub(super) async fn room_log(state: &GateState, context: &str) -> Reply {
    let note = state.note();
    let database = state.database.clone();
    let room = context.to_owned();
    let read = blocking(move || {
        let store = Store::open(&database).map_err(|e| e.to_string())?;
        let markers = store.recent_markers(&room, ROOM_LOG_ROWS).map_err(|e| e.to_string())?;
        let verdicts = store
            .recent_admissions(&room, ROOM_LOG_ROWS)
            .map_err(|e| e.to_string())?;
        Ok::<_, String>((markers, verdicts))
    })
    .await;
    match read {
        Ok(Ok((markers, verdicts))) => json(
            StatusCode::OK,
            body_with_note(
                serde_json::json!({
                    "context": context,
                    "rows": ROOM_LOG_ROWS,
                    "markers": markers.iter().map(marker).collect::<Vec<_>>(),
                    "verdicts": verdicts.iter().map(verdict).collect::<Vec<_>>(),
                    "stored_and_nothing_else":
                        "markers and admission verdicts; no challenge, no package, no address",
                }),
                note,
            ),
            note,
        ),
        Ok(Err(detail)) | Err(detail) => error(
            StatusCode::SERVICE_UNAVAILABLE,
            &format!("this gate could not read its own log: {detail}"),
            note,
        ),
    }
}

fn marker(row: &MarkerRow) -> serde_json::Value {
    serde_json::json!({ "marker": hex::encode(row.marker), "admitted_at": row.admitted_at })
}

fn verdict(row: &AdmissionRow) -> serde_json::Value {
    serde_json::json!({ "verdict": row.verdict, "code": row.code, "at": row.at })
}
