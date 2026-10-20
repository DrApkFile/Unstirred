#![doc = "Writing a gate manifest. Every rule that decides whether this file is one belongs to \
`unstirred-core`, so this command parses four values and hands them over: what it writes is what \
core itself accepts, or it writes nothing (A40)."]

use std::path::PathBuf;
use std::process::ExitCode;

use unstirred_core::{build_manifest, Network, Window};

use super::{local, usage};
use crate::args::{parse_flags, required, Flags};
use crate::refusal::print_refusal;
use crate::EXIT_REFUSED;

pub(super) const USAGE: &str = "unstirred gate init --network <test|main> --context <name> \\
    --h0 <height> --h1 <height> --out <file>

Writes one gate manifest. The context is 1-64 characters of a-z, 0-9 and '-', h0 must be below
h1, and h1 must be a multiple of 10 (the nullifier snapshot step). The threshold is fixed at
12,500,000 zatoshis (0.125 ZEC) in unstirred/v1. An existing --out file is never overwritten, and
nothing is written unless unstirred-core accepts the result.";

const FLAGS: [&str; 5] = ["--network", "--context", "--h0", "--h1", "--out"];

pub(super) fn run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &FLAGS) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem, USAGE),
    };
    let request = match Request::from_flags(&flags) {
        Ok(request) => request,
        Err(problem) => return usage(&problem, USAGE),
    };
    let json = match build_manifest(request.network, &request.context, request.window) {
        Ok(json) => json,
        Err(refusal) => {
            print_refusal(&refusal);
            return ExitCode::from(EXIT_REFUSED);
        }
    };
    if let Err(problem) = std::fs::write(&request.out, format!("{json}\n")) {
        return local(&format!("cannot write {}: {problem}", request.out.display()));
    }
    report(&request);
    ExitCode::SUCCESS
}

fn report(request: &Request) {
    println!("manifest written to {}", request.out.display());
    println!("network       {}", request.network);
    println!("context       {}", request.context);
    println!(
        "window        h0 {} through h1 {}",
        request.window.h0, request.window.h1
    );
    println!("threshold     12500000 zatoshis (0.125 ZEC), fixed in unstirred/v1");
    println!();
    println!("Publish this file to holders, and serve it with: unstirred gate serve --config <file>");
    println!("The gate's own h1 nullifier-set root is pinned in that config, not in this manifest.");
}

#[derive(Debug)]
struct Request {
    network: Network,
    context: String,
    window: Window,
    out: PathBuf,
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        let out = PathBuf::from(required(flags, "--out")?);
        if out.exists() {
            return Err(format!(
                "{} already exists; a manifest is never overwritten",
                out.display()
            ));
        }
        Ok(Self {
            network: network_from(required(flags, "--network")?)?,
            context: required(flags, "--context")?.to_owned(),
            window: Window {
                h0: height_from(required(flags, "--h0")?, "--h0")?,
                h1: height_from(required(flags, "--h1")?, "--h1")?,
            },
            out,
        })
    }
}

fn network_from(value: &str) -> Result<Network, String> {
    match value {
        "test" => Ok(Network::Test),
        "main" => Ok(Network::Main),
        other => Err(format!("--network is test or main, not {other:?}")),
    }
}

fn height_from(value: &str, flag: &str) -> Result<u32, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{flag} must be a block height, not {value:?}"))
}
