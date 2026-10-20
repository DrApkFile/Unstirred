#![doc = "What a knock prints: which room it is knocking on and on whose terms, then the gate's own \
verdict object, unchanged. The verdict comes from the gate; this command adds no judgement of its \
own and says plainly what it could not read."]

use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_gate::{Answer, IssuedChallenge};

use super::Request;
use crate::proving::Made;
use crate::{EXIT_ERROR, EXIT_INCONCLUSIVE, EXIT_REFUSED};

pub(super) fn print_header(request: &Request, manifest: &Manifest, challenge: &IssuedChallenge, json: bool) {
    let out = |line: String| {
        if json {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    };
    out(format!("gate          {}", request.gate));
    out(format!("room          {}", request.room));
    out(format!("network       {}", manifest.network));
    out(format!(
        "window        h0 {} through h1 {}",
        manifest.window.h0, manifest.window.h1
    ));
    out(format!("threshold     {} zatoshis", manifest.min_zatoshis));
    out(format!(
        "challenge     issued by the gate, expires {}",
        challenge.expires_at
    ));
    out("proving now; this takes seconds and reveals nothing to the gate but the package".to_owned());
}

/// The gate's answer. A verdict object is printed as the gate wrote it; an HTTP-level error is
/// printed as the gate's own words, and neither is restated as something the gate did not say.
pub(super) fn print_answer(answer: &Answer, made: &Made, json: bool) -> ExitCode {
    if json {
        println!("{}", answer.body);
    }
    match answer.verdict() {
        Some(verdict) => {
            if !json {
                if let Some(note) = &verdict.note {
                    println!("{note}");
                }
                println!("{}", verdict.verdict);
                if let Some(code) = &verdict.code {
                    println!("  {code}");
                }
                if let Some(message) = &verdict.message {
                    println!("  {message}");
                }
                if let Some(retryable) = verdict.retryable {
                    println!("  retryable: {retryable}");
                }
                print_what_was_sent(made);
            }
            exit_for(&verdict.verdict)
        }
        None => {
            let said = answer.error().unwrap_or_else(|| answer.body.clone());
            eprintln!("error: the gate answered {} and no verdict: {said}", answer.status);
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn print_what_was_sent(made: &Made) {
    println!(
        "the gate was sent one package proving {} note(s) held unmoved across the window, and nothing else",
        made.proven.positions.len()
    );
    println!(
        "wallet {} was opened read-only; no transaction was created",
        made.wallet_path.display()
    );
}

fn exit_for(verdict: &str) -> ExitCode {
    match verdict {
        "ADMITTED" => ExitCode::SUCCESS,
        "INCONCLUSIVE" => ExitCode::from(EXIT_INCONCLUSIVE),
        _ => ExitCode::from(EXIT_REFUSED),
    }
}
