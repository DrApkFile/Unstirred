#![doc = "Where one verify run's words go: the provenance of every root before the verdict, the \
verdict itself, and the same two things as JSON. A one-source run carries its note in both forms."]

use std::process::ExitCode;

use unstirred_core::{Manifest, Refusal};
use unstirred_verifier::{verdict_object, RootAuthorityReadout, Verdict};

use super::{inconclusive, ROOT_BYTES};
use crate::refusal::print_refusal;
use crate::root_sources::{print_readout_human, print_readout_json_stderr, print_refusal_json_stderr};
use crate::EXIT_REFUSED;

/// What an ADMITTED verdict from `verify` means, in both outputs: this command keeps no
/// database, so it enters the package nowhere. One constant, so the two outputs cannot drift.
const NO_MARKERS_RECORDED: &str = "verify does not record markers; use unstirred admit to enter a gate";

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
                eprintln!("{NO_MARKERS_RECORDED}");
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
                for line in admitted_report(note) {
                    println!("{line}");
                }
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

/// Every line an ADMITTED verdict prints, in order: the single-source note when only one source
/// was consulted, the verdict, and what this command did not do. The JSON path prints the verdict
/// as an object and the last line beside it, so both outputs end on the same sentence.
fn admitted_report(note: Option<&'static str>) -> Vec<&'static str> {
    let mut lines = Vec::new();
    lines.extend(note);
    lines.push("ADMITTED");
    lines.push(NO_MARKERS_RECORDED);
    lines
}

#[cfg(test)]
mod tests {
    use unstirred_verifier::SINGLE_SOURCE_NOTE;

    use super::{admitted_report, NO_MARKERS_RECORDED};

    /// A33: an admitted package is reported as admitted and as recorded nowhere, and the sentence
    /// that says so names the command that does record, in both outputs.
    #[test]
    fn a33_an_admitted_verdict_says_verify_recorded_nothing_and_names_admit() {
        assert_eq!(admitted_report(None), vec!["ADMITTED", NO_MARKERS_RECORDED]);
        assert_eq!(
            admitted_report(Some(SINGLE_SOURCE_NOTE)),
            vec![SINGLE_SOURCE_NOTE, "ADMITTED", NO_MARKERS_RECORDED],
            "a one-source run keeps its label above the verdict"
        );
        assert_eq!(
            admitted_report(None).last(),
            Some(&NO_MARKERS_RECORDED),
            "the line the JSON path prints beside its verdict is the line the human output ends on"
        );
        assert!(
            NO_MARKERS_RECORDED.contains("verify") && NO_MARKERS_RECORDED.contains("admit"),
            "the sentence must name the command that records nothing and the one that records: {NO_MARKERS_RECORDED}"
        );
        assert!(
            !NO_MARKERS_RECORDED.contains("not recorded"),
            "it must say what verify does, not read as a verdict about the markers: {NO_MARKERS_RECORDED}"
        );
    }

    /// A33: the sentence must not name a milestone. The line it replaced said "(M2)", which became
    /// false the moment M2 closed, and nothing failed when it did.
    #[test]
    fn a33_the_sentence_names_no_milestone_that_can_pass() {
        let milestone = NO_MARKERS_RECORDED
            .chars()
            .zip(NO_MARKERS_RECORDED.chars().skip(1))
            .any(|(letter, digit)| letter == 'M' && digit.is_ascii_digit());
        assert!(
            !milestone,
            "the sentence names a milestone, so it will go stale: {NO_MARKERS_RECORDED}"
        );
    }
}
