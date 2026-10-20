//! A body too large to be a proof package. Nothing about it is parsed, verified or stored, and
//! the gate claims no verdict about a holding it was never shown.

use unstirred_gate::MAX_ADMIT_BODY_BYTES;

use crate::gate::fixture::{Fixture, TestResult, HOLDERS_ROOM};
use crate::gate::support::expect_no_verdict;

/// A37: an admit body over the limit is 413, with a message and no verdict, and the gate's file is
/// untouched. A real package is about 24 KB; the limit is an order of magnitude above that.
#[test]
fn a37_an_oversized_admit_body_is_rejected_before_anything_is_read() -> TestResult {
    let fixture = Fixture::start()?;
    let too_large = "a".repeat(MAX_ADMIT_BODY_BYTES + 1);

    let answer = fixture.client.admit(HOLDERS_ROOM, &too_large)?;
    let said = expect_no_verdict(&answer, 413)?;
    assert!(
        said.contains(&MAX_ADMIT_BODY_BYTES.to_string()),
        "the gate says what it accepts: {said}"
    );

    let store = fixture.store()?;
    assert!(store.markers()?.is_empty(), "nothing oversized is ever stored");
    assert!(store.admissions()?.is_empty());
    assert!(
        fixture
            .log
            .lines()
            .iter()
            .all(|line| !line.contains("aaaaaaaaaaaaaaaa")),
        "no part of a rejected body reaches the log"
    );

    let within_limit = fixture.client.admit(HOLDERS_ROOM, "{\"version\":\"unstirred/v1\"}")?;
    assert_eq!(
        within_limit.status, 403,
        "a body under the limit is read and judged as a package: {}",
        within_limit.body
    );
    Ok(())
}
