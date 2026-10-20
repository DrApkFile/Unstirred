use std::error::Error;

use unstirred_core::{
    binding_message, build_manifest, check_window, derive_round_id, parse_manifest, BindingStatement, Code, Network,
    VerdictKind, Window, FINALITY_DEPTH,
};
use voting_crypto_deps::pasta_curves::pallas;

const VALID: &str = r#"{
  "version": "unstirred/v1",
  "network": "test",
  "context": "holders-room",
  "window": { "h0": 4505790, "h1": 4530000 },
  "min_zatoshis": "12500000"
}"#;

fn refusal_code(json: &str) -> Option<Code> {
    parse_manifest(json).err().map(|refusal| refusal.code)
}

#[test]
fn a13_accepts_the_holders_room_manifest() {
    let parsed = parse_manifest(VALID);
    assert!(parsed.is_ok(), "{parsed:?}");
}

#[test]
fn a13_refuses_unknown_fields_wrong_version_bad_context_and_other_thresholds() {
    let cases = [
        VALID.replace("\"min_zatoshis\"", "\"extra\": 1, \"min_zatoshis\""),
        VALID.replace("unstirred/v1", "unstirred/v2"),
        VALID.replace("holders-room", "Holders Room"),
        VALID.replace("holders-room", "-room"),
        VALID.replace("holders-room", &"a".repeat(65)),
        VALID.replace("12500000", "100000000"),
        VALID.replace("\"test\"", "\"regtest\""),
        VALID.replace("\"h1\": 4530000", "\"h1\": 4530000, \"h2\": 1"),
    ];
    let accepted: Vec<&String> = cases
        .iter()
        .filter(|json| refusal_code(json) != Some(Code::ManifestInvalid))
        .collect();
    assert!(accepted.is_empty(), "these should be MANIFEST_INVALID: {accepted:#?}");
}

#[test]
fn a13_window_requires_ordered_final_heights_on_the_nullifier_step() {
    let tip = 4_600_000;
    assert_eq!(
        check_window(Window { h0: 10, h1: 10 }, tip).map_err(|r| r.code),
        Err(Code::WindowInvalid)
    );
    assert_eq!(
        check_window(Window { h0: 20, h1: 10 }, tip).map_err(|r| r.code),
        Err(Code::WindowInvalid)
    );
    assert_eq!(
        check_window(Window { h0: 10, h1: 4_530_005 }, tip).map_err(|r| r.code),
        Err(Code::WindowInvalid)
    );
    let not_final = tip - FINALITY_DEPTH + 10;
    assert_eq!(
        check_window(Window { h0: 10, h1: not_final }, tip).map_err(|r| r.code),
        Err(Code::WindowInvalid)
    );
    assert_eq!(
        check_window(
            Window {
                h0: 4_505_790,
                h1: 4_530_000
            },
            tip
        ),
        Ok(())
    );
}

#[test]
fn a12_round_id_depends_on_network_and_context_only() {
    let room = derive_round_id(Network::Test, "holders-room");
    assert_eq!(room, derive_round_id(Network::Test, "holders-room"));
    assert_ne!(room, derive_round_id(Network::Test, "second-room"));
    assert_ne!(room, derive_round_id(Network::Main, "holders-room"));
}

#[test]
fn a12_republishing_a_gate_with_a_new_window_keeps_the_same_round_id() {
    let first = parse_manifest(VALID).map(|m| derive_round_id(m.network, &m.context));
    let moved = VALID.replace("4530000", "4560000");
    let second = parse_manifest(&moved).map(|m| derive_round_id(m.network, &m.context));
    assert!(first.is_ok());
    assert_eq!(first, second);
}

#[test]
fn a14_every_code_has_a_unique_wire_name_message_and_next_action() {
    let mut names: Vec<&str> = Code::ALL.iter().map(|c| c.spec().wire_name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), Code::ALL.len());
    let incomplete: Vec<Code> = Code::ALL
        .into_iter()
        .filter(|c| c.spec().message.is_empty() || c.spec().next_action.is_empty())
        .collect();
    assert!(incomplete.is_empty(), "{incomplete:?}");
}

#[test]
fn a14_unknown_is_never_admitted_root_problems_are_inconclusive() {
    assert_eq!(Code::RootSourcesDisagree.spec().kind, VerdictKind::Inconclusive);
    assert_eq!(Code::RootSourceUnavailable.spec().kind, VerdictKind::Inconclusive);
    assert_eq!(Code::AlreadyUsedInContext.spec().kind, VerdictKind::Refused);
}

#[test]
fn a07_binding_message_changes_with_session_challenge_and_every_statement_field() {
    let base = BindingStatement {
        round_id: pallas::Base::from(1),
        nc_root: pallas::Base::from(2),
        nf_imt_root: pallas::Base::from(3),
        van_comm: pallas::Base::from(4),
        markers: [5, 6, 7, 8, 9].map(pallas::Base::from),
    };
    let session = [7u8; 32];
    let challenge = [1u8; 32];
    let reference = binding_message(&base, &session, &challenge);

    let mut variants = vec![
        binding_message(&base, &[8u8; 32], &challenge),
        binding_message(&base, &session, &[2u8; 32]),
    ];
    let edits: [fn(&mut BindingStatement); 5] = [
        |s| s.round_id = pallas::Base::from(100),
        |s| s.nc_root = pallas::Base::from(100),
        |s| s.nf_imt_root = pallas::Base::from(100),
        |s| s.van_comm = pallas::Base::from(100),
        |s| s.markers[4] = pallas::Base::from(100),
    ];
    for edit in edits {
        let mut changed = base;
        edit(&mut changed);
        variants.push(binding_message(&changed, &session, &challenge));
    }
    let collisions = variants.iter().filter(|m| **m == reference).count();
    assert_eq!(collisions, 0);
}

/// A40: what `unstirred gate init` is allowed to write. The text comes back only when this
/// crate's own parser accepts it, and every value that cannot make a gate is refused with the
/// code that names why.
#[test]
fn a40_build_manifest_writes_only_a_manifest_core_accepts_and_refuses_the_rest() -> Result<(), Box<dyn Error>> {
    let window = Window {
        h0: 4_505_790,
        h1: 4_506_510,
    };
    let written = build_manifest(Network::Test, "holders-room", window)?;
    let parsed = parse_manifest(&written)?;
    assert_eq!(parsed.version, "unstirred/v1");
    assert_eq!(parsed.network, Network::Test);
    assert_eq!(parsed.context, "holders-room");
    assert_eq!(parsed.window, window);
    assert_eq!(parsed.min_zatoshis, "12500000");
    assert_eq!(
        parse_manifest(&build_manifest(Network::Main, "a", window)?)?.network,
        Network::Main
    );

    let refused = |network, context: &str, h0, h1| {
        build_manifest(network, context, Window { h0, h1 })
            .err()
            .map(|refusal| refusal.code)
    };
    assert_eq!(
        refused(Network::Test, "holders-room", 4_506_510, 4_505_790),
        Some(Code::WindowInvalid),
        "h0 must be below h1"
    );
    assert_eq!(
        refused(Network::Test, "holders-room", 4_505_790, 4_505_790),
        Some(Code::WindowInvalid),
        "a window of no length is no window"
    );
    assert_eq!(
        refused(Network::Test, "holders-room", 4_505_790, 4_506_511),
        Some(Code::WindowInvalid),
        "h1 must be a multiple of the nullifier snapshot step"
    );
    assert_eq!(
        refused(Network::Test, "Holders-Room", 4_505_790, 4_506_510),
        Some(Code::ManifestInvalid),
        "the context charset is a-z, 0-9 and '-'"
    );
    assert_eq!(
        refused(Network::Test, "", 4_505_790, 4_506_510),
        Some(Code::ManifestInvalid)
    );
    assert_eq!(
        refused(Network::Test, "-leading-dash", 4_505_790, 4_506_510),
        Some(Code::ManifestInvalid)
    );
    assert_eq!(
        refused(Network::Test, &"a".repeat(65), 4_505_790, 4_506_510),
        Some(Code::ManifestInvalid),
        "a context is at most 64 characters"
    );
    Ok(())
}
