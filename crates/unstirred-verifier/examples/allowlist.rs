//! The smallest thing that is actually a gate: an allowlist with a door.
//!
//! Two subcommands, which are the two halves of an admission:
//!
//! ```text
//! allowlist challenge --store allowlist.sqlite --manifest examples/holders-room.json
//! allowlist admit     --store allowlist.sqlite --manifest examples/holders-room.json \
//!     --package package.json \
//!     --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
//!     --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
//! ```
//!
//! `challenge` issues one and prints its hex; the holder proves against it with
//! `unstirred prove --challenge <that> --out package.json`. `admit` checks the package and, if it
//! holds, writes five markers and the verdict **in one transaction**. The second time the same
//! coins arrive, that transaction's `UNIQUE(context, marker)` refuses them and nothing is written.
//!
//! What this stores is what any Unstirred gate stores: markers and verdicts. Not an address, not a
//! balance, not the package, not the raw challenge — only a hash of it. `--list` prints the whole
//! allowlist, which is the quickest way to see that for yourself.

use std::process::ExitCode;

use unstirred_core::parse_manifest;
use unstirred_verifier::store::{Store, SystemClock};
use unstirred_verifier::{admit, consult, parse_root_source_spec, PinnedRoots, Verdict};

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
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = arguments.first().map(String::as_str).unwrap_or("--help");
    let flags = Flags::read(&arguments)?;

    match command {
        "challenge" => issue(&flags),
        "admit" => decide(&flags),
        "list" => list(&flags),
        _ => {
            println!("{USAGE}");
            Ok(ExitCode::from(2))
        }
    }
}

/// A challenge belongs to one room and expires in ten minutes. The store keeps a hash of it; this
/// hex is the only copy, and it goes to the holder.
fn issue(flags: &Flags) -> Result<ExitCode, String> {
    let manifest = manifest_of(flags)?;
    let store = Store::open(flags.require("--store")?).map_err(|e| format!("{e}"))?;
    let issued = store
        .issue(&manifest.context, &SystemClock)
        .map_err(|e| format!("{e}"))?;
    println!("challenge   {}", hex::encode(issued.raw));
    println!("room        {}", manifest.context);
    println!("expires     {} (unix seconds; ten minutes from now)", issued.expires_at);
    println!();
    println!("the holder answers it with:");
    println!(
        "  unstirred prove --challenge {} --out package.json \\",
        hex::encode(issued.raw)
    );
    println!("      --wallet <dir> --manifest <this room's manifest> --identity <age file> \\");
    println!("      --lightwalletd <url> --pir <url>");
    Ok(ExitCode::SUCCESS)
}

/// The whole decision: the roots from independent sources, then the engine, then — only if every
/// check held — one transaction that writes the markers and consumes the challenge.
fn decide(flags: &Flags) -> Result<ExitCode, String> {
    let manifest = manifest_of(flags)?;
    let sources = flags
        .every("--root-source")
        .iter()
        .map(|spec| parse_root_source_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;
    let agreed = match consult(&sources, manifest.network, manifest.window.h0).agreement {
        Ok(agreed) => agreed,
        Err(refusal) => {
            println!("INCONCLUSIVE  {}  {}", refusal.code, refusal.detail);
            println!("not on the allowlist, and nothing was written");
            return Ok(ExitCode::from(4));
        }
    };
    let roots = PinnedRoots {
        nc_root_at_h0: agreed.nc_root_at_h0,
        nf_imt_root_at_h1: thirty_two_bytes(flags.require("--h1-root")?)?,
    };
    let package = std::fs::read_to_string(flags.require("--package")?).map_err(|e| format!("{e}"))?;
    let mut store = Store::open(flags.require("--store")?).map_err(|e| format!("{e}"))?;

    let verdict = admit(&manifest, &package, &roots, agreed.tip, &mut store, &SystemClock)
        .map_err(|e| format!("the allowlist could not be written: {e}"))?;

    Ok(match verdict {
        Verdict::Admitted => {
            println!("ADMITTED");
            println!("this holder is on the allowlist for {}", manifest.context);
            println!("issue whatever you issue now: nothing here will recognise them again");
            ExitCode::SUCCESS
        }
        Verdict::Refused(refusal) => {
            println!("REFUSED  {}", refusal.code);
            println!("{}", refusal.code.spec().message);
            println!("not on the allowlist, and nothing was written");
            ExitCode::from(1)
        }
        Verdict::Inconclusive(refusal) => {
            println!("INCONCLUSIVE  {}  {}", refusal.code, refusal.detail);
            println!("no decision was reached; this is not a refusal of the holder");
            ExitCode::from(4)
        }
    })
}

/// Everything the allowlist holds, which is markers and verdicts and nothing else.
fn list(flags: &Flags) -> Result<ExitCode, String> {
    let store = Store::open(flags.require("--store")?).map_err(|e| format!("{e}"))?;
    let admissions = store.admissions().map_err(|e| format!("{e}"))?;
    let markers = store.markers().map_err(|e| format!("{e}"))?;
    println!("{} admission(s), {} marker(s)", admissions.len(), markers.len());
    for row in &admissions {
        println!("  {}  {}  at {}", row.context, row.verdict, row.at);
    }
    println!();
    println!("no address, balance, package or raw challenge is stored anywhere above.");
    Ok(ExitCode::SUCCESS)
}

fn manifest_of(flags: &Flags) -> Result<unstirred_core::Manifest, String> {
    let json = std::fs::read_to_string(flags.require("--manifest")?).map_err(|e| format!("{e}"))?;
    parse_manifest(&json).map_err(|refusal| format!("{refusal}"))
}

fn thirty_two_bytes(value: &str) -> Result<[u8; 32], String> {
    let decoded = hex::decode(value.trim()).map_err(|e| format!("--h1-root is not hex: {e}"))?;
    <[u8; 32]>::try_from(decoded.as_slice())
        .map_err(|_| format!("--h1-root decodes to {} bytes, not 32", decoded.len()))
}

const USAGE: &str = "allowlist <challenge|admit|list> --store <file> --manifest <file> [...]

  challenge  issue one for this room and print its hex
  admit      check --package against --root-source(s) and --h1-root, and write if it holds
  list       print every admission and marker this allowlist holds";

/// The flags this example was given, kept in order so `--root-source` can repeat.
struct Flags(Vec<(String, String)>);

impl Flags {
    fn read(arguments: &[String]) -> Result<Self, String> {
        let mut pairs = Vec::new();
        let mut rest = arguments.iter().skip(1);
        while let Some(flag) = rest.next() {
            if flag == "--single-root-source" {
                continue;
            }
            let value = rest.next().ok_or(format!("{flag} needs a value\n\n{USAGE}"))?;
            pairs.push((flag.clone(), value.clone()));
        }
        Ok(Self(pairs))
    }

    fn require(&self, flag: &str) -> Result<&str, String> {
        self.0
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, value)| value.as_str())
            .ok_or(format!("{flag} <value> is required\n\n{USAGE}"))
    }

    fn every(&self, flag: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(name, _)| name == flag)
            .map(|(_, value)| value.as_str())
            .collect()
    }
}
