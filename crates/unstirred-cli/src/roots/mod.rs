use std::process::ExitCode;

use crate::args::{parse_flags, required, strip_bool_flags, strip_repeatable_flags};
use crate::refusal::load_manifest;
use crate::root_sources::{print_readout_human, RootSourceArgs, SOURCE_RULES};
use crate::{EXIT_INCONCLUSIVE, EXIT_USAGE};

mod report;

use report::report_json;

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
