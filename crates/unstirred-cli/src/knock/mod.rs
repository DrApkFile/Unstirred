use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_gate::{GateClient, GateClientError};
use unstirred_prover::ProverError;

use crate::args::{optional, parse_flags, required, strip_bool_flags, Flags};
use crate::proving::{
    prepare, prepare_forced_demo, Advertised, NotForcible, Prepared, ProofInputs, DEMO_FLAG, PROOF_FLAGS,
};
use crate::refusal::print_refusal;
use crate::{EXIT_ERROR, EXIT_REFUSED, EXIT_USAGE};

mod report;

use report::{print_answer, print_challenge, print_demo, print_header, print_nothing_sent, print_proving_now};

pub const USAGE: &str = "unstirred knock --gate <url> --room <context> --wallet <dir> \\
    --identity <age file> [--pir <url>] [--lightwalletd <url>] [--expected-h1-root <hex>] \\
    [--account <uuid>] [--json]

Knocks on one room of a running gate: checks this wallet against the room's published rule, and
only if it has something to prove fetches a fresh challenge, proves that it held at least the
gate's threshold in Ironwood unmoved across the window, posts the package and prints the gate's
verdict. A wallet the rule refuses never asks for a challenge, so the gate keeps no record of it.
The wallet is opened read-only, no transaction is created, and the package carries no address,
note value, tree position or nullifier.

--pir and --lightwalletd may be left out when the gate advertises its own, which it does in
GET /rooms; a flag you give always wins over what the gate suggests. The gate is only asked when
one of them is missing.

Exit codes: 0 admitted, 1 refused, 2 usage, 3 local error, 4 inconclusive.";

pub fn run(args: &[String]) -> ExitCode {
    let (args, bools) = match strip_bool_flags(args, &["--json", DEMO_FLAG]) {
        Ok(stripped) => stripped,
        Err(problem) => return usage(&problem),
    };
    let allowed: Vec<&str> = PROOF_FLAGS.iter().copied().chain(["--gate", "--room"]).collect();
    let flags = match parse_flags(&args, &allowed) {
        Ok(flags) => flags,
        Err(problem) => return usage(&problem),
    };
    let json = bools.iter().any(|flag| flag == "--json");
    let forced = bools.iter().any(|flag| flag == DEMO_FLAG);
    match knock(&flags, json, forced) {
        Ok(exit) => exit,
        Err(exit) => exit,
    }
}

fn knock(flags: &Flags, json: bool, forced: bool) -> Result<ExitCode, ExitCode> {
    let gate = required(flags, "--gate").map_err(|problem| usage(&problem))?;
    let client = GateClient::new(gate).map_err(local)?;
    let request = Request::resolve(flags, gate, &client).map_err(|problem| usage(&problem))?;
    let request = &request;
    let manifest = client.manifest(&request.room).map_err(local)?;
    require_room_is_its_own_context(&manifest, &request.room)?;
    print_header(request, &manifest, json);
    let prepared = if forced {
        forced_demo(request, &manifest, json)?
    } else {
        prepare(&request.inputs, &manifest).map_err(|error| refusal_or_error(error, json, false))?
    };
    let challenge = client.challenge(&request.room).map_err(local)?;
    print_challenge(&challenge, json);
    print_proving_now(&prepared, json);
    let made = prepared
        .prove(&request.inputs, &manifest, challenge.raw)
        .map_err(|error| refusal_or_error(error, json, true))?;
    let package_json = made.proven.package.to_json().map_err(|refusal| {
        print_refusal(&refusal);
        print_nothing_sent(json, true);
        ExitCode::from(EXIT_ERROR)
    })?;
    let answer = client.admit(&request.room, &package_json).map_err(local)?;
    Ok(print_answer(&answer, &made, json))
}

/// The demo's forced submission (PRD section 9, act 4). It says what it is doing, and it only
/// works for a wallet this room refuses.
fn forced_demo(request: &Request, manifest: &Manifest, json: bool) -> Result<Prepared, ExitCode> {
    match prepare_forced_demo(&request.inputs, manifest) {
        Ok(demo) => {
            print_demo(&demo, json);
            Ok(demo.prepared)
        }
        Err(NotForcible::TheRoomWouldAdmitThis) => {
            eprintln!(
                "error: this wallet meets this room's rule, so there is nothing to force; \
                 knock without {DEMO_FLAG} and the proof is an honest one"
            );
            print_nothing_sent(json, false);
            Err(ExitCode::from(EXIT_USAGE))
        }
        Err(NotForcible::Local(error)) => Err(refusal_or_error(error, json, false)),
    }
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

/// A refusal or a fault on the holder's own side. Either way the gate was given nothing, which
/// the holder is told in those words rather than left to infer from a missing verdict.
fn refusal_or_error(error: ProverError, json: bool, challenge_was_issued: bool) -> ExitCode {
    let exit = match error {
        ProverError::Refused(refusal) => {
            print_refusal(&refusal);
            EXIT_REFUSED
        }
        other => {
            eprintln!("error: {other}");
            EXIT_ERROR
        }
    };
    print_nothing_sent(json, challenge_was_issued);
    ExitCode::from(exit)
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
    /// What this knock will do, once the gate has been asked what it advertises. The gate is only
    /// asked when the holder left a service flag out — a knock that names both addresses makes
    /// exactly the requests it made before this existed (A50).
    fn resolve(flags: &Flags, gate: &str, client: &GateClient) -> Result<Self, String> {
        let needs_advertisement = optional(flags, "--pir").is_none() || optional(flags, "--lightwalletd").is_none();
        let advertised = if needs_advertisement {
            let services = client
                .services()
                .map_err(|e| format!("this gate could not be asked what it advertises: {e}"))?;
            Advertised {
                pir: services.pir_url,
                lightwalletd: services.lightwalletd_url,
            }
        } else {
            Advertised::default()
        };
        Ok(Self {
            gate: gate.to_owned(),
            room: required(flags, "--room")?.to_owned(),
            inputs: ProofInputs::from_flags_or(flags, &advertised)?,
        })
    }
}
