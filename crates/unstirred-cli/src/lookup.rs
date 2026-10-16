use std::path::{Path, PathBuf};
use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_prover::{
    eligible_notes, look_up_nullifiers, root_from_hex, NullifierLookup, ProverError, ReadOnlyWallet,
};

use crate::args::{optional, parse_flags, required, Flags};
use crate::refusal::{load_manifest, print_refusal};
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred lookup --wallet <dir> --manifest <file> --pir <url> \\
    [--expected-h1-root <hex>] [--account <uuid>]

Asks a Valar nf-server, privately, whether each selected note's nullifier is absent from the
Ironwood nullifier set at h1, and checks every answer against the h1 root. Nullifiers are only
ever sent inside a PIR query, never in the clear, and none are printed. With --expected-h1-root,
a server serving any other root is refused before a single query is sent. The wallet is opened
read-only and no transaction is created.";

const FLAGS: [&str; 5] = ["--wallet", "--manifest", "--pir", "--expected-h1-root", "--account"];

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
        Ok(found) => report(&request, &manifest, &found),
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
    pir: String,
    expected_h1_root: Option<String>,
    account: Option<String>,
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        Ok(Self {
            wallet: PathBuf::from(required(flags, "--wallet")?),
            manifest: required(flags, "--manifest")?.to_owned(),
            pir: required(flags, "--pir")?.to_owned(),
            expected_h1_root: optional(flags, "--expected-h1-root").map(str::to_owned),
            account: optional(flags, "--account").map(str::to_owned),
        })
    }
}

#[derive(Debug)]
struct Found {
    wallet_path: PathBuf,
    lookup: NullifierLookup,
}

fn look(request: &Request, manifest: &Manifest) -> Result<Found, ProverError> {
    let expected_root = match request.expected_h1_root.as_deref() {
        Some(hex_root) => Some(root_from_hex(hex_root)?),
        None => None,
    };
    let wallet = ReadOnlyWallet::open(&request.wallet, manifest.network)?;
    let account = wallet.account(request.account.as_deref())?;
    let selected = eligible_notes(&wallet, account, manifest)?.selection;
    let positions: Vec<u64> = selected.notes.iter().map(|note| note.position).collect();
    let nullifiers = wallet.ironwood_nullifiers(account, manifest.window.h0, &positions)?;
    let lookup = look_up_nullifiers(
        &request.pir,
        manifest.network,
        manifest.window.h1,
        expected_root,
        &nullifiers,
    )?;
    Ok(Found {
        wallet_path: wallet.path().to_path_buf(),
        lookup,
    })
}

fn report(request: &Request, manifest: &Manifest, found: &Found) -> ExitCode {
    print_report(request, manifest, found);
    if found.lookup.all_proven_unspent() {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "error: the wallet recorded every selected note as unspent at h1, but the nullifier set at h1 \
             does not exclude all of them, so no proof can be made from this selection"
        );
        ExitCode::from(EXIT_ERROR)
    }
}

fn print_report(request: &Request, manifest: &Manifest, found: &Found) {
    print_header(&found.wallet_path, manifest, request);
    println!();
    println!("h1 Ironwood nullifier-set root  {}", hex::encode(found.lookup.root));
    println!();
    println!("selected notes");
    println!("  {:>13}  proven unspent at h1", "tree position");
    for note in &found.lookup.notes {
        println!("  {:>13}  {}", note.position, verdict(note.proven_unspent));
    }
    println!();
    println!("no nullifier was printed here or sent in the clear; each one was queried privately");
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
    println!("pir server    {}", request.pir);
    match request.expected_h1_root.as_deref() {
        Some(expected) => println!("expected root {expected}"),
        None => println!("expected root — (none given; the server's own h1 root is used)"),
    }
}

fn verdict(proven: bool) -> &'static str {
    if proven {
        "yes"
    } else {
        "no"
    }
}
