use std::process::ExitCode;

use unstirred_core::{parse_manifest, Refusal};
use unstirred_verifier::{verdict_object, verify_parsed, PinnedRoots, Verdict};

use crate::args::{parse_flags, required, strip_bool_flags, strip_repeatable_flags, Flags};
use crate::refusal::print_refusal;
use crate::root_sources::{
    print_readout_human, print_readout_json_stderr, print_refusal_json_stderr, RootSourceArgs, SOURCE_RULES,
};
use crate::{EXIT_ERROR, EXIT_INCONCLUSIVE, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred verify --package <file> --manifest <file> --root-source <spec> \\
    [--root-source <spec>] --h1-root <hex> --challenge <32 bytes hex> [--single-root-source] [--json]

Verifies one proof package against the gate's roots and prints the verdict (PRD section 3.5).
Exit codes: 0 admitted, 1 refused, 4 inconclusive, 2 usage, 3 local error.

The h0 Ironwood note-commitment root and the chain tip come from the gate's root sources, which
must agree on one root unless --single-root-source is given; the h1 nullifier-set root is pinned
by the operator on the command line and labelled as operator-pinned (nf-server verify-root is
deferred to P1). Nothing the package says about roots, heights, network or context is trusted;
only the manifest and the verifier's own sources are.

{SOURCE_RULES}";

const FLAGS: [&str; 4] = ["--package", "--manifest", "--h1-root", "--challenge"];

const ROOT_BYTES: usize = 32;
const CHALLENGE_BYTES: usize = 32;

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

    let report = Report {
        json: bools.iter().any(|f| f == "--json"),
        single: request.sources.single,
        package: &request.package,
        h1_root: request.h1_root,
    };
    let readout = request.sources.consult(manifest.network, manifest.window.h0);
    report.root_provenance(&readout, &manifest);
    let agreed = match readout.agreement {
        Ok(agreed) => agreed,
        Err(refusal) => {
            report.root_authority_failure(&refusal);
            return inconclusive();
        }
    };

    let roots = PinnedRoots {
        nc_root_at_h0: agreed.nc_root_at_h0,
        nf_imt_root_at_h1: request.h1_root,
    };
    let verdict = verify_parsed(&manifest, &package_json, &roots, agreed.tip, &request.challenge);
    report.verdict(&verdict, request.sources.note())
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

fn inconclusive() -> ExitCode {
    ExitCode::from(EXIT_INCONCLUSIVE)
}

#[derive(Debug)]
struct Request {
    package: String,
    manifest: String,
    sources: RootSourceArgs,
    h1_root: [u8; ROOT_BYTES],
    challenge: [u8; CHALLENGE_BYTES],
}

impl Request {
    fn from_flags(flags: &Flags, source_specs: &[String], single: bool) -> Result<Self, String> {
        Ok(Self {
            package: required(flags, "--package")?.to_owned(),
            manifest: required(flags, "--manifest")?.to_owned(),
            sources: RootSourceArgs::parse(source_specs, single)?,
            h1_root: fixed_hex(required(flags, "--h1-root")?, "--h1-root", ROOT_BYTES)?,
            challenge: fixed_hex(required(flags, "--challenge")?, "--challenge", CHALLENGE_BYTES)?,
        })
    }
}

fn fixed_hex<const N: usize>(text: &str, flag: &str, bytes: usize) -> Result<[u8; N], String> {
    let decoded = hex::decode(text.trim()).map_err(|_| format!("{flag} must be hex"))?;
    <[u8; N]>::try_from(decoded.as_slice()).map_err(|_| format!("{flag} must be {bytes} bytes of hex"))
}

/// Where the verdict and, around it, the provenance of every root went.
#[derive(Debug)]
struct Report<'a> {
    json: bool,
    single: bool,
    package: &'a str,
    h1_root: [u8; ROOT_BYTES],
}

impl Report<'_> {
    fn root_provenance(&self, readout: &unstirred_verifier::RootAuthorityReadout, manifest: &unstirred_core::Manifest) {
        if self.json {
            print_readout_json_stderr(readout, self.single);
            eprintln!(
                "h1 nullifier-set root pinned by the operator (--h1-root): {}",
                hex::encode(self.h1_root)
            );
            return;
        }
        println!(
            "verifying {} against manifest context {:?}, network {}, window h0 {} through h1 {}",
            self.package, manifest.context, manifest.network, manifest.window.h0, manifest.window.h1
        );
        println!();
        print_readout_human(readout, manifest.network, manifest.window.h0, self.single);
        println!("h1 nullifier-set root    {}", hex::encode(self.h1_root));
        println!("    pinned by the operator on the command line, labelled operator-pinned (nf-server verify-root is deferred to P1)");
        println!();
    }

    fn root_authority_failure(&self, refusal: &Refusal) {
        if self.json {
            print_refusal_json_stderr(refusal);
        } else {
            print_refusal(refusal);
        }
    }

    fn verdict(&self, verdict: &Verdict, note: Option<&'static str>) -> ExitCode {
        if self.json {
            println!("{}", verdict_object(verdict, note));
            if matches!(verdict, Verdict::Admitted) {
                eprintln!("markers not recorded (M2)");
                return ExitCode::SUCCESS;
            }
            return match verdict {
                Verdict::Refused(_) => ExitCode::from(EXIT_REFUSED),
                Verdict::Inconclusive(_) => inconclusive(),
                Verdict::Admitted => ExitCode::SUCCESS,
            };
        }
        match verdict {
            Verdict::Admitted => {
                if let Some(note) = note {
                    println!("{note}");
                }
                println!("ADMITTED");
                println!("markers not recorded (M2)");
                ExitCode::SUCCESS
            }
            Verdict::Refused(refusal) => {
                print_refusal(refusal);
                ExitCode::from(EXIT_REFUSED)
            }
            Verdict::Inconclusive(refusal) => {
                print_refusal(refusal);
                inconclusive()
            }
        }
    }
}
