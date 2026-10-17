#![doc = "The verifier's root authority (PRD section 4): the Ironwood note-commitment root at h0 \
and the chain tip, each read from independent root sources. The default is two sources that must \
agree on exactly one root; one source is an explicit opt-in that is labelled as unchecked in every \
output. Sources are lightwalletd gRPC (GetTreeState, GetLatestBlock) and a Zebra node's JSON-RPC \
(z_gettreestate, getblockchaininfo, getblockcount). Every source failure is INCONCLUSIVE: a \
verifier that cannot establish its own roots cannot decide anything."]

mod lightwalletd;
mod zebra;

pub use lightwalletd::LightwalletdSource;
pub use zebra::ZebraSource;

use unstirred_core::{Code, Network, Refusal};
use zcash_client_backend::proto::service::TreeState;

/// What one root-source read returned, and where it came from, so a report can say exactly which
/// server every root and the tip came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootsFromServer {
    pub server: String,
    pub nc_root_at_h0: [u8; 32],
    pub h0: u32,
    pub tip: u32,
}

/// One source's outcome in a consultation, in the order the sources were given.
#[derive(Debug)]
pub struct SourceOutcome {
    pub source: String,
    pub answer: Result<RootsFromServer, Refusal>,
}

/// The one root the sources agreed on, with the most conservative tip they reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgreedRoots {
    pub nc_root_at_h0: [u8; 32],
    pub tip: u32,
}

/// Everything one consultation produced: each source's answer or failure, plus the authority's
/// agreement result.
#[derive(Debug)]
pub struct RootAuthorityReadout {
    pub per_source: Vec<SourceOutcome>,
    pub agreement: Result<AgreedRoots, Refusal>,
}

/// One independent place the verifier asks for the h0 root and the chain tip (PRD section 4).
/// A read blocks and must not be called from inside an async runtime; the source owns its whole
/// transport and rejects answers for the wrong network or height itself.
pub trait RootSource: std::fmt::Debug {
    /// The name this source reports itself by in provenance output and refusals.
    fn describe(&self) -> String;

    fn read_roots(&self, network: Network, h0: u32) -> Result<RootsFromServer, Refusal>;
}

/// Consults every source in order and agrees on one root. Any source that fails, or that cannot
/// be used for this gate, is `ROOT_SOURCE_UNAVAILABLE`; two different h0 roots are
/// `ROOT_SOURCES_DISAGREE`; otherwise the tip is the lower of the answers, the conservative
/// choice for the finality check.
pub fn consult(sources: &[Box<dyn RootSource>], network: Network, h0: u32) -> RootAuthorityReadout {
    let per_source: Vec<SourceOutcome> = sources
        .iter()
        .map(|source| SourceOutcome {
            source: source.describe(),
            answer: source.read_roots(network, h0),
        })
        .collect();
    RootAuthorityReadout {
        agreement: agree(&per_source, h0),
        per_source,
    }
}

fn agree(per_source: &[SourceOutcome], h0: u32) -> Result<AgreedRoots, Refusal> {
    if per_source.is_empty() {
        return Err(Refusal::new(
            Code::RootSourceUnavailable,
            "no root sources were configured",
        ));
    }
    let mut unusable = Vec::new();
    let mut usable = Vec::new();
    for outcome in per_source {
        match &outcome.answer {
            Err(refusal) => unusable.push(format!("{}: {}: {}", outcome.source, refusal.code, refusal.detail)),
            Ok(answer) if answer.h0 != h0 => unusable.push(format!(
                "{}: answered for height {}, not the gate's h0 {h0}",
                outcome.source, answer.h0
            )),
            Ok(answer) if answer.tip < h0 => unusable.push(format!(
                "{}: reports tip {} behind h0 {h0}, so it cannot serve this gate",
                outcome.source, answer.tip
            )),
            Ok(answer) => usable.push(answer),
        }
    }
    if !unusable.is_empty() {
        return Err(Refusal::new(
            Code::RootSourceUnavailable,
            format!("root source failure: {}", unusable.join("; ")),
        ));
    }
    let first = &usable[0];
    if !usable.iter().all(|a| a.nc_root_at_h0 == first.nc_root_at_h0) {
        let detail = usable
            .iter()
            .map(|a| format!("{} says {}", a.server, hex::encode(a.nc_root_at_h0)))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(Refusal::new(
            Code::RootSourcesDisagree,
            format!("the root sources do not agree on the h0 root: {detail}"),
        ));
    }
    Ok(AgreedRoots {
        nc_root_at_h0: first.nc_root_at_h0,
        tip: usable.iter().fold(first.tip, |lowest, a| lowest.min(a.tip)),
    })
}

/// Parses one `--root-source` value: `lwd:<url>` for a lightwalletd gRPC server, `zebra:<url>`
/// for a Zebra node's JSON-RPC.
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

/// Reads the Ironwood root out of a tree state after checking the answer is for the height and
/// the network that were asked for, and that it carries an Ironwood tree at all. A wrong height
/// or network, or a tree state without the Ironwood pool, means this source cannot serve this
/// gate, which is a root-source failure, not a refusal of the holder.
pub(crate) fn ironwood_root_at(response: &TreeState, network: Network, requested: u32) -> Result<[u8; 32], String> {
    let returned = u32::try_from(response.height)
        .map_err(|_| format!("the server reported height {}, which does not fit", response.height))?;
    if returned != requested {
        return Err(format!(
            "the server answered a tree state for height {returned}, not the gate's h0 {requested}"
        ));
    }
    if !response.network.to_ascii_lowercase().starts_with(&network.to_string()) {
        return Err(format!(
            "the server reports network {:?}, not {:?}",
            response.network, network
        ));
    }
    if response.ironwood_tree.is_empty() {
        return Err(
            "the answer carries no Ironwood tree, so this source cannot serve an Ironwood-height gate".to_owned(),
        );
    }
    let tree = response
        .ironwood_tree()
        .map_err(|e| format!("the Ironwood tree field did not parse: {e}"))?;
    Ok(tree.to_frontier().root().to_bytes())
}

/// One source said nothing usable. Unknown is never success: no verdict of any kind is produced
/// from a guessed root, and the caller must report INCONCLUSIVE.
pub(crate) fn unavailable(source: &str, detail: impl Into<String>) -> Refusal {
    Refusal::new(
        Code::RootSourceUnavailable,
        format!("root source {source}: {}", detail.into()),
    )
}

/// The note a single-source run must carry in every output, so a one-source answer can never
/// pass as independently checked (A27).
pub const SINGLE_SOURCE_NOTE: &str = "ROOT FROM ONE SOURCE: not independently checked";

/// The label a run's outputs must carry: the single-source note when exactly one source was
/// consulted, nothing when the two-source rule ran.
pub fn root_source_note(source_count: usize) -> Option<&'static str> {
    (source_count == 1).then_some(SINGLE_SOURCE_NOTE)
}
