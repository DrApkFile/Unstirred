# CLAUDE.md: guide for the coding agent on Unstirred

Read `briefing/PRD.md` for the section you are working on before writing code. Do not derive requirements from memory. Read `briefing/BUILD_CONTRACT.md` once per session.

## What this is
A Rust toolkit that proves "held at least 0.125 ZEC in Ironwood, unmoved, from block H0 through H1" for one gate, using Valar Group's delegation circuit with the note-commitment root pinned at H0 and the nullifier-set root pinned at H1.

## Invariants the code cannot show
1. The verifier never uses a root, height, network or context from the proof package. Only from the manifest and its own root authority.
2. `round_id` is derived from network and context only. Never include heights.
3. A proof is only admitted after the binding signature verifies under rk rebuilt from the instance coordinates. The proof alone is NOT enough: a viewing-key holder can produce a valid proof (spike S6).
4. Marker insertion and challenge consumption happen in one database transaction with UNIQUE constraints.
5. The holder's demo wallet is read-only. Nothing in this repo may build or broadcast a transaction from it.
6. Unknown is never success. Root disagreement or unavailability is INCONCLUSIVE.
7. Only `unstirred-core` defines codes, the manifest schema and the binding message. Other crates import them.

## Versions and gotchas newer than your training data
- **Orchard is closed.** Since NU6.3 (2026-07-28) new shielded value lands in the **Ironwood** pool. Ironwood reuses Orchard-protocol receivers (UA typecode 0x03) and note version `NoteVersion::V3`. Code or docs that talk only about Orchard are out of date.
- **zcashd is dead.** Do not use zcashd RPC docs as a runtime reference. Node is Zebra; indexer is Zaino or lightwalletd.
- **Testnet is on NU7** (25-second blocks, observed ~27 s). Mainnet NU7 is targeted for 2026-11-05. Use block heights, never wall-clock assumptions about 75-second blocks.
- **voting-circuits 0.12.2**: build with `--no-default-features --features lrz`. Default features pull Zakura crates instead. Key API: `delegation::{build_delegation_bundle, create_delegation_proof, verify_delegation_proof, RealNoteInput, ImtProvider, SpacedLeafImtProvider}`; `Instance` exposes `nc_root`, `nf_imt_root`, `vote_round_id`, `rk_x`, `rk_y`, `gov_null[5]`, `van_comm`.
- **Rebuilding rk for verification**: take `rk_x.to_repr()`, set the top bit of byte 31 if `rk_y.is_odd()`, then `redpallas::VerificationKey::try_from(bytes)` (see `briefing/spikes/spend_authority_spike.rs`).
- **Historical witnesses**: `WalletDb::generate_ironwood_witnesses_at_historical_height(positions, frontier_at_h0, h0)` in `zcash_client_sqlite 0.22.0`. It builds an in-memory tree from stored shards plus the supplied frontier, so no wallet checkpoint at H0 is needed. The frontier comes from `GetTreeState(h0)`.
- **nf-server**: Ironwood nullifiers only; `SYNC_HEIGHT` must be a multiple of 10; `SVOTE_ZCASH_NETWORK=test` for testnet; `nf-server verify-root` rebuilds the root from raw blocks.
- **PIR to circuit**: `pir_client::ImtProofData` copies field for field into `voting_circuits::delegation::ImtProofData` (`root`, `nf_bounds`, `leaf_pos`, `path`).
- **Threshold**: the circuit requires at least one ballot = 12,500,000 zatoshis total across up to 5 notes.
- **Proving** is about 10–15 s per proof on one core in release mode. Never benchmark debug builds.

## Reference code that already works
- `briefing/spikes/split_height_spike.rs`: split-height proof, newcomer rejection, spent-in-window, markers per context.
- `briefing/spikes/spend_authority_spike.rs`: binding signature, viewing-key-only attack, replay.

## Conventions
- Names, not comments. Rename or split instead of commenting. No commented-out code.
- Split files past ~180 lines.
- Errors are typed and map to the closed code set in `unstirred-core`; every code has `message`, `retryable`, `next_action`.
- Pure core: `unstirred-core` has no network, no clock, no randomness except injected.
- Tests named as behaviour, with acceptance IDs in the title, e.g. `a05_refuses_same_coins_twice_at_one_gate_and_writes_nothing`.
- Adversarial tests live in `tests/adversarial/`.
- Never pipe test output through `tail` or `grep` in a way that can hide a failure. Report the full result line.
- Run `./scripts/gate.sh` from a clean clone before claiming anything works.
- Exact version pins; commit `Cargo.lock`.

## Honesty rules (from the build contract)
- Never show placeholder values as real; show `—`.
- Never report a number before a recorded run produced it; store runs in `evidence/`.
- Synthetic-tree tests are labelled synthetic; real testnet runs are labelled with heights and commit.
- Anything not built is listed as not built in README.

## Scope
New ideas go to `ROADMAP.md`. Do not start P1 work while any P0 item or A-test is open.

## Parallel work
The shared contract (`unstirred-core`) has exactly one owner per session. Other tracks consume it.
