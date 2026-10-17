use std::process::ExitCode;

use unstirred_core::parse_manifest;
use unstirred_verifier::store::{Store, StoreError, SystemClock};
use unstirred_verifier::{admit, verdict_object, PinnedRoots, Verdict};

use crate::args::{parse_flags, required, strip_bool_flags, strip_repeatable_flags, Flags};
use crate::refusal::print_refusal;
use crate::root_sources::{
    print_readout_human, print_readout_json_stderr, print_refusal_json_stderr, RootSourceArgs, SOURCE_RULES,
};
use crate::{EXIT_ERROR, EXIT_INCONCLUSIVE, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred admit --db <file> --package <file> --manifest <file> \\
    --root-source <spec> [--root-source <spec>] --h1-root <hex> [--single-root-source] [--json]

Admits one proof package into the gate database, or refuses it (PRD section 6, with the challenge
consumed only after the proof verifies). The challenge is the one inside the package. Exit codes:
0 admitted, 1 refused, 4 inconclusive, 2 usage, 3 local error.

{SOURCE_RULES}";

const FLAGS: [&str; 4] = ["--db", "--package", "--manifest", "--h1-root"];
const ROOT_BYTES: usize = 32;

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
    let request = match Request::from_flags(&flags, &source_specs, bools.iter().any(|f| f == "--single-root-source")) {
        Ok(request) => request,
        Err(problem) => return usage(&problem),
    };
    let manifest_json = match read_file(&request.manifest) {
        Ok(json) => json,
        Err(exit) => return exit,
    };
    let manifest = match parse_manifest(&manifest_json) {
        Ok(manifest) => manifest,
        Err(refusal) => {
            print_refusal(&refusal);
            return ExitCode::from(EXIT_REFUSED);
        }
    };
    let package_json = match read_file(&request.package) {
        Ok(json) => json,
        Err(exit) => return exit,
    };
    let mut store = match Store::open(&request.database) {
        Ok(store) => store,
        Err(error) => return local(&error),
    };

    let json = bools.iter().any(|f| f == "--json");
    let readout = request.sources.consult(manifest.network, manifest.window.h0);
    print_roots(json, &request, &readout, &manifest);
    let agreed = match readout.agreement {
        Ok(agreed) => agreed,
        Err(refusal) => {
            if json {
                print_refusal_json_stderr(&refusal);
            } else {
                print_refusal(&refusal);
            }
            return ExitCode::from(EXIT_INCONCLUSIVE);
        }
    };
    let verdict = match admit(
        &manifest,
        &package_json,
        &PinnedRoots {
            nc_root_at_h0: agreed.nc_root_at_h0,
            nf_imt_root_at_h1: request.h1_root,
        },
        agreed.tip,
        &mut store,
        &SystemClock,
    ) {
        Ok(verdict) => verdict,
        Err(error) => return local(&error),
    };
    finish(json, &verdict, request.sources.note())
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("{}\n\n{}", problem, USAGE.replace("{SOURCE_RULES}", SOURCE_RULES));
    ExitCode::from(EXIT_USAGE)
}

fn read_file(path: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|problem| {
        eprintln!("error: cannot read {path}: {problem}");
        ExitCode::from(EXIT_ERROR)
    })
}

fn local(error: &StoreError) -> ExitCode {
    eprintln!("error: {error}");
    ExitCode::from(EXIT_ERROR)
}

fn print_roots(
    json: bool,
    request: &Request,
    readout: &unstirred_verifier::RootAuthorityReadout,
    manifest: &unstirred_core::Manifest,
) {
    let h1 = hex::encode(request.h1_root);
    if json {
        print_readout_json_stderr(readout, request.sources.single);
        eprintln!("h1 nullifier-set root pinned by the operator (--h1-root): {h1}");
        return;
    }
    print_readout_human(readout, manifest.network, manifest.window.h0, request.sources.single);
    println!("h1 nullifier-set root    {h1}");
    println!(
        "    pinned by the operator on the command line, labelled operator-pinned (nf-server verify-root is deferred to P1)"
    );
}

fn finish(json: bool, verdict: &Verdict, note: Option<&'static str>) -> ExitCode {
    if json {
        println!("{}", verdict_object(verdict, note));
    }
    match verdict {
        Verdict::Admitted => {
            if !json {
                if let Some(note) = note {
                    println!("{note}");
                }
                println!("ADMITTED");
            }
            ExitCode::SUCCESS
        }
        Verdict::Refused(refusal) => {
            if !json {
                print_refusal(refusal);
            }
            ExitCode::from(EXIT_REFUSED)
        }
        Verdict::Inconclusive(refusal) => {
            if !json {
                print_refusal(refusal);
            }
            ExitCode::from(EXIT_INCONCLUSIVE)
        }
    }
}

#[derive(Debug)]
struct Request {
    database: String,
    package: String,
    manifest: String,
    sources: RootSourceArgs,
    h1_root: [u8; ROOT_BYTES],
}

impl Request {
    fn from_flags(flags: &Flags, source_specs: &[String], single: bool) -> Result<Self, String> {
        Ok(Self {
            database: required(flags, "--db")?.to_owned(),
            package: required(flags, "--package")?.to_owned(),
            manifest: required(flags, "--manifest")?.to_owned(),
            sources: RootSourceArgs::parse(source_specs, single)?,
            h1_root: fixed_hex(required(flags, "--h1-root")?)?,
        })
    }
}

fn fixed_hex(text: &str) -> Result<[u8; ROOT_BYTES], String> {
    let decoded = hex::decode(text.trim()).map_err(|_| "--h1-root must be hex".to_owned())?;
    <[u8; ROOT_BYTES]>::try_from(decoded.as_slice()).map_err(|_| "--h1-root must be 32 bytes of hex".to_owned())
}
