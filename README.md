# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message, proof package schema | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-17-gate-m1-step4.md)), and A28-A31 run the window rule, `round_id`, the context charset and the binding message over thousands of seeded random cases ([gate log](evidence/2026-10-20-gate-quality-pass.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing and selection, h0 witness rebuild checked against the h0 root, the private nullifier lookup at h1 through Valar's PIR client, the spending key derived in memory from the wallet's age-encrypted mnemonic, and a real delegation proof with its binding signature, written as a proof package. Tests A03, A04, A15, A15b, A16-A24 pass, and A32 checks the note selection against a brute-force oracle over every subset ([gate log](evidence/2026-10-20-gate-quality-pass.md)), the proof in them being a real K=12 proof over a synthetic wallet and a synthetic nullifier set ([gate log](evidence/2026-10-17-gate-m1-step5.md)). `notes`, `witness`, `lookup` and `prove` have run against the real testnet wallet ([notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt), [lookup](evidence/2026-10-16-lookup-real-testnet.txt), [first proof](evidence/2026-10-17-first-proof.json)) |
| `unstirred-verifier` | The verification engine (PRD §6, roots and tip injected) plus a two-source root authority and the gate's SQLite memory. The h0 Ironwood root and chain tip come from two `--root-source` backends (lightwalletd or Zebra JSON-RPC) that must agree exactly — disagreement or an unusable source is INCONCLUSIVE, and one source alone is an explicit `--single-root-source` labelled "ROOT FROM ONE SOURCE: not independently checked", with the agreement line "not checked (one source)". The h1 root is pinned by the operator. The store keeps a hash of each challenge, the five markers per context and the admission verdict; the same coins are refused at one gate and admitted at two. Tests A02b, A05, A05c, A05d, A06 (verifier half), A07, A08, A09, A10, A10b, A11, A25, A26 and A27 pass on synthetic packages ([gate log](evidence/2026-10-17-gate-m2-step2.md)). `verify` still records nothing. The real package was admitted in a fresh process against the real testnet roots ([verify run](evidence/2026-10-17-verify-first-proof.txt)) |
| `unstirred` CLI | `notes`, `witness`, `lookup`, `prove`, `knock`, `verify`, `roots`, `admit`, `gate serve`, `gate init`, `gate challenge`, `gate markers`, `codes` and `--version`. A27 checks that a one-source run is labelled as unchecked and A33 that an admitted package is reported as recorded nowhere ([gate log](evidence/2026-10-20-gate-quality-pass.md)). `knock` takes a challenge from a running gate, proves and posts the package in one command (A34-A36), and `gate init` writes only a manifest the core parser accepts (A40, [gate log](evidence/2026-10-20-gate-m3-step1.md)) |
| `unstirred-gate` | Built: the gate runs as a small web service hosting one room per manifest, with `GET /rooms`, and per room `GET manifest`, `POST challenge`, `POST admit` and `GET log`. It adds no verification of its own — the admit route calls the same engine and store `unstirred admit` calls — and it keeps only what the store already kept: hashed challenges, markers and admission verdicts. No request logging that records an address, no package body in any log, no third-party request. It also publishes `GET /rooms/<context>/recent`, a bounded in-memory feed of the verdicts it has just answered — a verdict, a code and a time, never written to disk and gone on restart — because a refusal is stored nowhere and the page could otherwise not show one (A43, LEAKS L16). Tests A34-A39 and A41 pass on a synthetic wallet with a real HTTP socket and real K=12 proofs ([gate log](evidence/2026-10-20-gate-m3-step1.md)), and A02c, A42-A44 cover the pages and the feed ([gate log](evidence/2026-10-21-gate-m3-step2.md)). It has run against a gate serving the real testnet wallet: one room admitted, the same coins there refused ALREADY_USED_IN_CONTEXT, a second room admitted, with disjoint marker sets ([two-room knock](evidence/2026-10-20-two-room-knock.txt)) |
| Holders' Room page | Built: the gate serves it itself — `GET /` is the room picker and `GET /room/<context>` is a room's page, each one self-contained HTML with inline styles and one pure `render(state)`, polling nothing but this gate's own JSON and asking no other origin for anything (A02c, A42). It shows the door rule from the manifest, the latest verdict with its plain message from the closed code set, what the gate wrote down beside what it never saw, the window as a bar, both rooms' markers with the count they share, and what a bot army would have to hold. `GET /room/<context>?replay=<act>` renders a recorded act of the demo from the real two-room run with no polling and no clock, which is what the video is rendered from (A44). Tests A02c, A42, A43 and A44 pass ([gate log](evidence/2026-10-21-gate-m3-step2.md)) |
| Real testnet admission | ADMITTED, recorded on testnet over h0 4,505,790 through h1 4,506,510 ([verify run](evidence/2026-10-17-verify-first-proof.txt)), and again against two independently agreeing root sources ([two-source verify](evidence/2026-10-20-verify-two-source.txt), [root sources](evidence/2026-10-20-roots-two-source.txt)) |

A proof has been made from the real testnet wallet ([first proof](evidence/2026-10-17-first-proof.json)), and `unstirred verify` admitted that package in a fresh process against the real testnet roots ([verify run](evidence/2026-10-17-verify-first-proof.txt)): milestone M1's real admission is recorded. The gate database then took a fresh proof for holders-room (ADMITTED), refused that same package (ALREADY_USED_IN_CONTEXT), and admitted the same coins under a second-room manifest with different markers ([admit run](evidence/2026-10-17-admit-two-rooms.txt)). Zebra was still behind h0 that day, so that run used one lightwalletd source and said so in every line of its output. On 2026-10-20 the owner's zebrad 7.0.0-rc.0 had synced past the window, and the two-source rule ran for real: both sources reported the same h0 root and `unstirred verify` admitted the same package with no single-source label ([root sources](evidence/2026-10-20-roots-two-source.txt), [two-source verify](evidence/2026-10-20-verify-two-source.txt)). Milestone M2 is closed.

`unstirred notes`, `unstirred witness`, `unstirred lookup` and `unstirred prove` have run against the real testnet wallet, over h0 4,505,790 through h1 4,506,510: see [notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt), [lookup](evidence/2026-10-16-lookup-real-testnet.txt) and the [first proof](evidence/2026-10-17-first-proof.json), the lookup and the proof against a local `nf-server` built from Valar's v0.12.3. The same synthetic proof machinery also runs in the verifier's tests, with a real K=12 proof: see the [gate log](evidence/2026-10-17-gate-m1-step5.md).

Feasibility spikes (split-height proofs, rejections, spend-key binding) passed before the build started; see `briefing/REALITY_SPIKES.md` and `briefing/spikes/evidence/`.

## Build

Rust 1.91, pinned by `rust-toolchain.toml`, and python3, which the gate's leak scan and claims
check use. The same gate runs in CI on every push and pull request.

```
cargo build --release
./scripts/gate.sh
```

## Docs

- `briefing/PRD.md`: what we are building and how it is verified
- `briefing/LEAKS.md`: every privacy channel, with its test or stated limit
- `briefing/BUILD_CONTRACT.md`: rules this repo follows
- `CLAIMS.md`: every claim on this page, its status, and the evidence file it rests on

## License

MIT OR Apache-2.0
