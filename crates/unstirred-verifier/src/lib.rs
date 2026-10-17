#![doc = "Unstirred verifier: admits or refuses one proof package against the gate's own pinned \
roots, never anything the package says about itself (PRD section 6). The engine is pure; the only \
network access is `roots`, which reads public tree state and the chain tip from a lightwalletd \
server. Markers and challenges arrive in milestone M2, the two-server root authority with them."]

pub mod engine;
pub mod roots;

pub use engine::{verify_package, verify_parsed, PinnedRoots, Verdict};
pub use roots::{fetch_h0_root_and_tip, RootsFromServer};
