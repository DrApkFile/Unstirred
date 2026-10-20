//! Root sources that disagree, answer for the wrong height, report a tip behind h0, or cannot
//! be reached at all. **Synthetic:** every source here is a fake in this process, except the
//! unreachable check, which points at a local port nothing listens on. None of these may
//! produce a verdict: contradicted or partial information is INCONCLUSIVE.

use unstirred_core::{Code, Network, VerdictKind};
use unstirred_verifier::{consult, AgreedRoots, LightwalletdSource, RootSource};

use crate::roots::fake_source::{expect_unavailable, sources, Answer, TestResult, H0, ROOT_A, ROOT_B};

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

/// A26: a root source that cannot be reached is INCONCLUSIVE and never a verdict of any other
/// kind. The only network this test touches is localhost on a port nothing listens on.
#[test]
fn a26_an_unreachable_root_source_is_inconclusive_never_admitted() -> TestResult {
    let source = LightwalletdSource::new("http://127.0.0.1:1");
    let refusal = match source.read_roots(Network::Test, 4_505_790) {
        Err(refusal) => refusal,
        Ok(roots) => return Err(format!("nothing listens on 127.0.0.1:1, yet the read returned {roots:?}").into()),
    };
    assert_eq!(
        refusal.code,
        Code::RootSourceUnavailable,
        "an unreachable root source must map to ROOT_SOURCE_UNAVAILABLE, got {refusal:?}"
    );
    assert_eq!(
        refusal.code.spec().kind,
        unstirred_core::VerdictKind::Inconclusive,
        "ROOT_SOURCE_UNAVAILABLE must be inconclusive"
    );
    Ok(())
}
