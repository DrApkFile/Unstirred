# CLAIMS

Every claim this repo makes in public text, with the one file that settles it. `scripts/claims-check.sh`
runs as a gate step: it fails if a row cites an evidence file that does not exist, if a link in
README.md or in this file does not resolve, or if a file in `evidence/` is cited by no row.

Status means exactly this:

- **PROVEN** — a recorded run against the real Zcash testnet, in `evidence/`, naming network,
  heights, commit, command and date.
- **SYNTHETIC** — true of a wallet, nullifier set or root built inside the test process. The proof
  machinery is real; the chain is not. Never written as a claim about the network.
- **GATED** — settled by a recorded `scripts/gate.sh` run: formatting, clippy and the test suite.
  It says nothing about the chain.
- **NOT BUILT** — named here so it cannot be read as working.

| Claim | Status | Evidence | Claimed in |
|---|---|---|---|
| A wallet's Ironwood notes held across a gate's window, and the selection, can be listed read-only | PROVEN | [evidence/2026-10-16-notes-real-testnet.txt](evidence/2026-10-16-notes-real-testnet.txt) | README status table |
| Each selected note's witness can be rebuilt at h0 and checked against the h0 root | PROVEN | [evidence/2026-10-16-witness-real-testnet.txt](evidence/2026-10-16-witness-real-testnet.txt) | README status table |
| Each selected note's nullifier can be shown absent from the h1 nullifier set through PIR, with no plaintext query | PROVEN | [evidence/2026-10-16-lookup-real-testnet.txt](evidence/2026-10-16-lookup-real-testnet.txt) | README status table |
| A real holding-duration proof and its binding signature can be made from the real testnet wallet over h0 4,505,790 through h1 4,506,510 | PROVEN | [evidence/2026-10-17-first-proof.json](evidence/2026-10-17-first-proof.json) | README status table, README prose |
| That package is ADMITTED in a fresh process against the real testnet roots | PROVEN | [evidence/2026-10-17-verify-first-proof.txt](evidence/2026-10-17-verify-first-proof.txt) | README status table, README prose |
| The gate database admits a package once, refuses the same package as ALREADY_USED_IN_CONTEXT, and admits the same coins at a second gate with a different marker set | PROVEN | [evidence/2026-10-17-admit-two-rooms.txt](evidence/2026-10-17-admit-two-rooms.txt) | README status table, README prose |
| That two-room run used one lightwalletd source, because the owner's Zebra node was still behind h0, and every output said so | PROVEN | [evidence/2026-10-17-two-rooms-summary.md](evidence/2026-10-17-two-rooms-summary.md) | README prose |
| The holder's wallet file was unchanged by that run (same sha256 before and after) | PROVEN | [evidence/2026-10-17-two-rooms-summary.md](evidence/2026-10-17-two-rooms-summary.md) | README prose |
| M1 step 1 (read-only wallet, held notes, selection) passed the gate | GATED | [evidence/2026-10-16-gate-m1-step1.md](evidence/2026-10-16-gate-m1-step1.md) | this ledger |
| M1 step 2 (h0 tree state, historical witnesses) passed the gate | GATED | [evidence/2026-10-16-gate-m1-step2.md](evidence/2026-10-16-gate-m1-step2.md) | this ledger |
| M1 step 3 (private nullifier lookup) passed the gate | GATED | [evidence/2026-10-16-gate-m1-step3.md](evidence/2026-10-16-gate-m1-step3.md) | this ledger |
| M1 step 4 (proof package, identity path, prove) passed the gate, with core's manifest, window, context and code tests | GATED | [evidence/2026-10-17-gate-m1-step4.md](evidence/2026-10-17-gate-m1-step4.md) | README status table |
| M1 step 5 (`unstirred verify`) passed the gate, with the prover's tests over a real K=12 proof on a synthetic wallet | SYNTHETIC | [evidence/2026-10-17-gate-m1-step5.md](evidence/2026-10-17-gate-m1-step5.md) | README status table |
| M2 step 1 (two-source root authority, `unstirred roots`) passed the gate | GATED | [evidence/2026-10-17-gate-m2-step1.md](evidence/2026-10-17-gate-m2-step1.md) | this ledger |
| M2 step 2 (challenge and marker store, `admit`, `gate markers`) passed the gate: A02b, A05, A05c, A05d, A07, A08 and A11 on synthetic packages | SYNTHETIC | [evidence/2026-10-17-gate-m2-step2.md](evidence/2026-10-17-gate-m2-step2.md) | README status table |
| Two root sources must agree on the h0 root, and a single source is an explicit opt-in labelled "ROOT FROM ONE SOURCE: not independently checked" | SYNTHETIC | [evidence/2026-10-17-gate-m2-step1.md](evidence/2026-10-17-gate-m2-step1.md) | README status table |
| A real two-source run (lightwalletd and the owner's Zebra node agreeing on one h0 root) | NOT BUILT | — | README status table, HANDOFF.md section 2 |
| `unstirred-gate` HTTP and the Holders' Room page | NOT BUILT | — | README status table |
| `unstirred gate init` | NOT BUILT | — | README status table, CLI usage |
| The h1 nullifier-set root checked by the verifier itself (nf-server `verify-root`) | NOT BUILT | — | README status table, HANDOFF.md section 6 |
