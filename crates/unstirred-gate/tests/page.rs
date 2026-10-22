//! The Holders' Room page: the routes that serve it, the feed it polls, and the recorded acts the
//! demo video is rendered from. Nothing here needs a proof, so this target builds no wallet and
//! pays for no Halo2 proof: what it checks is what the gate publishes to a browser.
//!
//! **Synthetic** wherever a gate is involved — the rooms, the roots and the database are this
//! process's own. The replay acts are the exception: their values come from the recorded
//! real-testnet run, which is why A44 checks them against the files in `evidence/` rather than
//! against anything this process made.

mod gate {
    pub mod page_support;
    pub mod plain;
    pub mod roots;
}

use std::error::Error;

use gate::page_support::{absolute_urls_in_code, network_call_sites, off_origin_markers, parts};
use gate::plain::{start_in, PlainGate, HOLDERS_ROOM, SECOND_ROOM};
use unstirred_core::Code;
use unstirred_gate::{replay_state, Mode, PageState, RECENT_PER_ROOM, REPLAY_ACTS, ROOM_LOG_LIMIT};

type TestResult = Result<(), Box<dyn Error>>;

/// Every act PRD section 9 has, whether or not a run has produced it yet. The gate may serve no
/// page that is not one of them, and `scripts/replay-from-evidence.py` writes the list of the ones
/// it could build from `evidence/`, so this is the only place an act's existence is a decision.
const DEMO_ACTS: [&str; 5] = [
    "act1-rule",
    "act2-admitted",
    "act3-refused",
    "act4-newcomer",
    "act5-second-room",
];

/// A42: the gate serves two kinds of page and nothing else. The picker names every room it hosts;
/// each hosted room has a page of its own carrying that room's live state; a room it does not host
/// is 404 and the answer names nothing the gate holds.
#[test]
fn a42_the_picker_and_one_page_per_room_are_served_and_nothing_else_is() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;

    let picker = gate.client.request(http::Method::GET, "/", Vec::new())?;
    assert_eq!(picker.status, 200, "the picker is the gate's root: {}", picker.body);
    assert!(
        picker.body.starts_with("<!doctype html>"),
        "the picker is one HTML document"
    );
    let picker_state = state_of(&picker.body)?;
    assert_eq!(picker_state.room, None, "the picker is about no single room");
    assert_eq!(picker_state.mode, Mode::Live);
    let contexts: Vec<&str> = picker_state.rooms.iter().map(|room| room.context.as_str()).collect();
    assert_eq!(contexts, vec![HOLDERS_ROOM, SECOND_ROOM]);

    for context in [HOLDERS_ROOM, SECOND_ROOM] {
        let page = gate
            .client
            .request(http::Method::GET, &format!("/room/{context}"), Vec::new())?;
        assert_eq!(page.status, 200, "{context} has a page: {}", page.body);
        let state = state_of(&page.body)?;
        assert_eq!(state.room.as_deref(), Some(context));
        assert_eq!(state.mode, Mode::Live);
        assert_eq!(state.replay, None, "a live page names no act");
        let room = state
            .rooms
            .iter()
            .find(|room| room.context == context)
            .ok_or("the page's own room is not in its state")?;
        assert_eq!(room.limit, ROOM_LOG_LIMIT);
        assert_eq!((room.window.h0, room.window.h1), (gate::plain::H0, gate::plain::H1));
        assert_eq!(room.min_zatoshis, "12500000");
        assert_eq!(room.h1_root_provenance, "operator-pinned");
    }

    let missing = gate
        .client
        .request(http::Method::GET, "/room/no-such-room", Vec::new())?;
    assert_eq!(missing.status, 404);
    for held in [HOLDERS_ROOM, SECOND_ROOM, "marker", "challenge", "verdict", "5a5a"] {
        assert!(
            !missing.body.contains(held),
            "a 404 for an unknown room must not mention {held:?}: {}",
            missing.body
        );
    }
    Ok(())
}

/// A02c: no page this gate serves asks for anything from anywhere else. No script, stylesheet,
/// font, image or frame from another origin, no beacon, and no absolute URL anywhere in the code
/// the browser runs. The root sources the page displays are in its state block, which is data.
#[test]
fn a02c_no_page_the_gate_serves_asks_for_anything_from_anywhere_else() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    let mut pages = vec![gate.client.request(http::Method::GET, "/", Vec::new())?.body];
    pages.push(
        gate.client
            .request(http::Method::GET, &format!("/room/{HOLDERS_ROOM}"), Vec::new())?
            .body,
    );
    for (act, _) in REPLAY_ACTS {
        let state = act_state(act)?;
        let room = state.room.clone().ok_or("an act names no room")?;
        pages.push(
            gate.client
                .request(http::Method::GET, &format!("/room/{room}?replay={act}"), Vec::new())?
                .body,
        );
    }
    for page in &pages {
        assert_eq!(
            off_origin_markers(page),
            Vec::<&str>::new(),
            "a page asked for something from somewhere else"
        );
        let split = parts(page)?;
        assert_eq!(
            absolute_urls_in_code(&split.code),
            Vec::<String>::new(),
            "the page's own code names an absolute URL"
        );
        assert!(
            !split.state.contains('<'),
            "the state block must carry no raw '<', so it cannot end its own script element"
        );
        assert!(
            page.contains("default-src 'none'"),
            "every page carries its own content-security-policy"
        );
    }
    Ok(())
}

/// A43: the recent-verdict feed. It shows a refusal, which the store never writes; every entry is
/// a verdict, a code and a time and nothing else; it is bounded; it writes nothing; and a gate
/// restarted on the same database answers with none of it.
#[test]
fn a43_the_recent_feed_shows_refusals_keeps_three_fields_and_forgets_on_restart() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    assert_eq!(feed(&gate, HOLDERS_ROOM)?.as_array().map(Vec::len), Some(0));

    let refused = gate.client.admit(HOLDERS_ROOM, "{\"not\":\"a package\"}")?;
    assert_eq!(
        refused.status, 403,
        "a body that is not a package is refused: {}",
        refused.body
    );

    let entries = feed(&gate, HOLDERS_ROOM)?;
    let shown = entries.as_array().ok_or("the feed is not a list")?;
    assert_eq!(shown.len(), 1, "the refusal is in the feed: {entries}");
    let entry = shown[0].as_object().ok_or("an entry is not an object")?;
    let mut fields: Vec<&str> = entry.keys().map(String::as_str).collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        vec!["at", "code", "verdict"],
        "an entry is a verdict, a code and a time, and nothing else"
    );
    assert_eq!(entry["verdict"], "REFUSED");
    let code = entry["code"].as_str().ok_or("the refusal names no code")?;
    assert!(
        Code::ALL.into_iter().any(|known| known.spec().wire_name == code),
        "{code:?} is not in the closed code set"
    );
    assert_eq!(
        feed(&gate, SECOND_ROOM)?.as_array().map(Vec::len),
        Some(0),
        "a verdict at one room is not in another room's feed"
    );

    let log = json_of(&gate.client.room_log(HOLDERS_ROOM)?)?;
    assert_eq!(
        log["markers"].as_array().map(Vec::len),
        Some(0),
        "the feed is not a write: a refusal still stores nothing"
    );
    assert_eq!(log["verdicts"].as_array().map(Vec::len), Some(0));

    for _ in 0..(RECENT_PER_ROOM + 5) {
        gate.client.admit(HOLDERS_ROOM, "{\"not\":\"a package\"}")?;
    }
    assert_eq!(
        feed(&gate, HOLDERS_ROOM)?.as_array().map(Vec::len),
        Some(RECENT_PER_ROOM),
        "the feed is bounded at {RECENT_PER_ROOM} per room"
    );

    let restarted = start_in(directory.path())?;
    assert_eq!(
        feed(&restarted, HOLDERS_ROOM)?.as_array().map(Vec::len),
        Some(0),
        "a restarted gate remembers no verdict: the feed is never written to disk"
    );
    Ok(())
}

/// A44: every act the gate ships renders, says the same things `unstirred-core` says, and carries
/// no way to reach the network. An act belongs to one room, and asking for it at another is 404.
#[test]
fn a44_every_replay_act_renders_in_a_document_with_no_network_call_site() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    let names: Vec<&str> = REPLAY_ACTS.iter().map(|(name, _)| *name).collect();
    for recorded in ["act1-rule", "act2-admitted", "act3-refused", "act5-second-room"] {
        assert!(names.contains(&recorded), "{recorded} must be served: {names:?}");
    }
    for name in &names {
        assert!(
            DEMO_ACTS.contains(name),
            "{name:?} is served but is no act of the demo script (PRD section 9): {DEMO_ACTS:?}"
        );
    }

    for (act, _) in REPLAY_ACTS {
        let state = act_state(act)?;
        assert_eq!(state.mode, Mode::Replay);
        assert_eq!(state.replay.as_deref(), Some(act));
        assert_eq!(state.source, "real-testnet");
        assert!(
            state.provenance.iter().any(|line| line.contains("evidence/")),
            "{act} must say on the page which recorded run it came from"
        );
        for (name, words) in &state.codes {
            let known = Code::ALL
                .into_iter()
                .find(|code| code.spec().wire_name == name)
                .ok_or_else(|| format!("{act} names {name:?}, which is not a code"))?
                .spec();
            assert_eq!(words.message, known.message, "{act} restates {name}'s message");
            assert_eq!(words.kind, known.kind.to_string(), "{act} restates {name}'s kind");
            assert_eq!(words.retryable, known.retryable, "{act} restates {name}'s retryable");
        }

        let room = state.room.clone().ok_or("an act names no room")?;
        let page = gate
            .client
            .request(http::Method::GET, &format!("/room/{room}?replay={act}"), Vec::new())?;
        assert_eq!(page.status, 200, "{act} is served for {room}: {}", page.body);
        let split = parts(&page.body)?;
        assert_eq!(
            network_call_sites(&split.code),
            Vec::<&str>::new(),
            "a replayed act has no way to reach the network"
        );
        assert!(
            page.body.contains("connect-src 'none'"),
            "a replayed act forbids the browser to connect anywhere"
        );
        let carried: PageState = serde_json::from_str(split.state.trim())?;
        assert_eq!(carried, state, "the page carries the act, unchanged");
    }

    let wrong_room = gate.client.request(
        http::Method::GET,
        &format!("/room/{HOLDERS_ROOM}?replay=act5-second-room"),
        Vec::new(),
    )?;
    assert_eq!(
        wrong_room.status, 404,
        "an act belongs to one room: {}",
        wrong_room.body
    );
    for asked in ["nope", "../../etc/passwd", "ACT2", ""] {
        let unknown = gate.client.request(
            http::Method::GET,
            &format!("/room/{HOLDERS_ROOM}?replay={asked}"),
            Vec::new(),
        )?;
        assert_eq!(unknown.status, 404, "{asked:?} is not an act: {}", unknown.body);
    }
    Ok(())
}

/// The room log's `limit` is the cap on the answer, not a count of what the gate stores. A room
/// holding nothing answers with the limit and an empty list.
#[test]
fn the_room_log_names_its_limit_and_not_a_count_of_stored_rows() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    let log = json_of(&gate.client.room_log(HOLDERS_ROOM)?)?;
    assert_eq!(log["limit"], ROOM_LOG_LIMIT);
    assert_eq!(log["markers"].as_array().map(Vec::len), Some(0));
    assert!(
        log.get("rows").is_none(),
        "the field that reads as a count of stored rows is gone: {log}"
    );
    Ok(())
}

/// Presentation mode is a way of reading a page, not part of what the page says: `?present=1`
/// works on a live page and on any act, changes `present` and nothing else about the state, and
/// takes the gate's own technical lines off the provenance foot without touching what a verdict
/// says about itself.
#[test]
fn present_mode_is_a_way_of_reading_a_page_and_not_part_of_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    let act = act_state("act3-refused")?;
    assert!(!act.present, "an act on disk is not a presentation");

    let plain = gate.client.request(
        http::Method::GET,
        &format!("/room/{HOLDERS_ROOM}?replay=act3-refused"),
        Vec::new(),
    )?;
    let presented = gate.client.request(
        http::Method::GET,
        &format!("/room/{HOLDERS_ROOM}?replay=act3-refused&present=1"),
        Vec::new(),
    )?;
    assert_eq!(presented.status, 200, "{}", presented.body);
    let plain_state = state_of(&plain.body)?;
    let presented_state = state_of(&presented.body)?;
    assert!(!plain_state.present);
    assert!(presented_state.present, "?present=1 is carried into the state");
    assert_eq!(
        PageState {
            present: false,
            ..presented_state.clone()
        },
        plain_state,
        "presentation mode changes nothing else about the act"
    );

    let technical = plain_state.rooms[0].h1_root.clone();
    assert!(
        plain.body.contains(&technical),
        "the pinned root is on the page a judge reads"
    );
    assert!(
        !presented.body.contains(&format!("h1 nullifier-set root {technical}")),
        "the presented page drops the gate's technical lines"
    );
    assert!(
        presented.body.contains("time derived"),
        "what a verdict says about its own time stays, wherever the page is read"
    );

    let live = gate.client.request(
        http::Method::GET,
        &format!("/room/{HOLDERS_ROOM}?present=1"),
        Vec::new(),
    )?;
    assert_eq!(live.status, 200, "{}", live.body);
    assert!(state_of(&live.body)?.present, "a live page can be presented too");
    Ok(())
}

/// A holder line is an act's own illustration of somebody against the rule, read from that
/// holder's own wallet by the knock that was recorded. A live gate never draws one, because it
/// knows no holder's block range; an act that does draw one draws it on the room it is about and
/// says on the page where it came from; and a line the page could not place is refused rather
/// than drawn wrong.
#[test]
fn a_holder_line_is_an_acts_own_and_is_refused_when_it_is_not_a_range() -> TestResult {
    let directory = tempfile::tempdir()?;
    let gate = start_in(directory.path())?;
    let live = gate
        .client
        .request(http::Method::GET, &format!("/room/{HOLDERS_ROOM}"), Vec::new())?;
    for room in state_of(&live.body)?.rooms {
        assert!(
            room.holder.is_none(),
            "a gate knows no holder's block range, so a live page draws no holder line"
        );
    }
    for (act, _) in REPLAY_ACTS {
        let state = act_state(act)?;
        let about = state.room.clone().ok_or("an act names no room")?;
        for room in &state.rooms {
            let Some(holder) = &room.holder else { continue };
            assert_eq!(
                room.context, about,
                "{act} draws a holder on a room the act is not about"
            );
            assert!(
                holder.from <= holder.to && !holder.label.is_empty(),
                "{act} draws a holder the page cannot place: {holder:?}"
            );
            assert!(
                state.provenance.iter().any(|line| line.contains("wallet")),
                "{act} draws a holder, so it must say on the page whose wallet that line was read from"
            );
        }
    }

    let mut backwards: serde_json::Value = serde_json::from_str(REPLAY_ACTS[0].1)?;
    backwards["rooms"][0]["holder"] = serde_json::json!({ "label": "backwards", "from": 9, "to": 8 });
    let refused: PageState = serde_json::from_value(backwards)?;
    let problem = refused.check().err().ok_or("a backwards holder line was accepted")?;
    assert!(
        problem.contains("backwards"),
        "the refusal says what is wrong: {problem}"
    );
    Ok(())
}

fn act_state(act: &str) -> Result<PageState, Box<dyn Error>> {
    match replay_state(act) {
        Some(Ok(state)) => Ok(state),
        Some(Err(detail)) => Err(format!("the act {act} is not a valid page state: {detail}").into()),
        None => Err(format!("{act} is in the act list but has no state").into()),
    }
}

fn state_of(document: &str) -> Result<PageState, Box<dyn Error>> {
    Ok(serde_json::from_str(parts(document)?.state.trim())?)
}

fn feed(gate: &PlainGate, context: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let answer = gate
        .client
        .request(http::Method::GET, &format!("/rooms/{context}/recent"), Vec::new())?;
    if answer.status != 200 {
        return Err(format!("the feed answered {}: {}", answer.status, answer.body).into());
    }
    let body: serde_json::Value = serde_json::from_str(&answer.body)?;
    Ok(body["verdicts"].clone())
}

fn json_of(answer: &unstirred_gate::Answer) -> Result<serde_json::Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&answer.body)?)
}
