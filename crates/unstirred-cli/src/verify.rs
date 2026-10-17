use std::process::ExitCode;

use unstirred_core::{parse_manifest, Refusal};
use unstirred_verifier::{fetch_h0_root_and_tip, verify_parsed, PinnedRoots, Verdict};

use crate::args::{parse_flags, required, Flags};
use crate::refusal::print_refusal;
use crate::{EXIT_ERROR, EXIT_INCONCLUSIVE, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred verify --package <file> --manifest <file> --lightwalletd <url> \\
    --h1-root <hex> --challenge <32 bytes hex> [--json]

Verifies one proof package against the gate's pinned roots and prints the verdict (PRD section
3.5). Exit codes: 0 admitted, 1 refused, 4 inconclusive, 2 usage, 3 local error.

Root sources in this milestone: the h0 Ironwood note-commitment root and the chain tip come from
one lightwalletd server (GetTreeState(h0) and GetLatestBlock), and the h1 nullifier-set root is
pinned by the operator on the command line. The two-server agreement check and the nf-server
verify-root check are milestone M2, and markers are not recorded yet (step 9 is M2). Nothing the
package says about roots, heights, network or context is trusted; only the manifest and the
verifier's own sources are.";

const FLAGS: [&str; 5] = ["--package", "--manifest", "--lightwalletd", "--h1-root", "--challenge"];

const ROOT_BYTES: usize = 32;
const CHALLENGE_BYTES: usize = 32;

pub fn run(args: &[String]) -> ExitCode {
    let (args, json) = match strip_json_flag(args) {
        Ok(stripped) => stripped,
        Err(problem) => return usage(&problem),
    };
    let flags = match parse_flags(&args, &FLAGS) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem),
    };
    let request = match Request::from_flags(&flags) {
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
        json,
        package: &request.package,
        h1_root: request.h1_root,
    };
    let from_server = match fetch_h0_root_and_tip(&request.lightwalletd, manifest.network, manifest.window.h0) {
        Ok(roots) => roots,
        Err(refusal) => {
            report.root_source_failure(&refusal);
            return inconclusive();
        }
    };
    report.root_provenance(&from_server, &manifest);

    let roots = PinnedRoots {
        nc_root_at_h0: from_server.nc_root_at_h0,
        nf_imt_root_at_h1: request.h1_root,
    };
    let verdict = verify_parsed(&manifest, &package_json, &roots, from_server.tip, &request.challenge);
    report.verdict(&verdict)
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("{problem}\n\n{USAGE}");
    ExitCode::from(EXIT_USAGE)
}

fn read_file(path: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|problem| {
        eprintln!("error: cannot read {path}: {problem}");
        ExitCode::from(EXIT_ERROR)
    })
}

/// `--json` is the one boolean flag; everything else takes a value.
fn strip_json_flag(args: &[String]) -> Result<(Vec<String>, bool), String> {
    let mut stripped = Vec::with_capacity(args.len());
    let mut json = false;
    for arg in args {
        if arg == "--json" {
            if json {
                return Err("--json was given twice".to_owned());
            }
            json = true;
        } else {
            stripped.push(arg.clone());
        }
    }
    Ok((stripped, json))
}

fn inconclusive() -> ExitCode {
    ExitCode::from(EXIT_INCONCLUSIVE)
}

#[derive(Debug)]
struct Request {
    package: String,
    manifest: String,
    lightwalletd: String,
    h1_root: [u8; ROOT_BYTES],
    challenge: [u8; CHALLENGE_BYTES],
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        Ok(Self {
            package: required(flags, "--package")?.to_owned(),
            manifest: required(flags, "--manifest")?.to_owned(),
            lightwalletd: required(flags, "--lightwalletd")?.to_owned(),
            h1_root: fixed_hex(required(flags, "--h1-root")?, "--h1-root", ROOT_BYTES)?,
            challenge: fixed_hex(required(flags, "--challenge")?, "--challenge", CHALLENGE_BYTES)?,
        })
    }
}

fn fixed_hex<const N: usize>(text: &str, flag: &str, bytes: usize) -> Result<[u8; N], String> {
    let decoded = hex::decode(text.trim()).map_err(|_| format!("{flag} must be hex"))?;
    <[u8; N]>::try_from(decoded.as_slice()).map_err(|_| format!("{flag} must be {bytes} bytes of hex"))
}

/// Where the verdict and, in human mode, the provenance of every root went.
#[derive(Debug)]
struct Report<'a> {
    json: bool,
    package: &'a str,
    h1_root: [u8; ROOT_BYTES],
}

impl Report<'_> {
    fn root_provenance(&self, from_server: &unstirred_verifier::RootsFromServer, manifest: &unstirred_core::Manifest) {
        if self.json {
            eprintln!(
                "h0 note-commitment root from GetTreeState({}) at {}",
                from_server.h0, from_server.server
            );
            eprintln!("h1 nullifier-set root pinned by the operator (--h1-root)");
            eprintln!(
                "chain tip {} from GetLatestBlock at {}",
                from_server.tip, from_server.server
            );
            eprintln!("one root server in M1; the two-server agreement check and nf-server verify-root are M2");
            return;
        }
        println!(
            "verifying {} against manifest context {:?}, network {}, window h0 {} through h1 {}",
            self.package, manifest.context, manifest.network, manifest.window.h0, manifest.window.h1
        );
        println!();
        println!("roots, and where each came from:");
        println!("  h0 note-commitment root  {}", hex::encode(from_server.nc_root_at_h0));
        println!(
            "    from GetTreeState({}) at {} (one server; the two-server agreement check is M2)",
            from_server.h0, from_server.server
        );
        println!("  h1 nullifier-set root    {}", hex::encode(self.h1_root));
        println!("    pinned by the operator on the command line (nf-server verify-root is M2)");
        println!("  chain tip                {}", from_server.tip);
        println!("    from GetLatestBlock at {}", from_server.server);
        println!();
    }

    fn root_source_failure(&self, refusal: &Refusal) {
        if self.json {
            eprintln!("{}: {}", refusal.code, refusal.detail);
        } else {
            print_refusal(refusal);
        }
    }

    fn verdict(&self, verdict: &Verdict) -> ExitCode {
        if self.json {
            return self.verdict_json(verdict);
        }
        match verdict {
            Verdict::Admitted => {
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

    /// The PRD section 3.5 verdict object, nothing else, on stdout.
    fn verdict_json(&self, verdict: &Verdict) -> ExitCode {
        match verdict {
            Verdict::Admitted => {
                println!("{{\"verdict\":\"ADMITTED\"}}");
                eprintln!("markers not recorded (M2)");
                ExitCode::SUCCESS
            }
            Verdict::Refused(refusal) | Verdict::Inconclusive(refusal) => {
                let spec = refusal.code.spec();
                println!(
                    "{{\"verdict\":\"{}\",\"code\":\"{}\",\"message\":\"{}\",\"retryable\":{}}}",
                    spec.kind, spec.wire_name, spec.message, spec.retryable
                );
                match verdict {
                    Verdict::Refused(_) => ExitCode::from(EXIT_REFUSED),
                    Verdict::Inconclusive(_) => inconclusive(),
                    Verdict::Admitted => ExitCode::SUCCESS,
                }
            }
        }
    }
}
