#![doc = "What a knock prints: which room it is knocking on and on whose terms, what this wallet \
can prove before the gate is asked for anything, then the gate's own verdict object, unchanged. \
The verdict comes from the gate; this command adds no judgement of its own and says plainly what \
it could not read, and what it never sent."]

use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_gate::{Answer, IssuedChallenge};

use super::Request;
use crate::proving::{ForcedDemo, Made, Prepared};
use crate::{EXIT_ERROR, EXIT_INCONCLUSIVE, EXIT_REFUSED};

/// In `--json` mode stdout carries the gate's answer and nothing else, so every human line goes
/// to stderr instead of being dropped.
fn out(json: bool, line: String) {
    if json {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}

pub(super) fn print_header(request: &Request, manifest: &Manifest, json: bool) {
    out(json, format!("gate          {}", request.gate));
    out(json, format!("room          {}", request.room));
    out(json, format!("network       {}", manifest.network));
    out(
        json,
        format!(
            "window        h0 {} through h1 {}",
            manifest.window.h0, manifest.window.h1
        ),
    );
    out(json, format!("threshold     {} zatoshis", manifest.min_zatoshis));
}

pub(super) fn print_challenge(challenge: &IssuedChallenge, json: bool) {
    out(
        json,
        format!("challenge     issued by the gate, expires {}", challenge.expires_at),
    );
}

/// Printed only once there is something to prove and a challenge to answer, so that the line is
/// never on screen for a knock that proved nothing.
pub(super) fn print_proving_now(prepared: &Prepared, json: bool) {
    out(
        json,
        format!(
            "proving now; {} note(s) against the note-commitment root at block {}; \
             this takes seconds and reveals nothing to the gate but the package",
            prepared.notes().len(),
            prepared.nc_root_height()
        ),
    );
}

/// The forced submission says what it is, before any proving starts: which rule it fails, what it
/// is proving instead, and what the gate is expected to answer. The heights here are read from
/// the holder's own wallet and are never sent: the package carries the room's published window
/// and no height of this holder's.
pub(super) fn print_demo(demo: &ForcedDemo, json: bool) {
    out(
        json,
        "DEMO: submitting a proof that does not meet this room's rule".to_owned(),
    );
    out(
        json,
        format!(
            "  this room refuses these coins: {}  {}",
            demo.refused_by_the_room.code.spec().wire_name,
            demo.refused_by_the_room.detail
        ),
    );
    out(
        json,
        format!(
            "  proving instead against the latest note-commitment root these coins appear in, at block {}",
            demo.prepared.nc_root_height()
        ),
    );
    if let Some(first) = demo.prepared.notes().iter().map(|note| note.mined_height).min() {
        out(
            json,
            format!(
                "  your coins: {} note(s), first mined at block {}, unspent through block {} \
                 (read from your own wallet; the gate is told no height and no value of yours)",
                demo.prepared.notes().len(),
                first,
                demo.prepared.notes_unspent_at()
            ),
        );
    }
    out(
        json,
        "  expect the gate to refuse this with ROOT_MISMATCH: it decides from the roots it pinned, \
         not from anything the package says"
            .to_owned(),
    );
}

/// A failure on the holder's own side. The gate was given nothing, and before a challenge is
/// issued it has not even been told that somebody tried.
pub(super) fn print_nothing_sent(json: bool, challenge_was_issued: bool) {
    out(json, "nothing was sent to the gate".to_owned());
    if !challenge_was_issued {
        out(
            json,
            "no challenge was asked for, so this gate has no record of this attempt".to_owned(),
        );
    }
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
        "the gate was sent one package and nothing else: {} note(s) behind it, and no address, \
         note value, tree position or nullifier",
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
