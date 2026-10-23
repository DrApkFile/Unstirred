# unstirred-verifier

The gate's half. Runs the ordered verification engine — manifest, window, `round_id`, roots, the
challenge, the binding signature, the Halo2 proof — over roots it authenticates itself from two
independent sources that must agree, and never from the package. Then the one write: five markers
under `UNIQUE(context, marker)` and the challenge consumed, in a single transaction, or nothing.

Unknown is never success: a root source that disagrees or cannot be reached is INCONCLUSIVE.

Part of **[Unstirred](https://github.com/DrApkFile/Unstirred)**: a Zcash holder proves to a gate
that they held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another,
without revealing their address, their balance or which coins. Built on
[Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits), used unchanged,
with the two roots pinned to two different heights.

- The protocol, the soundness argument and every limit: [`SPEC.md`](https://github.com/DrApkFile/Unstirred/blob/master/SPEC.md)
- Every claim and the recorded run that settles it: [`CLAIMS.md`](https://github.com/DrApkFile/Unstirred/blob/master/CLAIMS.md)
- Testnet only in 0.1.0. Nothing here has run against mainnet.

License: MIT OR Apache-2.0.
