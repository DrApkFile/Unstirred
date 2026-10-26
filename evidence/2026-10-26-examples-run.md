# The two library examples, run — 2026-10-26

`crates/unstirred-verifier/examples/` gained two programs that use nothing but the crate's public
API. Both compile as part of the workspace, and this file is the record of them being run, not a
description of running them.

| | |
|---|---|
| Repository | `1cd7148`, plus the uncommitted part-A tree |
| Toolchain | `rustc 1.91.1 (ed61e7d7e 2025-11-07)`, release profile |
| Package | `evidence/2026-10-17-first-proof.json`, sha256 `f13b6ce6c1f65c5a8e1b5e1bc2937e64eaf4e498c5d951420806bd68039bcc3f` |
| Manifest | `examples/holders-room.json`, sha256 `1208fc94c08fbd7bb5cba0785d35c678874b5a0cae3a2ad674a09d72b3a4102f` |
| Root source | `lwd:https://testnet.zec.rocks:443`, one source, explicitly opted into |
| Wallet | **none.** Neither example opens a wallet, and nothing was run against either wallet |

The package is the real proof made from the real testnet wallet on 2026-10-17. The manifest is the
one now committed at `examples/holders-room.json`, which `unstirred gate init` wrote from the four
numbers a gate publishes.

---

## 1. `verify_package` — ADMITTED against the live testnet root

```
cargo build --release --examples -p unstirred-verifier

./target/release/examples/verify_package \
    --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

```
  lwd:https://testnet.zec.rocks:443  root a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d  tip 4544493
ROOT FROM ONE SOURCE: not independently checked
ADMITTED
this example records nothing; `admit` is what enters a gate
```

**Exit code 0, in 36 seconds** — the Halo2 verification is most of it.

Three things in that output are the point of the example:

- The h0 root it fetched, `a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d`, is
  the same root the recorded runs of 2026-10-17 and 2026-10-20 reported for h0 4,505,790
  ([roots](2026-10-20-roots-two-source.txt)). It came from the live chain just now, at tip
  4,544,493 — about 38,000 blocks past h1 — and the proof still holds against it.
- The single-source label is printed because this run asked for one source. Two agreeing sources
  are the default; `--single-root-source` is the only way to get one, and it says so (A27).
- The last line is the `verify`/`admit` distinction: this example writes nothing, so the same
  package verifies again tomorrow. Single use is the store's, not the engine's.

## 2. `allowlist` — a challenge, a refusal, and an empty store

The three subcommands, run in order against a fresh SQLite file.

```
./target/release/examples/allowlist challenge \
    --store allowlist.sqlite --manifest examples/holders-room.json
```

```
challenge   66d5939a094631357d2ea7174ce92b852a5b76edb13652d50b8404c5943e2263
room        holders-room
expires     1793031344 (unix seconds; ten minutes from now)

the holder answers it with:
  unstirred prove --challenge 66d5939a094631357d2ea7174ce92b852a5b76edb13652d50b8404c5943e2263 --out package.json \
      --wallet <dir> --manifest <this room's manifest> --identity <age file> \
      --lightwalletd <url> --pir <url>
```

Exit code 0.

Then the recorded package was posted to it. **It answers a challenge this allowlist never issued** —
the package is from 2026-10-17 and this store was created seconds ago — so it is refused, which is
exactly the behaviour worth recording:

```
./target/release/examples/allowlist admit \
    --store allowlist.sqlite --manifest examples/holders-room.json \
    --package evidence/2026-10-17-first-proof.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
```

```
REFUSED  CHALLENGE_UNKNOWN_OR_EXPIRED
The challenge is unknown, already used or expired. Nothing was admitted.
not on the allowlist, and nothing was written
```

**Exit code 1.** The same package that `verify_package` admitted above is refused here, and the
difference is the whole point of a challenge: `verify` checks a proof against a rule, `admit`
checks it against *this gate's* open challenge and then records it. A package cannot be lifted from
one gate's records and replayed into another's.

```
./target/release/examples/allowlist list --store allowlist.sqlite
```

```
0 admission(s), 0 marker(s)

no address, balance, package or raw challenge is stored anywhere above.
```

Exit code 0. The refusal wrote nothing: the three tables (`challenges`, `markers`, `admissions`)
exist, and the only row in any of them is the challenge this store issued itself. That is A11's
claim — an invalid or unanswerable package writes nothing — observed here from the outside, in a
program that is 187 lines of public API.

---

## What this does and does not settle

**It settles** that both examples compile and run against the real crate API, that the recorded
proof is still ADMITTED against a root fetched from the live testnet today, and that a package
answering an unknown challenge is refused by `admit` while the store stays empty.

**It does not settle** anything about the two-source rule (this run used one source, deliberately),
anything about an ADMITTED path through `admit` (which needs a holder proving against a challenge
this store issued, and so a wallet — see [the two-room knock](2026-10-20-two-room-knock.txt) for
that run), or anything on mainnet.
