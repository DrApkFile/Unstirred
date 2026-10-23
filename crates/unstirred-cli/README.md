# unstirred

The command line. `cargo install unstirred` gives you the `unstirred` binary:

```
unstirred notes | witness | lookup | prove     # the holder's path
unstirred knock --gate <url> --room <context>  # prove and post, in one command
unstirred verify | admit | roots | codes       # the gate's path
unstirred gate init | serve | challenge | markers
```

`unstirred verify` records nothing and needs no database. The 60-second check a judge can run, on
a real recorded proof against the real testnet root, is in the repository README.

Part of **[Unstirred](https://github.com/DrApkFile/Unstirred)**: a Zcash holder proves to a gate
that they held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another,
without revealing their address, their balance or which coins. Built on
[Valar Group's delegation circuit](https://github.com/valargroup/voting-circuits), used unchanged,
with the two roots pinned to two different heights.

- The protocol, the soundness argument and every limit: [`SPEC.md`](https://github.com/DrApkFile/Unstirred/blob/master/SPEC.md)
- Every claim and the recorded run that settles it: [`CLAIMS.md`](https://github.com/DrApkFile/Unstirred/blob/master/CLAIMS.md)
- Testnet only in 0.1.0. Nothing here has run against mainnet.

License: MIT OR Apache-2.0.
