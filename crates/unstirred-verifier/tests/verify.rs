//! Verifying proof packages (PRD section 6, steps 1-8). **Synthetic:** the package, its roots
//! and the tip come from `verify/synthetic.rs`; no chain, wallet or server is involved in this
//! target. A01's real testnet admission is recorded separately in `evidence/` and closes on
//! that recorded run. Everything that attacks these checks is in `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod foreign_gate;
    mod tampered_package;
}

mod verify {
    pub mod support;
    pub mod synthetic;
}

use unstirred_verifier::{verify_package, Verdict};
use verify::support::TestResult;
use verify::synthetic::fixture;

/// The verifier half of A01 on synthetic ground: one real proof, admitted by the engine in a
/// fresh process with the roots injected. The real testnet package is verified separately and
/// recorded in `evidence/`.
#[test]
fn a01_a_valid_package_is_admitted() -> TestResult {
    let made = fixture()?;
    let verdict = verify_package(
        &made.manifest_json,
        &made.package_json,
        &made.roots,
        made.tip,
        &made.challenge,
    );
    assert_eq!(
        verdict,
        Verdict::Admitted,
        "a valid package must be admitted, got {verdict:?}"
    );
    Ok(())
}
