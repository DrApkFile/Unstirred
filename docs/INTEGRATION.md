# Integrating Unstirred

Putting a holding-duration door in front of something you build. Two routes — [over HTTP from any
language](#1-over-http-from-any-language), or [in Rust](#2-in-rust) — then what a verdict is
actually worth, and one limit you should know about before you design around it.

What is proved and why it holds is [`SPEC.md`](../SPEC.md). How to run the commands is
[`GETTING_STARTED.md`](GETTING_STARTED.md).

**The holder proves; you verify.** Your service never touches a wallet, never sees an address, and
never holds a key. That division is the whole design, and the easiest way to get this wrong is to
try to be helpful by doing the proving for someone.

---

## 1. Over HTTP, from any language

Three requests and one command the holder runs. Nothing here is Rust-specific.

```
  your service                     the gate                      the holder
       │                               │                              │
       │  POST /rooms/<c>/challenge    │                              │
       ├──────────────────────────────►│                              │
       │  { challenge, expires_at }    │                              │
       │◄──────────────────────────────┤                              │
       │                                                              │
       │  "prove this, you have 10 minutes"                           │
       ├─────────────────────────────────────────────────────────────►│
       │                                                              │
       │                    unstirred prove --challenge … --out package.json
       │                                                              │
       │  package.json                                                │
       │◄─────────────────────────────────────────────────────────────┤
       │  POST /rooms/<c>/admit        │                              │
       ├──────────────────────────────►│                              │
       │  { verdict, code, … }         │                              │
       │◄──────────────────────────────┤                              │
```

The holder can also do all three themselves with `unstirred knock`, which is the same flow in one
command. Use the split version when *your* service is the thing deciding what admission buys.

### Step 1: take a challenge

```
curl -sX POST http://gate.example:8080/rooms/holders-room/challenge
```

```json
{ "challenge": "17eeb99835dd…4e57", "expires_at": "2026-10-26T18:04:11Z" }
```

Single use, ten minutes. The gate stores only a BLAKE2b hash of it, so this response is the only
copy — hand it to the holder.

### Step 2: the holder proves

They run this; you never see their wallet:

```
unstirred prove --wallet ~/wallet --manifest holders-room.json --identity ~/key.age \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://localhost:3000 \
    --challenge 17eeb99835dd…4e57 --out package.json
```

10–15 seconds of Halo2 on one core in a release build. The result is a JSON package of nine
fields — the instance, the proof, a session key, the challenge and a binding signature. No
address, no value, no tree position, no nullifier; there is nothing in it to redact.

### Step 3: post it, read the verdict

```
curl -sX POST --data-binary @package.json \
    -H 'Content-Type: application/json' \
    http://gate.example:8080/rooms/holders-room/admit
```

```json
{ "verdict": "ADMITTED" }
```

```json
{
  "verdict": "REFUSED",
  "code": "ALREADY_USED_IN_CONTEXT",
  "message": "These coins have already been used at this gate. Nothing was admitted.",
  "retryable": false
}
```

### The same, in JavaScript

```js
const GATE = "http://gate.example:8080";
const ROOM = "holders-room";

// 1. A challenge for this holder.
async function challenge() {
  const response = await fetch(`${GATE}/rooms/${ROOM}/challenge`, { method: "POST" });
  if (!response.ok) throw new Error(`the gate answered ${response.status}`);
  return response.json(); // { challenge, expires_at }
}

// 3. The package they produced, posted as it stands. Do not reserialise it: the binding
//    signature is over these exact bytes' meaning, and a helpful reformat is a BAD_SIGNATURE.
async function admit(packageJson) {
  const response = await fetch(`${GATE}/rooms/${ROOM}/admit`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: packageJson,
  });

  // A verdict arrives on 200, 403 and 503 alike. Only a 4xx that is not 403 is a transport
  // problem: those carry { error } and no code, because no package was examined.
  const body = await response.json();
  if (body.code === undefined && body.verdict === undefined) {
    throw new Error(`not a verdict: ${body.error ?? response.status}`);
  }
  return body;
}

const verdict = await admit(await fs.readFile("package.json", "utf8"));
switch (verdict.verdict) {
  case "ADMITTED":
    await grantWhateverYouGrant();
    break;
  case "REFUSED":
    // verdict.retryable says whether a later attempt could differ at all.
    deny(verdict.code, verdict.message);
    break;
  case "INCONCLUSIVE":
    // Not a refusal of the holder. Something is down or disagreeing: say so, and let them retry.
    tryAgainLater(verdict.message);
    break;
}
```

### The routes

| Route | Method | Answers |
|---|---|---|
| `/rooms` | GET | the contexts this gate hosts, and the service addresses it advertises |
| `/rooms/<context>/manifest` | GET | that room's rule, byte for byte what its holders prove against |
| `/rooms/<context>/challenge` | POST | a fresh 32-byte challenge in hex and its `expires_at` |
| `/rooms/<context>/admit` | POST | a verdict: ADMITTED 200, REFUSED 403, INCONCLUSIVE 503 |
| `/rooms/<context>/log` | GET | markers and admission verdicts; never a challenge hash |
| `/rooms/<context>/recent` | GET | a bounded in-memory feed: verdict, code, time, nothing else |
| `/` and `/room/<context>` | GET | the room picker and the Holders' Room page |

An admit body is capped at 256 KiB. **HTTP-level refusals carry no verdict code**: a 404, 405, 413
or 400 answers `{"error": …}` and nothing from the closed set, because no package was examined and
so no verdict exists. Do not map them onto codes.

### Three outcomes, not two

**Unknown is never success, and it is never failure either.**

| Verdict | HTTP | Exit | What you do |
|---|---|---|---|
| `ADMITTED` | 200 | 0 | the markers are written; issue whatever you issue, now |
| `REFUSED` | 403 | 1 | a rule said no; `code` says which, `retryable` whether a later try could differ |
| `INCONCLUSIVE` | 503 | 4 | the gate could not decide; **not** a refusal, and not a tight retry loop |

`unstirred codes` prints the closed set. The ones an integration meets:

- `ALREADY_USED_IN_CONTEXT` — these coins entered this room before. Never retryable.
- `CHALLENGE_UNKNOWN_OR_EXPIRED` — retryable with a fresh one. Proving takes 10–15 seconds and the
  challenge is consumed against a clock read *after* the proof, so a slow machine can lose a race
  nobody told it about.
- `ROOT_MISMATCH` — the coins were not held unmoved across this window. This is what a newcomer gets.
- `COINS_TOO_NEW`, `INSUFFICIENT_HELD_BALANCE`, `TOO_MANY_NOTES_NEEDED` — the holder's own side
  refused before anything was sent. Your gate never saw the attempt and holds no record of it.
- `ROOT_SOURCES_DISAGREE`, `ROOT_SOURCE_UNAVAILABLE`, `PIR_UNAVAILABLE`, `TREE_STATE_UNAVAILABLE` —
  inconclusive. Something is down or disagreeing; nothing is wrong with the holder.

---

## 2. In Rust

### Depending on it

Until 0.1.0 is on crates.io, take it by git tag — exact, and the tag is what the recorded runs were
made from:

```toml
[dependencies]
unstirred-core = { git = "https://github.com/DrApkFile/Unstirred", tag = "v0.1.0" }
unstirred-verifier = { git = "https://github.com/DrApkFile/Unstirred", tag = "v0.1.0" }
```

Once 0.1.0 is published, the same code works against the registry:

```toml
[dependencies]
unstirred-core = "0.1"
unstirred-verifier = "0.1"
```

| Crate | What it is for |
|---|---|
| `unstirred-core` | the shared contract: manifest, window and context rules, package schema, binding message, the closed code set |
| `unstirred-verifier` | the gate's half: the engine over roots it authenticates itself, and the store that admits one set of coins per context exactly once |
| `unstirred-prover` | the holder's half: read-only wallet access, selection, witnesses, the private lookup, the proof and its signature |
| `unstirred-gate` | the HTTP service and the page it serves itself |
| `unstirred` | the command line over all of it |

### Verifying a package

Three steps: parse the rule, get the roots from sources you chose, check the package against
**those**. The verifier never reads a root, a height, a network or a context out of the package.

```rust
use unstirred_core::parse_manifest;
use unstirred_verifier::{consult, parse_root_source_spec, verify_parsed, PinnedRoots, Verdict};

let manifest = parse_manifest(&std::fs::read_to_string("holders-room.json")?)?;

// Two independent sources, which must agree. One is possible and labels itself.
let sources = ["lwd:https://testnet.zec.rocks:443", "zebra:http://127.0.0.1:18232"]
    .iter()
    .map(|spec| parse_root_source_spec(spec))
    .collect::<Result<Vec<_>, _>>()?;

let readout = consult(&sources, manifest.network, manifest.window.h0);
let agreed = match readout.agreement {
    Ok(agreed) => agreed,
    // Disagreement or an unreachable source is INCONCLUSIVE. Never fall through to a decision.
    Err(refusal) => return Ok(Outcome::Inconclusive(refusal)),
};

let roots = PinnedRoots {
    nc_root_at_h0: agreed.nc_root_at_h0,
    nf_imt_root_at_h1: operator_pinned_h1_root,
};

match verify_parsed(&manifest, &package_json, &roots, agreed.tip, &challenge) {
    Verdict::Admitted => { /* this proof holds — but nothing is recorded yet */ }
    Verdict::Refused(refusal) => { /* refusal.code is from the closed set */ }
    Verdict::Inconclusive(refusal) => { /* say so; do not treat as a refusal */ }
}
```

The whole of that, with argument parsing and output, is
[`examples/verify_package.rs`](../crates/unstirred-verifier/examples/verify_package.rs). Run it
against the recorded real-testnet proof:

```
cargo run --release --example verify_package -p unstirred-verifier -- \
    --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

### Admitting — verifying *and* recording

`verify_parsed` writes nothing, so the same package verifies for ever. Single use comes from
`admit`, which does the same checks and then, **in one transaction**, inserts five markers under
`UNIQUE(context, marker)`, consumes the challenge and records the verdict. A marker conflict rolls
the whole thing back, so a refusal writes nothing at all.

```rust
use unstirred_verifier::store::{Store, SystemClock};
use unstirred_verifier::{admit, PinnedRoots, Verdict};

let mut store = Store::open("gate.sqlite")?;

// Issue the challenge from the same store the admission will be written to.
let issued = store.issue(&manifest.context, &SystemClock)?;
println!("challenge {}", hex::encode(issued.raw));

// …the holder proves against it, and hands back package_json…

match admit(&manifest, &package_json, &roots, agreed.tip, &mut store, &SystemClock)? {
    Verdict::Admitted => grant(),
    Verdict::Refused(refusal) => deny(refusal.code),
    Verdict::Inconclusive(refusal) => retry_later(refusal.code),
}
```

A working allowlist built on exactly that is
[`examples/allowlist.rs`](../crates/unstirred-verifier/examples/allowlist.rs): `challenge`,
`admit` and `list`, in about 180 lines.

> A database fault is **not** a verdict. `admit` returns `Err(StoreError)` for one, and a caller
> that treats it as a refusal has just told a holder their coins were rejected because a disk was
> full.

### 3. Wallets embedding the prover

A wallet is the natural home for the holder's side: it already has the notes, the witnesses and the
spending key, and `unstirred-prover` is written so you do not have to hand any of them to a
separate process.

```toml
[dependencies]
unstirred-prover = { git = "https://github.com/DrApkFile/Unstirred", tag = "v0.1.0" }
```

What embedding buys your users: they never export a spending key, never run a second binary, and
never copy a package file around. What it asks of you:

- **Read-only, and keep it that way.** The prover opens the wallet read-only and nothing in this
  project can build or broadcast a transaction. A recorded real run left the wallet file's sha256
  unchanged. If you embed it, that property is now yours to keep.
- **The spending key stays with you.** It is read into zeroizing buffers, checked against the
  account's own viewing key before use, and never printed, logged or put in a package. One limit:
  `orchard 0.15.5` implements no `Zeroize` for its key types, so the derived key is not wiped on
  drop (`briefing/LEAKS.md` L15b).
- **Apply the room's rule before you touch the network.** `unstirred knock` checks eligibility
  against the published manifest *first*, and only asks for a challenge if the wallet has something
  to prove. Do the same: a user who does not qualify should never appear in a gate's records at
  all. That is A45, and it is the difference between "refused" and "never there".
- **Tell them what a proof costs.** 10–15 seconds of CPU on one core, and the private nullifier
  lookup needs a reachable nf-server. `PIR_UNAVAILABLE` and `TREE_STATE_UNAVAILABLE` are
  inconclusive, not refusals, and no plaintext nullifier query happens in either case.
- **Expect to be offered a gate's addresses.** `GET /rooms` may carry a `services` object with a
  `pir_url` and a `lightwalletd_url`. They are the gate operator's own public services. Use them if
  your user has nothing configured, and let anything your user did configure win.

---

## 4. One context per purpose

A **context** is a room's name, and it is the unit of single use. Markers are derived per context,
so the same coins admitted at two contexts produce two disjoint marker sets, and neither gate's log
matches the other's.

That means the design question is always: *what is the thing that should happen once?*

| Context | What happens once | What a verdict unlocks |
|---|---|---|
| `community-2026` | one membership per holder | a role, an invite, a seat |
| `airdrop-round-1` | one claim per holder | a payout, once, per round — a second round is a second context |
| `poll-treasury-q4` | one ballot per holder | a vote recorded, with no way to tie it to the next poll |
| `market-maker-tier` | one registration per holder | an allowance, a tier, a rate |

Rules of thumb that follow from the marker scope:

- **One purpose per context, and never reuse one.** `airdrop-round-2` is a different context from
  `airdrop-round-1`, which is exactly what lets the same holder claim both while neither round can
  tell it was the same person.
- **A context is public and is part of the rule.** It is in the manifest the holder proves against,
  and `round_id` is derived from `(network, context)` and nothing else — no height can change it.
- **Admission is a moment, not a session.** Unstirred will not recognise a holder a second time;
  that is the privacy property, not a gap. Whatever you grant, grant it at the moment of
  `ADMITTED`, and scope it yourself.
- **Do not try to join contexts up.** Correlating two rooms' marker sets is the thing the design
  prevents; if your product needs one identity across two purposes, Unstirred is the wrong tool for
  the second one.

---

## 5. A limit worth knowing before you design around it

**An application cannot yet bind its own data into the signed message.**

The binding signature covers the statement, the session key and the gate's challenge
([`SPEC.md` §3.6](../SPEC.md)). It does not cover anything of yours. So a verdict says *"a qualifying
holder answered this challenge"* and nothing more — it cannot say *"a qualifying holder answered
this challenge **and voted yes**"* or *"…**and this is their payout address**"*.

In practice that means the gap between admission and whatever you do next is yours to close:

- **A vote.** The ballot is not inside the proof. You are binding it yourself, out of band, at the
  moment of admission.
- **A payout address.** Same. An address collected beside an admission is linked to it by your
  service, not by the proof.
- **Anything a third party should be able to re-check later.** They can re-verify the package; they
  cannot re-verify that it went with your data, because the signature never covered your data.

The clean way to live with it today is to treat the challenge as the binding point: **you choose
the challenge**, so a challenge you issue per intent — one per ballot, one per address — ties the
two together through your own records. That is not the same as a signature over the ballot, and a
holder cannot prove the link to anyone but you. Design accordingly, and do not tell your users
otherwise.

Binding caller-supplied data into the signed message is a known gap, not a decision that it should
not exist.

---

## Before you put this in front of anyone

- **Testnet only.** Every recorded run is Zcash testnet, Ironwood, blocks 4,505,790 to 4,506,510.
  Nothing here has been exercised on mainnet.
- **The split-height soundness argument has had no external review.** It is tested (A03, A04) and
  written out in [`SPEC.md` §3](../SPEC.md). No cryptographer outside this project has read it.
- **The h1 nullifier-set root is operator-pinned.** The verifier takes the operator's word for it;
  checking it independently needs `nf-server verify-root`, which is [not built](../CLAIMS.md).
- **Network-level metadata is not defended.** The gate drops the peer address where it accepts the
  connection, and no log line or response carries one — but it cannot hide that a holder connected.
  Tell them to use Tor. [`briefing/LEAKS.md`](../briefing/LEAKS.md) lists every channel with its
  test or its stated non-defence.
- **Not post-quantum, and point-in-time.** The statement says nothing about after h1.

The whole list, with reasons, is [`SPEC.md` §10](../SPEC.md).
