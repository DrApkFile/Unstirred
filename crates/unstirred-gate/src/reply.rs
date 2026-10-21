#![doc = "The two kinds of answer a gate gives. One is a PRD section 3.5 verdict, which only the \
verification engine produces: ADMITTED is 200, a refusal is 403, an inconclusive root check is \
503. The other is an HTTP-level error for a request the gate never turned into a verdict, and it \
carries no code from the closed set, because no decision was made."]

use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE};
use hyper::{Response, StatusCode};
use unstirred_verifier::Verdict;

/// The header every answer carries when the gate reads one root source, so the label is on the
/// manifest route too, which cannot carry an extra JSON field (A27).
pub const ROOT_NOTE_HEADER: &str = "unstirred-root-note";

pub type Reply = Response<Full<Bytes>>;

/// One of our own JSON objects, with the single-source label added when one applies, so a
/// one-source gate says so in the body as well as the header. An absent label adds no field.
pub fn body_with_note(mut value: serde_json::Value, note: Option<&str>) -> String {
    if let (Some(note), Some(object)) = (note, value.as_object_mut()) {
        object.insert("note".to_owned(), serde_json::Value::String(note.to_owned()));
    }
    value.to_string()
}

/// One page, as its own document. Same headers as every other answer, so a page is as
/// uncacheable and as unremarkable as the JSON beside it.
pub fn html(body: String, note: Option<&str>) -> Reply {
    reply(StatusCode::OK, "text/html; charset=utf-8", body, note)
}

pub fn json(status: StatusCode, body: String, note: Option<&str>) -> Reply {
    reply(status, "application/json", body, note)
}

fn reply(status: StatusCode, content_type: &'static str, body: String, note: Option<&str>) -> Reply {
    let mut builder = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, "no-store");
    if let Some(note) = note {
        builder = builder.header(ROOT_NOTE_HEADER, note);
    }
    match builder.body(Full::new(Bytes::from(body))) {
        Ok(reply) => reply,
        Err(_) => unbuildable(),
    }
}

/// An HTTP-level refusal: a request the gate could not even read as a package. `error` is plain
/// words, never a verdict and never a code.
pub fn error(status: StatusCode, message: &str, note: Option<&str>) -> Reply {
    let body = serde_json::json!({ "error": message }).to_string();
    json(status, body, note)
}

pub fn status_for(verdict: &Verdict) -> StatusCode {
    match verdict {
        Verdict::Admitted => StatusCode::OK,
        Verdict::Refused(_) => StatusCode::FORBIDDEN,
        Verdict::Inconclusive(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub fn code_of(verdict: &Verdict) -> Option<&'static str> {
    match verdict {
        Verdict::Admitted => None,
        Verdict::Refused(refusal) | Verdict::Inconclusive(refusal) => Some(refusal.code.spec().wire_name),
    }
}

/// The last resort: a response whose own headers could not be built. It says nothing about the
/// request, because at this point the gate knows nothing it can safely say.
fn unbuildable() -> Reply {
    let mut reply = Response::new(Full::new(Bytes::from_static(
        b"{\"error\":\"the gate could not build a reply\"}",
    )));
    *reply.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    reply
}
