//! A holder who answers a challenge this gate never issued. **Synthetic:** the wallet and the
//! roots come from `gate/fixture.rs`; the proof, the challenge store and the refusal are real.

use unstirred_core::Code;

use crate::gate::fixture::{Fixture, TestResult, HOLDERS_ROOM};
use crate::gate::support::expect_code;

/// A36: a package whose challenge the gate never issued is refused before the signature and the
/// proof are even looked at, and the gate writes nothing: no marker, no admission row, and no
/// challenge row for a value it did not choose.
#[test]
fn a36_a_challenge_the_gate_never_issued_is_refused_and_writes_nothing() -> TestResult {
    let fixture = Fixture::start()?;
    let invented = [0x7c; 32];
    let package = fixture.prove_for(HOLDERS_ROOM, invented)?;

    let answer = fixture.client.admit(HOLDERS_ROOM, &package)?;
    expect_code(
        &answer,
        403,
        "REFUSED",
        Code::ChallengeUnknownOrExpired.spec().wire_name,
    )?;

    let store = fixture.store()?;
    assert!(
        store.lookup(&invented, HOLDERS_ROOM)?.is_none(),
        "a gate stores no challenge it did not issue"
    );
    assert!(store.markers()?.is_empty(), "a refused knock stores no marker");
    assert!(store.admissions()?.is_empty(), "a refused knock records no admission");
    assert!(
        store.challenges()?.is_empty(),
        "no challenge was issued in this test, so the file holds none"
    );
    Ok(())
}
