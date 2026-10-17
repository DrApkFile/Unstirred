use std::process::ExitCode;

use unstirred_verifier::RootAuthorityReadout;

use crate::args::{parse_flags, required, strip_bool_flags, strip_repeatable_flags};
use crate::refusal::load_manifest;
use crate::root_sources::{agreement_json, print_readout_human, RootSourceArgs, SOURCE_RULES};
use crate::{EXIT_INCONCLUSIVE, EXIT_USAGE};

pub const USAGE: &str = "unstirred roots --manifest <file> --root-source <spec> [--root-source <spec>] \\
    [--single-root-source] [--json]

Prints the h0 Ironwood note-commitment root each root source reports, whether the sources agree,
and the chain tip, without any proof package, so anyone can audit the roots a gate would pin.
Exit codes: 0 the sources agreed, 1 refused (bad manifest), 4 inconclusive, 2 usage, 3 local error.

{SOURCE_RULES}";

const FLAGS: [&str; 1] = ["--manifest"];

pub fn run(args: &[String]) -> ExitCode {
    let (args, bools) = match strip_bool_flags(args, &["--json", "--single-root-source"]) {
        Ok(stripped) => stripped,
        Err(problem) => return usage(&problem),
    };
    let (args, source_specs) = match strip_repeatable_flags(&args, "--root-source") {
        Ok(stripped) => stripped,
        Err(problem) => return usage(&problem),
    };
    let flags = match parse_flags(&args, &FLAGS) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem),
    };
    let manifest_path = match required(&flags, "--manifest") {
        Ok(path) => path,
        Err(problem) => return usage(&problem),
    };
    let manifest = match load_manifest(manifest_path) {
        Ok(manifest) => manifest,
        Err(exit) => return exit,
    };
    let sources = match RootSourceArgs::parse(&source_specs, bools.iter().any(|f| f == "--single-root-source")) {
        Ok(sources) => sources,
        Err(problem) => return usage(&problem),
    };

    let json = bools.iter().any(|f| f == "--json");
    let readout = sources.consult(manifest.network, manifest.window.h0);
    if json {
        println!("{}", report_json(&readout, &manifest, sources.note(), sources.single));
    } else {
        print_readout_human(&readout, manifest.network, manifest.window.h0, sources.single);
    }
    match readout.agreement {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(EXIT_INCONCLUSIVE),
    }
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("{}\n\n{}", problem, USAGE.replace("{SOURCE_RULES}", SOURCE_RULES));
    ExitCode::from(EXIT_USAGE)
}

fn report_json(
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
