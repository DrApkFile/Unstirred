# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove "I have held at least 0.125 ZEC in the Ironwood pool, unmoved, since block H0" to a gate (a community, an allowlist, a faucet) without revealing their address, balance or transactions. Each gate admits the same coins once, and two gates cannot link them.

Built for the Zecathon (Core & Tooling track) on Valar Group's audited delegation circuit ([voting-circuits](https://github.com/valargroup/voting-circuits)), which we use unchanged and credit throughout.

## Status

Work in progress. Nothing below is claimed until it links to a recorded run in `evidence/`.

| Part | Status |
|---|---|
| `unstirred-core`: manifest, window rules, context derivation, verdict codes, binding message, proof package schema | Built; tests A07, A12, A13, A14 pass ([gate log](evidence/2026-10-17-gate-m1-step4.md)) |
| `unstirred-prover` | Read-only wallet access, held-note listing and selection, h0 witness rebuild checked against the h0 root, the private nullifier lookup at h1 through Valar's PIR client, the spending key derived in memory from the wallet's age-encrypted mnemonic, and a real delegation proof with its binding signature, written as a proof package. Tests A03, A04, A15, A15b, A16-A24 pass, the proof in them being a real K=12 proof over a synthetic wallet and a synthetic nullifier set ([gate log](evidence/2026-10-17-gate-m1-step5.md)). `notes`, `witness`, `lookup` and `prove` have run against the real testnet wallet ([notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt), [lookup](evidence/2026-10-16-lookup-real-testnet.txt), [first proof](evidence/2026-10-17-first-proof.json)) |
| `unstirred-verifier` | The pure verification engine (PRD §6 steps 1-8, roots and tip injected) plus a one-server root read (h0 root and chain tip from one lightwalletd; the h1 root is pinned by the operator). Markers, the challenge store and the two-server root authority are M2. Tests A06 (verifier half), A09, A11, A25, A26 and the A07 verify half pass on a synthetic package ([gate log](evidence/2026-10-17-gate-m1-step5.md)), and the real package was admitted in a fresh process against the real testnet roots ([verify run](evidence/2026-10-17-verify-first-proof.txt)) |
| `unstirred` CLI | `notes`, `witness`, `lookup`, `prove`, `verify`, `codes` and `--version` |
| `unstirred-gate` + Holders' Room page | Not built yet (milestone M3) |
| Real testnet admission | ADMITTED, recorded on testnet over h0 4,505,790 through h1 4,506,510 ([verify run](evidence/2026-10-17-verify-first-proof.txt)) |

A proof has been made from the real testnet wallet ([first proof](evidence/2026-10-17-first-proof.json)), and `unstirred verify` admitted that package in a fresh process against the real testnet roots ([verify run](evidence/2026-10-17-verify-first-proof.txt)): milestone M1's real admission is recorded.

`unstirred notes`, `unstirred witness`, `unstirred lookup` and `unstirred prove` have run against the real testnet wallet, over h0 4,505,790 through h1 4,506,510: see [notes](evidence/2026-10-16-notes-real-testnet.txt), [witness](evidence/2026-10-16-witness-real-testnet.txt), [lookup](evidence/2026-10-16-lookup-real-testnet.txt) and the [first proof](evidence/2026-10-17-first-proof.json), the lookup and the proof against a local `nf-server` built from Valar's v0.12.3. The same synthetic proof machinery also runs in the verifier's tests, with a real K=12 proof: see the [gate log](evidence/2026-10-17-gate-m1-step5.md).

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
