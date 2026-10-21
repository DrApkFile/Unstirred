#![doc = "Answering one request. The dispatcher routes it, finds the room, hands the work to a \
blocking task (SQLite, the root sources and a Halo2 proof all block), and writes the one log line \
for the answer. Nothing here ever sees who asked: the connection's peer address is dropped where \
it is accepted, so there is nothing to pass down and nothing to log (A39)."]

mod admit;
mod challenge;
mod log;
mod page;
mod recent;
mod rooms;

use http_body_util::{BodyExt, Limited};
use hyper::body::Incoming;
use hyper::header::ALLOW;
use hyper::{Request, StatusCode};

use crate::events::request_line;
use crate::reply::{error, Reply};
use crate::router::{route, NoRoute, Route};
use crate::state::{GateState, MAX_ADMIT_BODY_BYTES};

/// Answers one request and logs the answer. The log line is written here, once, so no handler can
/// add a line of its own with something in it that this one leaves out.
pub async fn dispatch(state: &GateState, request: Request<Incoming>) -> Reply {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let query = request.uri().query().map(str::to_owned);
    let (reply, code) = answer(state, request, &path, query.as_deref()).await;
    state.log.line(&request_line(
        method.as_str(),
        &path,
        reply.status().as_u16(),
        code.as_deref(),
    ));
    reply
}

/// The reply, plus the verdict code when the answer was a verdict. An HTTP-level refusal has no
/// code: the gate decided nothing about the holding it was never shown.
async fn answer(
    state: &GateState,
    request: Request<Incoming>,
    path: &str,
    query: Option<&str>,
) -> (Reply, Option<String>) {
    let note = state.note();
    let route = match route(request.method(), path) {
        Ok(route) => route,
        Err(NoRoute::UnknownPath) => return (error(StatusCode::NOT_FOUND, "no such route at this gate", note), None),
        Err(NoRoute::WrongMethod { allowed }) => return (method_not_allowed(allowed, note), None),
    };
    match route {
        Route::Picker => (page::picker(state, query).await, None),
        Route::RoomPage(context) => (page::room(state, &context, query).await, None),
        Route::RoomList => (rooms::listing(state), None),
        Route::Manifest(context) => (with_room(state, &context, |room| rooms::manifest(room, note)), None),
        Route::Challenge(context) => match missing_room(state, &context, note) {
            Some(reply) => (reply, None),
            None => (challenge::issue(state, &context).await, None),
        },
        Route::Log(context) => match missing_room(state, &context, note) {
            Some(reply) => (reply, None),
            None => (log::room_log(state, &context).await, None),
        },
        Route::Recent(context) => match missing_room(state, &context, note) {
            Some(reply) => (reply, None),
            None => (recent::feed(state, &context), None),
        },
        Route::Admit(context) => match missing_room(state, &context, note) {
            Some(reply) => (reply, None),
            None => admit_body(state, &context, request).await,
        },
    }
}

async fn admit_body(state: &GateState, context: &str, request: Request<Incoming>) -> (Reply, Option<String>) {
    let note = state.note();
    match read_body(request).await {
        Ok(body) => admit::admit(state, context, body).await,
        Err(BodyProblem::TooLarge) => (
            error(
                StatusCode::PAYLOAD_TOO_LARGE,
                &format!("a proof package this gate accepts is at most {MAX_ADMIT_BODY_BYTES} bytes; nothing was read, verified or stored"),
                note,
            ),
            None,
        ),
        Err(BodyProblem::Unreadable) => (
            error(
                StatusCode::BAD_REQUEST,
                "the request body could not be read; nothing was verified or stored",
                note,
            ),
            None,
        ),
    }
}

enum BodyProblem {
    TooLarge,
    Unreadable,
}

/// Reads at most [`MAX_ADMIT_BODY_BYTES`]. A declared length above the limit is refused without
/// reading anything, and a body that grows past it stops the read.
async fn read_body(request: Request<Incoming>) -> Result<Vec<u8>, BodyProblem> {
    if declared_too_long(&request) {
        return Err(BodyProblem::TooLarge);
    }
    match Limited::new(request.into_body(), MAX_ADMIT_BODY_BYTES).collect().await {
        Ok(collected) => Ok(collected.to_bytes().to_vec()),
        Err(problem) if problem.downcast_ref::<http_body_util::LengthLimitError>().is_some() => {
            Err(BodyProblem::TooLarge)
        }
        Err(_) => Err(BodyProblem::Unreadable),
    }
}

fn declared_too_long(request: &Request<Incoming>) -> bool {
    request
        .headers()
        .get(hyper::header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length > MAX_ADMIT_BODY_BYTES)
}

/// The 404 for a room this gate does not host, or nothing when it does. The answer says nothing
/// else: not which rooms exist, not whether one ever did, and nothing from the store (A38).
fn missing_room(state: &GateState, context: &str, note: Option<&str>) -> Option<Reply> {
    state
        .room(context)
        .is_none()
        .then(|| error(StatusCode::NOT_FOUND, "this gate hosts no room by that name", note))
}

fn with_room(state: &GateState, context: &str, answer: impl FnOnce(&crate::config::Room) -> Reply) -> Reply {
    match state.room(context) {
        Some(room) => answer(room),
        None => error(
            StatusCode::NOT_FOUND,
            "this gate hosts no room by that name",
            state.note(),
        ),
    }
}

fn method_not_allowed(allowed: &'static str, note: Option<&str>) -> Reply {
    let mut reply = error(
        StatusCode::METHOD_NOT_ALLOWED,
        &format!("this route answers {allowed} only"),
        note,
    );
    if let Ok(value) = allowed.parse() {
        reply.headers_mut().insert(ALLOW, value);
    }
    reply
}

/// Runs the blocking half of a request: SQLite, the root sources, and the proof. A task that
/// could not be run at all is a local failure, never a verdict.
pub(crate) async fn blocking<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|problem| format!("{problem}"))
}
