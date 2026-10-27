# BUILD_CONTRACT: Unstirred

## Frozen decisions (changing one requires a DECISIONS.md entry first)
1. Rust workspace; crates `unstirred-core`, `unstirred-prover`, `unstirred-verifier`, `unstirred-cli`, `unstirred-gate` (PRD §4).
2. Cryptography only through `voting-circuits =0.12.2` with feature `lrz`. No circuit changes, no new circuits.
3. Wallet access through `zcash_client_sqlite =0.22.0` / `zcash_client_backend =0.24.0` (same cohort as the zcash-devtool wallet). We do not depend on `zcash_voting` (release candidate); we call the same lower-level functions it uses.
4. PIR through `pir-client =0.7.4`, `pir-types =0.6.4`; the gate operator runs Valar's `nf-server`.
5. Network for the demo: Zcash public testnet. H0 = 4,505,790 for the Holders' Room.
6. Context derivation excludes heights (PRD §3.2).
7. Verifier never reads a root, height, network or context from the proof package.
8. Threshold fixed at 12,500,000 zatoshis.
9. All dependency versions pinned exactly; `Cargo.lock` committed.

## Never fabricate
- No placeholder proof, root, marker, height or txid shown as real. Show `—` until a real value exists.
- No fallback that returns ADMITTED or a fake proof on any error.
- No test counts, timings or numbers reported before they are produced by a recorded run.
- Demo data is labelled demo. Real testnet runs are labelled with network, heights and commit.
- No simulated chain data presented as live. Synthetic-tree tests are labelled synthetic.

## No silent fallback
- No `unwrap_or_default` / `unwrap_or(…)` on roots, heights, markers, keys or config.
- No swallowed errors: every error maps to a code in the closed set or is propagated.
- Config validated at startup; the gate refuses to start without two root sources and a verified H1 root.
- Unknown is never success: any INCONCLUSIVE path returns INCONCLUSIVE.

## Traceable numbers
Every number in README, demo or submission records: network, h0, h1, commit, command, date, and whether it is real-testnet, synthetic or provisional. Stored under `evidence/`.

## Failures stay on record
Failed runs stay in `evidence/failed/` with the reason. Claims that do not replicate are demoted in README, not deleted silently. Spike history stays in `briefing/REALITY_SPIKES.md`.

## Protected critical path
- `unstirred-core` verification policy (PRD §6 steps 1–10)
- binding message construction and rk reconstruction
- marker insertion transaction
- root authority (two-source agreement, nf-server root check)
No change to these merges unless the gate passes, including A03–A12.

## Wallet safety
Never send, shield or self-send from the holder's demo wallet. Read-only access only. Any code path that could create a transaction from that wallet is a bug.

## Commits and deadline
First commit inside the event window. Last commit before 2026-10-27 08:00 UTC. Nothing pushed after submission.

## Shared contract
Only `unstirred-core` defines the closed code set, the manifest schema and the binding message. The
other crates import them; none adds a code of its own. One owner per session for that crate; the
other tracks consume it.

## Code bar
Planning quality does not excuse code quality.
- Names, not comments. Rename or split instead of commenting. No commented-out code.
- Split files past ~180 lines.
- Errors are typed and map to the closed code set in `unstirred-core`; every code has `message`,
  `retryable`, `next_action`.
- Pure core: `unstirred-core` has no network, no clock, no randomness except injected.
- Tests named as behaviour, with acceptance IDs in the title, e.g.
  `a05_refuses_same_coins_twice_at_one_gate_and_writes_nothing`.
- Adversarial tests live in `tests/adversarial/`.
- Never pipe test output through `tail` or `grep` in a way that can hide a failure. Report the full
  result line.
- Run `./scripts/gate.sh` from a clean clone before claiming anything works.
- Exact version pins; commit `Cargo.lock`.
- New ideas go to `ROADMAP.md`.

## Gate (one command from a clean clone)
`./scripts/gate.sh`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --release` (unit + adversarial), leak scanner (A02), claims check (every README claim links to an `evidence/` file).

## Proof priority (1 = most important)
1. A01 real testnet admission with recorded heights, roots and verdict
2. A06 viewing-key-only attack refused
3. A03 newcomer refused (prover and verifier)
4. A05 / A08 single use per gate, unlinkable across gates
5. A02 no leak in packages and logs
6. A10 disagreeing root servers never admit
