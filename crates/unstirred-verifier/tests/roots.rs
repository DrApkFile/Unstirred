//! The root authority (PRD section 4): two sources, one root. **Synthetic:** every source in
//! this target is a fake in-process object; no server is contacted. The z_gettreestate parsing
//! fixture lives in the verifier's unit tests (`src/roots/zebra/treestate.rs`), derived from
//! zebra-rpc 7.0.0-rc.0's own response type and height-0 snapshot. The sources that disagree,
//! lie about a height or cannot be reached are in `tests/adversarial/`.

#[path = "adversarial"]
mod adversarial {
    mod hostile_root_sources;
}

mod roots {
    pub mod fake_source;
    mod single_source;
}
