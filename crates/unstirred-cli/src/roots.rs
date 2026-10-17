use std::process::ExitCode;

use unstirred_verifier::RootAuthorityReadout;

use crate::args::{parse_flags, required, strip_bool_flags, strip_repeatable_flags};
use crate::refusal::load_manifest;
use crate::root_sources::{print_readout_human, RootSourceArgs, SOURCE_RULES};
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
        println!("{}", report_json(&readout, &manifest, sources.note()));
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
            ",\"agreement\":\"AGREED\",\"nc_root_at_h0\":\"{}\",\"tip\":{}",
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
