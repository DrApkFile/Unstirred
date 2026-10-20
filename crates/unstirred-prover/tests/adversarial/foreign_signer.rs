//! The signature, not the proof, is what shows the spending key was there: a viewing-key
//! holder can build a valid proof (spike S6) and still cannot sign the binding message, and
//! the signature does not carry over to another session key or another challenge.
//! **Synthetic:** the package comes from `prove/proof_fixture.rs`; the proof in it is real.

use unstirred_core::{binding_message, rk_from_coordinates, BindingStatement};
use unstirred_prover::testing::TestResult;
use unstirred_prover::OsRng;
use voting_crypto_deps::orchard::keys::{SpendAuthorizingKey, SpendingKey};
use voting_crypto_deps::pasta_curves::group::ff::Field;
use voting_crypto_deps::pasta_curves::pallas;

use crate::prove::proof_fixture::proved;

/// The prover side of A06: the signature, not the proof, is what shows the spending key was
/// there. A viewing-key holder can make a valid proof (spike S6) but cannot make this signature.
/// The last two assertions are the same rule under A07: the signature is useless in another
/// session or against another challenge.
#[test]
fn a06_a_signer_without_the_spending_key_cannot_produce_a_valid_binding_signature() -> TestResult {
    let made = proved()?;
    let package = &made.proven.package;
    let instance = package.instance()?;
    let rk = rk_from_coordinates(instance.rk_x, instance.rk_y)?;
    let statement = BindingStatement::of(&instance);
    let message = binding_message(&statement, &package.session_pubkey()?, &package.challenge()?);

    rk.verify(&message, &package.binding_signature()?)
        .map_err(|e| format!("the holder's own binding signature must verify under rk: {e}"))?;

    let mut rng = OsRng;
    let without_the_spend_key = SpendingKey::random(&mut rng);
    let forged = SpendAuthorizingKey::from(&without_the_spend_key)
        .randomize(&pallas::Scalar::random(&mut rng))
        .sign(rng, &message);
    assert!(
        rk.verify(&message, &forged).is_err(),
        "a signature made without the holder's spending key must not verify under rk"
    );

    let other_session = binding_message(&statement, &[0x11; 32], &package.challenge()?);
    assert!(
        rk.verify(&other_session, &package.binding_signature()?).is_err(),
        "the signature must not carry over to another session key"
    );
    let other_challenge = binding_message(&statement, &package.session_pubkey()?, &[0x22; 32]);
    assert!(
        rk.verify(&other_challenge, &package.binding_signature()?).is_err(),
        "the signature must not carry over to another challenge"
    );
    Ok(())
}
