# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-16-gate-m1-step2.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing, note selection, and h0 witness rebuild checked against the h0 root; tests A03, A04, A16-A21 pass on synthetic data ([gate log](evidence/2026-10-16-gate-m1-step2.md)), and `notes` has run against the real testnet wallet ([run](evidence/2026-10-16-notes-real-testnet.txt)). PIR, proving and signing not built yet (milestone M1) |
| `unstirred-verifier` | Not built yet (milestones M1-M2) |
| `unstirred` CLI | `notes`, `witness`, `codes` and `--version` |
| `unstirred-gate` + Holders' Room page | Not built yet (milestone M3) |
| Real testnet admission | Not yet run |

No proof has been built yet.

`unstirred notes` has run against the real testnet wallet, over h0 4,505,790 through h1 4,506,510: see [that run](evidence/2026-10-16-notes-real-testnet.txt). `unstirred witness` has only been exercised against synthetic Ironwood notes on a local test network; no live `GetTreeState` call is recorded yet.

Feasibility spikes (split-height proofs, rejections, spend-key binding) passed before the build started; see `briefing/REALITY_SPIKES.md` and `briefing/spikes/evidence/`.

## Build

Rust 1.91 or newer.

```
cargo build --release
./scripts/gate.sh
```

## Docs

- `briefing/PRD.md`: what we are building and how it is verified
- `briefing/LEAKS.md`: every privacy channel, with its test or stated limit
- `briefing/BUILD_CONTRACT.md`: rules this repo follows

## License

MIT OR Apache-2.0
