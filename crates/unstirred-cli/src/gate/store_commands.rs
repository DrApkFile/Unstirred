#![doc = "The two commands that read and write the gate database by hand, without a service: issuing \
one challenge, and printing everything the file holds."]

use std::process::ExitCode;

use unstirred_verifier::store::{Store, SystemClock};

use super::{local, usage};
use crate::args::{parse_flags, required};
use crate::refusal::load_manifest;

pub(super) const CHALLENGE_USAGE: &str = "unstirred gate challenge --db <file> --manifest <file>

Issues one challenge for the manifest's context and prints its hex. The database stores only a
hash of it. The challenge expires after 10 minutes.";

pub(super) const MARKERS_USAGE: &str = "unstirred gate markers --db <file>

Prints the markers and admission verdicts in the gate database. Challenge hashes are not printed.";

pub(super) fn challenge(args: &[String]) -> ExitCode {
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
        Ok(issued) => {
            println!("{}", hex::encode(issued.raw));
            ExitCode::SUCCESS
        }
        Err(error) => local(&error.to_string()),
    }
}

pub(super) fn markers(args: &[String]) -> ExitCode {
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
