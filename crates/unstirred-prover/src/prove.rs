#![doc = "One proof, end to end: the selected notes' witnesses at h0 and non-membership proofs at \
h1 go into Valar's delegation circuit unchanged, the proof is verified here before anything is \
written, and the binding signature is made under the randomized spend key the circuit published \
as rk. A proof alone is not enough to admit anyone (spike S6): the signature is what shows the \
spending key was present, so this refuses to emit a package whose own signature does not verify."]

use unstirred_core::{binding_message, derive_round_id, rk_from_coordinates, BindingStatement, Manifest, ProofPackage};
use voting_circuits::delegation::{
    build_delegation_bundle, create_delegation_proof, verify_delegation_proof, DelegationBundle,
};
use voting_circuits::ff::{Field, PrimeField};
use voting_circuits::rand::{CryptoRng, Rng, RngCore};
use voting_crypto_deps::orchard::keys::Scope;
use voting_crypto_deps::pasta_curves::pallas;
use zcash_client_sqlite::AccountUuid;
use zcash_protocol::consensus;

use crate::bundle::note_inputs;
use crate::error::ProverError;
use crate::pir::NullifierSetAtH1;
use crate::spend_authority::SpendAuthority;
use crate::wallet::ReadOnlyWallet;
use crate::witness::Witnesses;

/// The system randomness this prover uses, re-exported so a caller cannot hand in an RNG from a
/// different `rand` release than the circuit's own.
pub use voting_circuits::rand::rngs::OsRng;

/// The delegation circuit sends its output note to a voting hotkey. Unstirred delegates no votes
/// and broadcasts nothing, so the output goes to the holder's own address at this diversifier
/// index. Its commitment is a public input, but the commitment is blinded by fresh randomness
/// each time, so two proofs from one wallet share nothing a gate can compare.
const OUTPUT_DIVERSIFIER: u32 = 1;

const SESSION_KEY_BYTES: usize = 32;

/// What one proof is about: the gate, the witnesses at h0, the h1 root the gate pinned, and the
/// challenge this answer is bound to. In milestone M1 the challenge comes from the holder's
/// command line; from M3 the gate issues it.
#[derive(Debug, Clone, Copy)]
pub struct ProofRequest<'a> {
    /// The gate's published rule. Its network and context give the `round_id`, and its window is
    /// what the two roots are pinned at.
    pub manifest: &'a Manifest,
    /// The selected notes' authentication paths at h0, already checked against h0's root.
    pub witnesses: &'a Witnesses,
    /// The height at which the wallet must still report every witnessed note unspent, which is
    /// where its nullifier is read. A holding proof passes the gate's h1: witnessed at h0 and
    /// unspent at h1 is what makes the pair a statement about a window. The one caller that
    /// passes anything else is `knock --force-unqualified-demo`, whose coins did not exist at
    /// the gate's h1 and whose package the gate refuses on the roots.
    pub notes_unspent_at: u32,
    /// The h1 nullifier-set root the gate pinned. The lookup service must be serving this exact
    /// root, or nothing it answers is about this gate.
    pub expected_nf_imt_root: [u8; 32],
    /// The gate's challenge. It goes into the binding message, so a package answers one
    /// challenge at one gate and nothing else.
    pub challenge: [u8; 32],
}

/// A finished package, with the two roots it is about and the tree positions behind it. The
/// positions stay on the holder's side: they are not in the package.
#[derive(Debug)]
pub struct Proven {
    /// The package to hand the gate. This is the only part that leaves the holder's machine.
    pub package: ProofPackage,
    /// The tree positions the proof was made from. They stay here: the package carries none of
    /// them.
    pub positions: Vec<u64>,
    /// The note-commitment root at h0 this proof is about.
    pub nc_root: [u8; 32],
    /// The nullifier-set root at h1 this proof is about.
    pub nf_imt_root: [u8; 32],
}

/// Builds, proves and signs. Every step that could pass off an unchecked value as a proof is
/// checked here first: the nullifier set must be the pinned one, the proof must verify against
/// its own instance, and the signature must verify under rk rebuilt from that instance.
pub fn prove<P: consensus::Parameters>(
    wallet: &ReadOnlyWallet<P>,
    account: AccountUuid,
    authority: &SpendAuthority,
    request: &ProofRequest<'_>,
    nullifier_set: &impl NullifierSetAtH1,
    rng: &mut (impl Rng + CryptoRng),
) -> Result<Proven, ProverError> {
    let manifest = request.manifest;
    require_pinned_root(nullifier_set, request.expected_nf_imt_root)?;
    let nc_root = canonical_field(request.witnesses.root)?;
    let notes = note_inputs(
        wallet,
        account,
        authority.viewing_key(),
        request.witnesses,
        request.notes_unspent_at,
        nullifier_set,
        request.expected_nf_imt_root,
    )?;
    let round_id = derive_round_id(manifest.network, &manifest.context);
    let alpha = pallas::Scalar::random(&mut *rng);
    let van_comm_rand = pallas::Base::random(&mut *rng);
    let DelegationBundle { circuit, instance } = build_delegation_bundle(
        notes.inputs,
        authority.viewing_key(),
        alpha,
        authority.viewing_key().address_at(OUTPUT_DIVERSIFIER, Scope::External),
        round_id,
        nc_root,
        van_comm_rand,
        nullifier_set,
        rng,
        None,
    )
    .map_err(|unbuildable| {
        nullifier_set
            .last_failure()
            .unwrap_or_else(|| ProverError::BundleUnbuildable(format!("{unbuildable}")))
    })?;
    let proof = create_delegation_proof(circuit, &instance).map_err(|e| ProverError::ProvingFailed(format!("{e}")))?;
    verify_delegation_proof(&proof, &instance).map_err(ProverError::ProofDidNotVerifyHere)?;

    let session_pubkey = session_key(rng);
    let message = binding_message(&BindingStatement::of(&instance), &session_pubkey, &request.challenge);
    let signature = authority.randomized_signer(&alpha).sign(&mut *rng, &message);
    rk_from_coordinates(instance.rk_x, instance.rk_y)?
        .verify(&message, &signature)
        .map_err(|_| ProverError::BindingSignatureDidNotVerifyHere)?;

    Ok(Proven {
        package: ProofPackage::new(
            manifest,
            &instance,
            &proof,
            &session_pubkey,
            &request.challenge,
            &signature,
        ),
        positions: notes.positions,
        nc_root: request.witnesses.root,
        nf_imt_root: request.expected_nf_imt_root,
    })
}

/// A proof is only about the window the gate published if its nullifier-set root is the one the
/// gate pinned at h1. Checked before the first note is read.
fn require_pinned_root(nullifier_set: &impl NullifierSetAtH1, expected: [u8; 32]) -> Result<(), ProverError> {
    let serving = nullifier_set.root().to_repr();
    if serving == expected {
        Ok(())
    } else {
        Err(ProverError::NfImtRootMismatch {
            expected: hex::encode(expected),
            provider: hex::encode(serving),
        })
    }
}

fn canonical_field(root: [u8; 32]) -> Result<pallas::Base, ProverError> {
    Option::from(pallas::Base::from_repr(root)).ok_or_else(|| ProverError::NcRootNotCanonical(hex::encode(root)))
}

/// A fresh 32-byte session key, generated per proof and signed over, so a package cannot be
/// replayed into another session (spike S6). In M1 it is an opaque random value with no private
/// half in use; M3 gives the gate a session to bind it to.
fn session_key(rng: &mut impl RngCore) -> [u8; SESSION_KEY_BYTES] {
    let mut session = [0u8; SESSION_KEY_BYTES];
    rng.fill_bytes(&mut session);
    session
}
