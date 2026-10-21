#![doc = "The two pages the gate serves itself: the room picker at the root, and one page per \
room. Each is one self-contained HTML document assembled here from this gate's own state, or from \
a recorded act when the request asks for a replay. Nothing is read from disk at request time and \
nothing is fetched from anywhere: a page is the gate's own data and the gate's own renderer."]

use hyper::StatusCode;

use crate::handlers::blocking;
use crate::page::{is_act_name, live_state, picker_document, replay_state, room_document, Mode, PageState};
use crate::reply::{error, html, Reply};
use crate::state::GateState;

pub(super) async fn picker(state: &GateState, query: Option<&str>) -> Reply {
    match live(state, None, presenting(query)).await {
        Ok(page) => serve(picker_document(&page), state),
        Err(reply) => reply,
    }
}

pub(super) async fn room(state: &GateState, context: &str, query: Option<&str>) -> Reply {
    if state.room(context).is_none() {
        return error(
            StatusCode::NOT_FOUND,
            "this gate hosts no room by that name",
            state.note(),
        );
    }
    let present = presenting(query);
    match act_asked_for(query) {
        Some(act) => match replayed(&act, context, present) {
            Ok(page) => serve(room_document(&page), state),
            Err(message) => error(StatusCode::NOT_FOUND, &message, state.note()),
        },
        None => match live(state, Some(context), present).await {
            Ok(page) => serve(room_document(&page), state),
            Err(reply) => reply,
        },
    }
}

/// The act named in `?replay=<name>`, when one is named at all. A name outside the act charset is
/// not looked up: it is answered with the same 404 as a name no act has.
fn act_asked_for(query: Option<&str>) -> Option<String> {
    parameter(query, "replay")
}

/// `?present=1`: the page for a projector or a video frame rather than a desk. It works on a live
/// page and on a replayed act alike, and changes nothing but how the page is read.
fn presenting(query: Option<&str>) -> bool {
    parameter(query, "present").as_deref() == Some("1")
}

fn parameter(query: Option<&str>, wanted: &str) -> Option<String> {
    query?.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == wanted).then(|| value.to_owned())
    })
}

fn replayed(act: &str, context: &str, present: bool) -> Result<PageState, String> {
    if !is_act_name(act) {
        return Err("this gate serves no replay by that name".to_owned());
    }
    let mut page = match replay_state(act) {
        Some(Ok(page)) => page,
        Some(Err(detail)) => return Err(format!("this gate's own replay {act} cannot be rendered: {detail}")),
        None => return Err("this gate serves no replay by that name".to_owned()),
    };
    if page.room.as_deref() != Some(context) {
        return Err("this gate serves no replay by that name for this room".to_owned());
    }
    if page.mode != Mode::Replay {
        return Err(format!("this gate's own replay {act} is not a replay"));
    }
    page.present = present;
    Ok(page)
}

/// This gate's own state, read in a blocking task because it opens the database, like every other
/// route that reads it.
async fn live(state: &GateState, context: Option<&str>, present: bool) -> Result<PageState, Reply> {
    let reading = state.clone();
    let room = context.map(str::to_owned);
    let read = blocking(move || live_state(&reading, room.as_deref(), present)).await;
    match read {
        Ok(Ok(page)) => Ok(page),
        Ok(Err(detail)) | Err(detail) => Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            &format!("this gate could not read its own state for the page: {detail}"),
            state.note(),
        )),
    }
}

fn serve(assembled: Result<String, String>, state: &GateState) -> Reply {
    match assembled {
        Ok(document) => html(document, state.note()),
        Err(detail) => error(
            StatusCode::SERVICE_UNAVAILABLE,
            &format!("this gate could not render its own page: {detail}"),
            state.note(),
        ),
    }
}
