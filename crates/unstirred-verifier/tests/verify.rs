//! Verifying proof packages (PRD section 6, steps 1-8). **Synthetic:** the package, its roots
//! and the tip all come from `verify/synthetic.rs`; no chain, wallet or server is involved in
//! this file except the unreachable-localhost check, which names no holder data. A01's real
//! testnet admission is recorded separately in `evidence/` and closes on that recorded run.

mod verify {
    pub mod synthetic;
}

use std::error::Error;

use unstirred_core::{binding_message, BindingStatement, Code, ProofPackage};
use unstirred_prover::OsRng;
use unstirred_verifier::{fetch_h0_root_and_tip, verify_package, Verdict};
use verify::synthetic::{fixture, CHALLENGE};
use voting_crypto_deps::orchard::keys::{SpendAuthorizingKey, SpendingKey};
use voting_crypto_deps::pasta_curves::group::ff::Field;
use voting_crypto_deps::pasta_curves::pallas;

type TestResult = Result<(), Box<dyn Error>>;

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

/// A09: roots printed inside the package are diagnostics. The verifier decides from its pinned
/// roots, so a package claiming a different h0 root is a root mismatch, not a new pin.
#[test]
fn a09_a_root_changed_inside_the_package_is_a_root_mismatch() -> TestResult {
    let made = fixture()?;
    let mut rewritten = ProofPackage::parse(&made.package_json)?;
    let claimed = flip_one_hex_digit(&rewritten.instance.nc_root);
    rewritten.instance.nc_root = claimed;
    let tampered = rewritten.to_json()?;

    expect_code(
        verify_package(&made.manifest_json, &tampered, &made.roots, made.tip, &made.challenge),
        Code::RootMismatch,
    )
}

/// A11: the binding signature does not cover the proof bytes, so a tampered proof passes steps
/// 1-7 and dies at the circuit.
#[test]
fn a11_a_one_byte_tampered_proof_is_refused() -> TestResult {
    let made = fixture()?;
    let mut rewritten = ProofPackage::parse(&made.package_json)?;
    let mut bytes = rewritten.proof_bytes()?;
    bytes[0] ^= 0x01;
    rewritten.proof = hex::encode(bytes);
    let tampered = rewritten.to_json()?;

    expect_code(
        verify_package(&made.manifest_json, &tampered, &made.roots, made.tip, &made.challenge),
        Code::ProofInvalid,
    )
}

/// A06, verifier half: a package whose binding signature was made by a key that does not control
/// the notes is refused. The proof itself is untouched and still valid; the signature is the
/// line it cannot cross.
#[test]
fn a06_a_signature_by_a_foreign_key_is_refused() -> TestResult {
    let made = fixture()?;
    let mut rewritten = ProofPackage::parse(&made.package_json)?;
    let instance = rewritten.instance()?;
    let message = binding_message(
        &BindingStatement::of(&instance),
        &rewritten.session_pubkey()?,
        &rewritten.challenge()?,
    );
    let mut rng = OsRng;
    let foreign = SpendingKey::random(&mut rng);
    let forged = SpendAuthorizingKey::from(&foreign)
        .randomize(&pallas::Scalar::random(&mut rng))
        .sign(rng, &message);
    rewritten.binding_signature = hex::encode(<[u8; 64]>::from(&forged));
    let signed_by_a_stranger = rewritten.to_json()?;

    expect_code(
        verify_package(
            &made.manifest_json,
            &signed_by_a_stranger,
            &made.roots,
            made.tip,
            &made.challenge,
        ),
        Code::BadSignature,
    )
}

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

/// A26: a root source that cannot be reached is INCONCLUSIVE and never a verdict of any other
/// kind. The only network this test touches is localhost on a port nothing listens on.
#[test]
fn a26_an_unreachable_root_source_is_inconclusive_never_admitted() -> TestResult {
    let refusal = match fetch_h0_root_and_tip("http://127.0.0.1:1", unstirred_core::Network::Test, 4_505_790) {
        Err(refusal) => refusal,
        Ok(roots) => return Err(format!("nothing listens on 127.0.0.1:1, yet the read returned {roots:?}").into()),
    };
    assert_eq!(
        refusal.code,
        Code::RootSourceUnavailable,
        "an unreachable root source must map to ROOT_SOURCE_UNAVAILABLE, got {refusal:?}"
    );
    assert_eq!(
        refusal.code.spec().kind,
        unstirred_core::VerdictKind::Inconclusive,
        "ROOT_SOURCE_UNAVAILABLE must be inconclusive"
    );
    Ok(())
}

fn expect_code(verdict: Verdict, code: Code) -> TestResult {
    match verdict {
        Verdict::Refused(refusal) => {
            assert_eq!(
                refusal.code, code,
                "expected {code}, got {} ({})",
                refusal.code, refusal.detail
            );
            Ok(())
        }
        other => Err(format!("expected a refusal with {code}, got {other:?}").into()),
    }
}

/// Changes one hex digit without changing the field's length, so the package still parses and
/// the refusal has to come from the check under test, not from the reader.
fn flip_one_hex_digit(text: &str) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let first = chars.first().copied().unwrap_or('0');
    let replacement = if first == '0' { '1' } else { '0' };
    if let Some(slot) = chars.first_mut() {
        *slot = replacement;
    }
    chars.into_iter().collect()
}
