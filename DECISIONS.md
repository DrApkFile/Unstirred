# DECISIONS

Every change to a frozen decision in briefing/BUILD_CONTRACT.md is recorded here first.

| Date | Decision | Why | Evidence |
|---|---|---|---|
| 2026-10-15 | Do not depend on zcash_voting (release candidate); call zcash_client_sqlite and voting-circuits directly | Stable pins only | briefing/REALITY_SPIKES.md S5 |
| 2026-10-15 | Gate storage is an embedded SQLite file created on first run (`gate.sqlite`) | Transactions + UNIQUE constraints enforce single use under races; zero setup for judges; one gate server is enough for v1 | PRD §4, FAILURE_MODES gate storage |
| 2026-10-15 | Distribute via GitHub Releases (prebuilt Linux/macOS binaries) and crates.io; not npm | Rust project; npm only for a future JS client | — |
| 2026-10-15 | Publish to crates.io once, as 0.1.0, about 2026-10-26, from the final tagged commit, before the deadline; CLI package renamed to `unstirred` so `cargo install unstirred` works | Publishing is permanent; published code must match the judged repo | crates.io names checked free 2026-10-15 |
| 2026-10-15 | Three judge paths: live hosted Holders' Room (decide by M3), `unstirred gate serve` locally, and `unstirred verify` with no database | Judges never handle a database | PRD §16 |
| 2026-10-15 | Add `unstirred gate markers` to print everything the gate stores | Shows the gate keeps only markers and verdicts | PRD §16 |
| 2026-10-16 | Added acceptance IDs A17 (held-note listing and selection) and A18 (prover threshold and sync refusals) to PRD §10 | §10 had IDs for COINS_TOO_NEW (A03) and read-only wallet access (A16) but none for the selection rules M1 step 1 implements, and every test needs a registered ID | crates/unstirred-prover/tests/ |
| 2026-10-16 | The prover reads "held" as the intersection of the wallet's own unspent sets at h0 and h1, and passes the wallet's recorded chain tip to `unstirred_core::check_window` | Needs no network call and adds no rule outside core; the verifier still re-derives heights and roots from its own sources (PRD §6) | crates/unstirred-prover/src/wallet.rs |
| open | Reserve crate names now with a 0.0.1 placeholder? | Owner's call | — |
