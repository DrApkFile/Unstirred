# PRD: Unstirred

Version 1, 2026-10-15. Single source of truth. When evidence contradicts this file, record the change in `DECISIONS.md`, then update this file.

## 1. Goals and non-goals

### Goals
1. A holder proves "I have held at least 0.125 ZEC in Ironwood, unmoved, from block H0 through block H1" for one gate, revealing no address, balance, note or transaction.
2. Any gate owner verifies that claim with one CLI call or one HTTP call, without trusting the prover for any root, height or context.
3. One gate admits the same coins at most once. Two gates cannot link the same coins.
4. Everything runs on real Zcash testnet data, end to end, with evidence a judge can re-run.

### Non-goals (v1)
- Thresholds other than 0.125 ZEC, and any weighting by amount.
- Proof of personhood. Unstirred is stake-and-time weighted: one admission per 0.125 ZEC held per gate.
- Any statement about holdings after H1.
- Network-level anonymity of the prover (stated non-defence).
- Mobile wallets. v1 is CLI plus a web page for the gate.
- Mainnet production use. The demo runs on testnet (see `EVENT.md`: NU7 lands on mainnet during judging).
- New circuits. We reuse Valar's audited delegation circuit unchanged.

## 2. Hook, loop, boundary

- **Hook:** Unstirred: proof your coins stayed put. Nothing else.
- **Loop:** gate owner publishes a manifest → holder runs `unstirred prove` → gate verifies and admits or refuses with a named code → a second attempt with the same coins at the same gate is refused → the same coins at another gate are admitted with an unlinkable marker → the gate stores only markers.
- **Boundary:** the prover proposes; verifier code decides. The verifier takes roots, heights and context only from its own sources and the gate manifest, never from the proof package.

## 3. Flow and wire formats

### 3.1 Gate manifest (published by the gate owner, JSON, strict)
```json
{
  "version": "unstirred/v1",
  "network": "test",
  "context": "holders-room",
  "window": { "h0": 4505790, "h1": 4530000 },
  "min_zatoshis": "12500000"
}
```
Rules: `h0 < h1`; `h1 % 10 == 0`; `h1 <= tip - FINALITY_DEPTH` when verifying; `min_zatoshis` must equal `"12500000"` in v1; unknown fields rejected.

### 3.2 Context derivation
`round_id = hash_to_pallas_base("unstirred/v1/context", network || 0x00 || context)`.
Heights are deliberately excluded, so republishing a gate with a new window keeps the same markers (reuse stays refused).

### 3.3 Challenge (gate → holder)
`POST /challenge` → `{ "challenge": "<32 bytes hex>", "expires_at": "<RFC3339>" }`. Single use, 10-minute expiry, stored hashed.

### 3.4 Proof package (holder → gate)
```json
{
  "version": "unstirred/v1",
  "context": "holders-room",
  "network": "test",
  "window": { "h0": 4505790, "h1": 4530000 },
  "instance": { "...": "all public inputs of the delegation circuit, hex" },
  "proof": "<hex>",
  "session_pubkey": "<32 bytes hex>",
  "challenge": "<32 bytes hex>",
  "binding_signature": "<64 bytes hex>"
}
```
The window and context in the package are echoed for diagnostics only. The verifier uses the manifest's values.

### 3.5 Verdict (gate → holder)
```json
{ "verdict": "ADMITTED" }
{ "verdict": "REFUSED", "code": "ALREADY_USED_IN_CONTEXT", "message": "...", "retryable": false }
{ "verdict": "INCONCLUSIVE", "code": "ROOT_SOURCES_DISAGREE", "message": "...", "retryable": true }
```

### 3.6 Binding message (signed by the holder under rk)
`"unstirred/v1/bind" || round_id || nc_root || nf_imt_root || van_comm || gov_null[0..5] || session_pubkey || challenge`, each field element as its 32-byte canonical repr. Verified with rk rebuilt from the instance coordinates (spike S6).

## 4. Architecture and ownership

Rust workspace, edition 2021, all crypto via `voting-circuits` with the `lrz` backend.

| Crate | Role | May do network? |
|---|---|---|
| `unstirred-core` | Manifest parsing, context derivation, window rules, binding message, closed code set, verification policy as pure functions over injected root values | No |
| `unstirred-prover` | Open the zcash-devtool wallet DB, select notes, rebuild H0 witnesses, fetch nullifier non-membership via PIR, build the delegation bundle, prove, sign | Yes |
| `unstirred-verifier` | Root authority (H0 note-commitment root, H1 nullifier IMT root), proof verification, marker store, challenge store | Yes (roots only) |
| `unstirred-cli` | `prove`, `verify`, `gate init`, `roots` commands | via the crates above |
| `unstirred-gate` | HTTP service: `/manifest`, `/challenge`, `/admit`; serves the Holders' Room page | via verifier |

### Root authority (decided in advance)
- **H0 note-commitment root:** P0 source is `GetTreeState(h0)` from **two independent testnet lightwalletd servers**; exactly one agreeing answer is required, otherwise `INCONCLUSIVE / ROOT_SOURCES_DISAGREE`. Labelled in UI and README as "checked against two independent servers", not "trustless". P1 source: own Zebra node via `z_gettreestate`.
- **H1 nullifier IMT root:** the gate operator runs Valar's `nf-server sync` to `h1` and `nf-server verify-root`; the verifier refuses to start if the two disagree.

## 5. Vocabulary

- **Gate**: one manifest. **Context**: the gate's name; scopes markers.
- **Window**: `[h0, h1]`. **Held**: note commitment in the Ironwood tree at `h0`, nullifier absent from the Ironwood nullifier set at `h1`.
- **Marker**: a governance nullifier from the proof instance (5 per proof, padding included).
- **Verdicts**: `ADMITTED`, `REFUSED`, `INCONCLUSIVE`. Unknown is never success.

### Closed code set (`{message, retryable, next_action}` defined for each in `unstirred-core`)
Verifier: `MANIFEST_INVALID`, `WINDOW_INVALID`, `CONTEXT_MISMATCH`, `ROOT_MISMATCH`, `PROOF_INVALID`, `BAD_SIGNATURE`, `CHALLENGE_UNKNOWN_OR_EXPIRED`, `ALREADY_USED_IN_CONTEXT`, `ROOT_SOURCES_DISAGREE` (inconclusive), `ROOT_SOURCE_UNAVAILABLE` (inconclusive).
Prover (decides before proving): `WALLET_NOT_SYNCED_PAST_H1`, `COINS_TOO_NEW`, `INSUFFICIENT_HELD_BALANCE`, `TOO_MANY_NOTES_NEEDED` (more than 5); `PIR_UNAVAILABLE` and `TREE_STATE_UNAVAILABLE` (both inconclusive, retryable: a source the prover needs could not be reached, so no proof was made and nothing was revealed).

## 6. Verification engine (ordered; first failure returns)
1. Parse manifest strictly → `MANIFEST_INVALID`.
2. Check window rules against current tip → `WINDOW_INVALID`.
3. Recompute `round_id` from manifest; compare to instance `vote_round_id` → `CONTEXT_MISMATCH`.
4. Load pinned roots for `h0` and `h1` from the root authority → `ROOT_SOURCES_DISAGREE` / `ROOT_SOURCE_UNAVAILABLE`.
5. Compare instance `nc_root` and `nf_imt_root` to pinned roots → `ROOT_MISMATCH`.
6. Consume challenge (single use, not expired) → `CHALLENGE_UNKNOWN_OR_EXPIRED`.
7. Verify binding signature under rk from the instance → `BAD_SIGNATURE`.
8. Verify the Halo2 proof against the instance → `PROOF_INVALID`.
9. In one transaction: insert all 5 markers under the context with `UNIQUE(context, marker)`; any conflict → `ALREADY_USED_IN_CONTEXT` and nothing is written.
10. Return `ADMITTED`.

## 7. State machines
- **Challenge:** `ISSUED → CONSUMED` or `ISSUED → EXPIRED`. No reuse edge.
- **Admission:** `RECEIVED → (REFUSED | INCONCLUSIVE | ADMITTED)`. `INCONCLUSIVE` may be retried with a fresh challenge; `REFUSED` and `ADMITTED` are final.

## 8. Security model (attack lab, each with a test)
| Attack | Defence | Test |
|---|---|---|
| Viewing-key holder proves without the spend key | Binding signature under rk | A06 |
| Replay a seen package | Challenge single use; signature binds challenge and session | A07 |
| Coins bought after h0 | Prover refuses; verifier `ROOT_MISMATCH` because roots are pinned | A03 |
| Coins spent inside the window | No non-membership proof exists at h1 | A04 |
| Same coins twice at one gate | `UNIQUE(context, marker)` inside the admission transaction | A05 |
| Same coins at two gates, linkable? | Markers differ per context | A08 |
| Prover supplies its own roots | Verifier ignores them, uses pinned roots | A09 |
| Malicious root server | Two independent sources, exactly one answer | A10 |
| Proof tampered by one byte | Halo2 verification fails | A11 |
| Manifest with heights in the context | Context derivation excludes heights by construction | A12 |

## 9. Demo script (2 minutes, timed)
| Time | Act | On screen |
|---|---|---|
| 0:00–0:15 | The door rule | Holders' Room: "Held ≥ 0.125 ZEC, unmoved since block 4,505,790" |
| 0:15–0:40 | Alice proves | CLI prove (real testnet wallet) → door green. "What the door learned" panel: one line; address, balance, transactions crossed out |
| 0:40–0:55 | Same coins again | Refused: ALREADY_USED_IN_CONTEXT |
| 0:55–1:15 | Bob (coins newer than h0) | Prover: COINS_TOO_NEW. Forced submit: ROOT_MISMATCH. Timeline bar: Alice's line unbroken, Bob's starts later |
| 1:15–1:35 | Second room | Same coins admitted; both rooms' marker logs side by side, no match |
| 1:35–1:50 | Bot cost | Counter: 1,000 fake members = 125 ZEC unmoved for the whole window |
| 1:50–2:00 | The chain | Explorer and wallet history: no transaction since block 4,505,779 |

## 10. Test list (mirrors the demo and the attack lab)
- A01 a note held through the window is admitted (real testnet run recorded in `evidence/`)
- A02 the proof package and verifier logs contain no address, note value, txid or position (scanner test)
- A02b a dump of the gate database contains no IP address, proof-package body, note position or note value
- A02c no page the gate serves asks for anything from anywhere else: no script, stylesheet, font, image, frame or beacon from another origin, no analytics, and the only paths it fetches are this gate's own JSON routes (LEAKS L10)
- A03 a note created after h0 is refused by the prover (COINS_TOO_NEW) and by the verifier (ROOT_MISMATCH)
- A04 a note spent inside the window cannot be proven unspent
- A05 the same coins at the same gate are refused and nothing is written
- A05c two concurrent admissions of the same markers at one gate produce exactly one ADMITTED
- A05d a failure injected after the admission writes and before the commit leaves the database unchanged
- A06 a viewing-key-only prover is refused (BAD_SIGNATURE)
- A07 a replayed or expired challenge is refused (CHALLENGE_UNKNOWN_OR_EXPIRED) and the refusal writes nothing
- A08 the same coins at two gates produce disjoint markers
- A09 roots inside the package are never used
- A10 disagreeing root servers give INCONCLUSIVE, never ADMITTED
- A10b a root source that cannot be reached, or that answers with a malformed tree state or for the wrong network or height, makes the whole root check INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE, never ADMITTED
- A11 a one-byte tampered proof is refused (PROOF_INVALID)
- A12 republishing a gate with a new window keeps markers stable
- A13 a malformed manifest or unknown field is refused (MANIFEST_INVALID)
- A14 every code in the closed set has message, retryable and next action (compile-time coverage)
- A15 the prover refuses with PIR_UNAVAILABLE rather than making a plaintext nullifier query, and no nullifier reaches the wire in the clear
- A15b a lookup answer against any root but the expected H1 root is rejected
- A16 proving opens the wallet read-only and creates no transaction
- A17 the prover lists the notes held unmoved across the window and picks the fewest that reach the threshold
- A18 the prover refuses INSUFFICIENT_HELD_BALANCE, TOO_MANY_NOTES_NEEDED and WALLET_NOT_SYNCED_PAST_H1 before attempting a proof
- A19 witnesses rebuilt at h0 from the wallet's stored shards verify against the h0 note-commitment root, with the wallet synced past h0
- A20 a frontier that contradicts the wallet's own tree at h0 is rejected and no witness is returned
- A21 the prover's only network requests are GetTreeState for h0 and the private lookup at h1, and an unreachable source is TREE_STATE_UNAVAILABLE or PIR_UNAVAILABLE, never a guessed root
- A22 a proof package round-trips through JSON and its proof still verifies against the instance the package carries
- A23 a spending key that is not the account's, and a seed the account was not derived from, are both refused before anything is signed
- A24 the age-encrypted mnemonic in the wallet's `keys.toml` yields that wallet's seed and that account's Ironwood spending key, and no other identity can read it
- A25 a package whose instance `vote_round_id` belongs to another gate context is refused (CONTEXT_MISMATCH)
- A26 a verifier root source that cannot be reached or used gives INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE, never ADMITTED
- A27 single-source root checking exists only as an explicit `--single-root-source` opt-in: its human output and JSON verdict both carry "ROOT FROM ONE SOURCE: not independently checked", the agreement line says "not checked (one source)" instead of claiming the sources agree, and without the flag exactly two root sources are required
- A28 the window rule accepts exactly the windows it names and refuses every other, over thousands of random h0, h1 and tip triples, with every failing seed reported
- A29 `round_id` is derived from network and context only: the same pair always gives the same id, a different context or network always gives a different one, and no height can change it, over thousands of random contexts
- A30 the manifest context rule accepts exactly the documented charset and length, and a manifest that parses re-serialises and re-parses unchanged, over thousands of random strings
- A31 the binding message changes whenever any statement field, the session key or the challenge changes, over thousands of random statements
- A32 the note selection picks a smallest set of notes reaching the threshold, agreeing with a brute-force oracle over every subset, and does not depend on the order the notes arrive in
- A33 `unstirred verify`'s ADMITTED output says that this command recorded nothing and names `admit` as the command that does, in the human output and beside the JSON verdict, from one constant, and without naming a milestone that can pass
- A34 a full HTTP knock against a running gate is admitted: the holder fetches the room's manifest and a challenge over HTTP, proves against them, posts the package and reads the PRD §3.5 verdict (synthetic wallet, synthetic roots)
- A35 a second knock with the same coins at one room is refused ALREADY_USED_IN_CONTEXT over HTTP, re-posting an already admitted package is refused the same way, and neither attempt writes anything
- A36 a package answering a challenge the gate never issued is refused CHALLENGE_UNKNOWN_OR_EXPIRED over HTTP, before the signature and the proof, and the gate writes nothing
- A37 an admit body larger than the gate's limit is rejected before it is parsed, verified or stored, and the rejection invents no verdict code
- A38 a request for a room this gate does not host is 404 and names no room, marker, challenge or verdict of the gate
- A39 the gate's log lines carry no IP address other than its own listen address, and neither the log nor any response carries a proof-package body, a raw challenge or a session key (LEAKS L9, L10)
- A40 `unstirred gate init` writes only a manifest `unstirred-core` parses, and refuses h0 >= h1, an h1 that is not a multiple of 10, and a context outside the documented charset
- A41 `unstirred gate serve` refuses to start on an invalid room manifest, a missing setting, two rooms with the same context, the wrong number of root sources, or an h1 root that is not 32 bytes of hex
- A42 the gate serves the room picker at `GET /` and one Holders' Room page per hosted room at `GET /room/<context>`, 404 for any other room, naming no room, marker, challenge or verdict it holds
- A43 `GET /rooms/<context>/recent` answers a bounded feed of verdicts this process has answered, each one a verdict, a code and a time and nothing else; it shows refusals the store never wrote, writes nothing itself, and a gate restarted on the same database answers with none (LEAKS L16)
- A44 every replay act the gate ships is a valid page state whose code words are `unstirred-core`'s own, each act renders in a document that carries no network call site at all, and a replay page names an act only for the room that act belongs to
- A45 a knock the room's own rule refuses asks that gate for nothing but its published manifest: no challenge is requested, nothing is posted, and the command says both that nothing was sent and that no challenge was asked for, so the gate holds no record of the attempt
- A46 the demo's forced submission (`knock --force-unqualified-demo`) is in no usage text the CLI prints, and refuses to run at all for a wallet the room would admit, without asking that gate for a challenge
- A47 two proofs of the same coins at the same gate carry different `rk`, session key and `van_comm`, because `alpha` and the blinding are drawn fresh per proof, while exactly one marker per real note repeats — that collision is what single use rests on, and the rest of the five are the circuit's padding nullifiers (LEAKS L7)
- A48 a non-membership answer shows neither the IMT path nor the leaf bounds that bracket the holder's nullifier — not in its own `Debug`, nor in that of the lookup that carries it (LEAKS L5b)

## 11. Priorities
- **P0:** core, prover, verifier, CLI (incl. `gate markers`), gate service, Holders' Room page, GitHub Release binaries, A01–A14, `LEAKS.md`, `SPEC.md`, real testnet evidence, README, demo video.
- **P1:** Zebra root source; second room page polish; Telegram gate bot.
- **Never (this hackathon):** new circuits, thresholds above 0.125 ZEC, mainnet production, mobile, cross-chain.

## 12. Build order (record each act the moment it works)
| Milestone | Target date | Done when |
|---|---|---|
| M1 First real proof | 2026-10-17 | `unstirred prove` on the real testnet wallet produces a package; `unstirred verify` in a fresh process accepts it (closes S4/S5 live) |
| M2 Verifier engine | 2026-10-19 | Steps 1–10 implemented; A03–A14 green |
| M3 Gate + room | 2026-10-21 | `/challenge`, `/admit`, Holders' Room page; two contexts running |
| M4 Bob's wallet | 2026-10-21 | Second testnet wallet funded after h0, used for act 4 |
| M5 Proof and docs | 2026-10-24 | `LEAKS.md`, `SPEC.md`, README, `evidence/` complete |
| M6 Demo and audit | 2026-10-26 | Video recorded; pre-submit audit passes from a clean clone |
| Submit | 2026-10-27 08:00 UTC | Last commit before submission; nothing after |

## 13. Definition of finished
A judge clones the repo, runs one script against the recorded testnet heights, and sees: Alice admitted, reuse refused, newcomer refused, second gate admitted, tampered proof refused, with every verdict reproducible and every claim in the README backed by a file in `evidence/`.

## 14. Submission field mapping
Fill in from the site's submission form when visible. Default answers: track Core & Tooling; one-line pitch = the hook; demo link; repo link; "what is new" = private proof of holding duration via split-height roots on Valar's delegation circuit (credited).

## 15. Judge Q&A
- **Isn't this just Valar's voting proof?** Same audited circuit, different statement: we pin the two roots to different heights to prove holding duration, add context-scoped single use, spend-key binding, and a verifier that authenticates its own roots. Valar's docs require one height; our soundness argument is in `SPEC.md` and tested in A03, A04.
- **Can someone with my viewing key impersonate me?** No: A06.
- **Who do I have to trust?** The circuit (audited), two independent servers for the H0 root (or your own Zebra), your own nf-server for H1. Nothing from the prover.
- **Is it private?** See `LEAKS.md`: every channel, each with a test or a stated non-defence.
- **Why testnet?** NU7 activates on mainnet during judging; testnet is already on NU7 and stays valid.

## 16. Distribution and judge access (added 2026-10-15)
- **Storage:** the gate creates `gate.sqlite` next to itself on first run. Judges never handle it. Deleting it resets the gate.
- **Judge path 1, the demo:** the video, and the recorded acts any locally run gate serves at `GET /room/<context>?replay=<act>` with no chain, no wallet and no nf-server. No hosted room (DECISIONS 2026-10-22).
- **Judge path 2, local:** `unstirred gate serve`.
- **Judge path 3, no database:** `unstirred verify <package.json> --manifest <gate.json>` against recorded evidence.
- **Transparency:** `unstirred gate markers` prints everything the gate stores (markers, verdicts, timestamps; nothing else).
- **Distribution:** GitHub Releases with prebuilt Linux and macOS binaries; crates.io 0.1.0 published once around 2026-10-26 from the final tagged commit, before the deadline. CLI package renamed `unstirred`. Not npm.
