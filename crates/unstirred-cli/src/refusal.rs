use std::process::ExitCode;

use unstirred_core::{parse_manifest, Manifest, Refusal};
use unstirred_prover::HeldNote;

use crate::{EXIT_ERROR, EXIT_REFUSED};

/// Prints the verdict kind, the code, what happened, and what the holder can do about it. Every
/// word but the detail comes from the closed code set in `unstirred-core`.
pub fn print_refusal(refusal: &Refusal) {
    let spec = refusal.code.spec();
    println!("{}  {}", spec.kind, spec.wire_name);
    println!("  {}", refusal.detail);
    println!("  why:  {}", spec.message);
    println!("  next: {}", spec.next_action);
}

/// Loads a gate manifest, printing the refusal itself if the file is not one.
pub fn load_manifest(path: &str) -> Result<Manifest, ExitCode> {
    let json = std::fs::read_to_string(path).map_err(|problem| {
        eprintln!("error: cannot read {path}: {problem}");
        ExitCode::from(EXIT_ERROR)
    })?;
    parse_manifest(&json).map_err(|refusal| {
        print_refusal(&refusal);
        ExitCode::from(EXIT_REFUSED)
    })
}

pub fn print_note_table(notes: &[HeldNote]) {
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
