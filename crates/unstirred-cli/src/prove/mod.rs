use std::path::PathBuf;
use std::process::ExitCode;

use unstirred_prover::ProverError;

use crate::args::{parse_flags, required, Flags};
use crate::proving::{make_proof, ProofInputs, PROOF_FLAGS};
use crate::refusal::{load_manifest, print_refusal};
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

mod report;

use report::write_and_report;

pub const USAGE: &str = "unstirred prove --wallet <dir> --manifest <file> --lightwalletd <url> \\
    --pir <url> --identity <age file> --challenge <32 bytes hex> --out <file> \\
    [--expected-h1-root <hex>] [--account <uuid>]

Proves that the wallet held at least the gate's threshold in Ironwood, unmoved, from h0 through
h1, and writes the proof package to --out. The wallet is opened read-only and no transaction is
created. The spending key is derived from the age-encrypted mnemonic in memory only: it is never
printed, logged or written, and the package carries no address, value, tree position or nullifier.

The challenge is supplied by hand here. `unstirred knock` takes one from a running gate instead,
proves against it and posts the package in one command.";

const CHALLENGE_BYTES: usize = 32;

pub fn run(args: &[String]) -> ExitCode {
    let allowed: Vec<&str> = PROOF_FLAGS
        .iter()
        .copied()
        .chain(["--manifest", "--challenge", "--out"])
        .collect();
    let flags = match parse_flags(args, &allowed) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem),
    };
    let request = match Request::from_flags(&flags) {
        Ok(request) => request,
        Err(problem) => return usage(&problem),
    };
    let manifest = match load_manifest(&request.manifest) {
        Ok(manifest) => manifest,
        Err(exit) => return exit,
    };
    match make_proof(&request.inputs, &manifest, request.challenge) {
        Ok(made) => write_and_report(&request, &manifest, &made),
        Err(ProverError::Refused(refusal)) => {
            print_refusal(&refusal);
            ExitCode::from(EXIT_REFUSED)
        }
        Err(other) => {
            eprintln!("error: {other}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("{problem}\n\n{USAGE}");
    ExitCode::from(EXIT_USAGE)
}

#[derive(Debug)]
struct Request {
    inputs: ProofInputs,
    manifest: String,
    challenge: [u8; CHALLENGE_BYTES],
    out: PathBuf,
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        let out = PathBuf::from(required(flags, "--out")?);
        if out.exists() {
            return Err(format!(
                "{} already exists; a proof package is never overwritten",
                out.display()
            ));
        }
        Ok(Self {
            inputs: ProofInputs::from_flags(flags)?,
            manifest: required(flags, "--manifest")?.to_owned(),
            challenge: challenge_from_hex(required(flags, "--challenge")?)?,
            out,
        })
    }
}

pub fn challenge_from_hex(text: &str) -> Result<[u8; CHALLENGE_BYTES], String> {
    let bytes = hex::decode(text.trim()).map_err(|_| "--challenge must be hex".to_owned())?;
    <[u8; CHALLENGE_BYTES]>::try_from(bytes.as_slice())
        .map_err(|_| format!("--challenge must be {CHALLENGE_BYTES} bytes of hex"))
}
