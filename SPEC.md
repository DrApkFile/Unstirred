# Unstirred: what it proves, and why that holds

This is the protocol document. It says exactly what a holder proves, what a gate learns, what is
reused from other people's work, and what is ours and therefore ours to be wrong about. Every
claim below names the test that holds it (`A…`, listed in `briefing/PRD.md` §10) or the recorded
run in `evidence/` that produced it. `CLAIMS.md` is the ledger those runs are indexed in, and
nothing here is stated as working that is not in it.

Version `unstirred/v1`. Network: Zcash **testnet**, Ironwood pool (NU6.3 onward). Mainnet is not
claimed; see §10.

---

## 1. The statement

A holder proves, to one gate, one sentence:

> I control at least 0.125 ZEC in the Ironwood pool, in notes that existed at block **h0** and
> were still unspent at block **h1**, and I am the party that can spend them.

The gate learns the sentence is true, and one set of five opaque markers that make these coins
unusable at this gate a second time. It does not learn the address, the balance, which notes,
their positions, their values, the transactions behind them, or whether this holder has knocked
at any other gate. The proof package carries the circuit's fourteen public inputs, the proof, a
session key, the challenge and a signature, and nothing else (A02, and `briefing/LEAKS.md` L6b; recorded for a real
package in [evidence/2026-10-17-first-proof.json](evidence/2026-10-17-first-proof.json)).

"Unmoved" means exactly *unspent*. A wallet that consolidates, sends change, or migrates a note
spends it, and the clock for that value restarts. Nothing here proves anything about what happens
after h1.

---

## 2. What this reuses, and what it adds

### Reused, unchanged, credited

- **[Valar Group's `voting-circuits`](https://github.com/valargroup/voting-circuits)**, the
  delegation circuit, pinned at `=0.12.2` with `--no-default-features --features lrz`. It is used
  as published: this repository defines no circuit, modifies no constraint and re-proves nothing
  about it. The statement Unstirred proves is the circuit's own delegation statement with the two
  root inputs pinned to two different heights (§3).
- **[Valar Group's `vote-nullifier-pir`](https://github.com/valargroup/vote-nullifier-pir)**, tag
  `v0.12.3`, as the nullifier-set IMT and the YPIR server the prover queries privately.
- **The Zcash crates** (`zcash_client_sqlite 0.22.0`, `zcash_client_backend 0.24.0`,
  `zcash_protocol`, `zcash_primitives`, `orchard` via `voting-crypto-deps`), for wallet access,
  witnesses and key derivation. Every dependency is pinned with `=`, and `Cargo.lock` is committed.

The audit that covers the circuit covers the circuit. It does not cover anything in §3 to §6.

### Added here

| Added | Where | Held by |
|---|---|---|
| The two roots pinned to **two different heights**, h0 and h1, which is what turns a snapshot statement into a duration statement | `unstirred-prover`, `unstirred-verifier` | §3; A03, A04, and the real admission in [evidence/2026-10-17-verify-first-proof.txt](evidence/2026-10-17-verify-first-proof.txt) |
| Single use **per context**, with markers that cannot be linked across gates | `unstirred-core::context`, the store | §4; A05, A08, A29 |
| A **binding signature** under the spend-authorizing key, which the circuit alone does not require | `unstirred-core::binding` | §5; A06 |
| A verifier that **authenticates its own roots** from two independent sources and never reads one from the package | `unstirred-verifier::roots` | §6; A09, A10, A10b, A26, A27, [evidence/2026-10-20-roots-two-source.txt](evidence/2026-10-20-roots-two-source.txt) |
| A closed verdict code set, every code with a message, a retryability and a next action | `unstirred-core::codes` | §9; A14 |
| A gate service, a holder's `knock`, and a page the gate serves itself | `unstirred-gate` | §7; A34–A46 |

---

## 3. Soundness of the split height

This is **our argument**, not Valar's, and it is the one claim in this repository that rests on
reasoning rather than on a recorded run. It is tested (A03, A04) and it has not been reviewed by
anyone outside this project. Read it as a claim to be checked, not as a result.

### The two circuit conditions we lean on

The delegation circuit takes `nc_root` (a note-commitment tree root) and `nf_imt_root` (a
nullifier-set IMT root) as two **independent public inputs**. Nothing in the circuit ties them to
one another, and nothing in it ties either to a height: a height is not an input, and the circuit
cannot know one. For each real note the circuit enforces, among its other conditions:

- a Merkle path from the note's commitment to `nc_root`; and
- that the note's true nullifier, derived in-circuit from the note and the holder's nullifier key,
  is **not** a member of the IMT whose root is `nf_imt_root`.

### The argument

Let the verifier pin `nc_root` to the Ironwood note-commitment root at height **h0**, and
`nf_imt_root` to the Ironwood nullifier set at height **h1**, with h0 < h1.

1. The Merkle path puts the note's commitment in the tree as it stood at h0. A note-commitment
   tree is append-only, so a commitment in the tree at h0 was added at or before h0: **the note
   existed by h0.**
2. The non-membership proof puts the note's true nullifier outside the nullifier set as it stood
   at h1. The nullifier set is append-only too, and a spend publishes the nullifier in the block
   that contains it, so a nullifier absent at h1 was absent in every block up to h1: **the note
   was not spent at any height ≤ h1.**
3. A note that existed by h0 and was not spent through h1 was held, continuously, across the whole
   window [h0, h1]. There is no third state: an Ironwood note is either unspent or has published
   its nullifier.

Both halves are properties of append-only structures, which is what lets the two heights differ.
Nothing in the argument needs the two roots to come from the same block; what it needs is that
each root really is the root at the height the verifier believes, which is §6's job, and that
h0 < h1, which the window rule enforces (A28).

### Where it deviates from Valar

Valar's delegation protocol takes both roots **at the same height**. That is a rule of their
voting protocol — one round, one snapshot — and not a constraint of the circuit: the circuit
accepts any two field elements in those two inputs and says nothing about where they came from.
Unstirred breaks that rule deliberately and on purpose, and every consequence of breaking it is
the verifier's to carry:

- **h0 < h1 must be enforced outside the circuit.** The circuit cannot see it. The window rule
  does, and refuses `WINDOW_INVALID` otherwise (A28).
- **Both roots must come from the verifier's own authority, never the package.** A prover who
  could choose `nc_root` could choose a height at which their coins already existed, which is
  exactly the newcomer's forced submission — refused `ROOT_MISMATCH` against the gate's own pinned
  root, by the verifier (A03's verifier half) and on the real testnet
  ([evidence/2026-10-22-bob-forced-knock.txt](evidence/2026-10-22-bob-forced-knock.txt)).
- **h1 must be deep enough that a reorg cannot move it.** The window rule requires h1 to be final
  against the tip the verifier fetches for itself (A28).

### What is tested, and what is not

- A note created after h0 is refused on the holder's side (`COINS_TOO_NEW`) and, if forced through
  anyway against a later root, refused by the gate (`ROOT_MISMATCH`): A03, both halves, plus the
  real testnet pair [evidence/2026-10-22-bob-knock.txt](evidence/2026-10-22-bob-knock.txt) and
  [evidence/2026-10-22-bob-forced-knock.txt](evidence/2026-10-22-bob-forced-knock.txt).
- A note spent inside the window cannot be proven unspent at h1: A04.
- A note held across the window is admitted: A01, recorded on the real testnet in
  [evidence/2026-10-17-verify-first-proof.txt](evidence/2026-10-17-verify-first-proof.txt).
- **Not tested, and not claimed:** that the argument above is complete. It has had no external
  review. `briefing/knowledge/HOLDING_DURATION_PAPER_CHECK.md` is the working note it comes from,
  and it is marked "not yet reviewed by a cryptographer" there too.

---

## 4. Markers: once per gate, unlinkable across gates

The circuit publishes five field elements, `gov_null[5]`, each derived in-circuit as a Poseidon
hash of the holder's nullifier key, a domain element `dom`, and a note's true nullifier. The gate
stores these five as its **markers** and nothing else about the proof.

A proof covers up to five notes, and the circuit pads the rest: with one real note selected, one
marker is that note's and the other four come from the builder's own padding nullifiers, which are
fresh per proof. A gate therefore stores five markers of which only the real notes' repeat, and
the padding ones are noise it cannot match on (A47).

- **Deterministic per (note, context):** a real note's marker is the same every time that note is
  proved at that gate, so a second attempt with the same coins collides on it and is refused
  `ALREADY_USED_IN_CONTEXT` with nothing written — the `UNIQUE(context, marker)` constraint needs
  one collision, not five (A05, A35, A47, and on the real testnet in
  [evidence/2026-10-20-two-room-knock.txt](evidence/2026-10-20-two-room-knock.txt)).
- **Unrelated per proof, except for that collision:** `rk`, the session key and `van_comm` are
  all fresh per proof, so two admissions by one holder have nothing in common but the marker the
  gate is meant to match (A47).
- **Different per context:** `dom` comes from `round_id`, which is
  `BLAKE2b-512(personal = "Unstirred_ctx_v1", network byte ‖ 0x00 ‖ context) → pallas::Base`. The
  same coins at two gates therefore produce two disjoint marker sets, and neither gate's log can
  be matched against the other's (A08; the real run shows five markers per room and none shared,
  [evidence/2026-10-20-two-room-knock.txt](evidence/2026-10-20-two-room-knock.txt)).

### Why `round_id` excludes heights

`round_id` is derived from the **network and the context only**. No height goes into it, and A29
fails if one ever does. The reason is that a gate must be able to republish its window — move h0
forward for a new season, extend h1 — without the markers changing. If heights were in the
derivation, a gate could re-admit the same coins simply by shifting its window by one block, and
single use would mean nothing (A12 states this as the rule; it is held by the derivation, not by a
gate run, which §"Limits" repeats).

The cost of that choice is honest and worth naming: markers are stable for a context **forever**,
so a gate that wants a holder to be able to qualify again must publish a new context, not a new
window.

The gate stores the five markers, the admission verdict and its time, and a **hash** of each
challenge — never the raw challenge. A dump of the database contains no address, no package body,
no note position and no note value (A02b).

---

## 5. Spend-key binding, and the attack it closes

**The proof alone is not enough, and this is the single most important thing in this document.**
The delegation circuit's witness needs only full-viewing-key-level secrets. Anyone holding a
holder's viewing key — an exchange, an auditor, a wallet backup, a block explorer the holder once
pasted a key into — can produce a *valid* proof for that holder's coins. This was confirmed before
the build started, in `briefing/spikes/spend_authority_spike.rs` (reality spike S6).

So a package is only admitted if it also carries a RedPallas signature over

```
"unstirred/v1/bind" ‖ round_id ‖ nc_root ‖ nf_imt_root ‖ van_comm ‖ gov_null[0..5] ‖ session_pubkey ‖ challenge
```

(all field elements as their 32-byte canonical representation), verified under `rk` **rebuilt from
the instance's own coordinates**: take `rk_x.to_repr()`, set the top bit of byte 31 if `rk_y` is
odd, and read that as a `redpallas::VerificationKey`. `rk` is the holder's spend-authorizing key
randomized per proof, so signing it requires the spending key, which a viewing key holder does not
have.

- A viewing-key-only prover is refused `BAD_SIGNATURE` (A06).
- The message changes if any statement field, the session key or the challenge changes, over
  thousands of random statements (A31).
- Because the challenge is inside the message, a package seen on the wire cannot be replayed at
  the same gate against a different challenge, and cannot be front-run: the challenge it answers
  is consumed by the admission that uses it (A07, A36).
- The spending key is derived in memory from the wallet's age-encrypted mnemonic, checked against
  the account's own viewing key before use, and never printed or written (A23, A24). Its one
  stated limit is in `briefing/LEAKS.md` L15b: `orchard 0.15.5` does not zeroize its key types on
  drop.

`session_pubkey` is a fresh random 32 bytes per proof with no private half in v1. It makes each
package distinct and is covered by the signature; it is not yet a key anything challenges.

---

## 6. Root authority: the verifier's own roots, or no verdict

**Nothing in the package is ever used as a root, a height, a network or a context.** The verifier
takes the window, the context and the network from the manifest it was given, and both roots from
its own authority. The package's echoed copies are diagnostics (A09).

- **h0, the note-commitment root.** Read from **two** independent `--root-source` backends —
  `lwd:<url>` (lightwalletd `GetTreeState`) or `zebra:<url>` (Zebra JSON-RPC `z_gettreestate`) —
  which must return exactly the same root for h0. The governing chain tip is the **lower** of the
  two answers. Disagreement is `ROOT_SOURCES_DISAGREE`; an unreachable, malformed, wrong-network
  or wrong-height answer is `ROOT_SOURCE_UNAVAILABLE`. Both are **INCONCLUSIVE**, never a refusal
  of the holder and never an admission (A10, A10b, A26). Two agreeing sources have been run for
  real: [evidence/2026-10-20-roots-two-source.txt](evidence/2026-10-20-roots-two-source.txt) and
  [evidence/2026-10-20-verify-two-source.txt](evidence/2026-10-20-verify-two-source.txt).
- **One source is an explicit opt-in.** `--single-root-source` is required to run with one, and
  every answer it produces — human output and JSON alike — carries
  `ROOT FROM ONE SOURCE: not independently checked`, with the agreement line reading
  `not checked (one source)` rather than claiming agreement (A27).
- **h1, the nullifier-set root, is operator-pinned.** It is passed as `--h1-root` / `h1_root` and
  is labelled `operator-pinned` wherever it is shown. Checking it independently means running
  `nf-server verify-root`, which rebuilds the root from raw blocks and needs a full node serving
  blocks back to Ironwood activation plus an independently trusted block hash at h1. **That is not
  built** (`CLAIMS.md`, last row), and until it is, h1 is a trust assumption the operator carries,
  stated here and in the README rather than papered over.

**Unknown is never success.** No root trouble can produce ADMITTED.

---

## 7. Wire formats

### The manifest (what a gate publishes)

```json
{
  "version": "unstirred/v1",
  "network": "test",
  "context": "holders-room",
  "window": { "h0": 4505790, "h1": 4506510 },
  "min_zatoshis": "12500000"
}
```

Parsed strictly: an unknown field, a missing field, a version that is not `unstirred/v1`, a
threshold that is not `12500000`, or a context that is not 1 to 64 characters of `a-z`, `0-9` and `-` starting with a letter or a digit, is `MANIFEST_INVALID`
(A13, A30, A40). The window rule requires h0 < h1, h1 a multiple of 10, and h1 final against the
tip the verifier fetched for itself, or `WINDOW_INVALID` (A28).

### The proof package (what a holder hands a gate)

```json
{
  "version": "unstirred/v1",
  "context": "holders-room",
  "network": "test",
  "window": { "h0": 4505790, "h1": 4506510 },
  "instance": {
    "nf_signed": "…", "rk_x": "…", "rk_y": "…", "cmx_new": "…", "van_comm": "…",
    "vote_round_id": "…", "nc_root": "…", "nf_imt_root": "…",
    "gov_null": ["…", "…", "…", "…", "…"], "dom": "…"
  },
  "proof": "<hex>",
  "session_pubkey": "<32 bytes hex>",
  "challenge": "<32 bytes hex>",
  "binding_signature": "<64 bytes hex>"
}
```

Fourteen field elements, each the 32-byte canonical representation in hex; `rk_y` is carried whole
but only its parity is read. Parsed strictly: an unknown or missing field is `PROOF_INVALID`. A
package round-trips through JSON and still verifies (A22). A real one is
[evidence/2026-10-17-first-proof.json](evidence/2026-10-17-first-proof.json).

### The gate's HTTP surface

| Route | Method | Answers |
|---|---|---|
| `/rooms` | GET | the contexts this gate hosts |
| `/rooms/<context>/manifest` | GET | that room's manifest, byte for byte what its holders prove against |
| `/rooms/<context>/challenge` | POST | a fresh 32-byte challenge in hex and its `expires_at` |
| `/rooms/<context>/admit` | POST | a verdict: ADMITTED 200, REFUSED 403, INCONCLUSIVE 503 |
| `/rooms/<context>/log` | GET | markers and admission verdicts; never a challenge hash |
| `/rooms/<context>/recent` | GET | a bounded in-memory feed: verdict, code, time, nothing else (A43) |
| `/` and `/room/<context>` | GET | the room picker and the Holders' Room page (A42) |

A challenge is single use and expires 10 minutes after it is issued (`expires_at > now`); the gate
stores only a BLAKE2b hash of it, personalised `UnstirredChal_v1`. An admit body is capped at
256 KiB; a body refused on its declared length is read and thrown away up to four times that cap
before the 413 is written, so the refusal reaches the client, and nothing drained is parsed,
verified or stored (A37). HTTP-level refusals — 404, 405, 413, 400 — carry `{"error": …}` and
**no code from the closed set**, because no package was examined and so no verdict exists (A37,
A38). The peer address is dropped where the connection is accepted; no log line and no response
carries an address other than the gate's own listen address, a package body, a raw challenge or a
session key (A39).

---

## 8. The admission order

First failure returns. Nothing before step 9 writes anything.

| # | Step | Refusal |
|---|---|---|
| 1 | Parse the manifest strictly | `MANIFEST_INVALID` |
| 2 | Check the window against the tip the verifier fetched | `WINDOW_INVALID` |
| 3 | Recompute `round_id` from the manifest; compare to the instance's `vote_round_id` | `CONTEXT_MISMATCH` |
| 4 | Load the pinned h0 and h1 roots from the root authority | `ROOT_SOURCES_DISAGREE`, `ROOT_SOURCE_UNAVAILABLE` (INCONCLUSIVE) |
| 5 | Compare the instance's `nc_root` and `nf_imt_root` to the pinned roots | `ROOT_MISMATCH` |
| 6 | Check the challenge is this context's, open and unexpired (read only) | `CHALLENGE_UNKNOWN_OR_EXPIRED` |
| 7 | Verify the binding signature under `rk` rebuilt from the instance | `BAD_SIGNATURE` |
| 8 | Verify the Halo2 proof against the instance | `PROOF_INVALID` |
| 9 | **In one immediate transaction:** insert all five markers under `UNIQUE(context, marker)`, consume the challenge against `expires_at > now`, record the verdict | `ALREADY_USED_IN_CONTEXT` or `CHALLENGE_UNKNOWN_OR_EXPIRED`, transaction rolled back, nothing written |
| 10 | ADMITTED | — |

Step 6 is a read; the consume happens inside step 9's transaction, against a clock read **after**
the proof, because proving may have taken long enough for the challenge to expire
(`DECISIONS.md`, the row refining PRD §6). Two concurrent admissions of the same markers race
through SQLite's UNIQUE constraint, not through a mutex in one process, and exactly one wins
(A05c). A failure injected after the writes and before the commit leaves the database unchanged
(A05d). An invalid proof writes nothing (A11).

`unstirred verify` runs steps 1–8 and writes nothing at all; its ADMITTED output says so and names
`admit` as the command that records (A33).

---

## 9. The closed code set

Every verdict is ADMITTED, or one of these codes. Each has a wire name, a kind, a retryability, a
message for the holder and a next action, in one `match` so a code cannot be added without words
(A14). `unstirred codes` prints the set.

| Code | Kind | Retryable |
|---|---|---|
| `MANIFEST_INVALID` | REFUSED | no |
| `WINDOW_INVALID` | REFUSED | no |
| `CONTEXT_MISMATCH` | REFUSED | no |
| `ROOT_MISMATCH` | REFUSED | no |
| `PROOF_INVALID` | REFUSED | no |
| `BAD_SIGNATURE` | REFUSED | no |
| `CHALLENGE_UNKNOWN_OR_EXPIRED` | REFUSED | yes |
| `ALREADY_USED_IN_CONTEXT` | REFUSED | no |
| `ROOT_SOURCES_DISAGREE` | INCONCLUSIVE | yes |
| `ROOT_SOURCE_UNAVAILABLE` | INCONCLUSIVE | yes |
| `WALLET_NOT_SYNCED_PAST_H1` | REFUSED | yes |
| `COINS_TOO_NEW` | REFUSED | no |
| `INSUFFICIENT_HELD_BALANCE` | REFUSED | no |
| `PIR_UNAVAILABLE` | INCONCLUSIVE | yes |
| `TOO_MANY_NOTES_NEEDED` | REFUSED | no |
| `TREE_STATE_UNAVAILABLE` | INCONCLUSIVE | yes |

The last six are the prover's: they are reached on the holder's own machine, before anything is
sent. A holder the room's rule refuses asks the gate for nothing but its published manifest — no
challenge, nothing posted — so the gate has no record of the attempt (A45, and on the real testnet
[evidence/2026-10-22-bob-knock.txt](evidence/2026-10-22-bob-knock.txt)).

---

## 10. Limits

Stated, not worked around.

- **The threshold is fixed at 0.125 ZEC** (12,500,000 zatoshis), because the circuit requires at
  least one ballot and that is what one ballot is. A manifest naming any other threshold is
  refused (A13). A gate cannot ask for more or less in v1.
- **At most five notes per proof.** Reaching the threshold with more is `TOO_MANY_NOTES_NEEDED`
  (A18). The prover picks the fewest notes that reach it (A17, A32).
- **Testnet.** Every recorded run in `evidence/` is Zcash testnet, Ironwood, over h0 4,505,790
  through h1 4,506,510. Mainnet NU7 was targeted for 2026-11-05 and nothing here has run against
  it.
- **h1 is operator-pinned.** See §6. `nf-server verify-root` is not built.
- **A12 is a rule, not a run.** Marker stability when a gate republishes with a new window follows
  from `round_id` excluding heights (A29); no gate run exercises a republished window.
- **Not post-quantum.** Halo2 over Pallas, RedPallas signatures, BLAKE2b, Poseidon: all of it
  rests on the discrete log problem, exactly as Zcash's shielded pool does today.
- **Network-level metadata is out of scope.** A gate sees the connection a holder makes to it;
  this repository drops the peer address where it accepts the connection (A39) but cannot hide the
  connection itself. Use Tor. This is a stated non-defence, L12 in `briefing/LEAKS.md`, and the
  rest of that file is the full channel-by-channel list.
- **Point in time.** The statement says the coins were held through h1. It says nothing about what
  happened after h1, including whether they were spent one block later.
- **The split-height argument has had no external review** (§3).

---

## Reading further

- `README.md` — what it is, and how to check it yourself in 60 seconds.
- `CLAIMS.md` — every claim in this repository, its status, and the file that settles it.
- `briefing/LEAKS.md` — every privacy channel, each with a test or a stated non-defence.
- `briefing/PRD.md` — the requirements this was built against, §10 being the test list.
- `DECISIONS.md` — every decision that changed a frozen rule, with the reason and the date.
