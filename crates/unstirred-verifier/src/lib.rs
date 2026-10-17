#![doc = "Unstirred verifier: admits or refuses one proof package against the gate's own pinned \
roots, never anything the package says about itself (PRD section 6). The engine is pure; the only \
network access is `roots`, which consults the gate's root sources for the h0 tree state and the \
chain tip. Markers and challenges arrive later in milestone M2; the h1 nullifier-set root stays \
operator-pinned for M2, labelled as operator-pinned."]

pub mod engine;
pub mod roots;

pub use engine::{verdict_object, verify_package, verify_parsed, PinnedRoots, Verdict};
pub use roots::{
    consult, parse_root_source_spec, root_source_note, AgreedRoots, LightwalletdSource, RootAuthorityReadout,
    RootSource, RootsFromServer, SourceOutcome, ZebraSource, SINGLE_SOURCE_NOTE,
};
