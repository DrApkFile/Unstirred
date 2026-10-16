# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-16-gate-m1-step1.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing and note selection; tests A03, A04, A16, A17, A18 pass on synthetic data ([gate log](evidence/2026-10-16-gate-m1-step1.md)). Witnesses, PIR, proving and signing not built yet (milestone M1) |
| `unstirred-verifier` | Not built yet (milestones M1-M2) |
| `unstirred` CLI | `notes`, `codes` and `--version` |
| `unstirred-gate` + Holders' Room page | Not built yet (milestone M3) |
| Real testnet admission | Not yet run |

No proof has been built and no real-testnet run has been recorded yet. `unstirred notes` has been exercised against synthetic Ironwood notes on a local test network, not against Zcash testnet.

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
