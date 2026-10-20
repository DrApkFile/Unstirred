//! The gate over HTTP. **Synthetic** wherever a package is involved: the wallet, the tree and the
//! nullifier set come from `gate/fixture.rs`, and the roots the gate pins are that process's own.
//! The socket, the routes, the body limit, the store and the verification engine are the real
//! ones. Everything that attacks the service is in `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod foreign_challenge;
    mod oversized_body;
    mod unknown_room;
}

mod gate {
    pub mod fixture;
    pub mod roots;
    pub mod support;
}

use gate::fixture::{Fixture, TestResult, HOLDERS_ROOM, SECOND_ROOM};
use gate::support::{contains_address, expect_code, expect_verdict, json_of, require_absent_from_log};
use unstirred_prover::testing::{H0, H1};

/// A34: the whole knock over HTTP. The holder reads the room's manifest, takes a challenge the
/// gate issued, proves against both, posts the package and is admitted. Afterwards the room's log
/// shows the five markers and the verdict, and the gate's own log carries neither the challenge
/// nor any part of the package.
#[test]
fn a34_a_full_http_knock_is_admitted_and_the_room_log_shows_it() -> TestResult {
    let fixture = Fixture::start()?;
    let manifest = fixture.client.manifest(HOLDERS_ROOM)?;
    assert_eq!(manifest.context, HOLDERS_ROOM);
    assert_eq!((manifest.window.h0, manifest.window.h1), (H0, H1));
    assert_eq!(manifest.min_zatoshis, "12500000");

    let challenge = fixture.client.challenge(HOLDERS_ROOM)?;
    let challenge_hex = hex::encode(challenge.raw);
    assert_eq!(challenge_hex.len(), 64);
    assert_eq!(
        challenge.expires_at, "2023-11-14T22:23:20Z",
        "the expiry must be the stored one, ten minutes after the gate's clock"
    );

    let package = fixture.prove_for(HOLDERS_ROOM, challenge.raw)?;
    let answer = fixture.client.admit(HOLDERS_ROOM, &package)?;
    let verdict = expect_verdict(&answer, 200, "ADMITTED")?;
    assert_eq!(verdict.code, None, "an admission names no code");
    assert_eq!(verdict.note, None, "two agreeing sources carry no single-source label");

    let room_log = json_of(&fixture.client.room_log(HOLDERS_ROOM)?)?;
    assert_eq!(
        room_log["markers"].as_array().map(Vec::len),
        Some(5),
        "one admission stores the proof's five markers: {room_log}"
    );
    assert_eq!(room_log["verdicts"][0]["verdict"], "ADMITTED");
    assert!(
        !room_log.to_string().contains(&challenge_hex),
        "a room log must not carry the challenge it consumed"
    );
    let other_room = json_of(&fixture.client.room_log(SECOND_ROOM)?)?;
    assert_eq!(
        other_room["markers"].as_array().map(Vec::len),
        Some(0),
        "an admission at one room writes nothing at another: {other_room}"
    );

    let lines = fixture.log.lines();
    require_absent_from_log(&lines, &challenge_hex, "the raw challenge")?;
    let middle = package.len() / 2;
    require_absent_from_log(&lines, &package[middle..middle + 64], "a slice of the package body")?;
    Ok(())
}

/// A35: the same coins, knocked twice at one room. The second knock has its own challenge and its
/// own proof, and is refused because a marker of those coins is already stored. Re-posting the
/// admitted package is the same refusal, and neither attempt writes anything.
#[test]
fn a35_a_second_knock_with_the_same_coins_is_already_used_in_context() -> TestResult {
    let fixture = Fixture::start()?;
    let first_challenge = fixture.client.challenge(HOLDERS_ROOM)?;
    let first = fixture.prove_for(HOLDERS_ROOM, first_challenge.raw)?;
    expect_verdict(&fixture.client.admit(HOLDERS_ROOM, &first)?, 200, "ADMITTED")?;

    let store = fixture.store()?;
    let markers_after_first = store.markers()?;
    let admissions_after_first = store.admissions()?;
    assert_eq!(markers_after_first.len(), 5);

    let second_challenge = fixture.client.challenge(HOLDERS_ROOM)?;
    let second = fixture.prove_for(HOLDERS_ROOM, second_challenge.raw)?;
    expect_code(
        &fixture.client.admit(HOLDERS_ROOM, &second)?,
        403,
        "REFUSED",
        "ALREADY_USED_IN_CONTEXT",
    )?;
    expect_code(
        &fixture.client.admit(HOLDERS_ROOM, &first)?,
        403,
        "REFUSED",
        "ALREADY_USED_IN_CONTEXT",
    )?;

    let store = fixture.store()?;
    assert_eq!(store.markers()?, markers_after_first, "a refusal writes no marker");
    assert_eq!(
        store.admissions()?,
        admissions_after_first,
        "a refusal records no admission"
    );
    let unconsumed = store
        .lookup(&second_challenge.raw, HOLDERS_ROOM)?
        .ok_or("the second challenge was not stored")?;
    assert_eq!(
        unconsumed.consumed_at, None,
        "the refused knock must leave its own challenge unconsumed"
    );
    Ok(())
}

/// A39: what a gate says about who knocked. Its log carries one line per request with no address
/// in it, no request body and no challenge; the only address it ever prints is the one it bound
/// itself, at startup. The responses carry no address either.
#[test]
fn a39_no_log_line_or_response_carries_an_address_a_body_or_a_challenge() -> TestResult {
    let fixture = Fixture::start()?;
    let body =
        "{\"version\":\"unstirred/v1\",\"context\":\"holders-room\",\"marker-canary\":\"UNSTIRRED-CANARY-9f3c\"}";
    let challenge = fixture.client.challenge(HOLDERS_ROOM)?;
    let rooms = fixture.client.rooms()?;
    let posted = fixture.client.admit(HOLDERS_ROOM, body)?;
    let room_log = fixture.client.room_log(HOLDERS_ROOM)?;

    let lines = fixture.log.lines();
    let addressed: Vec<&String> = lines.iter().filter(|line| contains_address(line)).collect();
    assert_eq!(
        addressed.len(),
        1,
        "the gate may print one address, its own, at startup; it printed {addressed:?}"
    );
    assert!(
        addressed[0].starts_with("gate listening on "),
        "the only address a gate prints is its listen address, not {:?}",
        addressed[0]
    );
    require_absent_from_log(&lines, "UNSTIRRED-CANARY-9f3c", "the posted body")?;
    require_absent_from_log(&lines, &hex::encode(challenge.raw), "the issued challenge")?;
    for answer in [&rooms, &posted, &room_log] {
        assert!(
            !contains_address(&answer.body),
            "a gate response carries no address: {}",
            answer.body
        );
    }
    assert!(
        lines
            .iter()
            .any(|line| line.contains("POST /rooms/holders-room/admit -> ")),
        "the gate records that a request was answered, and what it answered: {lines:?}"
    );
    Ok(())
}

/// The room list is the gate's own description of itself: every room, its window, its threshold
/// and the h1 root its operator pinned, labelled as operator-pinned.
#[test]
fn the_room_list_names_every_room_and_how_its_h1_root_was_pinned() -> TestResult {
    let fixture = Fixture::start()?;
    let listing = json_of(&fixture.client.rooms()?)?;
    let rooms = listing["rooms"].as_array().ok_or("no rooms array")?;
    assert_eq!(rooms.len(), 2, "both rooms are hosted: {listing}");
    let contexts: Vec<&str> = rooms.iter().filter_map(|room| room["context"].as_str()).collect();
    assert_eq!(contexts, vec![HOLDERS_ROOM, SECOND_ROOM]);
    for room in rooms {
        assert_eq!(room["network"], "test");
        assert_eq!(room["window"]["h0"], H0);
        assert_eq!(room["min_zatoshis"], "12500000");
        assert_eq!(room["h1_root_provenance"], "operator-pinned");
        assert_eq!(
            room["h1_root"].as_str().map(str::len),
            Some(64),
            "the pinned h1 root is published so a judge can check it: {room}"
        );
    }
    Ok(())
}
