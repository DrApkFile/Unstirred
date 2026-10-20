use std::process::ExitCode;

mod admit;
mod args;
mod gate;
mod knock;
mod lookup;
mod notes;
mod prove;
mod proving;
mod refusal;
mod root_sources;
mod roots;
mod verify;
mod witness;

pub(crate) const EXIT_REFUSED: u8 = 1;
pub(crate) const EXIT_USAGE: u8 = 2;
pub(crate) const EXIT_ERROR: u8 = 3;
pub(crate) const EXIT_INCONCLUSIVE: u8 = 4;

const USAGE: &str = "unstirred: private proof of holding duration for shielded ZEC

Commands (status as of this build):
  notes      list the Ironwood notes a wallet held across a gate's window, and the selection
  witness    rebuild each selected note's witness at h0 and check it against the h0 root
  lookup     privately prove each selected note's nullifier absent from the h1 nullifier set
  prove      prove the holding and write a proof package
  knock      knock on a room of a running gate: fetch its manifest and challenge, prove, admit
  verify     verify a proof package against the gate's root sources and print the verdict
  roots      print the h0 root and tip each root source reports, and whether they agree
  admit      admit a package into the gate database, consuming its challenge and recording markers
  gate serve       serve one gate over HTTP: rooms, challenges, admissions, room logs
  gate init        write a gate manifest for one room
  gate challenge   issue a challenge and print its hex
  gate markers     print stored markers and admission verdicts
  codes      list every verdict code with its meaning

Exit codes: 0 success, 1 refused, 2 usage, 3 local error, 4 inconclusive";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("notes") => notes::run(&args[1..]),
        Some("witness") => witness::run(&args[1..]),
        Some("lookup") => lookup::run(&args[1..]),
        Some("prove") => prove::run(&args[1..]),
        Some("knock") => knock::run(&args[1..]),
        Some("verify") => verify::run(&args[1..]),
        Some("roots") => roots::run(&args[1..]),
        Some("admit") => admit::run(&args[1..]),
        Some("gate") => gate::run(&args[1..]),
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
