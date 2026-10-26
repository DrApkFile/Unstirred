# Gate run: the final audit, parts A and B together — 2026-10-26

One run covering both halves of the final audit: part A (committed as `456d19a` — the simpler
knock, the two guides, the two runnable examples, the in-repo manifests and rustdoc on every
public item) and part B (the published site, the clean-clone audit, the release preparation and
the hygiene pass), which was uncommitted when this ran.

```
$ CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh
== fmt
== clippy
    Finished `release` profile [optimized] target(s) in 1m 01s
== tests
...
== docs
== leaks
leak scan: evidence/2026-10-17-first-proof.json carries only the package schema's fields
leak scan: no third-party reference in crates/unstirred-gate/ui
leak scan: no key material and no address in any tracked file
== claims
claims check: 55 claims, 17 proven on the real testnet, 7 synthetic, 20 settled by a recorded gate run, 10 measured by a recorded tool run, 1 named as not built
== status
status table: README.md matches CLAIMS.md (55 claims)

GATE: PASS
```

**Exit 0.**

- **fmt**: `cargo fmt --all -- --check`, clean.
- **clippy**: `--release --all-targets --all-features -- -D warnings`, clean over all five crates.
  `unwrap_used`, `expect_used` and `panic` are denied in this workspace, and `missing_docs` is on
  in all four libraries, so an undocumented public item fails here as an error.
- **tests**: `cargo test --release --workspace --all-features` — **132 passed, 0 failed, 0
  ignored**, across 26 test binaries. **108** are unit, integration, property, oracle and
  adversarial tests; **24** are doc tests (12 in `unstirred-core`, 5 in `unstirred-verifier`, 4 in
  `unstirred-prover`, 3 in `unstirred-gate`). Every acceptance ID up to **A51** that this gate can
  settle is in them.
- **docs**: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace --all-features`, clean —
  no missing doc, no broken intra-doc link.
- **leaks**: the repository half of A02, plus `crates/unstirred-gate/ui/` scanned for any
  third-party reference (A02c).
- **claims**: every row of `CLAIMS.md` cites a file that exists, every link in it and in README
  resolves, and no file in `evidence/` is cited by nothing.
- **status**: the README status table is exactly what `scripts/status-table.py` writes from
  `CLAIMS.md`.

Machine: 4-core Intel i3 M 370 at 2.40 GHz, 7 GB of memory, `cargo 1.91.1` / `rustc 1.91.1` as
`rust-toolchain.toml` pins. `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1` because the machine is small,
not because anything needs serialising.

## What this run does not say

The two things named in the gate's own closing lines and in `briefing/BUILD_CONTRACT.md`: it
cannot see a leak through a channel rather than a string in a running process, and it cannot tell
whether a recorded run in `evidence/` says what `CLAIMS.md` says it says. It also says nothing
about the chain — the real-testnet claims are the PROVEN rows, each with its own recorded run.
