#![doc = "The roots command's JSON report: one object naming every source's answer, the agreement \
the authority reached, and the single-source note when there was only one source."]

use unstirred_verifier::RootAuthorityReadout;

use crate::root_sources::agreement_json;

pub(super) fn report_json(
    readout: &RootAuthorityReadout,
    manifest: &unstirred_core::Manifest,
    note: Option<&'static str>,
    single: bool,
) -> String {
    let sources = readout
        .per_source
        .iter()
        .map(|outcome| match &outcome.answer {
            Ok(answer) => format!(
                "{{\"source\":\"{}\",\"nc_root_at_h0\":\"{}\",\"tip\":{}}}",
                json_escape(&outcome.source),
                hex::encode(answer.nc_root_at_h0),
                answer.tip
            ),
            Err(refusal) => format!(
                "{{\"source\":\"{}\",\"unusable\":{{\"code\":\"{}\",\"detail\":\"{}\"}}}}",
                json_escape(&outcome.source),
                refusal.code,
                json_escape(&refusal.detail)
            ),
        })
        .collect::<Vec<_>>()
        .join(",");
    let mut object = format!(
        "{{\"network\":\"{}\",\"h0\":{},\"sources\":[{sources}]",
        manifest.network, manifest.window.h0
    );
    match &readout.agreement {
        Ok(agreed) => object.push_str(&format!(
            ",\"agreement\":\"{}\",\"nc_root_at_h0\":\"{}\",\"tip\":{}",
            agreement_json(single),
            hex::encode(agreed.nc_root_at_h0),
            agreed.tip
        )),
        Err(refusal) => object.push_str(&format!(
            ",\"agreement\":\"{}\",\"detail\":\"{}\"",
            refusal.code,
            json_escape(&refusal.detail)
        )),
    }
    if let Some(note) = note {
        object.push_str(&format!(",\"note\":\"{}\"", json_escape(note)));
    }
    object.push('}');
    object
}

fn json_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use unstirred_core::{parse_manifest, Network};
    use unstirred_verifier::{AgreedRoots, RootAuthorityReadout, RootsFromServer, SourceOutcome};

    use super::report_json;
    use crate::root_sources::{agreement_json, agreement_line};

    fn readout(tip: u32) -> RootAuthorityReadout {
        RootAuthorityReadout {
            per_source: vec![SourceOutcome {
                source: "fake:only".to_owned(),
                answer: Ok(RootsFromServer {
                    server: "fake:only".to_owned(),
                    nc_root_at_h0: [0xab; 32],
                    h0: 4_505_790,
                    tip,
                }),
            }],
            agreement: Ok(AgreedRoots {
                nc_root_at_h0: [0xab; 32],
                tip,
            }),
        }
    }

    #[test]
    fn a27_single_source_agreement_is_not_claimed_in_either_output() -> Result<(), String> {
        let root = hex::encode([0xab; 32]);
        assert_eq!(agreement_line(true, &root), "agreement: not checked (one source)");
        assert_eq!(agreement_json(true), "not checked (one source)");
        assert!(
            !agreement_line(true, &root).contains("the sources agree"),
            "a one-source line must not claim agreement"
        );

        let manifest = parse_manifest(
            r#"{"version":"unstirred/v1","network":"test","context":"holders-room","window":{"h0":4505790,"h1":4506510},"min_zatoshis":"12500000"}"#,
        )
        .map_err(|refusal| format!("the fixture manifest is valid: {refusal}"))?;
        assert_eq!(manifest.network, Network::Test);
        let json = report_json(
            &readout(4_506_700),
            &manifest,
            Some("ROOT FROM ONE SOURCE: not independently checked"),
            true,
        );
        assert!(
            json.contains("\"agreement\":\"not checked (one source)\""),
            "the JSON agreement field must say the check did not happen: {json}"
        );
        assert!(
            !json.contains("AGREED"),
            "single-source JSON must not say AGREED: {json}"
        );
        assert!(
            !json.contains("the sources agree"),
            "single-source JSON must not claim the sources agreed: {json}"
        );
        assert!(json.contains(&root), "the one source's root is still reported: {json}");

        let two = report_json(&readout(4_506_700), &manifest, None, false);
        assert!(
            two.contains("\"agreement\":\"AGREED\""),
            "two sources still agree: {two}"
        );
        assert_eq!(
            agreement_line(false, &root),
            format!("agreement: the sources agree on one h0 root {root}")
        );
        Ok(())
    }
}
