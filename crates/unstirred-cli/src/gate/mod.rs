use std::process::ExitCode;

mod init;
mod serve;
mod store_commands;

use crate::{EXIT_ERROR, EXIT_USAGE};

const SUBCOMMANDS: &str = "unstirred gate <serve|init|challenge|markers>";

pub fn run(args: &[String]) -> ExitCode {
    match args.first().map(String::as_str) {
        Some("serve") => serve::run(&args[1..]),
        Some("init") => init::run(&args[1..]),
        Some("challenge") => store_commands::challenge(&args[1..]),
        Some("markers") => store_commands::markers(&args[1..]),
        _ => {
            eprintln!(
                "{SUBCOMMANDS}\n\n{}\n\n{}\n\n{}\n\n{}",
                serve::USAGE,
                init::USAGE,
                store_commands::CHALLENGE_USAGE,
                store_commands::MARKERS_USAGE
            );
            ExitCode::from(EXIT_USAGE)
        }
    }
}

pub(crate) fn usage(problem: &str, text: &str) -> ExitCode {
    eprintln!("{problem}\n\n{text}");
    ExitCode::from(EXIT_USAGE)
}

pub(crate) fn local(detail: &str) -> ExitCode {
    eprintln!("error: {detail}");
    ExitCode::from(EXIT_ERROR)
}
