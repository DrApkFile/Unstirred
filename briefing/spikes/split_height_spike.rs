// Unstirred spike S2/S3: delegation proof with nc_root pinned to H0 and nf_imt_root pinned to H1.
// Run: cargo test --release --no-default-features --features lrz --test split_height_spike -- --ignored --nocapture

use incrementalmerkletree::{Hashable, Level};
use voting_circuits::delegation::{
    build_delegation_bundle, create_delegation_proof, verify_delegation_proof, ImtProvider,
    RealNoteInput, SpacedLeafImtProvider,
};
use voting_circuits::ff::Field;
use voting_circuits::rand::rngs::OsRng;
use voting_crypto_deps::orchard::{
    keys::{FullViewingKey, Scope, SpendingKey},
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

struct Wallet {
    fvk: FullViewingKey,
    held: Note,
    spent_in_window: Note,
    too_new: Note,
    tree_h0: Tree,
    tree_h1: Tree,
    imt_h1: SpacedLeafImtProvider,
}

fn scenario(rng: &mut OsRng) -> Wallet {
    let sk = SpendingKey::random(rng);
    let fvk: FullViewingKey = (&sk).into();
    let held = new_note(&fvk, 13_000_000, rng);
    let spent_in_window = new_note(&fvk, 13_000_000, rng);
    let too_new = new_note(&fvk, 13_000_000, rng);
    let tree_h0 = build_tree(&[leaf(&held), leaf(&spent_in_window)]);
    let tree_h1 = build_tree(&[leaf(&held), leaf(&spent_in_window), leaf(&too_new)]);
    let imt_h1 = SpacedLeafImtProvider::with_extra_nullifiers(&[spent_in_window.nullifier(&fvk).inner()]);
    Wallet { fvk, held, spent_in_window, too_new, tree_h0, tree_h1, imt_h1 }
}

fn bundle_for(
    w: &Wallet,
    note: &Note,
    path: MerklePath,
    nc_root: pallas::Base,
    imt: &SpacedLeafImtProvider,
    context: pallas::Base,
    rng: &mut OsRng,
) -> voting_circuits::delegation::DelegationBundle {
    let imt_proof = imt.non_membership_proof(note.nullifier(&w.fvk).inner()).expect("nullifier unspent at H1");
    let input = RealNoteInput { note: *note, fvk: w.fvk.clone(), merkle_path: path, imt_proof, scope: Scope::External };
    build_delegation_bundle(
        vec![input],
        &w.fvk,
        pallas::Scalar::random(&mut *rng),
        w.fvk.address_at(1u32, Scope::External),
        context,
        nc_root,
        pallas::Base::random(&mut *rng),
        imt,
        rng,
        None,
    )
    .expect("bundle")
}

#[test]
#[ignore = "real K=12 proofs; slow"]
fn held_note_proves_over_split_window_and_too_new_note_fails_against_h0_root() {
    let mut rng = OsRng;
    let w = scenario(&mut rng);
    let context = pallas::Base::random(&mut rng);

    let held = bundle_for(&w, &w.held, w.tree_h0.paths[0].clone(), w.tree_h0.root, &w.imt_h1, context, &mut rng);
    assert_eq!(held.instance.nc_root, w.tree_h0.root);
    assert_eq!(held.instance.nf_imt_root, w.imt_h1.root());
    let proof = create_delegation_proof(held.circuit, &held.instance).expect("prove held note");
    verify_delegation_proof(&proof, &held.instance).expect("S2: split-height proof must verify");
    println!("S2 PASS: note existing at H0 and unspent at H1 verifies with (root@H0, imt@H1)");

    let newcomer = bundle_for(&w, &w.too_new, w.tree_h1.paths[2].clone(), w.tree_h1.root, &w.imt_h1, context, &mut rng);
    let newcomer_proof = create_delegation_proof(newcomer.circuit, &newcomer.instance).expect("prove against its own root");
    let mut pinned_by_verifier = newcomer.instance;
    pinned_by_verifier.nc_root = w.tree_h0.root;
    assert!(
        verify_delegation_proof(&newcomer_proof, &pinned_by_verifier).is_err(),
        "S3: a note created after H0 must not verify against the verifier-pinned H0 root"
    );
    println!("S3 PASS: note created after H0 rejected (COINS_TOO_NEW)");
}

#[test]
fn note_spent_inside_window_has_no_non_membership_proof_at_h1() {
    let mut rng = OsRng;
    let w = scenario(&mut rng);
    let attempt = w.imt_h1.non_membership_proof(w.spent_in_window.nullifier(&w.fvk).inner());
    assert!(attempt.is_err(), "S3: a nullifier revealed by H1 must have no non-membership proof");
    println!("S3 PASS: note spent between H0 and H1 cannot be proven unspent at H1");
}

#[test]
fn markers_repeat_within_a_context_and_differ_across_contexts() {
    let mut rng = OsRng;
    let w = scenario(&mut rng);
    let context_a = pallas::Base::random(&mut rng);
    let context_b = pallas::Base::random(&mut rng);
    let later_imt = SpacedLeafImtProvider::with_extra_nullifiers(&[
        w.spent_in_window.nullifier(&w.fvk).inner(),
        pallas::Base::random(&mut rng),
    ]);

    let a_first = bundle_for(&w, &w.held, w.tree_h0.paths[0].clone(), w.tree_h0.root, &w.imt_h1, context_a, &mut rng);
    let a_other_window = bundle_for(&w, &w.held, w.tree_h1.paths[0].clone(), w.tree_h1.root, &later_imt, context_a, &mut rng);
    let b = bundle_for(&w, &w.held, w.tree_h0.paths[0].clone(), w.tree_h0.root, &w.imt_h1, context_b, &mut rng);

    assert_eq!(a_first.instance.gov_null[0], a_other_window.instance.gov_null[0], "same coins, same context, any window → same marker");
    assert_ne!(a_first.instance.gov_null[0], b.instance.gov_null[0], "same coins, different context → unlinkable marker");
    println!("S3 PASS: marker repeats in one context regardless of window; differs across contexts");
}
