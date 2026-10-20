#![doc = "Agreeing on one root. Every source is asked, and only answers for the gate's h0 from a \
chain that has reached it can be used; those must then carry the same root. The governing tip is the \
lower of the answers, the conservative choice for the finality check."]

use unstirred_core::{Code, Network, Refusal};

use super::{RootSource, RootsFromServer};

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
