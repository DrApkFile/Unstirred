use std::path::{Path, PathBuf};
use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, prove, root_from_hex, witnesses_at, OsRng, PirNullifierOracle,
    ProofRequest, Proven, ProverError, ReadOnlyWallet,
};

use crate::args::{optional, parse_flags, required, Flags};
use crate::refusal::{load_manifest, print_refusal};
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred prove --wallet <dir> --manifest <file> --lightwalletd <url> \\
    --pir <url> --identity <age file> --challenge <32 bytes hex> --out <file> \\
    [--expected-h1-root <hex>] [--account <uuid>]

Proves that the wallet held at least the gate's threshold in Ironwood, unmoved, from h0 through
h1, and writes the proof package to --out. The wallet is opened read-only and no transaction is
created. The spending key is derived from the age-encrypted mnemonic in memory only: it is never
printed, logged or written, and the package carries no address, value, tree position or nullifier.

In this milestone the challenge is supplied by hand; from milestone M3 the gate issues it.";

const FLAGS: [&str; 9] = [
    "--wallet",
    "--manifest",
    "--lightwalletd",
    "--pir",
    "--identity",
    "--challenge",
    "--out",
    "--expected-h1-root",
    "--account",
];

const CHALLENGE_BYTES: usize = 32;

pub fn run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &FLAGS) {
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
    match make_proof(&request, &manifest) {
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
    wallet: PathBuf,
    manifest: String,
    lightwalletd: String,
    pir: String,
    identity: PathBuf,
    challenge: [u8; CHALLENGE_BYTES],
    out: PathBuf,
    expected_h1_root: Option<String>,
    account: Option<String>,
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
            wallet: PathBuf::from(required(flags, "--wallet")?),
            manifest: required(flags, "--manifest")?.to_owned(),
            lightwalletd: required(flags, "--lightwalletd")?.to_owned(),
            pir: required(flags, "--pir")?.to_owned(),
            identity: PathBuf::from(required(flags, "--identity")?),
            challenge: challenge_from_hex(required(flags, "--challenge")?)?,
            out,
            expected_h1_root: optional(flags, "--expected-h1-root").map(str::to_owned),
            account: optional(flags, "--account").map(str::to_owned),
        })
    }
}

fn challenge_from_hex(text: &str) -> Result<[u8; CHALLENGE_BYTES], String> {
    let bytes = hex::decode(text.trim()).map_err(|_| "--challenge must be hex".to_owned())?;
    <[u8; CHALLENGE_BYTES]>::try_from(bytes.as_slice())
        .map_err(|_| format!("--challenge must be {CHALLENGE_BYTES} bytes of hex"))
}

fn make_proof(request: &Request, manifest: &Manifest) -> Result<Made, ProverError> {
    let expected_root = match request.expected_h1_root.as_deref() {
        Some(hex_root) => Some(root_from_hex(hex_root)?),
        None => None,
    };
    let wallet = ReadOnlyWallet::open(&request.wallet, manifest.network)?;
    let account = wallet.account(request.account.as_deref())?;
    let selected = eligible_notes(&wallet, account, manifest)?.selection;
    let tree_state = fetch_ironwood_tree_state(&request.lightwalletd, manifest.network, manifest.window.h0)?;
    let witnesses = witnesses_at(&wallet, &tree_state, &selected.notes)?;
    let nullifier_set = PirNullifierOracle::connect(&request.pir, manifest.network, manifest.window.h1, expected_root)?;
    let authority = wallet.ironwood_spend_authority(account, &request.identity)?;
    let proven = prove(
        &wallet,
        account,
        &authority,
        &ProofRequest {
            manifest,
            witnesses: &witnesses,
            expected_nf_imt_root: nullifier_set.root_bytes(),
            challenge: request.challenge,
        },
        &nullifier_set,
        &mut OsRng,
    )?;
    Ok(Made {
        wallet_path: wallet.path().to_path_buf(),
        proven,
    })
}

#[derive(Debug)]
struct Made {
    wallet_path: PathBuf,
    proven: Proven,
}

fn write_and_report(request: &Request, manifest: &Manifest, made: &Made) -> ExitCode {
    let json = match made.proven.package.to_json() {
        Ok(json) => json,
        Err(refusal) => {
            print_refusal(&refusal);
            return ExitCode::from(EXIT_ERROR);
        }
    };
    if let Err(problem) = std::fs::write(&request.out, json) {
        eprintln!("error: cannot write {}: {problem}", request.out.display());
        return ExitCode::from(EXIT_ERROR);
    }
    print_report(request, manifest, made);
    ExitCode::SUCCESS
}

fn print_report(request: &Request, manifest: &Manifest, made: &Made) {
    print_header(&made.wallet_path, manifest, request);
    println!();
    println!("h0 Ironwood note-commitment root  {}", hex::encode(made.proven.nc_root));
    println!(
        "h1 Ironwood nullifier-set root    {}",
        hex::encode(made.proven.nf_imt_root)
    );
    println!();
    println!(
        "proved from {} note(s) held unmoved across the window",
        made.proven.positions.len()
    );
    println!("  {:>13}", "tree position");
    for position in &made.proven.positions {
        println!("  {position:>13}");
    }
    println!();
    println!("package written to {}", request.out.display());
    println!("the proof verified here before the package was written, and so did its binding signature");
    println!("the package carries no address, note value, tree position or nullifier");
    println!("the spending key was derived in memory only; no transaction was created");
}

fn print_header(wallet_path: &Path, manifest: &Manifest, request: &Request) {
    println!("wallet        {} (read-only)", wallet_path.display());
    println!("identity      {}", request.identity.display());
    println!("network       {}", manifest.network);
    println!("context       {}", manifest.context);
    println!(
        "window        h0 {} through h1 {}",
        manifest.window.h0, manifest.window.h1
    );
    println!(
        "lightwalletd  {} (one request: GetTreeState {})",
        request.lightwalletd, manifest.window.h0
    );
    println!("pir server    {}", request.pir);
    match request.expected_h1_root.as_deref() {
        Some(expected) => println!("expected root {expected}"),
        None => println!("expected root — (none given; the server's own h1 root is used)"),
    }
}
