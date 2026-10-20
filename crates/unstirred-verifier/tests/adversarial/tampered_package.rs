//! Packages rewritten after they were proved: a changed root, a flipped proof byte, a
//! signature from a stranger's key. **Synthetic:** the package comes from
//! `verify/synthetic.rs`. Each test asserts the code, not just the verdict kind, so a change
//! in the order of the checks shows up here.

use unstirred_core::{binding_message, BindingStatement, Code, ProofPackage};
use unstirred_prover::OsRng;
use unstirred_verifier::verify_package;
use voting_crypto_deps::orchard::keys::{SpendAuthorizingKey, SpendingKey};
use voting_crypto_deps::pasta_curves::group::ff::Field;
use voting_crypto_deps::pasta_curves::pallas;

use crate::verify::support::{expect_code, flip_one_hex_digit, TestResult};
use crate::verify::synthetic::fixture;

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
