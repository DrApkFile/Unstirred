use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::slice::Iter;

use unstirred_core::{parse_manifest, Manifest, Refusal};
use unstirred_prover::{eligible_notes, EligibleNotes, HeldNote, ProverError, ReadOnlyWallet};

use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

pub const USAGE: &str = "unstirred notes --wallet <dir> --manifest <file> [--account <uuid>]

Lists the Ironwood notes the wallet held unmoved across the gate's window, and the fewest of them
that reach the gate's threshold. The wallet is opened read-only and no transaction is created.";

pub fn run(args: &[String]) -> ExitCode {
    let request = match Request::from_args(args) {
        Ok(request) => request,
        Err(problem) => {
            eprintln!("{problem}\n\n{USAGE}");
            return ExitCode::from(EXIT_USAGE);
        }
    };
    let json = match std::fs::read_to_string(&request.manifest) {
        Ok(json) => json,
        Err(problem) => {
            eprintln!("error: cannot read {}: {problem}", request.manifest.display());
            return ExitCode::from(EXIT_ERROR);
        }
    };
    let manifest = match parse_manifest(&json) {
        Ok(manifest) => manifest,
        Err(refusal) => {
            print_refusal(&refusal);
            return ExitCode::from(EXIT_REFUSED);
        }
    };
    match look(&request, &manifest) {
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

#[derive(Debug)]
struct Request {
    wallet: PathBuf,
    manifest: PathBuf,
    account: Option<String>,
}

impl Request {
    fn from_args(args: &[String]) -> Result<Self, String> {
        let mut wallet = None;
        let mut manifest = None;
        let mut account = None;
        let mut remaining = args.iter();
        while let Some(flag) = remaining.next() {
            match flag.as_str() {
                "--wallet" => wallet = Some(PathBuf::from(value_after(flag, &mut remaining)?)),
                "--manifest" => manifest = Some(PathBuf::from(value_after(flag, &mut remaining)?)),
                "--account" => account = Some(value_after(flag, &mut remaining)?),
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        Ok(Self {
            wallet: wallet.ok_or_else(|| "--wallet <dir> is required".to_owned())?,
            manifest: manifest.ok_or_else(|| "--manifest <file> is required".to_owned())?,
            account,
        })
    }
}

fn value_after(flag: &str, remaining: &mut Iter<'_, String>) -> Result<String, String> {
    remaining.next().cloned().ok_or_else(|| format!("{flag} needs a value"))
}

fn look(request: &Request, manifest: &Manifest) -> Result<(PathBuf, EligibleNotes), ProverError> {
    let wallet = ReadOnlyWallet::open(&request.wallet, manifest.network)?;
    let account = wallet.account(request.account.as_deref())?;
    let found = eligible_notes(&wallet, account, manifest)?;
    Ok((wallet.path().to_path_buf(), found))
}

fn print_refusal(refusal: &Refusal) {
    let spec = refusal.code.spec();
    println!("{}  {}", spec.kind, spec.wire_name);
    println!("  {}", refusal.detail);
    println!("  why:  {}", spec.message);
    println!("  next: {}", spec.next_action);
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

fn print_note_table(notes: &[HeldNote]) {
    println!(
        "  {:>18}  {:>12}  {:>13}",
        "value (zatoshis)", "mined height", "tree position"
    );
    for note in notes {
        println!(
            "  {:>18}  {:>12}  {:>13}",
            note.value_zatoshis, note.mined_height, note.position
        );
    }
}
