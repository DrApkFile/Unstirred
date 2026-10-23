# unstirred-gate

The gate as a small HTTP service: one room per manifest, with routes for the manifest, a challenge,
an admission and the room's log. It adds no check of its own — the admit route calls the same
engine the CLI calls — and keeps only what the store keeps: hashed challenges, markers, verdicts.

It also serves the Holders' Room page itself, assembled in-process, asking no other origin for
anything, with the recorded demo acts rendered from the gate's own state.

Part of **[Unstirred](https://github.com/DrApkFile/Unstirred)**: a Zcash holder proves to a gate
that they held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another,
without revealing their address, their balance or which coins. Built on
[Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits), used unchanged,
with the two roots pinned to two different heights.

- The protocol, the soundness argument and every limit: [`SPEC.md`](https://github.com/DrApkFile/Unstirred/blob/master/SPEC.md)
- Every claim and the recorded run that settles it: [`CLAIMS.md`](https://github.com/DrApkFile/Unstirred/blob/master/CLAIMS.md)
- Testnet only in 0.1.0. Nothing here has run against mainnet.

License: MIT OR Apache-2.0.
