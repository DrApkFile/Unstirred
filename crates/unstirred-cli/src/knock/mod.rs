use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_gate::{GateClient, GateClientError};
use unstirred_prover::ProverError;

use crate::args::{parse_flags, required, strip_bool_flags, Flags};
use crate::proving::{make_proof, ProofInputs, PROOF_FLAGS};
use crate::refusal::print_refusal;
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

mod report;

use report::{print_answer, print_header};

pub const USAGE: &str = "unstirred knock --gate <url> --room <context> --wallet <dir> \\
    --pir <url> --lightwalletd <url> --identity <age file> [--expected-h1-root <hex>] \\
    [--account <uuid>] [--json]

Knocks on one room of a running gate: fetches the room's manifest and a fresh challenge, proves
that this wallet held at least the gate's threshold in Ironwood unmoved across the window, posts
the package, and prints the gate's verdict. The wallet is opened read-only, no transaction is
created, and the package carries no address, note value, tree position or nullifier.

Exit codes: 0 admitted, 1 refused, 2 usage, 3 local error, 4 inconclusive.";

pub fn run(args: &[String]) -> ExitCode {
    let (args, bools) = match strip_bool_flags(args, &["--json"]) {
        Ok(stripped) => stripped,
        Err(problem) => return usage(&problem),
    };
    let allowed: Vec<&str> = PROOF_FLAGS.iter().copied().chain(["--gate", "--room"]).collect();
    let flags = match parse_flags(&args, &allowed) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem),
    };
    let request = match Request::from_flags(&flags) {
        Ok(request) => request,
        Err(problem) => return usage(&problem),
    };
    let json = bools.iter().any(|flag| flag == "--json");
    match knock(&request, json) {
        Ok(exit) => exit,
        Err(exit) => exit,
    }
}

fn knock(request: &Request, json: bool) -> Result<ExitCode, ExitCode> {
    let client = GateClient::new(&request.gate).map_err(local)?;
    let manifest = client.manifest(&request.room).map_err(local)?;
    require_room_is_its_own_context(&manifest, &request.room)?;
    let challenge = client.challenge(&request.room).map_err(local)?;
    print_header(request, &manifest, &challenge, json);
    let made = make_proof(&request.inputs, &manifest, challenge.raw).map_err(|error| match error {
        ProverError::Refused(refusal) => {
            print_refusal(&refusal);
            ExitCode::from(EXIT_REFUSED)
        }
        other => {
            eprintln!("error: {other}");
            ExitCode::from(EXIT_ERROR)
        }
    })?;
    let package_json = made.proven.package.to_json().map_err(|refusal| {
        print_refusal(&refusal);
        ExitCode::from(EXIT_ERROR)
    })?;
    let answer = client.admit(&request.room, &package_json).map_err(local)?;
    Ok(print_answer(&answer, &made, json))
}

/// A gate serves one manifest per room, so the manifest it hands out under a room's path must be
/// that room's own. A gate that answers with another context is asking this holder to prove for a
/// gate they did not knock on, and nothing is proved.
fn require_room_is_its_own_context(manifest: &Manifest, room: &str) -> Result<(), ExitCode> {
    if manifest.context == room {
        return Ok(());
    }
    eprintln!(
        "error: the gate published a manifest for context {:?} under the room {room:?}; \
         nothing was proved",
        manifest.context
    );
    Err(ExitCode::from(EXIT_ERROR))
}

fn usage(problem: &str) -> ExitCode {
    eprintln!("{problem}\n\n{USAGE}");
    ExitCode::from(EXIT_USAGE)
}

fn local(error: GateClientError) -> ExitCode {
    eprintln!("error: {error}");
    ExitCode::from(EXIT_ERROR)
}

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) gate: String,
    pub(crate) room: String,
    pub(crate) inputs: ProofInputs,
}

impl Request {
    fn from_flags(flags: &Flags) -> Result<Self, String> {
        Ok(Self {
            gate: required(flags, "--gate")?.to_owned(),
            room: required(flags, "--room")?.to_owned(),
            inputs: ProofInputs::from_flags(flags)?,
        })
    }
}
