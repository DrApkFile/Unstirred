// Unstirred spike S6 (spend-authority binding). Shared helpers copied from split_height_spike.rs.
// Original header: Unstirred spike S2/S3: delegation proof with nc_root pinned to H0 and nf_imt_root pinned to H1.
// Run: cargo test --release --no-default-features --features lrz --test split_height_spike -- --ignored --nocapture

use incrementalmerkletree::{Hashable, Level};
use voting_circuits::delegation::{
    build_delegation_bundle, create_delegation_proof, verify_delegation_proof, ImtProvider,
    RealNoteInput, SpacedLeafImtProvider,
};
use voting_circuits::ff::{Field, PrimeField};
use voting_circuits::rand::rngs::OsRng;
use voting_crypto_deps::orchard::{
    keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    primitives::redpallas::{self, SpendAuth},
    note::{commitment::ExtractedNoteCommitment, Note, NoteVersion, Rho},
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
};
use voting_crypto_deps::pasta_curves::pallas;

const DEPTH: usize = 32;

struct Tree {
    root: pallas::Base,
    paths: Vec<MerklePath>,
}

fn build_tree(leaves: &[MerkleHashOrchard]) -> Tree {
    let mut layers: Vec<Vec<MerkleHashOrchard>> = vec![leaves.to_vec()];
    for level in 0..DEPTH {
        let below = &layers[level];
        let mut above = Vec::with_capacity(below.len().div_ceil(2));
        for pair in below.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 {
                pair[1]
            } else {
                MerkleHashOrchard::empty_root(Level::from(level as u8))
            };
            above.push(MerkleHashOrchard::combine(Level::from(level as u8), &left, &right));
        }
        layers.push(above);
    }
    let root = layers[DEPTH][0].inner();
    let paths = (0..leaves.len())
        .map(|position| {
            let mut auth = [MerkleHashOrchard::empty_leaf(); DEPTH];
            let mut index = position;
            for level in 0..DEPTH {
                let sibling = index ^ 1;
                auth[level] = layers[level]
                    .get(sibling)
                    .copied()
                    .unwrap_or_else(|| MerkleHashOrchard::empty_root(Level::from(level as u8)));
                index >>= 1;
            }
            MerklePath::from_parts(position as u32, auth)
        })
        .collect();
    Tree { root, paths }
}

fn leaf(note: &Note) -> MerkleHashOrchard {
    MerkleHashOrchard::from_cmx(&ExtractedNoteCommitment::from(note.commitment()))
}

fn new_note(fvk: &FullViewingKey, value: u64, rng: &mut OsRng) -> Note {
    let (_, _, dummy) = Note::dummy(rng, None, NoteVersion::V3);
    Note::new(
        fvk.address_at(0u32, Scope::External),
        NoteValue::from_raw(value),
        Rho::from_nf_old(dummy.nullifier(fvk)),
        NoteVersion::V3,
        rng,
    )
}


/// Verifier side: rebuild rk from the public instance coordinates (never from the prover's message).
fn rk_from_instance(inst: &voting_circuits::delegation::Instance) -> redpallas::VerificationKey<SpendAuth> {
    let mut bytes = inst.rk_x.to_repr();
    if bool::from(inst.rk_y.is_odd()) {
        bytes[31] |= 0x80;
    }
    redpallas::VerificationKey::try_from(bytes).expect("rk decodes")
}

/// Message binds the proof's public statement to a gate context, a session key and a fresh challenge.
fn binding_message(inst: &voting_circuits::delegation::Instance, session: &[u8; 32], challenge: &[u8; 32]) -> Vec<u8> {
    let mut m = b"unstirred/v1/bind".to_vec();
    for f in [inst.vote_round_id, inst.nc_root, inst.nf_imt_root, inst.van_comm] {
        m.extend_from_slice(&f.to_repr());
    }
    for g in inst.gov_null {
        m.extend_from_slice(&g.to_repr());
    }
    m.extend_from_slice(session);
    m.extend_from_slice(challenge);
    m
}

fn verifier_accepts(inst: &voting_circuits::delegation::Instance, session: &[u8; 32], challenge: &[u8; 32], sig: &redpallas::Signature<SpendAuth>) -> bool {
    rk_from_instance(inst).verify(&binding_message(inst, session, challenge), sig).is_ok()
}

#[test]
#[ignore = "real K=12 proof; slow"]
fn spend_authority_binding_accepts_holder_and_rejects_viewing_key_holder_and_replay() {
    let mut rng = OsRng;
    let sk = SpendingKey::random(&mut rng);
    let fvk: FullViewingKey = (&sk).into();
    let note = new_note(&fvk, 13_000_000, &mut rng);
    let tree = build_tree(&[leaf(&note)]);
    let imt = SpacedLeafImtProvider::with_extra_nullifiers(&[]);
    let context = pallas::Base::random(&mut rng);
    let alpha = pallas::Scalar::random(&mut rng);

    // Built with the FULL VIEWING KEY only: no spending key involved.
    let input = RealNoteInput {
        note,
        fvk: fvk.clone(),
        merkle_path: tree.paths[0].clone(),
        imt_proof: imt.non_membership_proof(note.nullifier(&fvk).inner()).unwrap(),
        scope: Scope::External,
    };
    let bundle = build_delegation_bundle(
        vec![input], &fvk, alpha, fvk.address_at(1u32, Scope::External), context, tree.root,
        pallas::Base::random(&mut rng), &imt, &mut rng, None,
    ).unwrap();
    let proof = create_delegation_proof(bundle.circuit, &bundle.instance).unwrap();
    verify_delegation_proof(&proof, &bundle.instance).expect("proof alone verifies");
    println!("S6 FINDING: a viewing-key holder can produce a VALID proof alone -> signature check is mandatory");

    let inst = bundle.instance;
    let rk_check = rk_from_instance(&inst);
    let expected_vk: [u8; 32] = (&redpallas::VerificationKey::from(&SpendAuthorizingKey::from(&sk).randomize(&alpha))).into();
    assert_eq!(<[u8; 32]>::from(&rk_check), expected_vk, "rk rebuilt from instance matches holder's randomized key");

    let session_a = [7u8; 32];
    let session_b = [9u8; 32];
    let challenge = [1u8; 32];

    let holder_sig = SpendAuthorizingKey::from(&sk).randomize(&alpha).sign(&mut rng, &binding_message(&inst, &session_a, &challenge));
    assert!(verifier_accepts(&inst, &session_a, &challenge, &holder_sig));
    println!("S6 PASS: holder's spend-key signature accepted");

    let attacker_sk = SpendingKey::random(&mut rng);
    let forged = SpendAuthorizingKey::from(&attacker_sk).randomize(&alpha).sign(&mut rng, &binding_message(&inst, &session_a, &challenge));
    assert!(!verifier_accepts(&inst, &session_a, &challenge, &forged));
    println!("S6 PASS: viewing-key-only attacker (no spend key) rejected (BAD_SIGNATURE)");

    assert!(!verifier_accepts(&inst, &session_b, &challenge, &holder_sig));
    assert!(!verifier_accepts(&inst, &session_a, &[2u8; 32], &holder_sig));
    println!("S6 PASS: replay to another session or stale challenge rejected");
}
