//! A27: one source is an explicit opt-in, and every output of such a run says so. The JSON
//! object checked here is what `unstirred verify --json` puts on stdout.

use unstirred_core::{Code, Network, Refusal};
use unstirred_verifier::{consult, root_source_note, verdict_object, Verdict, SINGLE_SOURCE_NOTE};

use super::fake_source::{Answer, FakeSource, TestResult, H0, ROOT_A};

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
