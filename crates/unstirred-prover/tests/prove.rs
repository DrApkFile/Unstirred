//! Proving, signing and packaging. **Synthetic:** the wallet, its notes and the nullifier set
//! at h1 are all built in this process (see `unstirred_prover::testing`); the nullifier set is
//! the delegation circuit's own reference tree, not a snapshot of any chain. The proof itself
//! is a real K=12 Halo2 proof over Valar's unmodified delegation circuit, made once for the
//! whole target because it costs seconds. The attack on its signature is in
//! `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod foreign_signer;
}

mod prove {
    pub mod proof_fixture;
}

use prove::proof_fixture::proved;
use unstirred_core::ProofPackage;
use unstirred_prover::testing::TestResult;
use voting_circuits::delegation::verify_delegation_proof;
use voting_crypto_deps::pasta_curves::group::ff::PrimeField;
use voting_crypto_deps::pasta_curves::pallas;

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
