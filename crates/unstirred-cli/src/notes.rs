use std::path::{Path, PathBuf};
use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_prover::{eligible_notes, EligibleNotes, ProverError, ReadOnlyWallet};

use crate::args::{optional, parse_flags, required, Flags};
use crate::refusal::{load_manifest, print_note_table, print_refusal};
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred notes --wallet <dir> --manifest <file> [--account <uuid>]

Lists the Ironwood notes the wallet held unmoved across the gate's window, and the fewest of them
that reach the gate's threshold. The wallet is opened read-only and no transaction is created.";

const FLAGS: [&str; 3] = ["--wallet", "--manifest", "--account"];

pub fn run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &FLAGS) {
        Ok(flags) => flags,
        Err(problem) => {
            eprintln!("{problem}\n\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let (wallet, manifest_path) = match wallet_and_manifest(&flags) {
        Ok(paths) => paths,
        Err(problem) => {
            eprintln!("{problem}\n\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let manifest = match load_manifest(&manifest_path) {
        Ok(manifest) => manifest,
        Err(exit) => return exit,
    };
    match look(&wallet, &manifest, optional(&flags, "--account")) {
        Ok((wallet_path, found)) => {
            print_report(&wallet_path, &manifest, &found);
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

fn wallet_and_manifest(flags: &Flags) -> Result<(PathBuf, String), String> {
    let wallet = PathBuf::from(required(flags, "--wallet")?);
    let manifest = required(flags, "--manifest")?.to_owned();
    Ok((wallet, manifest))
}

fn look(wallet: &Path, manifest: &Manifest, account: Option<&str>) -> Result<(PathBuf, EligibleNotes), ProverError> {
    let wallet = ReadOnlyWallet::open(wallet, manifest.network)?;
    let account = wallet.account(account)?;
    let found = eligible_notes(&wallet, account, manifest)?;
    Ok((wallet.path().to_path_buf(), found))
}

fn print_report(wallet_path: &Path, manifest: &Manifest, found: &EligibleNotes) {
    println!("wallet       {} (read-only)", wallet_path.display());
    println!("network      {}", manifest.network);
    println!("context      {}", manifest.context);
    println!("account      {}", found.account.expose_uuid());
    println!("chain tip    {} (as the wallet knows it)", found.chain_tip);
    println!("scanned to   {}", found.fully_scanned);
    println!(
        "window       h0 {} through h1 {}",
        manifest.window.h0, manifest.window.h1
    );
    println!("threshold    {} zatoshis", found.min_zatoshis);
    println!();
    println!(
        "unspent Ironwood notes at h0: {}, of those still unspent at h1: {}",
        found.notes.unspent_at_h0.len(),
        found.notes.held_through_h1.len()
    );
    println!();
    println!("eligible notes");
    print_note_table(&found.notes.held_through_h1);
    println!();
    println!(
        "selection    {} of {} notes, {} zatoshis",
        found.selection.notes.len(),
        found.notes.held_through_h1.len(),
        found.selection.total_zatoshis
    );
    print_note_table(&found.selection.notes);
    println!();
    println!("no transaction was created; the wallet was never opened for writing");
}
