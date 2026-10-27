//! What `unstirred knock` does before it asks a gate for anything, and where the demo's forced
//! submission is not. These run the real binary against a gate that records every path it is
//! asked for, so "no challenge was requested" is checked at the socket rather than promised.
//!
//! The wallet here is one the command cannot read: the synthetic wallet's keys are for a regtest
//! network and a gate manifest can only say `test` or `main`, so no wallet a test can build is
//! one this binary will open. That is enough for what these
//! tests are about — the wallet is read, and the room's rule applied, before a challenge is ever
//! requested, so the step that fails is the local one either way. The rule's own refusals are the
//! prover's tests (A03, A17, A18) and the guard on the forced flag is a unit test beside it.
//!
//! The flag's own name is written out here rather than imported, because `unstirred-cli` is a
//! binary and has nothing to import from.

mod knock {
    pub mod fake_gate;
}

use knock::fake_gate::{
    run_unstirred, unreadable_wallet, FakeGate, Ran, TestResult, ADVERTISED_LIGHTWALLETD, ADVERTISED_PIR,
    SCANNED_THROUGH,
};
use unstirred_gate::GateClient;
use unstirred_prover::testing::H0;

const DEMO_FLAG: &str = "--force-unqualified-demo";

/// A window this wallet's notes do not reach back to, had it been readable at all.
const BEFORE_THE_COINS: u32 = H0 - 20;

/// A45: a knock that cannot prove anything asks that gate for nothing but its published rule. The
/// wallet is read first, so no challenge is issued and the gate holds no record that anybody
/// tried — and the command says so instead of leaving it to be inferred from a missing verdict.
#[test]
fn a45_a_knock_with_nothing_to_prove_asks_for_no_challenge_and_posts_nothing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let wallet = unreadable_wallet(directory.path())?;
    let gate = FakeGate::serving(&[("too-new", BEFORE_THE_COINS, SCANNED_THROUGH)])?;

    let ran = knock_at(&gate, "too-new", &wallet, &[])?;

    assert_eq!(
        gate.asked.paths(),
        vec!["/rooms/too-new/manifest".to_owned()],
        "the knock read the room's rule and asked for nothing else: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said("nothing was sent to the gate"),
        "a failure on the holder's own side says nothing was sent: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said("no challenge was asked for"),
        "a failure before the challenge says the gate has no record of it: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert_ne!(ran.code, Some(0), "nothing was proved, so this is not a success");

    // The canary: this recorder does see a challenge request, so the assertion above is a
    // statement about the knock and not about the recorder.
    GateClient::new(&gate.url)?.challenge("too-new").ok();
    assert_eq!(
        gate.asked.paths(),
        vec![
            "/rooms/too-new/manifest".to_owned(),
            "/rooms/too-new/challenge".to_owned()
        ],
        "a challenge request against this gate is recorded when one is made"
    );
    Ok(())
}

/// A46, the half a usage text can show: the demo's forced submission is in nothing the CLI
/// prints, so it cannot be found by reading the help, and it reads the wallet before the gate is
/// asked for anything either. Its guard — that it refuses a wallet the room would admit — is the
/// unit test in `src/proving/demo.rs`.
#[test]
fn a46_the_forced_demo_is_in_no_usage_text_and_still_asks_for_no_challenge() -> TestResult {
    for args in [vec![], vec!["knock"], vec!["prove"], vec!["gate"], vec!["codes"]] {
        let ran = run_unstirred(&args)?;
        assert!(
            !ran.said(DEMO_FLAG),
            "{args:?} must not advertise {DEMO_FLAG}: {}{}",
            ran.stdout,
            ran.stderr
        );
    }

    let directory = tempfile::tempdir()?;
    let wallet = unreadable_wallet(directory.path())?;
    let gate = FakeGate::serving(&[("qualifies", H0, SCANNED_THROUGH)])?;

    let ran = knock_at(&gate, "qualifies", &wallet, &[DEMO_FLAG])?;

    assert_eq!(
        gate.asked.paths(),
        vec!["/rooms/qualifies/manifest".to_owned()],
        "the forced path reads the wallet before the gate is asked for a challenge: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said("nothing was sent to the gate"),
        "the forced path says what it sent, which is nothing: {}{}",
        ran.stdout,
        ran.stderr
    );
    Ok(())
}

/// A50: where the two service addresses come from. A flag the holder gave always wins, the gate
/// is asked only when one is missing, and a gate that advertises nothing leaves the holder with
/// the error they would have had anyway — before a challenge is requested.
#[test]
fn a50_a_flag_beats_the_gates_advertisement_and_the_gate_is_asked_only_when_one_is_missing() -> TestResult {
    let directory = tempfile::tempdir()?;
    let wallet = unreadable_wallet(directory.path())?;
    let rooms = [("too-new", BEFORE_THE_COINS, SCANNED_THROUGH)];

    // Both flags given: the gate is never asked what it advertises, and the holder's own
    // addresses are the ones printed.
    let gate = FakeGate::advertising(&rooms)?;
    let ran = knock_at(&gate, "too-new", &wallet, &[])?;
    assert_eq!(
        gate.asked.paths(),
        vec!["/rooms/too-new/manifest".to_owned()],
        "a knock that names both addresses asks for nothing but the rule: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        !ran.said(ADVERTISED_PIR),
        "the gate's suggestion was not taken: {}",
        ran.stdout
    );
    assert!(
        !ran.said("advertised"),
        "nothing is said about an advertisement that was never read: {}",
        ran.stdout
    );

    // Neither flag given: the gate is asked once, and both of its addresses are used and named
    // as its own.
    let gate = FakeGate::advertising(&rooms)?;
    let ran = knock_without(&gate, "too-new", &wallet, &["--pir", "--lightwalletd"])?;
    assert_eq!(
        gate.asked.paths(),
        vec!["/rooms".to_owned(), "/rooms/too-new/manifest".to_owned()],
        "the room list is read once, before the rule: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said(ADVERTISED_PIR) && ran.said(ADVERTISED_LIGHTWALLETD),
        "both advertised addresses are the ones used: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said("this gate's own, advertised"),
        "the holder is told whose address they are using: {}{}",
        ran.stdout,
        ran.stderr
    );

    // One of each: the holder's flag wins for the one they gave, the gate's fills the other.
    let gate = FakeGate::advertising(&rooms)?;
    let ran = knock_without(&gate, "too-new", &wallet, &["--lightwalletd"])?;
    assert!(
        ran.said(ADVERTISED_LIGHTWALLETD),
        "the missing one came from the gate: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        !ran.said(ADVERTISED_PIR),
        "the one the holder named is the holder's: {}{}",
        ran.stdout,
        ran.stderr
    );

    // A gate that advertises nothing: usage, naming the flag, with no challenge asked for.
    let gate = FakeGate::serving(&rooms)?;
    let ran = knock_without(&gate, "too-new", &wallet, &["--pir"])?;
    assert_eq!(
        ran.code,
        Some(2),
        "a missing address is usage: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert!(
        ran.said("--pir <url> is required") && ran.said("this gate advertises none"),
        "the holder is told which flag and why there was no fallback: {}{}",
        ran.stdout,
        ran.stderr
    );
    assert_eq!(
        gate.asked.paths(),
        vec!["/rooms".to_owned()],
        "nothing beyond the advertisement was asked for: {}{}",
        ran.stdout,
        ran.stderr
    );
    Ok(())
}

/// A knock with some of its service flags left out, so the gate's advertisement is what fills
/// them. `without` names the flags to drop, each one taking its value with it.
fn knock_without(
    gate: &FakeGate,
    room: &str,
    wallet: &std::path::Path,
    without: &[&str],
) -> Result<Ran, Box<dyn std::error::Error>> {
    let wallet = wallet.to_string_lossy().into_owned();
    let full = [
        "knock",
        "--gate",
        &gate.url,
        "--room",
        room,
        "--wallet",
        &wallet,
        "--pir",
        "http://127.0.0.1:1",
        "--lightwalletd",
        "http://127.0.0.1:1",
        "--identity",
        "/dev/null",
    ];
    let mut args: Vec<&str> = Vec::with_capacity(full.len());
    let mut skip_value = false;
    for argument in full {
        if skip_value {
            skip_value = false;
            continue;
        }
        if without.contains(&argument) {
            skip_value = true;
            continue;
        }
        args.push(argument);
    }
    run_unstirred(&args)
}

fn knock_at(
    gate: &FakeGate,
    room: &str,
    wallet: &std::path::Path,
    extra: &[&str],
) -> Result<Ran, Box<dyn std::error::Error>> {
    let wallet = wallet.to_string_lossy().into_owned();
    let mut args = vec![
        "knock",
        "--gate",
        &gate.url,
        "--room",
        room,
        "--wallet",
        &wallet,
        // Neither of these is reached: the wallet is read, and the rule applied, before anything
        // but the room's manifest is fetched. Nothing listens on port 1.
        "--pir",
        "http://127.0.0.1:1",
        "--lightwalletd",
        "http://127.0.0.1:1",
        "--identity",
        "/dev/null",
    ];
    args.extend_from_slice(extra);
    run_unstirred(&args)
}
