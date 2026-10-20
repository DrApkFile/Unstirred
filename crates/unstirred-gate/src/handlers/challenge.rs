#![doc = "Issuing a challenge (PRD section 3.3). The gate stores a hash of it and answers with the \
raw value and the expiry the file now holds. The raw challenge goes to the holder and nowhere \
else: it is not logged, and the store never writes it."]

use hyper::StatusCode;
use unstirred_verifier::store::Store;

use crate::handlers::blocking;
use crate::reply::{body_with_note, error, json, Reply};
use crate::rfc3339::from_unix_seconds;
use crate::state::GateState;

pub(super) async fn issue(state: &GateState, context: &str) -> Reply {
    let note = state.note();
    let database = state.database.clone();
    let clock = state.clock.clone();
    let context = context.to_owned();
    let issued = blocking(move || {
        let store = Store::open(&database).map_err(|e| e.to_string())?;
        store.issue(&context, &*clock).map_err(|e| e.to_string())
    })
    .await;
    match issued {
        Ok(Ok(issued)) => json(
            StatusCode::OK,
            body_with_note(
                serde_json::json!({
                    "challenge": hex::encode(issued.raw),
                    "expires_at": from_unix_seconds(issued.expires_at),
                }),
                note,
            ),
            note,
        ),
        Ok(Err(detail)) | Err(detail) => unavailable(&detail, note),
    }
}

/// A gate that cannot write a challenge admits nothing and says so: this is the gate's own
/// failure, not a verdict about a holder.
fn unavailable(detail: &str, note: Option<&str>) -> Reply {
    error(
        StatusCode::SERVICE_UNAVAILABLE,
        &format!("this gate could not issue a challenge: {detail}"),
        note,
    )
}
