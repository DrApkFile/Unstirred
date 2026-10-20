//! The fake root sources the authority tests consult: each one answers from a value in this
//! process, so no server is contacted and every disagreement is arranged on purpose.

use std::error::Error;

use unstirred_core::{Code, Network, Refusal};
use unstirred_verifier::{RootSource, RootsFromServer};

pub type TestResult = Result<(), Box<dyn Error>>;

pub const H0: u32 = 4_505_790;
pub const ROOT_A: [u8; 32] = [0xa5; 32];
pub const ROOT_B: [u8; 32] = [0x5a; 32];

#[derive(Debug)]
pub enum Answer {
    /// An answer for `h0`, carrying this root and tip.
    Roots(u32, [u8; 32], u32),
    /// The source cannot be reached.
    Fails,
}

#[derive(Debug)]
pub struct FakeSource {
    pub name: String,
    pub answer: Answer,
}

impl RootSource for FakeSource {
    fn describe(&self) -> String {
        self.name.clone()
    }

    fn read_roots(&self, _network: Network, _h0: u32) -> Result<RootsFromServer, Refusal> {
        match &self.answer {
            Answer::Roots(answered_h0, root, tip) => Ok(RootsFromServer {
                server: self.describe(),
                nc_root_at_h0: *root,
                h0: *answered_h0,
                tip: *tip,
            }),
            Answer::Fails => Err(Refusal::new(
                Code::RootSourceUnavailable,
                format!("{} cannot be reached", self.name),
            )),
        }
    }
}

pub fn sources(first: Answer, second: Answer) -> Vec<Box<dyn RootSource>> {
    vec![
        Box::new(FakeSource {
            name: "fake:first".to_owned(),
            answer: first,
        }),
        Box::new(FakeSource {
            name: "fake:second".to_owned(),
            answer: second,
        }),
    ]
}

pub fn expect_unavailable(readout: &unstirred_verifier::RootAuthorityReadout) -> Result<Refusal, Box<dyn Error>> {
    match &readout.agreement {
        Err(refusal) if refusal.code == Code::RootSourceUnavailable => Ok(refusal.clone()),
        Err(refusal) => Err(format!(
            "expected ROOT_SOURCE_UNAVAILABLE, got {}: {}",
            refusal.code, refusal.detail
        )
        .into()),
        Ok(agreed) => Err(format!(
            "expected ROOT_SOURCE_UNAVAILABLE, the sources agreed on {}",
            hex::encode(agreed.nc_root_at_h0)
        )
        .into()),
    }
}
