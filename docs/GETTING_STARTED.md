# Getting started with Unstirred

Three paths. Pick the one you are:

- **[A holder](#a-holder)** — you have ZEC and want to prove you held it, without saying who you are.
- **[A gate owner](#a-gate-owner)** — you run a community, an allowlist or a faucet and want a door.
- **[A developer](#a-developer)** — you are putting this behind something you build.

If you only want to see that any of it is real, there is a shorter path than all three: the
[60-second check](../README.md#check-it-yourself-in-60-seconds) re-verifies a recorded real-testnet
proof in your own process, with no wallet, no node and nothing to configure.

Everything here is **Zcash testnet**. Nothing in this project has been run on mainnet.

---

## Install

Either take a binary, or build one.

**The binary.** From [the releases page](https://github.com/DrApkFile/Unstirred/releases), take the
tarball for your platform, check it against the `.sha256` beside it, and unpack it:

```
curl -LO https://github.com/DrApkFile/Unstirred/releases/download/v0.1.0/unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/DrApkFile/Unstirred/releases/download/v0.1.0/unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
sha256sum --check unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256
tar -xzf unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
```

The binary is inside, beside the README, the spec and both licences. Put it on your `PATH`.

**From source.** Rust 1.91, pinned by `rust-toolchain.toml`:

```
git clone https://github.com/DrApkFile/Unstirred
cd Unstirred
cargo build --release
```

That is a cold build of the Halo2 and Zcash dependency graph and it is not quick: a recorded
clean-clone build of 350 crates took **27 minutes 34 seconds** with `CARGO_BUILD_JOBS=2` on a
four-core 2.40 GHz i3 with 7 GB of memory and a warm cargo registry
([the audit](../evidence/2026-10-26-clean-clone-audit.md)). A faster machine with all four jobs
running is considerably quicker. The binary lands at `target/release/unstirred`.

Check it runs, and read the verdicts it can give you, before anything else:

```
unstirred --version
unstirred codes
```

---

## A holder

What you are about to prove: **that you held at least 0.125 ZEC in the Ironwood pool, unmoved,
from one block to another.** Not your address, not your balance, not which coins, not anything you
have ever spent.

### 1. A wallet, with coins that have sat still

You need a wallet built on `zcash_client_sqlite` 0.22. [zcash-devtool](https://github.com/zcash/zcash-devtool)
is the one this project was developed against; check its README for Ironwood / NU6.3 support
before you start, because a wallet from an older cohort cannot see Ironwood notes at all.

Create a testnet wallet, take a unified address, receive TAZ, and let it sync. Then **leave those
coins alone.** The statement is about coins that did not move; spending one inside the window ends
the proof for that coin, and there is no way to un-spend it.

Two things matter about timing:

- **Your coins must already exist at the gate's `h0`.** A coin mined after `h0` cannot be proved to
  have been held from `h0`, and the answer is `COINS_TOO_NEW` before anything is sent anywhere.
  Write down the block height at which your coins confirmed; `h0` must be at or after it.
- **Your wallet must be synced past the gate's `h1`.** Otherwise your own side cannot know the
  coins were still unspent at `h1`, and it refuses with `WALLET_NOT_SYNCED_PAST_H1` rather than
  guessing.

You also need the **age identity file** that decrypts your wallet's `keys.toml` — one
`AGE-SECRET-KEY-…` line. That is what `--identity` takes. The spending key is read from it into
zeroizing buffers, checked against the account's own viewing key, and never printed, logged or
sent (`briefing/LEAKS.md` L15b states the one limit on that).

### 2. Find a gate

A gate publishes a URL and a room name. Ask it what it hosts:

```
curl http://the-gate.example:8080/rooms
```

You will get the rooms, each with its window and threshold, the `h1` root its operator pinned, and
the root sources it reads. If the gate advertises them, you will also see a `services` object with
a `pir_url` and a `lightwalletd_url` — those are the operator's own public services, and `knock`
will use them so you do not have to find your own.

### 3. Knock

```
unstirred knock --gate http://the-gate.example:8080 --room holders-room \
    --wallet ~/wallet --identity ~/unstirred.age
```

If the gate does not advertise its services, or you would rather use your own, add them and your
flags win:

```
unstirred knock --gate http://the-gate.example:8080 --room holders-room \
    --wallet ~/wallet --identity ~/unstirred.age \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443
```

What happens, in order:

1. It fetches the room's rule and prints it, along with which service addresses it is about to use
   and whose they are.
2. **It applies that rule to your wallet before it asks the gate for anything else.** If your coins
   do not qualify, it stops here: no challenge is requested, nothing is posted, and it tells you in
   those words that the gate has no record you tried. That is not a courtesy — it is the point. A
   gate cannot refuse you if it never heard from you.
3. If you do qualify, it takes a fresh challenge, proves (10–15 seconds of Halo2 on one core, in a
   release build), posts the package, and prints the gate's verdict.

Exit codes: `0` admitted, `1` refused, `2` usage, `3` local error, `4` inconclusive.

**The same coins at the same room a second time are `ALREADY_USED_IN_CONTEXT`.** At a *different*
room they are admitted again, under five markers that share nothing with the first set — the two
gates cannot tell they saw the same holder.

**What the gate learns about you:** that someone who held at least the threshold across its window
knocked, and five markers that mean nothing outside that room. What it does not learn: your
address, your balance, how much you held, which coins, when they arrived, or anything you ever
spent. What it can still see, because no software can hide it, is that a connection happened —
use Tor if that matters (`briefing/LEAKS.md` L12).

### The steps, separately

`knock` is five commands in one. Each runs alone, which is how you find out where something went
wrong:

```
unstirred notes   --wallet ~/wallet --manifest room.json
unstirred witness --wallet ~/wallet --manifest room.json --lightwalletd https://testnet.zec.rocks:443
unstirred lookup  --wallet ~/wallet --manifest room.json --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443
unstirred prove   --wallet ~/wallet --manifest room.json --identity ~/unstirred.age \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://localhost:3000 --out package.json
```

`prove` refuses to overwrite an existing `--out` file.

---

## A gate owner

You need three things running: a Zcash node, a nullifier-PIR server, and the gate itself. The
first two are not ours and are the real work; the gate is one process and one file.

### 1. Install, and a node

[Install](#install) the binary. Then run a testnet **zebrad** and let it sync past your window plus
finality. This project was developed against `zebrad 7.0.0-rc.0` with no cookie auth, on
`http://127.0.0.1:18232`.

You need it because **two independent root sources that agree are the default**, and there is no
second public lightwalletd that answers `z_gettreestate` on testnet — so in practice the second
source is your own node. One source is possible, and every line of output says it was not
independently checked. Sources that disagree, or that cannot be reached, give `INCONCLUSIVE` and
never `ADMITTED`.

### 2. Valar's nf-server, at v0.12.3

The private "is this nullifier absent at h1?" lookup is answered by
[vote-nullifier-pir](https://github.com/valargroup/vote-nullifier-pir), **tag `v0.12.3`** —
not HEAD. The pin is deliberate and recorded in `DECISIONS.md`.

```
git clone https://github.com/valargroup/vote-nullifier-pir
cd vote-nullifier-pir
git checkout v0.12.3
cargo build --release
```

Export the dataset for your `h1` and serve it. `SYNC_HEIGHT` must be a multiple of 10, which is
also why a gate manifest refuses an `h1` that is not:

```
./target/release/nf-server serve --zcash-network test \
    --pir-data-dir ../pir-data/test-v0123 --port 3000 --pir-poly-len 4096
```

The dataset's own `circuit_root` is the h1 nullifier-set root you are about to pin. Write it down.

Holders need this server, and so does anyone proving. A gate that is only *verifying* does not.

### 3. Write the room's rule

`h0` and `h1` are your rule and they are the whole statement: "held from here to here". `h1` must
be a multiple of 10, below the tip by enough to be final, and above `h0`.

```
unstirred gate init --network test --context my-room --h0 4505790 --h1 4506510 --out my-room.json
```

That writes a manifest, and nothing else — six fields, which `unstirred-core`'s own parser
accepted. This is the file you publish.

### 4. Write the gate's config

This file is yours and is never published. [`examples/gate.toml`](../examples/gate.toml) is a
working one:

```toml
listen = "127.0.0.1:8080"
database = "gate.sqlite"
root_sources = ["lwd:https://testnet.zec.rocks:443", "zebra:http://127.0.0.1:18232"]
h1_root = "8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26"

# Optional, and public: advertised in GET /rooms so holders do not have to be told them.
pir_url = "http://127.0.0.1:3000"
lightwalletd_url = "https://testnet.zec.rocks:443"

[[rooms]]
manifest = "my-room.json"
```

Set `pir_url` and `lightwalletd_url` unless you have a reason not to. They are the two addresses a
holder cannot discover for themselves, and leaving them out means telling every holder out of band.
They are your own public services, the same for everyone who knocks, and they carry nothing about
any holder; a holder who prefers their own passes `--pir` or `--lightwalletd` and their flag wins.

**`h1_root` is operator-pinned, and the verifier takes your word for it.** Checking it
independently needs `nf-server verify-root` against a full archive node, which is
[not built](../CLAIMS.md). A holder who trusts your gate is, on this one value, trusting you. Say so
when you publish.

A gate refuses to start on an unreadable manifest, a missing setting, two rooms with the same
context, the wrong number of root sources, an `h1_root` that is not 32 bytes of hex, or a service
address a holder could not reach. Every one of those is a refusal before the socket is bound.

### 5. Serve, and publish

```
unstirred gate serve --config gate.toml
```

Publish two things: **the gate's URL and the room's name.** That is all a holder needs — the rule
itself is served at `/rooms/my-room/manifest`, byte for byte what they will prove against, so
nobody has to take your word for what it says.

Open `http://127.0.0.1:8080/` yourself: the gate serves a room picker and one Holders' Room page
per room, assembled in-process, fetching nothing from anywhere. It also serves five recorded acts
of the demo at `/room/<context>?replay=<act>`, which need no chain, no wallet and no nf-server —
useful for showing people what your door does before they knock on it.

### What you keep

One SQLite file. Per admission: five markers scoped to that room's context, and a verdict. Not an
address, not a package, not a raw challenge, not an IP. `unstirred gate markers` prints everything
it holds, which is the fastest way to satisfy yourself about that.

---

## A developer

Two routes, both in [`INTEGRATION.md`](INTEGRATION.md):

- **[Over HTTP, from any language](INTEGRATION.md#1-over-http-from-any-language)** — challenge, the
  holder proves locally, post the package, read the verdict. There is a `curl` version and a
  JavaScript version. Your service never touches a wallet.
- **[In Rust](INTEGRATION.md#2-in-rust)** — the crates, by git tag today and from crates.io once
  0.1.0 is published, with working code against the real public API. `unstirred-verifier` is the
  one you want if you are embedding a gate; `unstirred-prover` is the one a wallet embeds.

Two runnable examples are in the repository and compile in the gate:

```
cargo run --release --example verify_package -p unstirred-verifier -- \
    --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57

cargo run --release --example allowlist -p unstirred-verifier -- --help
```

---

## Where to go next

- [`INTEGRATION.md`](INTEGRATION.md) — the HTTP flow, the crates, and what a verdict is worth
- [`SPEC.md`](../SPEC.md) — what is proved, why the split height is sound, the wire formats, the limits
- [`CLAIMS.md`](../CLAIMS.md) — every claim this project makes and the file that settles it
- [`briefing/LEAKS.md`](../briefing/LEAKS.md) — every privacy channel, with its test or its stated non-defence
