#![doc = "What the prove command prints once the package is written: the two roots it pinned, how many \
notes it proved from, and the plain statement of what the package does not carry."]

use std::path::Path;
use std::process::ExitCode;

use unstirred_core::Manifest;

use super::Request;
use crate::proving::Made;
use crate::refusal::print_refusal;
use crate::EXIT_ERROR;

pub(super) fn write_and_report(request: &Request, manifest: &Manifest, made: &Made) -> ExitCode {
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
    println!("identity      {}", request.inputs.identity.display());
    println!("network       {}", manifest.network);
    println!("context       {}", manifest.context);
    println!(
        "window        h0 {} through h1 {}",
        manifest.window.h0, manifest.window.h1
    );
    println!(
        "lightwalletd  {} (one request: GetTreeState {})",
        request.inputs.lightwalletd, manifest.window.h0
    );
    println!("pir server    {}", request.inputs.pir);
    match request.inputs.expected_h1_root.as_deref() {
        Some(expected) => println!("expected root {expected}"),
        None => println!("expected root — (none given; the server's own h1 root is used)"),
    }
}
