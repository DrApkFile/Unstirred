use std::path::{Path, PathBuf};
use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_prover::{
    eligible_notes, fetch_ironwood_tree_state, witnesses_at, IronwoodTreeState, ProverError, ReadOnlyWallet, Witnesses,
};

use crate::args::{optional, parse_flags, required, Flags};
use crate::refusal::{load_manifest, print_refusal};
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred witness --wallet <dir> --manifest <file> --lightwalletd <url> [--account <uuid>]

Rebuilds the note commitment witness of each selected note against the Ironwood root at h0, and
checks that every path hashes to that root. The wallet is opened read-only and no transaction is
created. One network request is made: GetTreeState for h0.";

const FLAGS: [&str; 4] = ["--wallet", "--manifest", "--lightwalletd", "--account"];

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
    match look(&request, &manifest) {
        Ok(found) => {
            print_report(&request, &manifest, &found);
            ExitCode::SUCCESS
        }
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
    account: Option<String>,
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        Ok(Self {
            wallet: PathBuf::from(required(flags, "--wallet")?),
            manifest: required(flags, "--manifest")?.to_owned(),
            lightwalletd: required(flags, "--lightwalletd")?.to_owned(),
            account: optional(flags, "--account").map(str::to_owned),
        })
    }
}

#[derive(Debug)]
struct Found {
    wallet_path: PathBuf,
    witnesses: Witnesses,
}

fn look(request: &Request, manifest: &Manifest) -> Result<Found, ProverError> {
    let wallet = ReadOnlyWallet::open(&request.wallet, manifest.network)?;
    let account = wallet.account(request.account.as_deref())?;
    let selected = eligible_notes(&wallet, account, manifest)?.selection;
    let tree_state = ironwood_tree_state_at_h0(request, manifest)?;
    let witnesses = witnesses_at(&wallet, &tree_state, &selected.notes)?;
    Ok(Found {
        wallet_path: wallet.path().to_path_buf(),
        witnesses,
    })
}

fn ironwood_tree_state_at_h0(request: &Request, manifest: &Manifest) -> Result<IronwoodTreeState, ProverError> {
    fetch_ironwood_tree_state(&request.lightwalletd, manifest.network, manifest.window.h0)
}

fn print_report(request: &Request, manifest: &Manifest, found: &Found) {
    print_header(&found.wallet_path, manifest, request);
    println!();
    println!(
        "h0 Ironwood note-commitment root  {}",
        hex::encode(found.witnesses.root)
    );
    println!();
    println!("selected notes");
    println!(
        "  {:>13}  {:>18}  path verifies against the h0 root",
        "tree position", "value (zatoshis)"
    );
    for witness in &found.witnesses.notes {
        println!(
            "  {:>13}  {:>18}  {}",
            witness.note.position,
            witness.note.value_zatoshis,
            verdict(witness.path_root == found.witnesses.root)
        );
    }
    println!();
    println!("no transaction was created; the wallet was never opened for writing");
}

fn print_header(wallet_path: &Path, manifest: &Manifest, request: &Request) {
    println!("wallet        {} (read-only)", wallet_path.display());
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
}

fn verdict(verified: bool) -> &'static str {
    if verified {
        "yes"
    } else {
        "no"
    }
}
