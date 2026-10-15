use std::process::ExitCode;

const USAGE: &str = "unstirred: private proof of holding duration for shielded ZEC

Commands (status as of this build):
  prove      not built yet (milestone M1)
  verify     not built yet (milestone M1)
  gate init  not built yet (milestone M3)
  roots      not built yet (milestone M2)
  codes      list every verdict code with its meaning";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("codes") => {
            for code in unstirred_core::Code::ALL {
                let spec = code.spec();
                println!(
                    "{:<30} {:?}  retryable={}  {}",
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
            ExitCode::from(2)
        }
    }
}
