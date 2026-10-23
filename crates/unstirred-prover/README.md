# unstirred-prover

The holder's half. Opens a wallet read-only, lists the notes held unmoved across a gate's window
and picks the fewest that reach the threshold, rebuilds each one's witness at h0 from the wallet's
own shards plus the public frontier, proves each nullifier absent from the h1 set through a private
(YPIR) lookup, and produces the delegation proof with its binding signature as a proof package.

It creates no transaction, and it cannot: nothing here can build or broadcast one.

Part of **[Unstirred](https://github.com/DrApkFile/Unstirred)**: a Zcash holder proves to a gate
that they held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another,
without revealing their address, their balance or which coins. Built on
[Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits), used unchanged,
with the two roots pinned to two different heights.

- The protocol, the soundness argument and every limit: [`SPEC.md`](https://github.com/DrApkFile/Unstirred/blob/master/SPEC.md)
- Every claim and the recorded run that settles it: [`CLAIMS.md`](https://github.com/DrApkFile/Unstirred/blob/master/CLAIMS.md)
- Testnet only in 0.1.0. Nothing here has run against mainnet.

License: MIT OR Apache-2.0.
