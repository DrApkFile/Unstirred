//! Knocking on a room this gate does not host, and asking for routes it does not serve. The gate
//! answers what it does not have, and nothing about what it does.

use http::Method;

use crate::gate::fixture::{Fixture, TestResult, HOLDERS_ROOM, SECOND_ROOM};
use crate::gate::support::expect_no_verdict;

/// A38: every route of a room that does not exist is 404, and the answer names no room the gate
/// hosts, no marker, no challenge and no verdict. A request to a route the gate does not serve is
/// 404 as well, and the wrong method on a real route is 405.
#[test]
fn a38_an_unknown_room_is_not_found_and_names_nothing_the_gate_holds() -> TestResult {
    let fixture = Fixture::start()?;
    let ghost = "ghost-room";
    for (method, path) in [
        (Method::GET, format!("/rooms/{ghost}/manifest")),
        (Method::POST, format!("/rooms/{ghost}/challenge")),
        (Method::POST, format!("/rooms/{ghost}/admit")),
        (Method::GET, format!("/rooms/{ghost}/log")),
    ] {
        let answer = fixture.client.request(method.clone(), &path, b"{}".to_vec())?;
        let said = expect_no_verdict(&answer, 404)?;
        for secret in [HOLDERS_ROOM, SECOND_ROOM, "marker", "challenge hash", "sqlite"] {
            assert!(
                !said.contains(secret),
                "a 404 for {path} must not mention {secret:?}: {said}"
            );
        }
    }
    expect_no_verdict(&fixture.client.request(Method::GET, "/gate.sqlite", Vec::new())?, 404)?;
    expect_no_verdict(
        &fixture.client.request(Method::GET, "/rooms/holders-room", Vec::new())?,
        404,
    )?;
    expect_no_verdict(
        &fixture
            .client
            .request(Method::GET, "/rooms/holders-room/admit", Vec::new())?,
        405,
    )?;

    let store = fixture.store()?;
    assert!(store.challenges()?.is_empty(), "no room, no challenge");
    assert!(store.markers()?.is_empty());
    Ok(())
}
