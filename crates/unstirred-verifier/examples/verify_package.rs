//! Verifying one proof package the way a gate does, in about sixty lines.
//!
//! This is the whole of what a verifier is: read the gate's rule, ask independent sources for the
//! root that rule's `h0` names, and check the package against **those** roots — never against
//! anything the package says about itself. Nothing here writes, so the same package verifies
//! twice; recording a single use is `admit`, which the `allowlist` example beside this one shows.
//!
//! Run it against the real testnet proof recorded in this repository:
//!
//! ```text
//! cargo run --release --example verify_package -p unstirred-verifier -- \
//!     --package evidence/2026-10-17-first-proof.json \
//!     --manifest examples/holders-room.json \
//!     --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
//!     --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
//!     --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
//! ```
//!
//! Two agreeing sources are the default; one is an explicit opt-in that labels every line of its
//! own output. Exit codes match the command line's: 0 admitted, 1 refused, 3 local error,
//! 4 inconclusive.

use std::process::ExitCode;

use unstirred_core::parse_manifest;
use unstirred_verifier::{consult, parse_root_source_spec, root_source_note, verify_parsed, PinnedRoots, Verdict};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(problem) => {
            eprintln!("error: {problem}");
            ExitCode::from(3)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let arguments = Arguments::from_environment()?;

    // 1. The gate's rule. Everything the verifier is about to check comes from here.
    let manifest_json = read(&arguments.manifest)?;
    let manifest = parse_manifest(&manifest_json).map_err(|refusal| format!("{refusal}"))?;

    // 2. The h0 root, from sources of the verifier's own choosing. Two that agree by default;
    //    one only because this run asked for one, and it says so below.
    let sources = arguments
        .root_sources
        .iter()
        .map(|spec| parse_root_source_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;
    let readout = consult(&sources, manifest.network, manifest.window.h0);
    for outcome in &readout.per_source {
        match &outcome.answer {
            Ok(roots) => println!(
                "  {}  root {}  tip {}",
                outcome.source,
                hex::encode(roots.nc_root_at_h0),
                roots.tip
            ),
            Err(refusal) => println!("  {}  unusable: {}", outcome.source, refusal.detail),
        }
    }
    if let Some(note) = root_source_note(sources.len()) {
        println!("{note}");
    }
    let agreed = match readout.agreement {
        Ok(agreed) => agreed,
        Err(refusal) => {
            println!("INCONCLUSIVE  {}  {}", refusal.code, refusal.detail);
            return Ok(ExitCode::from(4));
        }
    };

    // 3. The verdict. `nf_imt_root_at_h1` is operator-pinned: the verifier cannot yet check it
    //    for itself, which is a stated limit (SPEC.md section 10).
    let roots = PinnedRoots {
        nc_root_at_h0: agreed.nc_root_at_h0,
        nf_imt_root_at_h1: arguments.h1_root,
    };
    let package_json = read(&arguments.package)?;
    Ok(
        match verify_parsed(&manifest, &package_json, &roots, agreed.tip, &arguments.challenge) {
            Verdict::Admitted => {
                println!("ADMITTED");
                println!("this example records nothing; `admit` is what enters a gate");
                ExitCode::SUCCESS
            }
            Verdict::Refused(refusal) => {
                println!("REFUSED  {}  {}", refusal.code, refusal.detail);
                ExitCode::from(1)
            }
            Verdict::Inconclusive(refusal) => {
                println!("INCONCLUSIVE  {}  {}", refusal.code, refusal.detail);
                ExitCode::from(4)
            }
        },
    )
}

struct Arguments {
    package: String,
    manifest: String,
    root_sources: Vec<String>,
    h1_root: [u8; 32],
    challenge: [u8; 32],
}

const USAGE: &str = "verify_package --package <file> --manifest <file> \\
    --root-source <lwd:url|zebra:url> --root-source <...> \\
    --h1-root <64 hex> --challenge <64 hex> [--single-root-source]";

impl Arguments {
    fn from_environment() -> Result<Self, String> {
        let (mut package, mut manifest, mut h1_root, mut challenge) = (None, None, None, None);
        let mut root_sources = Vec::new();
        let mut single = false;
        let mut arguments = std::env::args().skip(1);
        while let Some(flag) = arguments.next() {
            if flag == "--single-root-source" {
                single = true;
                continue;
            }
            let value = arguments.next().ok_or(format!("{flag} needs a value\n\n{USAGE}"))?;
            match flag.as_str() {
                "--package" => package = Some(value),
                "--manifest" => manifest = Some(value),
                "--root-source" => root_sources.push(value),
                "--h1-root" => h1_root = Some(value),
                "--challenge" => challenge = Some(value),
                other => return Err(format!("unknown argument {other:?}\n\n{USAGE}")),
            }
        }
        match (root_sources.len(), single) {
            (1, true) | (2, false) => {}
            (count, _) => {
                return Err(format!(
                    "exactly two root sources are required, or one with --single-root-source; \
                     {count} were given\n\n{USAGE}"
                ))
            }
        }
        Ok(Self {
            package: required(package, "--package")?,
            manifest: required(manifest, "--manifest")?,
            root_sources,
            h1_root: thirty_two_bytes(&required(h1_root, "--h1-root")?, "--h1-root")?,
            challenge: thirty_two_bytes(&required(challenge, "--challenge")?, "--challenge")?,
        })
    }
}

fn required(value: Option<String>, flag: &str) -> Result<String, String> {
    value.ok_or(format!("{flag} <value> is required\n\n{USAGE}"))
}

fn thirty_two_bytes(value: &str, flag: &str) -> Result<[u8; 32], String> {
    let decoded = hex::decode(value.trim()).map_err(|e| format!("{flag} is not hex: {e}"))?;
    <[u8; 32]>::try_from(decoded.as_slice()).map_err(|_| format!("{flag} decodes to {} bytes, not 32", decoded.len()))
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}
