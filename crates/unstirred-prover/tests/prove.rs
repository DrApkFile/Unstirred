//! Proving, signing and packaging. **Synthetic:** the wallet, its notes and the nullifier set at
//! h1 are all built in this process (see `harness/mod.rs`); the nullifier set is the delegation
//! circuit's own reference tree, not a snapshot of any chain. The proof itself is a real K=12
//! Halo2 proof over Valar's unmodified delegation circuit.
//!
//! One proof is made for the whole file, because a real proof costs seconds, and the three things
//! worth checking about a package are all checks on that one package.

mod harness;

use std::error::Error;
use std::sync::OnceLock;

use harness::{manifest, NoteA, SyntheticNullifierSet, SyntheticWallet, TestResult, H0, H1};
use unstirred_core::{binding_message, rk_from_coordinates, BindingStatement, ProofPackage};
use unstirred_prover::{eligible_notes, prove, witnesses_at, HeldNote, OsRng, ProofRequest, Proven, ReadOnlyWallet};
use voting_circuits::delegation::verify_delegation_proof;
use voting_crypto_deps::orchard::keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey};
use voting_crypto_deps::pasta_curves::group::ff::{Field, PrimeField};
use voting_crypto_deps::pasta_curves::pallas;

/// Stands in for the gate's challenge, which the holder passes by hand until the gate issues one.
const CHALLENGE: [u8; 32] = [0x5c; 32];

#[test]
fn a22_a_package_round_trips_through_json_and_the_proof_still_verifies() -> TestResult {
    let made = proved()?;
    let json = made.proven.package.to_json()?;

    let read_back = ProofPackage::parse(&json)?;

    assert_eq!(
        read_back, made.proven.package,
        "a package must survive being written and read"
    );
    let instance = read_back.instance()?;
    verify_delegation_proof(&read_back.proof_bytes()?, &instance)
        .map_err(|e| format!("the proof must still verify against the instance the package carries: {e}"))?;
    assert_eq!(
        read_back.instance.nc_root,
        hex::encode(made.proven.nc_root),
        "the instance must be about the h0 note commitment root the prover pinned"
    );
    assert_eq!(
        read_back.instance.nf_imt_root,
        hex::encode(made.proven.nf_imt_root),
        "the instance must be about the h1 nullifier-set root the prover pinned"
    );
    Ok(())
}

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

/// A first version of the leak scan: the package is the only thing that leaves the holder's
/// machine in M1, and none of what the wallet knows may be in it.
///
/// A needle has to be long enough to be evidence. The byte forms are 16 hex characters or more,
/// so they are scanned against the whole package. A decimal is short and made of characters that
/// are also hex digits, so it would collide with the proof blob by chance; decimals are scanned
/// against every field but the proof, which is where a leaked number would actually appear.
#[test]
fn a02_the_package_carries_no_note_value_tree_position_address_or_nullifier() -> TestResult {
    let made = proved()?;
    let json = made.proven.package.to_json()?;
    let fields = json.replace(&made.proven.package.proof, "<proof>");
    let note = made.note;

    for (what, appearance) in [
        (
            "the note value as eight bytes",
            hex::encode(note.value_zatoshis.to_le_bytes()),
        ),
        ("the note value as a field element", field_hex(note.value_zatoshis)),
        (
            "the tree position as eight bytes",
            hex::encode(note.position.to_le_bytes()),
        ),
        ("the tree position as a field element", field_hex(note.position)),
        ("the note commitment", hex::encode(note.commitment)),
        ("the note's nullifier", hex::encode(made.nullifier)),
        ("the holder's address", hex::encode(made.address)),
    ] {
        assert!(
            appearance.len() >= 16,
            "{what} is too short a needle to be evidence: {appearance}"
        );
        assert!(
            !json.contains(&appearance),
            "{what} appears in the proof package as {appearance}"
        );
    }

    for (what, appearance) in [
        ("the note value in zatoshis", note.value_zatoshis.to_string()),
        ("the mined height", note.mined_height.to_string()),
    ] {
        assert!(
            appearance.len() >= 6,
            "{what} is too short a needle to be evidence: {appearance}"
        );
        assert!(
            !fields.contains(&appearance),
            "{what} appears in a field of the proof package as {appearance}"
        );
    }
    Ok(())
}

fn field_hex(value: u64) -> String {
    hex::encode(pallas::Base::from(value).to_repr())
}

/// One proof, made once for the whole file. A failure is reported to every test that needed it.
fn proved() -> Result<&'static Proved, String> {
    static PROVED: OnceLock<Result<Proved, String>> = OnceLock::new();

    PROVED
        .get_or_init(|| make_one_proof().map_err(|e| format!("{e}")))
        .as_ref()
        .map_err(String::clone)
}

/// A package, plus what the wallet knows about the note behind it, so the leak scan has
/// something to look for.
#[derive(Debug)]
struct Proved {
    proven: Proven,
    note: HeldNote,
    nullifier: [u8; 32],
    address: [u8; 43],
}

fn make_one_proof() -> Result<Proved, Box<dyn Error>> {
    let synthetic = SyntheticWallet::build(NoteA::StaysPut)?;
    let directory = tempfile::tempdir()?;
    synthetic.write_to(directory.path())?;
    let wallet = ReadOnlyWallet::open_with_params(directory.path(), synthetic.network())?;
    let account = synthetic.account()?;
    let spend_key = synthetic.ironwood_spending_key()?;
    let authority = wallet.spend_authority_for(account, spend_key)?;

    let manifest = manifest(H0, H1)?;
    let selected = eligible_notes(&wallet, account, &manifest)?.selection;
    let note = *selected
        .notes
        .first()
        .ok_or("the selection must hold at least one note")?;
    let witnesses = witnesses_at(&wallet, &synthetic.tree_state_at(H0)?, &selected.notes)?;
    let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
    let nullifiers = wallet.ironwood_nullifiers(account, H1, &positions)?;
    let nullifier = nullifiers
        .first()
        .ok_or("the wallet must derive a nullifier for the selected note")?
        .value()
        .to_repr();
    let nullifier_set = SyntheticNullifierSet::with_spent(&[]);

    let proven = prove(
        &wallet,
        account,
        &authority,
        &ProofRequest {
            manifest: &manifest,
            witnesses: &witnesses,
            expected_nf_imt_root: nullifier_set.root_bytes(),
            challenge: CHALLENGE,
        },
        &nullifier_set,
        &mut OsRng,
    )?;

    Ok(Proved {
        proven,
        note,
        nullifier,
        address: FullViewingKey::from(&spend_key)
            .address_at(0u32, Scope::External)
            .to_raw_address_bytes(),
    })
}
