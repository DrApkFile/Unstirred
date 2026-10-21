# Gate run: the Holders' Room page after its visual review

Commit `bf3fe87` (M3 step 2) plus the uncommitted fixes for the owner's browser review of
2026-10-21: the page's layout and hierarchy, the window as a timeline with an optional holder line,
`?present=1`, the two new page-state fields (`holder` per room, `present` per page), the
regenerated replay acts, `scripts/page-layout-check.mjs`, the `MEASURED` status in `CLAIMS.md` and
`scripts/claims-check.sh`, and the CLAIMS, README, HANDOFF §9 and BUILD_NOTES entries for all of it.
Date: 2026-10-21T16:28:38Z. Toolchain: `rustc 1.91.1 (ed61e7d7e 2025-11-07)`, pinned by
`rust-toolchain.toml`. Command: `./scripts/gate.sh`. Exit: 0.

```
GATE: PASS
```

The suite is 97 tests, two more than the run that built this step
(`evidence/2026-10-21-gate-m3-step2.md`, 95): the page state gained two fields, and each has a test
saying what it is for.

No chain was contacted by this run and nothing in it touched the holder's real wallet.

## Steps

| Step | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --release --all-targets --all-features -- -D warnings` | clean, no warnings |
| `cargo test --release --workspace --all-features` | 97 passed, 0 failed, 0 ignored |
| `./scripts/leak-scan.sh` | no key material and no address in any tracked file; no third-party reference in `crates/unstirred-gate/ui`; `evidence/2026-10-17-first-proof.json` carries only the package schema's fields |
| `./scripts/claims-check.sh` | 32 claims: 11 proven on the real testnet, 5 synthetic, 13 settled by a recorded gate run, 2 measured by a recorded tool run, 1 named as not built |

## What the review added to the suite (2 tests)

| ID | Test | Target |
|---|---|---|
| — | presentation mode is a way of reading a page and not part of it | `unstirred-gate/tests/page.rs` |
| — | a holder line is an act's own, and is refused when it is not a range | `unstirred-gate/tests/page.rs` |

The first asserts that `?present=1` changes `present` and nothing else about an act's state, that it
works on a live page too, that the gate's technical lines leave the provenance foot and that what a
verdict says about its own time does not. The second asserts that a live page draws no holder line
(a gate knows no holder's block range), that no recorded act draws one, and that a backwards range
is refused by the state contract rather than drawn wrong.

## Tests, by target

| Target | Passed | Time |
|---|---|---|
| `unstirred` (cli unit, A27, A33 x2) | 3 | 0.00s |
| `unstirred-core` unit | 0 | — |
| `unstirred-core` `tests/properties.rs` (A28-A31) | 4 | 0.05s |
| `unstirred-core` `tests/rules.rs` (A07, A12-A14, A40) | 9 | 0.00s |
| `unstirred-gate` unit (router x2, RFC 3339 x2, design tokens) | 5 | 0.15s |
| `unstirred-gate` `tests/config.rs` (A41) | 1 | 0.00s |
| `unstirred-gate` `tests/http.rs` (A34-A39) | 7 | 139.52s |
| `unstirred-gate` `tests/page.rs` (A02c, A42-A44, present, holder) | 7 | 0.07s |
| `unstirred-prover` unit | 0 | — |
| `unstirred-prover` `tests/identity.rs` (A24) | 5 | 0.99s |
| `unstirred-prover` `tests/pir.rs` (A04, A15, A15b) | 5 | 3.38s |
| `unstirred-prover` `tests/properties.rs` (A32) | 2 | 0.01s |
| `unstirred-prover` `tests/prove.rs` (A02, A06, A22) | 3 | 54.35s |
| `unstirred-prover` `tests/selection.rs` (A17, A18) | 8 | 0.00s |
| `unstirred-prover` `tests/synthetic_wallet.rs` | 4 | 2.81s |
| `unstirred-prover` `tests/wallet_read_only.rs` (A16) | 3 | 0.77s |
| `unstirred-prover` `tests/witness.rs` (A19-A21) | 5 | 3.00s |
| `unstirred-verifier` unit | 4 | 1.62s |
| `unstirred-verifier` `tests/roots.rs` (A10, A10b, A27) | 6 | 0.00s |
| `unstirred-verifier` `tests/store.rs` (A02b, A05, A05c, A05d, A07, A08) | 10 | 96.18s |
| `unstirred-verifier` `tests/verify.rs` (A09, A11, A25, A26) | 6 | 52.29s |
| doc-tests (4 crates) | 0 | — |

## What this run does not prove

- **Nothing about how the page looks.** The gate cannot see a line box, which is exactly how the
  clipping the owner found got past the run before this one: every test passed while the browser
  was cutting text off at 1326x683. What the layout now rests on is
  `evidence/2026-10-21-page-layout-check.md`, a browser measuring it, and that is run by hand.
- The same limits as the run that built this step: nothing about the Zcash network, nothing about
  two root sources agreeing, nothing about a challenge expiring, and nothing about a running
  process keeping a secret off a channel.
