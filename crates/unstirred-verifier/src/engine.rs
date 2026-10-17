#![doc = "The verification engine (PRD section 6). `verify_package` runs steps 1-8 over an injected \
challenge and records nothing. `examine` and `authenticate` are the same checks split where the \
admission path inserts its read-only challenge test and, only after the proof verifies, its one \
write. Pure by construction: roots, tip and challenge are injected, and this file never reads a \
root, height, network or context from the package, and never touches the network or a clock."]

use unstirred_core::{
    binding_message, check_window, derive_round_id, parse_manifest, rk_from_coordinates, BindingStatement, Code,
    Manifest, ProofPackage, Refusal,
};
use voting_circuits::delegation::{verify_delegation_proof, Instance};
use voting_circuits::ff::PrimeField;

use crate::verdict::as_verdict;

/// The two roots a gate pins for its window: the Ironwood note-commitment root at h0 and the
/// Ironwood nullifier-set root at h1. The h0 root comes from the root authority. The h1 root is
/// pinned by the operator until nf-server `verify-root` lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedRoots {
    pub nc_root_at_h0: [u8; 32],
    pub nf_imt_root_at_h1: [u8; 32],
}

/// What checking one package produced. `Refused` and `Inconclusive` both carry a `Refusal` from
/// the closed code set; which verdict kind a code maps to is fixed in `unstirred-core`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Admitted,
    Refused(Refusal),
    Inconclusive(Refusal),
}

/// A package that has passed the manifest, window, context and pinned-root checks (steps 2-5).
/// The challenge, the signature and the proof are still unchecked.
#[derive(Debug)]
pub(crate) struct Examined {
    pub package: ProofPackage,
    pub instance: Instance,
}

pub use crate::verdict::verdict_object;

/// Runs steps 1-8 in order against a challenge the caller already chose. This is the
/// no-database path: admission here records nothing.
pub fn verify_package(
    manifest_json: &str,
    package_json: &str,
    roots: &PinnedRoots,
    chain_tip: u32,
    expected_challenge: &[u8; 32],
) -> Verdict {
    match parse_manifest(manifest_json) {
        Ok(manifest) => verify_parsed(&manifest, package_json, roots, chain_tip, expected_challenge),
        Err(refusal) => as_verdict(refusal),
    }
}

/// `verify_package` for a manifest the caller already parsed. The command line parses first
/// because it needs the window and network to know what to ask the root sources for.
pub fn verify_parsed(
    manifest: &Manifest,
    package_json: &str,
    roots: &PinnedRoots,
    chain_tip: u32,
    expected_challenge: &[u8; 32],
) -> Verdict {
    match check_all(manifest, package_json, roots, chain_tip, expected_challenge) {
        Ok(()) => Verdict::Admitted,
        Err(refusal) => as_verdict(refusal),
    }
}

fn check_all(
    manifest: &Manifest,
    package_json: &str,
    roots: &PinnedRoots,
    chain_tip: u32,
    expected_challenge: &[u8; 32],
) -> Result<(), Refusal> {
    let examined = examine(manifest, package_json, roots, chain_tip)?;
    require_challenge(&examined.package, expected_challenge)?;
    authenticate(&examined.package, &examined.instance)
}

/// Steps 2-5. Step 1 (parsing the manifest) belongs to the caller, who already needed it to
/// choose which roots to fetch. A package that cannot be read is `PROOF_INVALID`.
pub(crate) fn examine(
    manifest: &Manifest,
    package_json: &str,
    roots: &PinnedRoots,
    chain_tip: u32,
) -> Result<Examined, Refusal> {
    check_window(manifest.window, chain_tip)?;
    let package = ProofPackage::parse(package_json)?;
    let instance = package.instance()?;
    require_same_context(manifest, &instance)?;
    require_pinned_roots(&instance, roots)?;
    Ok(Examined { package, instance })
}

/// The binding signature under rk from the instance, then the Halo2 proof. Both run only after
/// the challenge has been looked up, and neither of them writes.
pub(crate) fn authenticate(package: &ProofPackage, instance: &Instance) -> Result<(), Refusal> {
    require_binding_signature(package, instance)?;
    let proof = package.proof_bytes()?;
    verify_delegation_proof(&proof, instance)
        .map_err(|e| Refusal::new(Code::ProofInvalid, format!("the delegation proof did not verify: {e}")))
}

fn require_same_context(manifest: &Manifest, instance: &Instance) -> Result<(), Refusal> {
    let round_id = derive_round_id(manifest.network, &manifest.context);
    if round_id == instance.vote_round_id {
        return Ok(());
    }
    Err(Refusal::new(
        Code::ContextMismatch,
        format!(
            "the instance was proven for another gate context (round id {}, not this gate's); \
             heights are never part of a context, so this proof belongs somewhere else",
            hex_field(instance.vote_round_id)
        ),
    ))
}

fn require_pinned_roots(instance: &Instance, roots: &PinnedRoots) -> Result<(), Refusal> {
    let nc_root = instance.nc_root.to_repr();
    let nf_imt_root = instance.nf_imt_root.to_repr();
    if nc_root == roots.nc_root_at_h0 && nf_imt_root == roots.nf_imt_root_at_h1 {
        return Ok(());
    }
    let mut detail = String::from("the instance does not match this gate's pinned roots: ");
    if nc_root != roots.nc_root_at_h0 {
        detail.push_str(&format!(
            "the note-commitment root at h0 is pinned as {} but the instance carries {}",
            hex::encode(roots.nc_root_at_h0),
            hex::encode(nc_root)
        ));
    }
    if nf_imt_root != roots.nf_imt_root_at_h1 {
        if nc_root != roots.nc_root_at_h0 {
            detail.push_str("; ");
        }
        detail.push_str(&format!(
            "the nullifier-set root at h1 is pinned as {} but the instance carries {}",
            hex::encode(roots.nf_imt_root_at_h1),
            hex::encode(nf_imt_root)
        ));
    }
    detail.push_str(". Roots carried by the package are diagnostics only and are never used");
    Err(Refusal::new(Code::RootMismatch, detail))
}

fn require_challenge(package: &ProofPackage, expected_challenge: &[u8; 32]) -> Result<(), Refusal> {
    let answered = package.challenge()?;
    if &answered == expected_challenge {
        Ok(())
    } else {
        Err(Refusal::new(
            Code::ChallengeUnknownOrExpired,
            "the package answers a different challenge than the one this check was given",
        ))
    }
}

fn require_binding_signature(package: &ProofPackage, instance: &Instance) -> Result<(), Refusal> {
    let session_pubkey = package.session_pubkey()?;
    let challenge = package.challenge()?;
    let signature = package.binding_signature()?;
    let rk = rk_from_coordinates(instance.rk_x, instance.rk_y)?;
    let message = binding_message(&BindingStatement::of(instance), &session_pubkey, &challenge);
    rk.verify(&message, &signature).map_err(|_| {
        Refusal::new(
            Code::BadSignature,
            "the binding signature does not verify under the randomized key the circuit published; \
             a viewing-key holder can build the proof but cannot sign this message",
        )
    })
}

pub(crate) fn hex_field(value: voting_crypto_deps::pasta_curves::pallas::Base) -> String {
    hex::encode(value.to_repr())
}
