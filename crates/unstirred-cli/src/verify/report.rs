#![doc = "Where one verify run's words go: the provenance of every root before the verdict, the \
verdict itself, and the same two things as JSON. A one-source run carries its note in both forms."]

use std::process::ExitCode;

use unstirred_core::{Manifest, Refusal};
use unstirred_verifier::{verdict_object, RootAuthorityReadout, Verdict};

use super::{inconclusive, ROOT_BYTES};
use crate::refusal::print_refusal;
use crate::root_sources::{print_readout_human, print_readout_json_stderr, print_refusal_json_stderr};
use crate::EXIT_REFUSED;

/// Where the verdict and, around it, the provenance of every root went.
#[derive(Debug)]
pub(super) struct Report<'a> {
    pub(super) json: bool,
    pub(super) single: bool,
    pub(super) package: &'a str,
    pub(super) h1_root: [u8; ROOT_BYTES],
}

impl Report<'_> {
    pub(super) fn root_provenance(&self, readout: &RootAuthorityReadout, manifest: &Manifest) {
        if self.json {
            print_readout_json_stderr(readout, self.single);
            eprintln!(
                "h1 nullifier-set root pinned by the operator (--h1-root): {}",
                hex::encode(self.h1_root)
            );
            return;
        }
        println!(
            "verifying {} against manifest context {:?}, network {}, window h0 {} through h1 {}",
            self.package, manifest.context, manifest.network, manifest.window.h0, manifest.window.h1
        );
        println!();
        print_readout_human(readout, manifest.network, manifest.window.h0, self.single);
        println!("h1 nullifier-set root    {}", hex::encode(self.h1_root));
        println!("    pinned by the operator on the command line, labelled operator-pinned (nf-server verify-root is deferred to P1)");
        println!();
    }

    pub(super) fn root_authority_failure(&self, refusal: &Refusal) {
        if self.json {
            print_refusal_json_stderr(refusal);
        } else {
            print_refusal(refusal);
        }
    }

    pub(super) fn verdict(&self, verdict: &Verdict, note: Option<&'static str>) -> ExitCode {
        if self.json {
            println!("{}", verdict_object(verdict, note));
            if matches!(verdict, Verdict::Admitted) {
                eprintln!("markers not recorded (M2)");
                return ExitCode::SUCCESS;
            }
            return match verdict {
                Verdict::Refused(_) => ExitCode::from(EXIT_REFUSED),
                Verdict::Inconclusive(_) => inconclusive(),
                Verdict::Admitted => ExitCode::SUCCESS,
            };
        }
        match verdict {
            Verdict::Admitted => {
                if let Some(note) = note {
                    println!("{note}");
                }
                println!("ADMITTED");
                println!("markers not recorded (M2)");
                ExitCode::SUCCESS
            }
            Verdict::Refused(refusal) => {
                print_refusal(refusal);
                ExitCode::from(EXIT_REFUSED)
            }
            Verdict::Inconclusive(refusal) => {
                print_refusal(refusal);
                inconclusive()
            }
        }
    }
}
