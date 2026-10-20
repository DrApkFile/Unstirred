//! The root authority the HTTP tests hand the gate: two sources that agree, in this process. No
//! server is contacted, so the h0 root and the tip under test are exactly the ones the synthetic
//! wallet's own tree produced. **Synthetic:** this says nothing about any chain.

use unstirred_core::Network;
use unstirred_gate::RootAuthority;
use unstirred_verifier::{root_source_note, AgreedRoots, RootAuthorityReadout, RootsFromServer, SourceOutcome};

#[derive(Debug, Clone)]
pub struct FixedRoots {
    pub nc_root_at_h0: [u8; 32],
    pub tip: u32,
    pub sources: Vec<String>,
}

impl RootAuthority for FixedRoots {
    fn consult(&self, _network: Network, h0: u32) -> RootAuthorityReadout {
        let per_source = self
            .sources
            .iter()
            .map(|source| SourceOutcome {
                source: source.clone(),
                answer: Ok(RootsFromServer {
                    server: source.clone(),
                    nc_root_at_h0: self.nc_root_at_h0,
                    h0,
                    tip: self.tip,
                }),
            })
            .collect();
        RootAuthorityReadout {
            per_source,
            agreement: Ok(AgreedRoots {
                nc_root_at_h0: self.nc_root_at_h0,
                tip: self.tip,
            }),
        }
    }

    fn note(&self) -> Option<&'static str> {
        root_source_note(self.sources.len())
    }

    fn describe(&self) -> Vec<String> {
        self.sources.clone()
    }
}
