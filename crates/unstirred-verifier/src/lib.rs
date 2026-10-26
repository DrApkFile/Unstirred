#![doc = "Unstirred verifier: admits or refuses one proof package against the gate's own pinned \
roots, never anything the package says about itself (PRD section 6). The engine is pure. `roots` \
is the only network access, and `store` is the only database: hashed challenges, markers and \
admission rows. The h1 nullifier-set root stays operator-pinned, labelled as operator-pinned."]
#![warn(missing_docs)]

pub mod admit;
pub mod engine;
pub mod roots;
pub mod store;
mod verdict;

pub use admit::admit;
pub use engine::{verdict_object, verify_package, verify_parsed, PinnedRoots, Verdict};
pub use roots::{
    consult, parse_root_source_spec, root_source_note, AgreedRoots, LightwalletdSource, RootAuthorityReadout,
    RootSource, RootsFromServer, SourceOutcome, ZebraSource, SINGLE_SOURCE_NOTE,
};
pub use verdict::as_verdict;
