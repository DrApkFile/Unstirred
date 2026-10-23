# Gate run: the release-prep pass

Commit `3f79c4e` plus the uncommitted release-prep tree: act 4's broken axis in `ui/render.js` and
`ui/tokens.css` with the `aside` field it needed in `src/page/contract.rs` and the regenerated
`act4-newcomer.json`; `SPEC.md`; the rebuilt `README.md` with its status table generated from
`CLAIMS.md` by the new `scripts/status-table.py`, which `scripts/gate.sh` now re-runs with
`--check`; `briefing/LEAKS.md` finalised so every row names a test or is a stated non-defence, and
the two tests that took two rows out of "planned" — **A47** (`unstirred-verifier`
`tests/store/fresh_randomizer.rs`) and **A48** (`unstirred-prover`
`tests/adversarial/pir_lookup.rs`); `scripts/claims-check.sh` widened to hold `SPEC.md` and
`briefing/LEAKS.md` to their evidence links as well as the README's; the release workflow
`.github/workflows/release.yml`; both licence files; and the crates.io metadata on every crate,
including the CLI package renamed `unstirred-cli` → `unstirred` and the `Cargo.lock` that rename
rewrote.
Date: 2026-10-23T00:48:21Z. Toolchain: `rustc 1.91.1 (ed61e7d7e 2025-11-07)`, pinned by
`rust-toolchain.toml`. Command: `./scripts/gate.sh`. Exit: 0.

```
GATE: PASS
```

**Run under a memory limit.** This machine has 7 GB of RAM, and an earlier attempt at the same
gate was killed while `tests/http.rs` was building its K=12 proofs in parallel. The recorded run
was made as `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh`, with zebrad, the nf-server
and every running gate stopped first. Those settings change how much of the suite runs at once and
nothing about what it asserts: every target is the same target, and the serialised run is slower
(`tests/http.rs` 140.89s against 111.43s in the parallel run of 2026-10-22).

No chain was contacted by this run, and nothing in it touched either real wallet. The
real-testnet values in it are the five replay acts', copied by the generator from
`evidence/2026-10-20-two-room-knock.txt`, `evidence/2026-10-22-bob-forced-knock.txt` and
`evidence/2026-10-22-bob-knock.txt`.

## Steps

| Step | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --release --all-targets --all-features -- -D warnings` | clean, no warnings |
| `cargo test --release --workspace --all-features` | 104 passed, 0 failed, 0 ignored |
| `./scripts/leak-scan.sh` | no key material and no address in any tracked file; no third-party reference in `crates/unstirred-gate/ui`; `evidence/2026-10-17-first-proof.json` carries only the package schema's fields |
| `./scripts/claims-check.sh` | 44 claims: 14 proven on the real testnet, 6 synthetic, 18 settled by a recorded gate run, 5 measured by a recorded tool run, 1 named as not built |
| `python3 -I scripts/status-table.py --check` | README.md matches CLAIMS.md (44 claims) |

The three rows for this run itself were written after it, as every gate log's are: re-running
`./scripts/claims-check.sh` afterwards reports **47 claims: 14 proven on the real testnet, 7
synthetic, 20 settled by a recorded gate run, 5 measured by a recorded tool run, 1 named as not
built**, exit 0, and `scripts/status-table.py --check` matches at 47. A47's row is SYNTHETIC
rather than GATED because what it is about is two real proofs over a wallet built in the test
process; the other two rows are what a gate run settles.

## What this pass added to the suite

102 tests before it, 104 now. Both new ones exist because `briefing/LEAKS.md` was being finalised
and two of its rows had a real defence in the code and no test, which is neither a test nor a
stated non-defence. Both IDs were registered in `briefing/PRD.md` §10 with a `DECISIONS.md` row
before they were written.

| ID | Test | What it holds |
|---|---|---|
| A47 | `a47_two_proofs_of_the_same_coins_repeat_one_marker_per_note_and_no_key` | LEAKS L7. Two proofs of the same notes, from the store fixture's existing corpus, so it costs no new proving: `rk`, the session key and `van_comm` all differ, and exactly one marker per real note repeats |
| A48 | `a48_a_non_membership_answer_prints_neither_its_path_nor_its_bounds` | LEAKS L5b. Neither the lookup answer nor the lookup that carries it shows an IMT path element or a leaf bound when printed; the bounds bracket the holder's own nullifier |

**A47 failed the first time it ran, and the documentation was what was wrong.** It was written to
assert that two proofs of the same coins carry identical markers; only one of the five matched.
The delegation builder pads a bundle to five notes with padding nullifiers of its own, so a proof
of one real note publishes one real marker and four fresh ones. Single use is unaffected —
`UNIQUE(context, marker)` needs one collision, not five — but `SPEC.md` §4 and LEAKS L4 had both
been written as "the same coins produce the same markers", and both were corrected before this
run. The test now asserts the shared count equals the number of real notes.

One other test changed, because what it asserts changed: the holder-line test in
`crates/unstirred-gate/tests/page.rs` now also requires an act that draws a holder to carry the
`aside` naming the wallet those blocks were read from, and refuses a holder whose `aside` is an
empty string.

## Tests, by target

| Target | Passed | Time |
|---|---|---|
| `unstirred` (cli unit: A27, A33 x2, A46's guard) | 4 | 0.00s |
| `unstirred-cli` `tests/knock.rs` (A45, A46) | 2 | 1.17s |
| `unstirred-core` unit | 0 | — |
| `unstirred-core` `tests/properties.rs` (A28-A31) | 4 | 0.07s |
| `unstirred-core` `tests/rules.rs` (A07, A12-A14, A40) | 9 | 0.00s |
| `unstirred-gate` unit (router x2, RFC 3339 x2, design tokens) | 5 | 0.17s |
| `unstirred-gate` `tests/config.rs` (A41) | 1 | 0.01s |
| `unstirred-gate` `tests/http.rs` (A34-A39) | 7 | 140.89s |
| `unstirred-gate` `tests/page.rs` (A02c, A42-A44) | 7 | 0.08s |
| `unstirred-prover` unit | 0 | — |
| `unstirred-prover` `tests/identity.rs` (A24) | 5 | 0.99s |
| `unstirred-prover` `tests/pir.rs` (A04, A15, A15b, **A48**) | 6 | 2.81s |
| `unstirred-prover` `tests/properties.rs` (A32) | 2 | 0.02s |
| `unstirred-prover` `tests/prove.rs` (A02, A06, A22) | 3 | 55.50s |
| `unstirred-prover` `tests/selection.rs` (A17, A18) | 8 | 0.00s |
| `unstirred-prover` `tests/synthetic_wallet.rs` | 4 | 2.55s |
| `unstirred-prover` `tests/wallet_read_only.rs` (A16) | 3 | 0.74s |
| `unstirred-prover` `tests/witness.rs` (A19-A21) | 5 | 2.51s |
| `unstirred-verifier` unit | 4 | 1.52s |
| `unstirred-verifier` `tests/roots.rs` (A10, A10b, A27) | 6 | 0.00s |
| `unstirred-verifier` `tests/store.rs` (A02b, A03, A05, A05c, A05d, A07, A08, **A47**) | 13 | 129.14s |
| `unstirred-verifier` `tests/verify.rs` (A01, A06, A07, A09, A11, A25) | 6 | 63.72s |
| doc-tests (4 crates) | 0 | — |

## What this run does not prove

- **Nothing about the release itself.** No tag exists, nothing is published, and
  `.github/workflows/release.yml` has never run: the gate does not execute workflows. The
  crates.io side is a separate recorded run, `evidence/2026-10-23-publish-dry-run.md`.
- **Nothing about the chain.** No wallet, no node and no nf-server was involved — all three were
  stopped for this run — and the act pages carry values the generator copied from runs recorded
  earlier.
- **Nothing about how act 4 reads.** The gate shows it is a valid page state whose words are the
  closed code set's own; that it fits and hides nothing is the browser's run,
  `evidence/2026-10-23-page-layout-check-act4-broken-axis.md`, and whether the break reads as a
  break is a review.
- **That the suite passes on a machine like this one without the two settings.** It does not: the
  parallel run was killed for memory here. The settings are a property of this machine, not of the
  suite, and CI runs the same gate with neither.
