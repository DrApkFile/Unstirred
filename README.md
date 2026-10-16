# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message, proof package schema | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-17-gate-m1-step4.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing and selection, h0 witness rebuild checked against the h0 root, the private nullifier lookup at h1 through Valar's PIR client, the spending key derived in memory from the wallet's age-encrypted mnemonic, and a real delegation proof with its binding signature, written as a proof package. Tests A03, A04, A15, A15b, A16-A24 pass, the proof in them being a real K=12 proof over a synthetic wallet and a synthetic nullifier set ([gate log](evidence/2026-10-17-gate-m1-step4.md)). `notes`, `witness` and `lookup` have run against the real testnet wallet ([notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt), [lookup](evidence/2026-10-16-lookup-real-testnet.txt)); `prove` has not |
| `unstirred-verifier` | Not built yet (milestones M1-M2) |
| `unstirred` CLI | `notes`, `witness`, `lookup`, `prove`, `codes` and `--version` |
| `unstirred-gate` + Holders' Room page | Not built yet (milestone M3) |
| Real testnet admission | Not yet run |

No proof has been made from the real testnet wallet yet, and nothing verifies a package yet: `unstirred verify` is milestone M1's closing act and is not built.

`unstirred notes`, `unstirred witness` and `unstirred lookup` have run against the real testnet wallet, over h0 4,505,790 through h1 4,506,510: see [notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt) and [lookup](evidence/2026-10-16-lookup-real-testnet.txt), the last against a local `nf-server` built from Valar's v0.12.3. Proving has only run on a synthetic wallet against a synthetic nullifier set, with a real K=12 proof: see the [gate log](evidence/2026-10-17-gate-m1-step4.md).

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
