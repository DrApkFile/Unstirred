#![doc = "Admission: the verification engine plus the one write. Steps 1-5, a read-only challenge \
check, the binding signature, the Halo2 proof, and only then the admission transaction. A refusal \
from any earlier step leaves the database untouched."]

use unstirred_core::Manifest;
use voting_circuits::ff::PrimeField;

use crate::engine::{authenticate, examine, Examined, PinnedRoots, Verdict};
use crate::store::{challenge_is_open, challenge_refused, AdmissionWrite, Clock, CommitOutcome, Store, StoreError};
use crate::verdict::as_verdict;

/// Admits one package into `store`, or refuses it without writing. A database fault is not a
/// verdict: the caller reports it as a local error and must not treat it as a decision.
pub fn admit(
    manifest: &Manifest,
    package_json: &str,
    roots: &PinnedRoots,
    chain_tip: u32,
    store: &mut Store,
    clock: &impl Clock,
) -> Result<Verdict, StoreError> {
    let examined = match examine(manifest, package_json, roots, chain_tip) {
        Ok(examined) => examined,
        Err(refusal) => return Ok(as_verdict(refusal)),
    };
    let challenge = match examined.package.challenge() {
        Ok(challenge) => challenge,
        Err(refusal) => return Ok(as_verdict(refusal)),
    };
    if !precheck(store, &challenge, &manifest.context, clock.unix_seconds())? {
        return Ok(as_verdict(challenge_refused()));
    }
    if let Err(refusal) = authenticate(&examined.package, &examined.instance) {
        return Ok(as_verdict(refusal));
    }
    let now = instant_the_consume_is_checked_against(clock);
    match store.commit_admission(&admission(&examined, &manifest.context, challenge, now))? {
        CommitOutcome::Admitted => Ok(Verdict::Admitted),
        CommitOutcome::Refused(refusal) => Ok(as_verdict(refusal)),
    }
}

/// The clock read after the proof, not before it: proving may have taken long enough for the
/// challenge to expire, and the admission transaction tests this instant against `expires_at`
/// before it consumes anything.
fn instant_the_consume_is_checked_against(clock: &impl Clock) -> i64 {
    clock.unix_seconds()
}

fn precheck(store: &Store, challenge: &[u8; 32], context: &str, now: i64) -> Result<bool, StoreError> {
    Ok(match store.lookup(challenge, context)? {
        Some(row) => challenge_is_open(&row, now),
        None => false,
    })
}

fn admission(examined: &Examined, context: &str, challenge: [u8; 32], now: i64) -> AdmissionWrite {
    AdmissionWrite {
        context: context.to_owned(),
        challenge,
        markers: examined.instance.gov_null.map(|marker| marker.to_repr()),
        now,
    }
}
