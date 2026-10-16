# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-16-gate-m1-step2.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing, note selection, h0 witness rebuild checked against the h0 root, and the private nullifier lookup at h1 through Valar's PIR client; tests A03, A04, A15, A15b, A16-A21 pass on synthetic data ([gate log](evidence/2026-10-16-gate-m1-step3.md)), and `notes` and `witness` have run against the real testnet wallet ([notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt)). Proving and signing not built yet (milestone M1) |
| `unstirred-verifier` | Not built yet (milestones M1-M2) |
| `unstirred` CLI | `notes`, `witness`, `lookup`, `codes` and `--version` |
| `unstirred-gate` + Holders' Room page | Not built yet (milestone M3) |
| Real testnet admission | Not yet run |

No proof has been built yet.

`unstirred notes` and `unstirred witness` have run against the real testnet wallet, over h0 4,505,790 through h1 4,506,510: see [notes](evidence/2026-10-16-notes-real-testnet.txt) and [witness](evidence/2026-10-16-witness-real-testnet.txt). `unstirred lookup` has only been exercised against a synthetic nullifier set and a local stand-in HTTP server; no live `nf-server` lookup is recorded yet.

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
