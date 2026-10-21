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
- **MEASURED** — settled by a recorded run of a tool that is not the gate, named in the evidence
  file with its version and its command, against something this repo serves. It says nothing about
  the chain either, and nothing about anything the tool did not measure.
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
| The quality pass passes the gate: fmt, clippy with `-D warnings`, 76 tests including the property, oracle and adversarial suites, the leak scan and the claims check | GATED | [evidence/2026-10-20-gate-quality-pass.md](evidence/2026-10-20-gate-quality-pass.md) | README status table |
| Two independent root sources agree on the h0 root on the real testnet (zec.rocks lightwalletd and the owner's zebrad 7.0.0-rc.0, both reporting `a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` at h0 4,505,790), and the governing tip is the lower of the answers | PROVEN | [evidence/2026-10-20-roots-two-source.txt](evidence/2026-10-20-roots-two-source.txt) | README status table |
| A single root source is an explicit opt-in, labelled "ROOT FROM ONE SOURCE: not independently checked" in human output and in the JSON verdict | SYNTHETIC | [evidence/2026-10-17-gate-m2-step1.md](evidence/2026-10-17-gate-m2-step1.md) | README status table |
| The first real proof is ADMITTED against two agreeing root sources, with no single-source label in the output | PROVEN | [evidence/2026-10-20-verify-two-source.txt](evidence/2026-10-20-verify-two-source.txt) | README status table, README prose |
| A gate hosts one room per manifest over HTTP, and `unstirred knock` fetches a room's manifest and a gate-issued challenge, proves, posts the package and reads the PRD §3.5 verdict: admitted once (A34), the same coins at that room refused ALREADY_USED_IN_CONTEXT (A35), and a challenge the gate never issued refused with nothing written (A36) | SYNTHETIC | [gate log](evidence/2026-10-20-gate-m3-step1.md) | README status table |
| The gate rejects an admit body over its limit before parsing or storing it and claims no verdict for it (A37), answers 404 for a room it does not host without naming anything it holds (A38), and refuses to start on an invalid manifest, a missing setting, duplicate contexts, the wrong number of root sources or a bad h1 root (A41) | GATED | [gate log](evidence/2026-10-20-gate-m3-step1.md) | README status table |
| The gate's log lines carry no address but the one it bound itself, and neither its log nor any response carries a proof-package body, a raw challenge or a session key (A39) | SYNTHETIC | [gate log](evidence/2026-10-20-gate-m3-step1.md) | README status table, briefing/LEAKS.md L9 |
| `unstirred gate init` writes only a manifest `unstirred-core` parses, refusing h0 >= h1, an h1 off the nullifier step and a context outside the charset (A40) | GATED | [gate log](evidence/2026-10-20-gate-m3-step1.md) | README status table, CLI usage |
| A gate serving the real testnet wallet admits a knock at one room over HTTP, refuses the same coins there as ALREADY_USED_IN_CONTEXT, and admits them at a second room, with two agreeing root sources (zec.rocks lightwalletd and the owner's zebrad) and no single-source label; each room's log shows five markers and the two rooms share none | PROVEN | [evidence/2026-10-20-two-room-knock.txt](evidence/2026-10-20-two-room-knock.txt), header [evidence/2026-10-20-two-room-knock.meta.md](evidence/2026-10-20-two-room-knock.meta.md) | HANDOFF.md section 8 |
| The gate serves the Holders' Room page itself: a room picker at `GET /`, a page per hosted room at `GET /room/<context>`, 404 for any other room, each one self-contained HTML rendered by one pure function from the gate's own state (A42) | GATED | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) | README status table, CLI usage |
| No page the gate serves asks another origin for anything — no script, stylesheet, font, image, frame or beacon, no absolute URL in the page's own code, and every page carries `default-src 'none'` — checked over every page the gate serves and over `crates/unstirred-gate/ui/` as a gate step, both confirmed against a planted third-party stylesheet and a planted beacon (A02c) | GATED | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) | README status table, briefing/LEAKS.md L10 |
| The gate's recent-verdict feed shows a refusal the store never wrote, keeps only a verdict, a code and a time, is bounded per room, writes nothing, and is empty again after a restart on the same database (A43) | GATED | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) | README status table, briefing/LEAKS.md L16 |
| Each demo act is a replay state built from the recorded two-room knock by `scripts/replay-from-evidence.py`, restates the closed code set's own words, and renders in a document with no network call site and `connect-src 'none'` (A44) | GATED | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) | README status table, crates/unstirred-gate/ui/frame.md |
| A room page may draw one holder's own line across the window, drawn short of the band and in the refusal colour when it does not cover it; a live gate draws none, because it knows no holder's block range, and a range that is not a range is refused by the state contract | GATED | [after the review](evidence/2026-10-21-gate-page-review.md) | crates/unstirred-gate/ui/frame.md |
| No page the gate serves hides text to make itself fit: on every page, live, replayed and presented, at 1920x1080 and at 1326x683, the stage does not outgrow its 1080px, no box hides what does not fit, and no text has left the stage — measured in a real browser | MEASURED | [layout check](evidence/2026-10-21-page-layout-check.md) | README status table, crates/unstirred-gate/ui/frame.md |
| `?present=1` raises the smallest type on any page from 15-16px to 19-20px and takes the gate's own technical lines off the provenance foot, on a live page and on a replayed act alike | MEASURED | [layout check](evidence/2026-10-21-page-layout-check.md) | crates/unstirred-gate/ui/frame.md, CLI usage |
| The h1 nullifier-set root checked by the verifier itself (nf-server `verify-root`) | NOT BUILT | — | README status table, HANDOFF.md section 6 |
