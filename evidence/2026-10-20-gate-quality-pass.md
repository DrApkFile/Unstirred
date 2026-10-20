# Gate run: quality pass, M2 closed, A28-A33

Commit `8398181` plus six uncommitted files (`CLAIMS.md`, `DECISIONS.md`, `HANDOFF.md`,
`README.md`, `briefing/PRD.md`, `crates/unstirred-cli/src/verify/report.rs`), which hold the
M2-closed wording, the A33 registration and the A33 test.
Date: 2026-10-20T08:13:36Z. Toolchain: `rustc 1.91.1 (ed61e7d7e 2025-11-07)`, pinned by
`rust-toolchain.toml`. Command: `./scripts/gate.sh`. Exit: 0.

```
GATE: PASS
```

No chain was contacted by this run. It records what the code does, not what the network did: the
real-testnet runs are the other files in this directory.

## Steps

| Step | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --release --all-targets --all-features -- -D warnings` | clean, no warnings |
| `cargo test --release --workspace --all-features` | 76 passed, 0 failed, 0 ignored |
| `./scripts/leak-scan.sh` | no key material and no address in any tracked file; `evidence/2026-10-17-first-proof.json` carries only the package schema's fields |
| `./scripts/claims-check.sh` | 21 claims: 10 proven on the real testnet, 3 synthetic, 5 settled by a recorded gate run, 3 named as not built |

## Tests, by target

| Target | Passed | Time |
|---|---|---|
| `unstirred` (cli unit, A27, A33 x2) | 3 | 0.00s |
| `unstirred-core` lib | 0 | — |
| `unstirred-core` `tests/properties.rs` (A28-A31) | 4 | 0.04s |
| `unstirred-core` `tests/rules.rs` | 8 | 0.00s |
| `unstirred-gate` lib | 0 | — |
| `unstirred-prover` lib | 0 | — |
| `unstirred-prover` `tests/identity.rs` | 5 | 0.78s |
| `unstirred-prover` `tests/pir.rs` | 5 | 2.78s |
| `unstirred-prover` `tests/properties.rs` (A32) | 2 | 0.01s |
| `unstirred-prover` `tests/prove.rs` | 3 | 51.17s |
| `unstirred-prover` `tests/selection.rs` | 8 | 0.02s |
| `unstirred-prover` `tests/synthetic_wallet.rs` | 4 | 2.84s |
| `unstirred-prover` `tests/wallet_read_only.rs` | 3 | 0.73s |
| `unstirred-prover` `tests/witness.rs` | 5 | 2.70s |
| `unstirred-verifier` lib (zebra treestate parse) | 4 | 1.64s |
| `unstirred-verifier` `tests/roots.rs` | 6 | 0.00s |
| `unstirred-verifier` `tests/store.rs` | 10 | 104.53s |
| `unstirred-verifier` `tests/verify.rs` | 6 | 53.94s |
| doc-tests (4 crates) | 0 | — |

The long targets are the ones that build a real K=12 proof: one in `prove.rs`, one in `verify.rs`
and three in the `store.rs` corpus, each built once per target.

## What this run does not prove

- Nothing about the Zcash network. Every test here is synthetic or pure.
- Not that a recorded run in `evidence/` says what `CLAIMS.md` says it says: the claims check
  tests that the files exist and that every run is cited, not that the words match.
- Not that a running process keeps a secret off a channel; the leak scan reads tracked files.
