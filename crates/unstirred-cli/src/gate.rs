use std::process::ExitCode;

use unstirred_verifier::store::{Store, SystemClock};

use crate::args::{parse_flags, required};
use crate::refusal::load_manifest;
use crate::{EXIT_ERROR, EXIT_USAGE};

const CHALLENGE_USAGE: &str = "unstirred gate challenge --db <file> --manifest <file>

Issues one challenge for the manifest's context and prints its hex. The database stores only a
hash of it. The challenge expires after 10 minutes.";

const MARKERS_USAGE: &str = "unstirred gate markers --db <file>

Prints the markers and admission verdicts in the gate database. Challenge hashes are not printed.";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("challenge") => challenge(&args[1..]),
        Some("markers") => markers(&args[1..]),
        Some("init") => {
            eprintln!("unstirred gate init is not built yet (milestone M3)\n");
            ExitCode::from(EXIT_USAGE)
        }
        _ => {
            eprintln!("unstirred gate <challenge|markers>\n\n{CHALLENGE_USAGE}\n\n{MARKERS_USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn challenge(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &["--db", "--manifest"]) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem, CHALLENGE_USAGE),
    };
    let database = match required(&flags, "--db") {
        Ok(path) => path,
        Err(problem) => return usage(&problem, CHALLENGE_USAGE),
    };
    let manifest_path = match required(&flags, "--manifest") {
        Ok(path) => path,
        Err(problem) => return usage(&problem, CHALLENGE_USAGE),
    };
    let manifest = match load_manifest(manifest_path) {
        Ok(manifest) => manifest,
        Err(exit) => return exit,
    };
    let store = match Store::open(database) {
        Ok(store) => store,
        Err(error) => return local(&error.to_string()),
    };
    match store.issue(&manifest.context, &SystemClock) {
        Ok(raw) => {
            println!("{}", hex::encode(raw));
            ExitCode::SUCCESS
        }
        Err(error) => local(&error.to_string()),
    }
}

fn markers(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &["--db"]) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem, MARKERS_USAGE),
    };
    let database = match required(&flags, "--db") {
        Ok(path) => path,
        Err(problem) => return usage(&problem, MARKERS_USAGE),
    };
    let store = match Store::open(database) {
        Ok(store) => store,
        Err(error) => return local(&error.to_string()),
    };
    let markers = match store.markers() {
        Ok(markers) => markers,
        Err(error) => return local(&error.to_string()),
    };
    let admissions = match store.admissions() {
        Ok(admissions) => admissions,
        Err(error) => return local(&error.to_string()),
    };
    println!("markers:");
    for row in &markers {
        println!(
            "  context={} marker={} admitted_at={}",
            row.context,
            hex::encode(row.marker),
            row.admitted_at
        );
    }
    println!("admissions:");
    for row in &admissions {
        let code = row.code.as_deref().unwrap_or("—");
        println!(
            "  context={} verdict={} code={code} at={}",
            row.context, row.verdict, row.at
        );
    }
    ExitCode::SUCCESS
}

fn usage(problem: &str, text: &str) -> ExitCode {
    eprintln!("{problem}\n\n{text}");
    ExitCode::from(EXIT_USAGE)
}

fn local(detail: &str) -> ExitCode {
    eprintln!("error: {detail}");
    ExitCode::from(EXIT_ERROR)
}
