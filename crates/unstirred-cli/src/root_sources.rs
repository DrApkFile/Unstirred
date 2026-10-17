use unstirred_core::{Network, Refusal};
use unstirred_verifier::{
    consult, parse_root_source_spec, root_source_note, RootAuthorityReadout, RootSource, SINGLE_SOURCE_NOTE,
};

/// What the commands tell a holder or judge about the root-source rules.
pub const SOURCE_RULES: &str = "--root-source <spec> is lwd:<url> for a lightwalletd gRPC server \
or zebra:<url> for a Zebra node's JSON-RPC. Without --single-root-source, exactly two \
--root-source values are required and must agree on one h0 root; --single-root-source takes \
exactly one source and labels every output as not independently checked.";

/// The root sources one command run consults, as parsed from `--root-source` and
/// `--single-root-source`.
pub struct RootSourceArgs {
    pub sources: Vec<Box<dyn RootSource>>,
    pub single: bool,
}

impl std::fmt::Debug for RootSourceArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RootSourceArgs")
            .field(
                "sources",
                &self.sources.iter().map(|source| source.describe()).collect::<Vec<_>>(),
            )
            .field("single", &self.single)
            .finish()
    }
}

impl RootSourceArgs {
    pub fn parse(specs: &[String], single: bool) -> Result<Self, String> {
        let sources = specs
            .iter()
            .map(String::as_str)
            .map(parse_root_source_spec)
            .collect::<Result<Vec<_>, _>>()?;
        match (sources.len(), single) {
            (0, _) => Err("--root-source <lwd:<url>|zebra:<url>> is required".to_owned()),
            (1, true) | (2, false) => Ok(Self { sources, single }),
            (_, true) => Err("--single-root-source takes exactly one --root-source".to_owned()),
            (_, false) => Err(format!(
                "exactly two --root-source values are required (or one with \
                 --single-root-source), got {}",
                sources.len()
            )),
        }
    }

    pub fn consult(&self, network: Network, h0: u32) -> RootAuthorityReadout {
        consult(&self.sources, network, h0)
    }

    pub fn note(&self) -> Option<&'static str> {
        root_source_note(self.sources.len())
    }
}

/// The human provenance block: what each source answered, whether they agreed, and which tip the
/// window check will use.
pub fn print_readout_human(readout: &RootAuthorityReadout, network: Network, h0: u32, single: bool) {
    println!("root sources for the h0 Ironwood note-commitment root (network {network}, h0 {h0}):");
    for outcome in &readout.per_source {
        match &outcome.answer {
            Ok(answer) => println!(
                "  {}  root {}  tip {}",
                outcome.source,
                hex::encode(answer.nc_root_at_h0),
                answer.tip
            ),
            Err(refusal) => println!("  {}  unusable: {} ({})", outcome.source, refusal.code, refusal.detail),
        }
    }
    match &readout.agreement {
        Ok(agreed) => {
            println!(
                "agreement: the sources agree on one h0 root {}",
                hex::encode(agreed.nc_root_at_h0)
            );
            println!(
                "chain tip for the window check: {} (the lower of the answers)",
                agreed.tip
            );
        }
        Err(refusal) => println!("agreement: {} ({})", refusal.code, refusal.detail),
    }
    if single {
        println!("{SINGLE_SOURCE_NOTE}");
    }
}

/// The same provenance for `--json`, on stderr, so stdout stays one verdict object.
pub fn print_readout_json_stderr(readout: &RootAuthorityReadout, single: bool) {
    for outcome in &readout.per_source {
        match &outcome.answer {
            Ok(answer) => eprintln!(
                "source {} root {} tip {}",
                outcome.source,
                hex::encode(answer.nc_root_at_h0),
                answer.tip
            ),
            Err(refusal) => eprintln!(
                "source {} unusable: {} ({})",
                outcome.source, refusal.code, refusal.detail
            ),
        }
    }
    if single {
        eprintln!("{SINGLE_SOURCE_NOTE}");
    }
}

/// One refusal on stderr for `--json`, where stdout is reserved for the verdict object.
pub fn print_refusal_json_stderr(refusal: &Refusal) {
    eprintln!("{}: {}", refusal.code, refusal.detail);
}
