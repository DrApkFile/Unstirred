#![doc = "Starting the gate service. Every reason not to start is reported before the socket is \
bound, and once it is listening the service's own lines say what it hosts and what it reads its \
roots from."]

use std::process::ExitCode;

use unstirred_gate::{load_config, GateState};

use super::{local, usage};
use crate::args::{parse_flags, required};

pub(super) const USAGE: &str = "unstirred gate serve --config <file>

Serves one gate over HTTP. The config is TOML:

    listen = \"127.0.0.1:8080\"
    database = \"gate.sqlite\"
    root_sources = [\"lwd:https://testnet.zec.rocks:443\", \"zebra:http://127.0.0.1:18232\"]
    h1_root = \"<64 hex characters>\"
    # single_root_source = true        # one source, labelled as not independently checked
    [[rooms]]
    manifest = \"holders-room.json\"
    [[rooms]]
    manifest = \"second-room.json\"
    # h1_root = \"<64 hex characters>\" # per room, when its window ends at another height

Paths are relative to the config file. Each room is one manifest, and its context is the
manifest's own.

JSON routes: GET /rooms, GET /rooms/<context>/manifest, POST /rooms/<context>/challenge, POST
/rooms/<context>/admit, GET /rooms/<context>/log, GET /rooms/<context>/recent.
Pages: GET / is the room picker and GET /room/<context> is that room's Holders' Room page, one
self-contained document that fetches nothing from anywhere but this gate. GET
/room/<context>?replay=<act> renders a recorded act of the demo with no polling at all; the acts
are act1-rule, act2-admitted, act3-refused and act5-second-room.

The gate refuses to start on an invalid manifest or a missing setting, and it logs no addresses,
no request bodies and no challenges.";

pub(super) fn run(args: &[String]) -> ExitCode {
    let flags = match parse_flags(args, &["--config"]) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem, USAGE),
    };
    let path = match required(&flags, "--config") {
        Ok(path) => path,
        Err(problem) => return usage(&problem, USAGE),
    };
    let config = match load_config(path) {
        Ok(config) => config,
        Err(problem) => return local(&format!("this gate will not start: {problem}")),
    };
    let state = match GateState::from_config(config) {
        Ok(state) => state,
        Err(problem) => return local(&format!("this gate will not start: {problem}")),
    };
    match unstirred_gate::serve(state) {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => local(&problem.to_string()),
    }
}
