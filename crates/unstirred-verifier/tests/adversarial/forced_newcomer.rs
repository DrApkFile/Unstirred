//! A newcomer who submits anyway. **Synthetic:** the package comes from `store/fixture.rs`, and
//! it is a real proof — real notes, real witnesses, a real binding signature — of a note that did
//! not exist at this room's own h0, built against the later note-commitment root it does appear
//! in. That is what `unstirred knock --force-unqualified-demo` builds for the demo's act 4. The
//! gate decides from the roots it pinned, so the package is refused and the file is untouched.

use unstirred_core::{Code, ProofPackage};

use crate::store::assertions::{admit_made, assert_unconsumed};
use crate::store::fixture::corpus;
use crate::store::support::{clock_at, expect_code, open_gate, snapshot, TestResult, NOW};

/// A03, verifier half: a proof whose note-commitment root is not the one this gate pinned at h0
/// is REFUSED with ROOT_MISMATCH, and the refusal writes nothing — not a marker, not an admission,
/// and the challenge it answered is still unconsumed. The challenge is a real one this gate
/// issued, so nothing but the root check can be doing the refusing.
#[test]
fn a03_a_proof_against_a_later_root_is_refused_root_mismatch_and_writes_nothing() -> TestResult {
    let made = &corpus()?.newcomer;
    let (_directory, mut store) = open_gate()?;
    store.record(&made.challenge, &made.manifest.context, &clock_at(NOW))?;
    let before = snapshot(&store)?;

    expect_code(admit_made(&mut store, made, NOW)?, Code::RootMismatch)?;

    assert_eq!(
        snapshot(&store)?,
        before,
        "a refused newcomer must change nothing in the gate's file"
    );
    assert_unconsumed(&store, &made.challenge, &made.manifest.context)?;
    Ok(())
}

/// The package claims this room's own published window, because that is what its holder was
/// answering. The verifier never reads that claim: it compares the instance's root with the root
/// it pinned itself, and those are the two that differ here.
#[test]
fn a03_the_verifier_refuses_on_its_own_pinned_root_not_on_what_the_package_claims() -> TestResult {
    let made = &corpus()?.newcomer;
    let package = ProofPackage::parse(&made.package_json)?;
    assert_eq!(
        (package.window.h0, package.window.h1),
        (made.manifest.window.h0, made.manifest.window.h1),
        "the package echoes the room's window, so the refusal cannot come from reading it"
    );
    let instance = package.instance()?;
    assert_ne!(
        hex::encode(root_of(&instance)),
        hex::encode(made.roots.nc_root_at_h0),
        "the newcomer's proof must be about a root this gate did not pin"
    );
    Ok(())
}

fn root_of(instance: &voting_circuits::delegation::Instance) -> [u8; 32] {
    use voting_circuits::ff::PrimeField;
    instance.nc_root.to_repr()
}
