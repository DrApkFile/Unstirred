use std::process::ExitCode;

mod args;
mod lookup;
mod notes;
mod refusal;
mod witness;

pub(crate) const EXIT_REFUSED: u8 = 1;
pub(crate) const EXIT_USAGE: u8 = 2;
pub(crate) const EXIT_ERROR: u8 = 3;

const USAGE: &str = "unstirred: private proof of holding duration for shielded ZEC

Commands (status as of this build):
  notes      list the Ironwood notes a wallet held across a gate's window, and the selection
  witness    rebuild each selected note's witness at h0 and check it against the h0 root
  lookup     privately prove each selected note's nullifier absent from the h1 nullifier set
  prove      not built yet (milestone M1)
  verify     not built yet (milestone M1)
  gate init  not built yet (milestone M3)
  roots      not built yet (milestone M2)
  codes      list every verdict code with its meaning

Exit codes: 0 success, 1 refused, 2 usage, 3 local error";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("notes") => notes::run(&args[1..]),
        Some("witness") => witness::run(&args[1..]),
        Some("lookup") => lookup::run(&args[1..]),
        Some("codes") => {
            for code in unstirred_core::Code::ALL {
                let spec = code.spec();
                println!(
                    "{:<30} {}  retryable={}  {}",
                    spec.wire_name, spec.kind, spec.retryable, spec.message
                );
            }
            ExitCode::SUCCESS
        }
        Some("--version") => {
            println!("unstirred {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}
