# unstirred-core

The shared contract, and the only crate allowed to define it: the gate manifest and its rules, the
window rule, `round_id` derivation from network and context, the proof package schema, the binding
message, and the closed set of verdict codes. No network, no clock, no randomness except what is
injected. Every other crate imports these definitions rather than restating them.

Part of **[Unstirred](https://github.com/DrApkFile/Unstirred)**: a Zcash holder proves to a gate
that they held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another,
without revealing their address, their balance or which coins. Built on
[Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits), used unchanged,
with the two roots pinned to two different heights.

- The protocol, the soundness argument and every limit: [`SPEC.md`](https://github.com/DrApkFile/Unstirred/blob/master/SPEC.md)
- Every claim and the recorded run that settles it: [`CLAIMS.md`](https://github.com/DrApkFile/Unstirred/blob/master/CLAIMS.md)
- Testnet only in 0.1.0. Nothing here has run against mainnet.

License: MIT OR Apache-2.0.
