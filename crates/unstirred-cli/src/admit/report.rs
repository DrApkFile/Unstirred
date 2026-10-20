#![doc = "What an admit run prints: the provenance of the roots it pinned before it decided, then the \
verdict, in words or as the JSON verdict object. The database write itself is the verifier's."]

use std::process::ExitCode;

use unstirred_core::Manifest;
use unstirred_verifier::{verdict_object, RootAuthorityReadout, Verdict};

use super::Request;
use crate::refusal::print_refusal;
use crate::root_sources::{print_readout_human, print_readout_json_stderr};
use crate::{EXIT_INCONCLUSIVE, EXIT_REFUSED};

pub(super) fn print_roots(json: bool, request: &Request, readout: &RootAuthorityReadout, manifest: &Manifest) {
    let h1 = hex::encode(request.h1_root);
    if json {
        print_readout_json_stderr(readout, request.sources.single);
        eprintln!("h1 nullifier-set root pinned by the operator (--h1-root): {h1}");
        return;
    }
    print_readout_human(readout, manifest.network, manifest.window.h0, request.sources.single);
    println!("h1 nullifier-set root    {h1}");
    println!(
        "    pinned by the operator on the command line, labelled operator-pinned (nf-server verify-root is deferred to P1)"
    );
}

pub(super) fn finish(json: bool, verdict: &Verdict, note: Option<&'static str>) -> ExitCode {
    if json {
        println!("{}", verdict_object(verdict, note));
    }
    match verdict {
        Verdict::Admitted => {
            if !json {
                if let Some(note) = note {
                    println!("{note}");
                }
                println!("ADMITTED");
            }
            ExitCode::SUCCESS
        }
        Verdict::Refused(refusal) => {
            if !json {
                print_refusal(refusal);
            }
            ExitCode::from(EXIT_REFUSED)
        }
        Verdict::Inconclusive(refusal) => {
            if !json {
                print_refusal(refusal);
            }
            ExitCode::from(EXIT_INCONCLUSIVE)
        }
    }
}
