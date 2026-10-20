# HANDOFF

State of Unstirred at commit `b9374b0` — M1 closed, and M2 steps 1 and 2 built, gated and
committed. The one thing M2 still owes is a real two-source run, which waits on the owner's Zebra
node (§2). Written 2026-10-20, revised from the 2026-10-17 version. For the next session (agent
or human).
Read `briefing/CLAUDE.md` and `briefing/BUILD_CONTRACT.md` first; this file only says where the
work stopped, what is next, and what the machine around the repo is holding.

## 1. Where the build is

Milestone M1 ("first real proof", target 2026-10-17) is **closed and committed**: all five steps
are built, and `unstirred verify` admitted the first real package in a fresh process against the
real testnet roots — recorded in `evidence/2026-10-17-verify-first-proof.txt` (A01's real run),
and the owner re-ran the same command and confirmed ADMITTED.

| Step | What it added | Real-testnet evidence |
|---|---|---|
| M1 step 1 (`ce7c213`) | read-only wallet, held-note set, selection, `unstirred notes` | `evidence/2026-10-16-notes-real-testnet.txt` |
| M1 step 2 (`0573f04`, `cf3e189`) | h0 tree state over gRPC, historical witnesses, `unstirred witness` | `evidence/2026-10-16-witness-real-testnet.txt` |
| M1 step 3 (`825c3e5`, `c15e092`) | PIR non-membership at h1, `unstirred lookup` | `evidence/2026-10-16-lookup-real-testnet.txt` |
| M1 step 4 (`4c975c4`, `118564f`) | proof package schema, the identity and spending-key path, bundle, prove, sign, `unstirred prove`; first real proof | `evidence/2026-10-17-first-proof.json` |
| M1 step 5 (`8c52f45`) | `unstirred-verifier`: pure engine over injected roots (PRD §6 steps 1-8), one-server root read, `unstirred verify` with `--json` | `evidence/2026-10-17-verify-first-proof.txt` (synthetic tests only in CI; the real admission is the recorded run) |

Step 5's additions: `crates/unstirred-verifier/src/{engine,roots}/`,
`crates/unstirred-verifier/tests/verify.rs`, `crates/unstirred-verifier/tests/verify/`,
`crates/unstirred-cli/src/verify/`, and the synthetic harness moved from
`crates/unstirred-prover/tests/harness/` to `crates/unstirred-prover/src/testing/` behind a
`test-harness` cargo feature.

**M2 step 1 (two-source root authority)** is built and committed (`0c5cc1f`, gate log
`evidence/2026-10-17-gate-m2-step1.md`): `unstirred verify` and the new
`unstirred roots` take repeatable `--root-source lwd:<url>|zebra:<url>` (exactly two required) or
one source behind the explicit `--single-root-source`, whose human output and JSON verdict both
carry "ROOT FROM ONE SOURCE: not independently checked". The lightwalletd read moved behind a
`RootSource` trait; a Zebra JSON-RPC source (`z_gettreestate`, `getblockchaininfo`,
`getblockcount`, shape from zebra-rpc 7.0.0-rc.0) is the second kind. The sources must agree on
exactly one h0 root or the run is INCONCLUSIVE (`ROOT_SOURCES_DISAGREE`, `ROOT_SOURCE_UNAVAILABLE`);
the tip is the lower of the answers. The h1 root stays operator-pinned and labelled as such
(nf-server `verify-root` is deferred to P1). Additions: `crates/unstirred-verifier/src/roots/`
(now a module: `mod.rs` authority, `lightwalletd.rs`, `zebra.rs` with the parser's unit tests),
`crates/unstirred-verifier/tests/roots.rs`, `unstirred_verifier::verdict_object`, and the CLI
gained `crates/unstirred-cli/src/{roots,root_sources}.rs`. A10 got its positive and refusal tests
and new IDs A10b and A27 are registered in PRD §10 with `DECISIONS.md` rows.

**M2 step 2 (challenge and marker store)** is built and committed (`2639fdd`, then `b9374b0`,
which adds nothing but the session summary now at `evidence/2026-10-17-two-rooms-summary.md`;
gate log `evidence/2026-10-17-gate-m2-step2.md`, `GATE: PARTIAL`, exit 0). One SQLite file,
created on first open, WAL, 10s busy timeout on that connection. Tables are hashed challenges,
markers (`UNIQUE(context, marker)`) and admission verdicts. The raw challenge is never stored.
Challenges expire after 10 minutes (`expires_at > now`); the clock is an injected trait.
`unstirred gate challenge` prints the hex of a new challenge. `unstirred admit` verifies, then
in one immediate transaction inserts the five markers, consumes the challenge and records
ADMITTED. A marker conflict rolls that transaction back (`ALREADY_USED_IN_CONTEXT`) and the
attempt consumes nothing, which is also what a replay of an already-admitted package reports.
A consumed challenge presented with new markers fails the consume inside the transaction and is
`CHALLENGE_UNKNOWN_OR_EXPIRED`, rolled back. Unknown, expired and wrong-context challenges are
refused before the proof, and an invalid proof writes nothing. `unstirred gate markers` prints
markers and admission verdicts, not challenge hashes. `unstirred verify` is still the
no-database path and still prints "markers not recorded (M2)" on ADMITTED. A one-source run's
agreement line is "not checked (one source)" in the human output and in the roots JSON.
Additions: `crates/unstirred-verifier/src/admit.rs`, `crates/unstirred-verifier/src/store/`,
`crates/unstirred-verifier/src/verdict.rs`, `crates/unstirred-verifier/tests/store.rs` with
`crates/unstirred-verifier/tests/store/`, `crates/unstirred-cli/src/admit/`,
`crates/unstirred-cli/src/gate.rs`. A02b, A05, A05c, A05d,
A07, A08 and the A11 no-write check pass on synthetic packages. The order is the `DECISIONS.md`
row that refines PRD §6. The same sequence on the real testnet wallet is
`evidence/2026-10-17-admit-two-rooms.txt`: holders-room ADMITTED, the same package
ALREADY_USED_IN_CONTEXT, second-room ADMITTED with a different marker set. One lightwalletd
source. The wallet file's sha256 did not change.

`unstirred verify` still runs PRD §6 steps 1-8 with nothing written: manifest, window against
the tip the command fetches, `round_id` against the instance, instance roots against the pinned
ones, challenge equality against the challenge on the command line, the binding signature, the
Halo2 proof. Roots come from the M2 step 1 authority. Every root-source failure is INCONCLUSIVE.
A25 and A26 stay registered for that path.

`unstirred-gate` HTTP is still a one-line doc stub. `unstirred gate init` is not built (M3).

Last gate run: `evidence/2026-10-17-gate-m2-step2.md` — `GATE: PARTIAL`, exit 0; fmt and clippy
clean, all tests pass (the step 1 set plus A02b, A05, A05c, A05d, A07, A08 and the A11 no-write
check, with A01 still closed by `evidence/2026-10-17-verify-first-proof.txt`). Open: A02's
repo/log scanner as a gate step, A12 (marker stability when a gate is republished with a new
window), and every live HTTP path in `unstirred-gate`.

**Cleanup pass, 2026-10-20 (uncommitted, nothing behavioural):** every source file over ~180
lines was split into modules named for what they hold, so several paths this file names are now
directories. `unstirred-core`: `codes/{mod,spec}.rs`, `package/{mod,instance}.rs`.
`unstirred-prover`: `pir/{mod,lookup,oracle}.rs`. `unstirred-verifier`: `engine/{mod,checks}.rs`,
`roots/{mod,agreement,tree_state}.rs`, `roots/zebra/{mod,jsonrpc,treestate}.rs` with the parser's
unit tests in `roots/zebra/treestate/tests.rs`, `store/{mod,challenge,clock,commit,query}.rs`.
`unstirred-cli`: `admit/`, `prove/`, `roots/` and `verify/`, each the command plus its `report.rs`.
Tests: the store's proof tests are now
`tests/store/{same_coins,challenge_rules,two_contexts,dump,assertions}.rs`, and `tests/roots/`,
`tests/verify/`, `tests/prove/` and `tests/pir/` hold their fixtures and helpers. No test body,
assertion, acceptance ID or public name changed; the only visibility changes are `pub(super)` and
`pub` where a split moved an item away from its caller. `crates/unstirred-prover/src/error.rs`
(194 lines) is the one file still over the limit: it is a single `enum ProverError`, and one enum
cannot be spread over two files without changing the public error type.

**Quality pass, 2026-10-20 (uncommitted, nothing behavioural):** run against the code bar in
`research-and-coding-skill-guide`. The adversarial tests moved into `crates/*/tests/adversarial/`
as CLAUDE.md always required (package tampering, a foreign gate, replayed coins, challenge misuse,
hostile root sources, a foreign signer, the PIR lookup under attack, a forged frontier, keys that
are not the account's), with `#[path = "adversarial"]` on each target's wrapper module so rustfmt
and rustc agree (see `briefing/BUILD_NOTES.md`). Five property, oracle and determinism tests were
added as A28-A32, registered in PRD §10 and `DECISIONS.md` first, seeded by case number with no
new dependency. `scripts/gate.sh` now also runs `scripts/leak-scan.sh` (A02's repository half) and
`scripts/claims-check.sh` against the new `CLAIMS.md` ledger, and reports **GATE: PASS** instead of
PARTIAL. CI runs the same gate on push and pull request (`.github/workflows/gate.yml`, actions
pinned by SHA), `rust-toolchain.toml` pins 1.91, and `Cargo.toml` now carries the real repository
URL. The two narrating comments in `crates/unstirred-verifier/src/admit.rs` became a named
function. The README rebuild (a claims table and a 60-second judge path) is deliberately left
until after M3, at the owner's instruction.

## 2. What to do next, precisely

M2 steps 1 and 2 (the verifier's root authority; the store and `admit`) are committed. Two agreeing sources
behind `--root-source` (two required, or one with `--single-root-source`), `ROOT_SOURCES_DISAGREE`
/ `ROOT_SOURCE_UNAVAILABLE` otherwise, h1 still operator-pinned and labelled, nf-server
`verify-root` deferred (P1, see §6). No second public lightwalletd is reachable (Tatum's gateway
has no z_gettreestate, testnet.lightwalletd.com no longer resolves, the Logos onion times out over
Tor), so the second source is the owner's own Zebra node. On 2026-10-17 it was still syncing:
`getblockcount` at `http://127.0.0.1:18232` climbed through 1,675,600 and 1,791,200 and was
1,836,400 at the start of the live admit, all short of h0 4,505,790, so that admit used
`--single-root-source`. Remaining:

1. A real two-source run recorded in `evidence/` once the Zebra node has synced past h1 +
   finality (`unstirred roots`, then `unstirred verify` with lwd + zebra; commands in §4). This
   is the last open item in M2, and nothing in the repo can close it: it waits on the node.

M3, then: `unstirred-gate` (`/manifest`, `/challenge`, `/admit`, `gate markers`), exposing the M2
challenge store over HTTP, and the Holders' Room page. The session key stays a fresh random 32
bytes with no private half for v1. The judge path decision (hosted Holders' Room) is due by M3,
2026-10-21.

The standing rule holds: register any new acceptance IDs in `briefing/PRD.md` §10 **before**
writing their tests, with a `DECISIONS.md` row.

## 3. The machine around the repo (not in git)

Paths are absolute because none of this is inside the repo.

- **Holder's wallet (read-only, real testnet):**
  `/home/kingghidorah/Documents/projects/bounties/zecathon/wallet/data.sqlite`
  account `56099e58-aab0-4f60-9c38-b84ae7211c2d`, scanned to 4,506,611.
  Two held Ironwood notes at h0; the selection takes position 552213 (1 ZEC).
  **Never write to it.** Nothing in this repo may create a transaction from it.
- **Gate manifest used for every run so far:**
  `/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json`
  — `test` / `holders-room` / h0 4,505,790 / h1 4,506,510 / 12,500,000 zatoshis.
- **lightwalletd:** `https://testnet.zec.rocks:443`. The second root source is the owner's Zebra
  node, `http://127.0.0.1:18232` (§2, still syncing on 2026-10-17).
- **nf-server:** built from `vote-nullifier-pir` tag `v0.12.3` (`ac15ed2`) — *not* HEAD, see
  `DECISIONS.md`. Needed only for `lookup` and `prove`, not for `verify`. It was running during
  the step 3 and step 4 runs as:
  ```
  cd /home/kingghidorah/Documents/projects/bounties/zecathon/vote-nullifier-pir/nf-server
  ../target/release/nf-server serve --zcash-network test \
      --pir-data-dir ../pir-data/test-v0123 --port 3000 --pir-poly-len 4096
  ```
  Dataset `pir-data/test-v0123`: ironwood, dataset version 2, height 4,506,510, 276,626 ranges,
  depth 19 (12+7), poly_len 4096. Its `circuit_root` is the expected h1 root
  `8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26`.
- **The holder's age identity:** `/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age`,
  an age identity file (one `AGE-SECRET-KEY-…` line). It is what `--identity` takes, and it is the
  key that decrypts the `mnemonic` field of `wallet/keys.toml`. **Its secret-key line was printed
  into the 2026-10-17 session's transcript** while identifying the file's format, so treat that
  identity as exposed and rotate it (re-encrypt `keys.toml` to a fresh identity) before this
  wallet holds anything that matters.

## 4. Reproducing the recorded runs

From a release build (`cargo build --release`), with `M=/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json`
and `W=/home/kingghidorah/Documents/projects/bounties/zecathon/wallet`:

```
./target/release/unstirred notes   --wallet "$W" --manifest "$M"
./target/release/unstirred witness --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443
./target/release/unstirred lookup  --wallet "$W" --manifest "$M" --pir http://localhost:3000 \
    --expected-h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
```

```
./target/release/unstirred prove   --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://localhost:3000 \
    --identity /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age \
    --challenge $(openssl rand -hex 32) --out ./package.json
```

Recorded 2026-10-17 as `evidence/2026-10-17-first-proof.json`. `lookup` and `prove` need the
nf-server above running. `prove` refuses to overwrite an existing `--out` file.

**The run that closed M1** — accepting that proof in a fresh process — is recorded in
`evidence/2026-10-17-verify-first-proof.txt`. It was made at commit `8c52f45` with the then-only
syntax `--lightwalletd https://testnet.zec.rocks:443`; since M2 step 1 that flag is `--root-source`.
To reproduce against one source today (output carries "ROOT FROM ONE SOURCE: not independently
checked"):

```
./target/release/unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest /home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

Without `--single-root-source`, exactly two `--root-source` values are required and must agree on
the h0 root; the tip is the lower of the two answers. Once the Zebra node (§2) has synced past
h1 + finality, the two-source form is:

```
./target/release/unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest /home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 \
    --root-source zebra:http://127.0.0.1:18232 \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

`unstirred roots` takes the same `--manifest` + `--root-source` flags with no package and prints
each source's h0 root, tip, and the agreement. A one-source run says
`agreement: not checked (one source)` in the human output and
`"agreement":"not checked (one source)"` in `--json`. It does not say the sources agreed.

**Admitting into the gate database** (M2 step 2). `verify` still writes nothing.
`admit` uses the challenge inside the package, so issue one, prove with it, then admit. With
Zebra still behind h0, pass `--single-root-source`. The second room below is the holders-room
manifest with `"context":"second-room"` and the same window; it is not a file in the repo.

```
DB=/tmp/unstirred-gate.sqlite
M=/home/kingghidorah/Documents/projects/bounties/zecathon/holders-room.json
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26
CHAL=$(./target/release/unstirred gate challenge --db "$DB" --manifest "$M")
./target/release/unstirred prove --wallet "$W" --manifest "$M" \
    --lightwalletd https://testnet.zec.rocks:443 --pir http://localhost:3000 \
    --identity /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age \
    --challenge "$CHAL" --expected-h1-root "$H1" --out /tmp/unstirred-holders.json
./target/release/unstirred admit --db "$DB" --package /tmp/unstirred-holders.json \
    --manifest "$M" --root-source lwd:https://testnet.zec.rocks:443 \
    --single-root-source --h1-root "$H1"
```

Admitting that same package again is exit 1, `ALREADY_USED_IN_CONTEXT`. A second manifest with
a different context, a new challenge and a new proof is a second `ADMITTED`, with different
markers. `unstirred gate markers --db "$DB"` prints both contexts and does not print challenge
hashes. That sequence, including the exits, is recorded in
`evidence/2026-10-17-admit-two-rooms.txt`. Zebra's `getblockcount` at the start of it was
1,836,400.

Recorded result: `ADMITTED`, exit 0, with the h0 root from GetTreeState(4,505,790) equal to
`a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` and the chain tip well past
h1 + finality.

`unstirred codes` lists the closed code set.

## 5. Traps that already cost time

`briefing/BUILD_NOTES.md` is the full list and should be read before touching the PIR, witness,
proving or verifying paths. The ones most likely to bite the next step:

- The synthetic harness moved into the prover library behind the `test-harness` cargo feature.
  Plain `cargo test -p unstirred-prover` no longer compiles the integration tests; use
  `--features test-harness`. `scripts/gate.sh` runs clippy and tests with `--all-features`.
- `voting-circuits 0.12.2` cannot verify an IMT proof out of circuit (`protocol_hash` private,
  no `ImtProofData::verify`); out-of-circuit checking goes through `imt-tree` via `pir-client`.
- The PIR stack must stay on the `v0.12.3` cohort (pasta_curves 0.5 / orchard 0.15 / halo2 0.3).
  HEAD moved to Zakura Common 3.0.0 and its field types cannot meet `voting-circuits 0.12.2`.
- A spent nullifier surfaces two different ways depending on parity: either `fetch_proof` errors
  or the proof verifies false. Both mean spent.
- `orchard::note::Nullifier::inner()` is only public under a feature another crate happens to
  enable; use `to_bytes()` + `Fp::from_repr`.
- Two concurrent `cargo test` runs make `gate.sh` look hung at `== clippy`. Kill stray cargo first.
- The delegation builder asks the IMT provider for proofs for its own *padding* nullifiers, so the
  PIR oracle has to stay live during `build_delegation_bundle` (one note selected = four extra
  private queries per proof).
- `age 0.12.1` brings `secrecy 0.10.3` alongside the zcash cohort's `secrecy 0.8.0`: read age's own
  secrets through `age::secrecy`, ours through `secrecy`.
- The `zcash_client_backend` test harness account comes from 32 zero bytes, not a mnemonic, so a
  synthetic wallet cannot be driven through the age path; `tests/prove.rs` uses
  `spend_authority_for` with the harness's own key, and `tests/identity.rs` covers the age path
  against the published BIP 39 vector.
- One real K=12 proof costs about 70 s in `tests/prove.rs` and about 95 s across the verifier's
  seven tests (one shared proof). All tests in a file share one proof on purpose.
- `redpallas::Signature` converts to `[u8; 64]` by reference only; `SpendAuthorizingKey::randomize`
  takes `&pallas::Scalar`.

## 6. Open items

- `scripts/gate.sh` prints `GATE: PASS` since 2026-10-20: fmt, clippy, the whole suite, the leak
  scan and the claims check. What it still cannot see is a leak through a channel rather than a
  string in a running process, and whether a file in `evidence/` says what `CLAIMS.md` says it
  says. A02's package and database scans remain in-process tests.
- `unstirred-gate`: M3. M2 builds the challenge store in `unstirred-verifier` (issue, single-use
  consume with expiry, hashed); M3 only exposes it over HTTP. The session key stays a fresh
  random 32 bytes with no private half for v1.
- h1 via nf-server `verify-root` is deferred (P1): it needs a full node serving raw blocks back to
  Ironwood activation plus an independently trusted block hash at h1. Until then h1 stays
  operator-pinned via `--h1-root` and labelled operator-pinned in output and docs.
- `DECISIONS.md` has one `open` row: whether to reserve the crates.io names now with a 0.0.1
  placeholder. Owner's call. The planned publish is a single 0.1.0 around 2026-10-26.
- `crates/unstirred-prover/src/error.rs` is 194 lines, over the ~180-line rule, because it is one
  enum (see the cleanup note in §1). `DECISIONS.md` rows written before 2026-10-20 still cite the
  pre-split file paths; they are a record of decisions, so they were left as written.
- The README is still dense prose rather than a claims table and a 60-second judge path. `CLAIMS.md`
  now holds the ledger it should be built from. Owner's instruction: rebuild it after M3.
- Second root source for the verifier's two-source rule: no public second lightwalletd is
  reachable (Tatum's gateway has no z_gettreestate, testnet.lightwalletd.com no longer resolves,
  the Logos onion times out over Tor), so it is the owner's Zebra node (§2), still syncing.
- Judge path 1 (hosted Holders' Room) has to be decided by M3, 2026-10-21.
- Stated limit worth re-reading before any claim about key hygiene: `orchard 0.15.5` does not
  zeroize its key types on drop (`briefing/BUILD_NOTES.md`, `LEAKS.md` L15b).

## 7. Rules that are easy to break under time pressure

- Never show a placeholder as real; show `—`. No number in README, demo or submission without a
  file in `evidence/` naming network, h0, h1, commit, command and date.
- Unknown is never success. Root trouble is INCONCLUSIVE, never ADMITTED.
- A frozen decision in `briefing/BUILD_CONTRACT.md` changes only after a `DECISIONS.md` row.
- Run `./scripts/gate.sh` before claiming anything works, and record the run.
- Deadline: last commit before **2026-10-27 08:00 UTC**, nothing after — not even docs.
