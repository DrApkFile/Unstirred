#![doc = "Where a running gate gets its h0 root and the chain tip. The authority is a trait so a \
test can hand the service fixed roots without a server; the gate itself holds the \
`--root-source` specs it was started with and builds the sources again for each admission, \
because a source is a boxed trait object that cannot be shared across the blocking tasks \
verification runs in."]

use unstirred_core::{Code, Network, Refusal};
use unstirred_verifier::{consult, parse_root_source_spec, root_source_note, RootAuthorityReadout};

/// The h0 root and tip for one window, with the provenance of every answer. Both implementations
/// decide nothing: an unusable source is INCONCLUSIVE inside the readout, as it is for the CLI.
pub trait RootAuthority: std::fmt::Debug + Send + Sync {
    /// Asks every source for the h0 root and the tip, and reports what each one said alongside
    /// the agreement they reached or the refusal that stopped them.
    fn consult(&self, network: Network, h0: u32) -> RootAuthorityReadout;

    /// The single-source label, when only one source was configured.
    fn note(&self) -> Option<&'static str>;

    /// How the gate names its sources in its startup lines and its room listing.
    fn describe(&self) -> Vec<String>;
}

/// The authority a served gate uses: the specs from its config, read over the network.
#[derive(Debug)]
pub struct SpecRootAuthority {
    specs: Vec<String>,
}

impl SpecRootAuthority {
    /// Fails for any spec the verifier cannot turn into a source, so a gate refuses to start
    /// rather than discovering it at the first knock.
    pub fn new(specs: &[String]) -> Result<Self, String> {
        for spec in specs {
            parse_root_source_spec(spec)?;
        }
        Ok(Self { specs: specs.to_vec() })
    }
}

impl RootAuthority for SpecRootAuthority {
    fn consult(&self, network: Network, h0: u32) -> RootAuthorityReadout {
        let built = self
            .specs
            .iter()
            .map(|spec| parse_root_source_spec(spec))
            .collect::<Result<Vec<_>, _>>();
        match built {
            Ok(sources) => consult(&sources, network, h0),
            Err(detail) => RootAuthorityReadout {
                per_source: Vec::new(),
                agreement: Err(Refusal::new(
                    Code::RootSourceUnavailable,
                    format!("this gate's root sources cannot be built: {detail}"),
                )),
            },
        }
    }

    fn note(&self) -> Option<&'static str> {
        root_source_note(self.specs.len())
    }

    fn describe(&self) -> Vec<String> {
        self.specs.clone()
    }
}
