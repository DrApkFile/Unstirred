//! A package offered at a gate it was not proved for: another context's manifest, and another
//! challenge than the one this check was handed. **Synthetic:** the package comes from
//! `verify/synthetic.rs`; only the gate around it changes.

use unstirred_core::Code;
use unstirred_verifier::verify_package;

use crate::verify::support::{expect_code, TestResult};
use crate::verify::synthetic::{fixture, CHALLENGE};

/// A25: round_id binds a proof to a gate context and nothing else; another context's manifest
/// recomputes another round_id and the instance no longer belongs.
#[test]
fn a25_a_package_for_another_gate_context_is_refused() -> TestResult {
    let made = fixture()?;
    let foreign_manifest = format!(
        r#"{{"version":"unstirred/v1","network":"test","context":"attic-room",
             "window":{{"h0":{},"h1":{}}},"min_zatoshis":"12500000"}}"#,
        made.manifest.window.h0, made.manifest.window.h1
    );

    expect_code(
        verify_package(
            &foreign_manifest,
            &made.package_json,
            &made.roots,
            made.tip,
            &made.challenge,
        ),
        Code::ContextMismatch,
    )
}

/// A07, the verify step's half: the package must answer the challenge this check was handed.
#[test]
fn a07_a_package_answering_another_challenge_is_refused() -> TestResult {
    let made = fixture()?;
    let another_challenge = [0x9e; 32];
    assert_ne!(
        another_challenge, CHALLENGE,
        "the wrong-challenge test must actually change the challenge"
    );

    expect_code(
        verify_package(
            &made.manifest_json,
            &made.package_json,
            &made.roots,
            made.tip,
            &another_challenge,
        ),
        Code::ChallengeUnknownOrExpired,
    )
}
