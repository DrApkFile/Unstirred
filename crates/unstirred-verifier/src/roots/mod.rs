#![doc = "The verifier's root authority (PRD section 4): the Ironwood note-commitment root at h0 \
and the chain tip, each read from independent root sources. The default is two sources that must \
agree on exactly one root; one source is an explicit opt-in that is labelled as unchecked in every \
output. Sources are lightwalletd gRPC (GetTreeState, GetLatestBlock) and a Zebra node's JSON-RPC \
(z_gettreestate, getblockchaininfo, getblockcount). Every source failure is INCONCLUSIVE: a \
verifier that cannot establish its own roots cannot decide anything."]

mod agreement;
mod lightwalletd;
mod tree_state;
mod zebra;

pub use agreement::{consult, AgreedRoots, RootAuthorityReadout, SourceOutcome};
pub use lightwalletd::LightwalletdSource;
pub use zebra::ZebraSource;

pub(crate) use tree_state::{ironwood_root_at, unavailable};

use unstirred_core::{Network, Refusal};

/// What one root-source read returned, and where it came from, so a report can say exactly which
/// server every root and the tip came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootsFromServer {
    /// How the source names itself, as [`RootSource::describe`] gives it.
    pub server: String,
    /// The Ironwood note-commitment root this source reports for `h0`.
    pub nc_root_at_h0: [u8; 32],
    /// The height the root is for. The agreement step refuses an answer for any other height.
    pub h0: u32,
    /// The chain tip this source reports, used for the finality check.
    pub tip: u32,
}

/// One independent place the verifier asks for the h0 root and the chain tip (PRD section 4).
/// A read blocks and must not be called from inside an async runtime; the source owns its whole
/// transport and rejects answers for the wrong network or height itself.
pub trait RootSource: std::fmt::Debug {
    /// The name this source reports itself by in provenance output and refusals.
    fn describe(&self) -> String;

    /// Reads the h0 root and the chain tip. Every failure, including a wrong network or a chain
    /// that has not reached `h0`, is a `Refusal` with an INCONCLUSIVE code: a source that cannot
    /// answer says nothing about the holder.
    fn read_roots(&self, network: Network, h0: u32) -> Result<RootsFromServer, Refusal>;
}

/// Parses one `--root-source` value: `lwd:<url>` for a lightwalletd gRPC server, `zebra:<url>`
/// for a Zebra node's JSON-RPC. Parsing reaches no network; the source is contacted only when it
/// is read.
///
/// ```
/// use unstirred_verifier::parse_root_source_spec;
///
/// assert_eq!(
///     parse_root_source_spec("lwd:https://zcash-testnet.chainsafe.dev")?.describe(),
///     "lwd:https://zcash-testnet.chainsafe.dev",
/// );
/// assert!(parse_root_source_spec("https://example.invalid").is_err()); // no kind
/// assert!(parse_root_source_spec("rpc:http://127.0.0.1:8232").is_err()); // unknown kind
/// # Ok::<(), String>(())
/// ```
pub fn parse_root_source_spec(spec: &str) -> Result<Box<dyn RootSource>, String> {
    let (kind, url) = spec
        .split_once(':')
        .ok_or_else(|| format!("--root-source must be lwd:<url> or zebra:<url>, got {spec:?}"))?;
    match kind {
        "lwd" => Ok(Box::new(LightwalletdSource::new(url))),
        "zebra" => Ok(Box::new(ZebraSource::new(url)?)),
        other => Err(format!(
            "unknown root-source kind {other:?}; use lwd:<url> or zebra:<url>"
        )),
    }
}

/// The note a single-source run must carry in every output, so a one-source answer can never
/// pass as independently checked (A27).
pub const SINGLE_SOURCE_NOTE: &str = "ROOT FROM ONE SOURCE: not independently checked";

/// The label a run's outputs must carry: the single-source note when exactly one source was
/// consulted, nothing when the two-source rule ran.
///
/// ```
/// use unstirred_verifier::{root_source_note, SINGLE_SOURCE_NOTE};
///
/// assert_eq!(root_source_note(1), Some(SINGLE_SOURCE_NOTE));
/// assert_eq!(root_source_note(2), None);
/// ```
pub fn root_source_note(source_count: usize) -> Option<&'static str> {
    (source_count == 1).then_some(SINGLE_SOURCE_NOTE)
}
