#![doc = "The checks a package has to pass, each one refusing with the code the closed set names: the \
gate context behind round_id, the two pinned roots, the challenge this run was handed, and the binding \
signature under rk rebuilt from the instance. None of them reads anything the package says about \
roots, heights, network or context."]

use unstirred_core::{
    binding_message, derive_round_id, rk_from_coordinates, BindingStatement, Code, Manifest, ProofPackage, Refusal,
};
use voting_circuits::delegation::Instance;
use voting_circuits::ff::PrimeField;

use super::{hex_field, PinnedRoots};

pub(super) fn require_same_context(manifest: &Manifest, instance: &Instance) -> Result<(), Refusal> {
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

pub(super) fn require_pinned_roots(instance: &Instance, roots: &PinnedRoots) -> Result<(), Refusal> {
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

pub(super) fn require_challenge(package: &ProofPackage, expected_challenge: &[u8; 32]) -> Result<(), Refusal> {
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

pub(super) fn require_binding_signature(package: &ProofPackage, instance: &Instance) -> Result<(), Refusal> {
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
