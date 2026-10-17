//! The root authority (PRD section 4): two sources, one root. **Synthetic:** every source in
//! this file is a fake in-process object; no server is contacted. The z_gettreestate parsing
//! fixture lives in the verifier's unit tests (`src/roots/zebra.rs`), derived from zebra-rpc
//! 7.0.0-rc.0's own response type and height-0 snapshot.

use std::error::Error;

use unstirred_core::{Code, Network, Refusal, VerdictKind};
use unstirred_verifier::{
    consult, root_source_note, verdict_object, AgreedRoots, RootSource, RootsFromServer, Verdict, SINGLE_SOURCE_NOTE,
};

type TestResult = Result<(), Box<dyn Error>>;

const H0: u32 = 4_505_790;
const ROOT_A: [u8; 32] = [0xa5; 32];
const ROOT_B: [u8; 32] = [0x5a; 32];

#[derive(Debug)]
enum Answer {
    /// An answer for `h0`, carrying this root and tip.
    Roots(u32, [u8; 32], u32),
    /// The source cannot be reached.
    Fails,
}

#[derive(Debug)]
struct FakeSource {
    name: String,
    answer: Answer,
}

impl RootSource for FakeSource {
    fn describe(&self) -> String {
        self.name.clone()
    }

    fn read_roots(&self, _network: Network, _h0: u32) -> Result<RootsFromServer, Refusal> {
        match &self.answer {
            Answer::Roots(answered_h0, root, tip) => Ok(RootsFromServer {
                server: self.describe(),
                nc_root_at_h0: *root,
                h0: *answered_h0,
                tip: *tip,
            }),
            Answer::Fails => Err(Refusal::new(
                Code::RootSourceUnavailable,
                format!("{} cannot be reached", self.name),
            )),
        }
    }
}

fn sources(first: Answer, second: Answer) -> Vec<Box<dyn RootSource>> {
    vec![
        Box::new(FakeSource {
            name: "fake:first".to_owned(),
            answer: first,
        }),
        Box::new(FakeSource {
            name: "fake:second".to_owned(),
            answer: second,
        }),
    ]
}

fn expect_unavailable(readout: &unstirred_verifier::RootAuthorityReadout) -> Result<Refusal, Box<dyn Error>> {
    match &readout.agreement {
        Err(refusal) if refusal.code == Code::RootSourceUnavailable => Ok(refusal.clone()),
        Err(refusal) => Err(format!(
            "expected ROOT_SOURCE_UNAVAILABLE, got {}: {}",
            refusal.code, refusal.detail
        )
        .into()),
        Ok(agreed) => Err(format!(
            "expected ROOT_SOURCE_UNAVAILABLE, the sources agreed on {}",
            hex::encode(agreed.nc_root_at_h0)
        )
        .into()),
    }
}

/// A10, positive half: two agreeing sources yield one root, and the tip is the lower of the two
/// answers, the conservative choice for the finality check.
#[test]
fn a10_agreeing_sources_yield_one_root_and_the_lower_tip() -> TestResult {
    let readout = consult(
        &sources(
            Answer::Roots(H0, ROOT_A, 4_506_700),
            Answer::Roots(H0, ROOT_A, 4_506_550),
        ),
        Network::Test,
        H0,
    );
    let agreed = match &readout.agreement {
        Ok(agreed) => *agreed,
        Err(refusal) => return Err(format!("the sources should agree: {refusal:?}").into()),
    };
    assert_eq!(
        agreed,
        AgreedRoots {
            nc_root_at_h0: ROOT_A,
            tip: 4_506_550,
        },
        "the tip must be the lower of the two answers"
    );
    Ok(())
}

/// A10, refusal half: disagreeing sources are INCONCLUSIVE / ROOT_SOURCES_DISAGREE, never a
/// verdict, and the report names which source said which root.
#[test]
fn a10_disagreeing_sources_are_inconclusive_never_admitted() -> TestResult {
    let readout = consult(
        &sources(
            Answer::Roots(H0, ROOT_A, 4_506_700),
            Answer::Roots(H0, ROOT_B, 4_506_700),
        ),
        Network::Test,
        H0,
    );
    let refusal = match &readout.agreement {
        Err(refusal) => refusal,
        Ok(agreed) => {
            return Err(format!(
                "disagreeing sources produced a root {}: {}",
                hex::encode(agreed.nc_root_at_h0),
                agreed.tip
            )
            .into())
        }
    };
    assert_eq!(refusal.code, Code::RootSourcesDisagree);
    assert_eq!(
        refusal.code.spec().kind,
        VerdictKind::Inconclusive,
        "ROOT_SOURCES_DISAGREE must be inconclusive"
    );
    assert!(
        refusal.detail.contains("fake:first") && refusal.detail.contains("fake:second"),
        "the refusal must name every source's answer, got: {}",
        refusal.detail
    );
    Ok(())
}

/// A10b: one unreachable source makes the whole check INCONCLUSIVE, even though the other source
/// answered; partial information is never a decision.
#[test]
fn a10b_one_unreachable_source_is_root_source_unavailable() -> TestResult {
    let readout = consult(
        &sources(Answer::Roots(H0, ROOT_A, 4_506_700), Answer::Fails),
        Network::Test,
        H0,
    );
    let refusal = expect_unavailable(&readout)?;
    assert!(
        refusal.detail.contains("fake:second"),
        "the refusal must name the source that failed, got: {}",
        refusal.detail
    );
    Ok(())
}

/// A10b: a source that answers for the wrong height, or that reports a tip below the h0 it
/// claims a root for, is rejected as unusable, never trusted with the pin.
#[test]
fn a10b_a_source_answering_for_the_wrong_height_or_below_h0_is_rejected() -> TestResult {
    let wrong_height = consult(
        &sources(
            Answer::Roots(H0 - 1, ROOT_A, 4_506_700),
            Answer::Roots(H0, ROOT_A, 4_506_700),
        ),
        Network::Test,
        H0,
    );
    let refusal = expect_unavailable(&wrong_height)?;
    assert!(
        refusal.detail.contains("answered for height"),
        "the refusal must name the wrong height, got: {}",
        refusal.detail
    );

    let tip_behind_h0 = consult(
        &sources(Answer::Roots(H0, ROOT_A, H0 - 1), Answer::Roots(H0, ROOT_A, 4_506_700)),
        Network::Test,
        H0,
    );
    let refusal = expect_unavailable(&tip_behind_h0)?;
    assert!(
        refusal.detail.contains("tip"),
        "the refusal must name the tip behind h0, got: {}",
        refusal.detail
    );
    Ok(())
}

/// A27: single-source checking exists only as the explicit opt-in, and everything a one-source
/// run prints carries the fixed note; a two-source run carries none. The JSON object is what
/// `unstirred verify --json` puts on stdout.
#[test]
fn a27_single_source_mode_is_labelled_in_both_outputs() -> TestResult {
    assert_eq!(root_source_note(1), Some(SINGLE_SOURCE_NOTE));
    assert_eq!(root_source_note(2), None);
    assert!(
        SINGLE_SOURCE_NOTE.starts_with("ROOT FROM ONE SOURCE:"),
        "the label must name the one-source condition verbatim"
    );

    let one = consult(
        &[Box::new(FakeSource {
            name: "fake:only".to_owned(),
            answer: Answer::Roots(H0, ROOT_A, 4_506_700),
        })],
        Network::Test,
        H0,
    );
    assert!(
        one.agreement.is_ok(),
        "one source consulted on purpose still yields the root it reported"
    );

    let admitted = verdict_object(&Verdict::Admitted, root_source_note(1));
    assert_eq!(
        admitted,
        format!("{{\"verdict\":\"ADMITTED\",\"note\":\"{SINGLE_SOURCE_NOTE}\"}}"),
        "the JSON verdict must carry the note verbatim"
    );
    let two_sources = verdict_object(&Verdict::Admitted, root_source_note(2));
    assert_eq!(
        two_sources, "{\"verdict\":\"ADMITTED\"}",
        "a two-source run adds no note"
    );

    let refused = Verdict::Refused(Refusal::new(Code::RootSourcesDisagree, "the sources do not agree"));
    let refused_object = verdict_object(&refused, root_source_note(1));
    assert!(
        refused_object.contains("\"code\":\"ROOT_SOURCES_DISAGREE\"") && refused_object.contains("\"note\":"),
        "a refused verdict keeps the closed-set fields and still carries the note: {refused_object}"
    );
    Ok(())
}
