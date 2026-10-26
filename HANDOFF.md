# HANDOFF

State of Unstirred at commit `d2dfc12`, with the demo video built on top of it and uncommitted. **Act 4's visual fix, the docs and the
release prep are committed** (§11: the broken axis, `SPEC.md`, the rebuilt README, LEAKS
finalised, the release workflow and crates.io readiness). **M1, M2, M3 and M4 are closed** (M2's last open item, a real two-source root run, was recorded
on 2026-10-20, §1; M3 step 1 is `6c2a288` and `34c3146`, §8; M3 step 2, the Holders' Room page, is
`bf3fe87` with the browser review's fixes in `e02230d`, §9). **M4, the demo's act 4, is built and
recorded** (§10): the newcomer's wallet was refused by the room's own rule on 2026-10-22 with
nothing sent, and its forced submission was refused by the gate itself, REFUSED / ROOT_MISMATCH,
writing nothing — both against the real testnet, and `act4-newcomer` is generated from them. M4 is
committed, `2e24559` and `527f871`. **The demo video is done** (§12): the 2:00 narrated walkthrough
is rendered and delivered at `video/out/unstirred-demo.mp4` — 120.000000 s, 3,600 frames,
1920x1080, h264 High + aac LC, 16,218,341 bytes, sha256
`9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399` — with narration, captions, a
ducked music bed, twelve stills cut out of the delivered file, and `npm run check` passing every
pass against the delivered tree. Its generators are in `video/tools/` and reproduce `index.html`
byte for byte from any working directory. Nothing about the video is open.
**What is next: the final audit, parts A and B. That is the only thing left before the release.**
Written 2026-10-20, revised 2026-10-21, 2026-10-22, 2026-10-23 and 2026-10-26,
from the 2026-10-17 version. For the next session (agent or human).
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
no-database path and says so on ADMITTED ("verify does not record markers; use unstirred admit to
enter a gate", changed from the M2-era wording on 2026-10-20). A one-source run's
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

`unstirred-gate` is now the HTTP service and `unstirred gate init` and `unstirred knock` are built;
that is M3 step 1, recorded in section 8 below. It also serves the Holders' Room page itself, which
is M3 step 2, recorded in section 9.

Last gate run of M2's own work: `evidence/2026-10-17-gate-m2-step2.md` — `GATE: PARTIAL`, exit 0;
fmt and clippy clean, all tests pass (the step 1 set plus A02b, A05, A05c, A05d, A07, A08 and the
A11 no-write check, with A01 still closed by `evidence/2026-10-17-verify-first-proof.txt`). Its
open items are closed since: A02's repository scanner is a gate step
(`evidence/2026-10-20-gate-quality-pass.md`) and the live HTTP paths are M3 step 1
(`evidence/2026-10-20-gate-m3-step1.md`). A12 (marker stability when a gate is republished with a
new window) is still only covered by the `round_id` rule in core, not by a gate run.

**Cleanup pass, 2026-10-20 (committed in `8398181`, nothing behavioural):** every source file over ~180
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

**Quality pass, 2026-10-20 (committed in `8398181`, nothing behavioural):** run against the code bar in
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
function. Afterwards, `unstirred verify`'s ADMITTED line stopped naming a milestone
("verify does not record markers; use unstirred admit to enter a gate", one constant for both
outputs) and A33 was registered to keep it from going stale again; the run that covers all of it
is `evidence/2026-10-20-gate-quality-pass.md` (76 tests, GATE: PASS). The README rebuild (a claims
table and a 60-second judge path) is deliberately left until after M3, at the owner's
instruction; its per-crate test lists name A28-A33 in the meantime.

Two commits carry all of this, and the first of them is mislabelled. `8398181`, subject
"Evidence: two-source root agreement and verify (zec.rocks + own Zebra node)", contains the whole
quality pass as well as that evidence; its subject describes a part of it. It was already pushed
when the mismatch was noticed, so it stands as written rather than being rewritten, and the
`DECISIONS.md` row dated 2026-10-20 records what is actually in it. The M2 wrap-up — the claims
rows for the two-source run, the `verify` wording constant and A33, the README test IDs and
`evidence/2026-10-20-gate-quality-pass.md` — was rebased on top as `15bd859`. `master` tracks
`origin/master` and both are pushed, so look for this work under two subjects, not one.

## 2. What to do next, precisely

**The build is finished.** Everything is committed and green at `c2751fd` (GATE: PASS, 132 tests,
`evidence/2026-10-26-gate-final-audit.md`), the site is published and live, and **everything left
is the owner's to do by hand**. Nothing in this repository is waiting on more code.

**Done, and closed:** the five milestones, the demo video (§12), the final audit parts A (§13) and
B (§14), the clean-clone audit of both README paths
(`evidence/2026-10-26-clean-clone-audit.md`), and the published site —
**https://drapkfile.github.io/Unstirred/**, deployed by `.github/workflows/pages.yml` on the push
of `c2751fd`, with the Pages source switched to "GitHub Actions" and the social preview uploaded.
Every page of it answers 200: the landing page, the five acts in both readings, the picker, the
three guides and the API reference.

**What is left, in order, all of it the owner's:**

1. **`docs/RELEASING.md`, top to bottom.** Tag `v0.1.0` and push it (this is what makes the
   download links in README and both guides resolve — they already name `v0.1.0`, not the earlier
   `v0.1.0-rc.1`); wait for the three release build jobs; paste the release notes, which are
   written out in full in that file; `gh release upload v0.1.0 video/out/unstirred-demo.mp4
   --clobber` — or drag the file onto the release page, since `gh` is not installed on the build
   machine; then `cargo publish` in dependency order: **core, prover, verifier, gate, unstirred**.
2. **The video link.** Once the release carries the `.mp4`, put its URL into README's
   "Watch the demo" placeholder, which carries an HTML comment saying it is the only place in the
   repository that needs it. Commit and push; the site rebuilds itself.
3. **A 1:30 re-cut of the walkthrough, with the real links in it** — the published site and the
   release — replacing the 2:00 cut that had neither. `video/` is the project, `video/STORYBOARD.md`
   its plan, and `video/tools/` its generators, which reproduce `index.html` byte for byte.

The historical record of how it got here is below, oldest last. Deadline: last commit before
**2026-10-27 08:00 UTC**.

### How it got here

**Everything built so far is committed and green** (`3d2bf57`, GATE: PASS, 104 tests,
`evidence/2026-10-23-gate-release-prep.md`). **Next, in order:**

1. ~~**The demo video**~~ — **done, rendered and delivered, 2026-10-26, and uncommitted** (§12).
   `video/` is a HyperFrames project; `video/STORYBOARD.md` is its plan with the evidence file named
   for every scene, and `video/out/unstirred-demo.mp4` is the delivered 2:00 narrated render with
   narration, captions and a ducked music bed. `video/tools/` holds its generators and has been
   verified to reproduce `index.html` byte for byte. Nothing about the video is open; do not
   re-render it.
2. **The final audit, parts A and B — the only thing left before the release.** The audit is the
   pre-submit check in `briefing/PRD.md` §M6 ("pre-submit audit passes from a clean clone") plus the
   GitHub Pages replay site. `scripts/export-replay-site.sh` writes the five acts, each in normal
   and presentation mode, plus the room picker, into `site/` as static HTML, straight out of the
   gate's own page module, so an exported file is byte for byte what the gate serves — re-verified
   2026-10-26 against `video/assets/site/`, all eleven files identical. **Settled in part B
   (§14):** the picker's room cards now open the first recorded act of each room (A51), and the
   site is built by `scripts/build-site.sh` and published by `.github/workflows/pages.yml`, which
   deployed it on the push of `c2751fd`. *(Part A turned out to be the simpler
   knock, the guides, the examples and the rustdoc; part B the publish-ready pass.)*

The release is prepared and not performed: the tag and the crates.io publish are still open, with
the order and the exact commands in `docs/RELEASING.md` (§11 has the older, shorter version).

**M2 is closed. Nothing in it is open.** Both steps are committed, and the two-source rule has
now run for real: on 2026-10-20 the owner's zebrad 7.0.0-rc.0 had synced past the window, and
`unstirred roots` and `unstirred verify` were recorded with lightwalletd and Zebra both reporting
h0 root `a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` at h0 4,505,790, tip
4,521,439 from each, and ADMITTED with no single-source label
(`evidence/2026-10-20-roots-two-source.txt`, `evidence/2026-10-20-verify-two-source.txt`).
Two agreeing sources behind `--root-source` stay the default (two required, or one with
`--single-root-source`), `ROOT_SOURCES_DISAGREE` / `ROOT_SOURCE_UNAVAILABLE` otherwise, h1 still
operator-pinned and labelled, nf-server `verify-root` still deferred (P1, see §6). Note for any
future second source: no public second lightwalletd is reachable (Tatum's gateway has no
z_gettreestate, testnet.lightwalletd.com no longer resolves, the Logos onion times out over Tor),
so the owner's own node is the second source.

**M3 step 1 is built, gated and committed** (`6c2a288`, with the knock evidence in `34c3146`;
section 8): the `unstirred-gate` HTTP service with five routes per room, `unstirred knock`, and
`unstirred gate init`, with A34-A41 green in `evidence/2026-10-20-gate-m3-step1.md` (88 tests,
GATE: PASS). It has also **been run against a gate serving the real testnet wallet**: on
2026-10-20 a knock at `holders-room` was ADMITTED, the same coins there were REFUSED
`ALREADY_USED_IN_CONTEXT`, and `second-room` was ADMITTED, each room's `/log` showing five
markers with none shared between the two, over two agreeing root sources with no single-source
label (`evidence/2026-10-20-two-room-knock.txt`, header in
`evidence/2026-10-20-two-room-knock.meta.md`; the commands are in section 8).

**M3 step 2 is built and gated** (section 9): the gate serves the room picker at `GET /` and one
Holders' Room page per room at `GET /room/<context>`, with `GET /rooms/<context>/recent` beside
them, and renders a recorded act of the demo at `GET /room/<context>?replay=<act>`. A02c, A42, A43
and A44 are green in `evidence/2026-10-21-gate-m3-step2.md`. It is **committed as `bf3fe87`**, and
it has been reviewed in a browser since, and the fixes for that review are committed as `e02230d`
(section 9).

**That closes M3, and M4 — act 4 — is done too: see §10 for all of it.** The newcomer's wallet
holds two Ironwood coins mined at 4,526,839 and 4,526,842, both after h0 4,505,790. On 2026-10-22
the room's own rule refused it `COINS_TOO_NEW` with no challenge requested and nothing sent
(`evidence/2026-10-22-bob-knock.txt`), and `knock --force-unqualified-demo` built a real proof
against the latest note-commitment root one of those coins appears in and posted it, which the gate
refused `ROOT_MISMATCH` while writing nothing
(`evidence/2026-10-22-bob-forced-knock.txt`). `act4-newcomer` is generated from those two runs and
is the fifth act the gate serves. **M4 is committed, `2e24559` and `527f871`, with this file and
`briefing/BUILD_NOTES.md` brought up to date in `3f79c4e`.** **Next, in order:**

All three are **done and committed as `3d2bf57`** — §11 has the detail:

1. **Act 4's visual fix** — done. The newcomer is no longer a faded stub at the end of the axis:
   the rail is broken after h1 and the coins are marked past the break with the distance on the
   mark. Twenty-eight page variants measured again, nothing hidden.
2. **Docs** — done. `SPEC.md` is new; the README is rebuilt around the 60-second judge path with a
   status table generated from `CLAIMS.md`; `briefing/LEAKS.md` has no "planned" row left.
3. **Release** — prepared, not performed. The tag-triggered workflow is written, the crates carry
   their crates.io metadata, and the publish dry run passes for the whole workspace. **Nothing is
   committed, tagged or published**, which is §11's last part and the owner's to do. Deadline:
   last commit before **2026-10-27 08:00 UTC**.

What is left of the demo script is act 7 (the chain itself), which has no run behind it. The
session key stays a fresh random 32 bytes with no private half for v1.

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
  node, `http://127.0.0.1:18232` (zebrad 7.0.0-rc.0, no cookie auth). It was still syncing on
  2026-10-17; by 2026-10-20 it had reached tip 4,521,439, past h1 + finality, and the two-source
  run in §1 used it.
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
the h0 root; the tip is the lower of the two answers. That is the form that closed M2, recorded as
`evidence/2026-10-20-roots-two-source.txt` and `evidence/2026-10-20-verify-two-source.txt`; the
verify run there was made at commit `b9374b0`, whose last line still read "markers not recorded
(M2)" before that wording changed on 2026-10-20:

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

- `scripts/gate.sh` prints `GATE: PASS` since 2026-10-20: fmt, clippy, the whole suite, the API
  reference with no rustdoc warning (added in part A, 2026-10-26), the leak scan, the claims check
  and the README status table against `CLAIMS.md`. What it still cannot see is a leak through a
  channel rather than a string in a running process, and whether a file in `evidence/` says what
  `CLAIMS.md` says it says. A02's package and database scans remain in-process tests.
- `unstirred-gate` HTTP is built and committed (M3 step 1, section 8) and exposes the M2 store
  without adding a check of its own. Its real two-room run was made on 2026-10-20 — ADMITTED,
  `ALREADY_USED_IN_CONTEXT` for the same coins at that room, ADMITTED at the second room, with
  disjoint marker sets (`evidence/2026-10-20-two-room-knock.txt`). The session key stays a fresh
  random 32 bytes with no private half for v1.
- The Holders' Room page **has** been opened in a browser now, and reviewed: the owner's review of
  2026-10-21 and the fixes for it are in §9, and `evidence/2026-10-21-page-layout-check.md` records
  a headless Chromium measuring all twelve page variants at 1920×1080 and at 1326×683 with nothing
  clipped and nothing hidden. What is still unverified: any browser but that one Chromium build,
  any viewport but those two (the stage is fixed and scaled, so the layout is the same at every
  size, but an engine that lays text out differently could still differ), and whether the result
  reads well to anyone who has not been staring at it.
- The demo script's act 7 (the chain itself) has no replay file, because no run has produced its
  numbers. Act 4 now has one: both of its halves were recorded on 2026-10-22 (§10) and
  `scripts/replay-from-evidence.py` wrote `act4-newcomer` from them.
- h1 via nf-server `verify-root` is deferred (P1): it needs a full node serving raw blocks back to
  Ironwood activation plus an independently trusted block hash at h1. Until then h1 stays
  operator-pinned via `--h1-root` and labelled operator-pinned in output and docs.
- `DECISIONS.md` has one `open` row: whether to reserve the crates.io names now with a 0.0.1
  placeholder. Owner's call. The planned publish is a single 0.1.0, and `docs/RELEASING.md` has the
  order, the exact commands and the release-notes text; the dry run for it passes on the whole
  workspace.
- The ~180-line file rule keeps **ten recorded exceptions**, listed with their line counts and the
  reason for each in the `DECISIONS.md` row of 2026-10-26; `crates/unstirred-prover/src/error.rs`
  (377, one enum of 57 documented variants) is the largest of the source files among them. Every
  other tracked Rust file is at or under 195 lines, which that row counts as inside `~180`.
  `DECISIONS.md` rows written before 2026-10-20 still cite the pre-split file paths; they are a
  record of decisions, so they were left as written.
- Two page files are also over the ~180-line rule, for the same kind of reason.
  `crates/unstirred-gate/ui/render.js` (470) is one pure function and its private helpers in one
  closure; splitting it would mean a namespace object quoted on every call, which is worse to read
  than the length it saves. `ui/tokens.css` (441) is the one `:root` token block the video's design
  system mirrors, plus the styles that use it, and the only way CSS has to split a file is
  `@import`, which is a third-party request waiting to happen and is what LEAKS L10 forbids.
- The README is rebuilt (§11): a 60-second judge path, and a status table generated from
  `CLAIMS.md` by `scripts/status-table.py`, which the gate re-runs with `--check`. `SPEC.md` is
  the protocol document beside it, and `briefing/LEAKS.md` has no "planned" row left.
- Judge path 1 is decided: **no hosted Holders' Room.** There is now a published site
  (`.github/workflows/pages.yml`, §14), but it is static: the five recorded acts, the guides and the
  API reference, with no gate, no chain, no wallet and no nf-server behind any of it. The paths are
  that site, the video, the replay acts a gate the judge runs serves, and `unstirred verify` against
  the recorded evidence (owner's decision, `DECISIONS.md` 2026-10-22, PRD §16 rewritten to match).
  Nothing has to be deployed or kept running after the deadline.
- Stated limit worth re-reading before any claim about key hygiene: `orchard 0.15.5` does not
  zeroize its key types on drop (`briefing/BUILD_NOTES.md`, `LEAKS.md` L15b).

## 7. Rules that are easy to break under time pressure

- Never show a placeholder as real; show `—`. No number in README, demo or submission without a
  file in `evidence/` naming network, h0, h1, commit, command and date.
- Unknown is never success. Root trouble is INCONCLUSIVE, never ADMITTED.
- A frozen decision in `briefing/BUILD_CONTRACT.md` changes only after a `DECISIONS.md` row.
- Run `./scripts/gate.sh` before claiming anything works, and record the run. **It wants about
  7 GB free**: the proof targets build and run in parallel, and on 2026-10-23 a run on this
  machine was killed for memory part-way through `tests/http.rs`. Stop zebrad, the nf-server and
  any running gate first, and if memory is still short use
  `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh`, which asserts exactly the same
  things and takes longer (`tests/http.rs` 140.89s serialised against 111.43s parallel).
- Deadline: last commit before **2026-10-27 08:00 UTC**, nothing after — not even docs.

## 8. M3 step 1, built and gated 2026-10-20, committed as `6c2a288` and `34c3146`

`./scripts/gate.sh` reports **GATE: PASS**, exit 0, with 88 tests and the leak and claims checks:
`evidence/2026-10-20-gate-m3-step1.md`. A34-A41 are green, the twelve new tests are listed in
that file, and the CLAIMS, README and LEAKS entries for them are written. The code and those
entries are **committed as `6c2a288`**, and the step's HTTP paths have been run against a gate
serving the holder's real wallet: the two-room knock below was made on 2026-10-20, recorded in
`evidence/2026-10-20-two-room-knock.txt` (committed as `34c3146`) with its header in
`evidence/2026-10-20-two-room-knock.meta.md`.

What was built:

- **`unstirred-gate` is now the service** (`src/{config,roots,state,router,reply,events,rfc3339,
  serve,handlers,client}`): `GET /rooms`, `GET /rooms/<context>/manifest`,
  `POST /rooms/<context>/challenge`, `POST /rooms/<context>/admit`, `GET /rooms/<context>/log`.
  hyper 1.12 and hyper-util 0.1.21 directly, no axum, no middleware (DECISIONS 2026-10-20). The
  admit route calls the same `unstirred_verifier::admit` the CLI calls; the gate adds no check.
  Body limit `MAX_ADMIT_BODY_BYTES` = 256 KiB. ADMITTED 200, REFUSED 403, INCONCLUSIVE 503;
  404/405/413/400 carry `{"error":…}` and no code from the closed set.
- **Privacy:** the peer address is dropped at `accept`, one log line per request
  (`method path -> status CODE`), plus a root-provenance line per admission; no body, no raw
  challenge, no marker in the log. The single-source label travels in every body of ours and in
  the `unstirred-root-note` header (the manifest body cannot carry extra fields).
- **`unstirred gate serve --config <file>`** (TOML; `unstirred gate serve` with no flags prints a
  full sample config), **`unstirred gate init --network --context --h0 --h1 --out`**, and
  **`unstirred knock --gate --room --wallet --pir --lightwalletd --identity [--expected-h1-root]
  [--account] [--json]`**, which fetches the room's manifest and a challenge, proves and posts.
- **Shared code, not copies:** `crates/unstirred-cli/src/proving.rs` is the one proof path for
  both `prove` and `knock`; `unstirred-core` gained `build_manifest` and `check_window_shape`;
  `Store::issue`/`record` now return `Issued { raw, issued_at, expires_at }`; the store gained
  `recent_markers`/`recent_admissions` (`store/recent.rs`); `admit` and the store take
  `&dyn Clock`; `as_verdict` is public.
- **Tests, IDs registered first** (PRD §10 and `DECISIONS.md` rows are already written):
  A34 full HTTP knock admitted, A35 second knock with the same coins ALREADY_USED_IN_CONTEXT,
  A36 a challenge the gate never issued (in `tests/adversarial/foreign_challenge.rs`),
  A37 oversized body (`adversarial/oversized_body.rs`), A38 unknown room
  (`adversarial/unknown_room.rs`), A39 no address, body or challenge in the log or responses,
  A40 `build_manifest` (core's `tests/rules.rs`), A41 config refusals
  (`unstirred-gate/tests/config.rs`), plus unit tests for the router and the RFC 3339 clock.

The service was also exercised by hand on this machine after that run, with a one-room gate whose
root sources pointed at `127.0.0.1:1` (nothing listening) and a wallet path that does not exist:
`gate init` wrote a manifest and refused a reversed window and a bad context, all five routes
answered, `POST admit` was `503 INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE` rather than a refusal of
the holder, `knock` fetched the manifest and a challenge over HTTP before failing locally on the
missing wallet, and the gate's log held one line per request with no address but its own. That was
a smoke test against no chain and no wallet, so it is not in `evidence/`; A34-A41 are the record.

Nothing is open on this step's runs: the real two-room run below was made.
What cost time is in `briefing/BUILD_NOTES.md` under "M3 step 1": `&impl Clock` cannot take a
trait object, so the store and `admit` take `&dyn Clock`; `RootSource` and `Store` are not `Send`,
so the gate keeps its source specs and builds both per request; a non-`Sync` synthetic wallet
cannot live in a `OnceLock`, so each HTTP test pays its own ~50 s proof; `serde_json::json!`
renders `Option::None` as `null`, hence `reply::body_with_note`; and clippy's `ptr_arg` and
`result_large_err` cost one gate run, neither of which `cargo check` can see.

### The real two-room run, made 2026-10-20

Recorded in `evidence/2026-10-20-two-room-knock.txt`: holders-room ADMITTED, the same coins at
that room REFUSED `ALREADY_USED_IN_CONTEXT`, second-room ADMITTED, and each room's `/log`
showing five markers with none shared between the two. Both root sources were live
(zec.rocks lightwalletd and the owner's zebrad), so no answer carried the single-source label.
That closes the `CLAIMS.md` row for this run.

The commands, as they were run. `./scripts/gate.sh` runs `cargo test --release`, which does not
refresh `target/release/unstirred`, so **build the binary first** or the old one (without
`knock`, `gate serve` or `gate init`) is what runs. With `nf-server` up as in §3 and zebrad past
the window:

```
cd /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred
cargo build --release
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
mkdir -p gate && cd gate
$U gate init --network test --context holders-room --h0 4505790 --h1 4506510 --out holders-room.json
$U gate init --network test --context second-room  --h0 4505790 --h1 4506510 --out second-room.json
cat > gate.toml <<'TOML'
listen = "127.0.0.1:8080"
database = "gate.sqlite"
root_sources = ["lwd:https://testnet.zec.rocks:443", "zebra:http://127.0.0.1:18232"]
h1_root = "8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26"

[[rooms]]
manifest = "holders-room.json"

[[rooms]]
manifest = "second-room.json"
TOML
$U gate serve --config gate.toml
```

Both rooms share h1 4,506,510, so one `h1_root` is correct for both (the startup check only
refuses a shared root when two rooms' windows end at different heights). In a second terminal:

```
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
W=/home/kingghidorah/Documents/projects/bounties/zecathon/wallet
ID=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred.age
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26

curl -s http://127.0.0.1:8080/rooms
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # expect ADMITTED, exit 0
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # expect ALREADY_USED_IN_CONTEXT, exit 1
$U knock --gate http://127.0.0.1:8080 --room second-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"          # expect ADMITTED, different markers
curl -s http://127.0.0.1:8080/rooms/holders-room/log
curl -s http://127.0.0.1:8080/rooms/second-room/log
```

The wallet is opened read-only on every knock and no transaction is created. If zebrad is behind
h0, every knock is INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE (exit 4) until the config is changed to
one source plus `single_root_source = true`, which labels every answer as not independently
checked.

## 9. M3 step 2, the Holders' Room page, committed as `bf3fe87` 2026-10-21

`./scripts/gate.sh` reported **GATE: PASS**, exit 0, with 95 tests and the leak and claims checks
when this step was built: `evidence/2026-10-21-gate-m3-step2.md`. A02c, A42, A43 and A44 are
registered in PRD §10 with `DECISIONS.md` rows, and the CLAIMS, README and LEAKS (L10, L16)
entries are written. The page was then **reviewed in a browser**, and the fixes for that review are
at the end of this section and are **committed as `e02230d`**.

What was built:

- **Two page routes, served by the gate itself**: `GET /` is the room picker and
  `GET /room/<context>` is that room's page, 404 for a room it does not host. Each is one
  self-contained HTML document assembled per request from
  `crates/unstirred-gate/ui/{picker,room}.html` + `tokens.css` + `render.js` (+ `live.js` when the
  page is live), with the state inlined as JSON. Nothing is read from disk at request time and
  nothing is fetched from any other origin; every page carries its own
  `Content-Security-Policy: default-src 'none'`.
- **One state, one renderer.** `crates/unstirred-gate/src/page/contract.rs` is the page state as a
  Rust type with `deny_unknown_fields`; `ui/render.js` is one pure `render(state)` that reads
  exactly those fields and no clock, no randomness and no network. A live page serialises the type;
  a replay act is deserialised into it; the same `check` runs over both, so a field cannot be added
  to an act without being added to the renderer.
- **What the page shows**, all of it computed from the gate's own data: the door rule in words from
  the manifest ("Held at least 0.125 ZEC privately, unmoved from block 4,505,790 through
  4,506,510."), the latest verdict large and in colour with the plain message from
  `unstirred-core`'s closed code set, what the gate wrote down beside a struck-through list of what
  it never saw, the window as a bar with its block count, both rooms' markers with the computed
  "markers shared between rooms: 0", and the bot-cost counter (fake members × 0.125 ZEC held
  unmoved across the window).
- **`GET /rooms/<context>/recent`**: a bounded in-memory feed (20 per room) of the verdicts this
  process has answered — a verdict, a code and the second it was answered, nothing else, never
  written to disk, gone on restart. It exists because the store writes nothing for a refusal, so
  the demo's third act could not otherwise appear on the page (`DECISIONS.md` 2026-10-21, LEAKS
  L16).
- **Replay mode**: `GET /room/<context>?replay=<act>` renders a recorded act with no polling, no
  clock and no randomness. The document is assembled *without* `live.js`, so it has no network call
  site at all, and its policy is `connect-src 'none'`. The four acts are `act1-rule`,
  `act2-admitted`, `act3-refused` and `act5-second-room`, generated from
  `evidence/2026-10-20-two-room-knock.txt` by `scripts/replay-from-evidence.py` and compiled in
  with `include_str!`. An act belongs to one room and is 404 at any other.
- **`ui/frame.md`** mirrors the `:root` token block as the design system the demo video is cut
  against, and a unit test fails if the two drift.
- **The leak scan** gained A02c's repository half over `crates/unstirred-gate/ui/`. Both halves
  were confirmed against a planted third-party stylesheet and a planted beacon.
- Also in this step: the room log's `rows` field is now `limit`, because it published the cap on
  the answer under a name that read as a count of stored rows.

### Opening it

The page needs no wallet, no nf-server and no proof — only a gate. Live, with the two rooms of §8:

```
cd /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred
cargo build --release
cd /home/kingghidorah/Documents/projects/bounties/zecathon/gate
/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred \
    gate serve --config gate.toml
```

Then open `http://127.0.0.1:8080/` (the picker) and
`http://127.0.0.1:8080/room/holders-room`. The live page polls `/rooms/<context>/log` and
`/rooms/<context>/recent` every two seconds, so a knock from §8 turns the verdict green while it is
open, and a second knock with the same coins turns it red. The two root sources in that config are
only read when something is admitted; the page itself needs neither.

The four acts, which need nothing running but the gate and no chain at all:

```
http://127.0.0.1:8080/room/holders-room?replay=act1-rule
http://127.0.0.1:8080/room/holders-room?replay=act2-admitted
http://127.0.0.1:8080/room/holders-room?replay=act3-refused
http://127.0.0.1:8080/room/second-room?replay=act5-second-room
```

Any gate hosting rooms named `holders-room` and `second-room` serves all four, whatever its own
database holds and whatever its root sources are: an act carries its own numbers, and the route
only checks that the gate hosts a room by that name. So `unstirred gate init` two manifests, point
a `gate.toml` at them, and the acts render. Render the video at exactly 1920×1080 and the stage is
at scale 1, which is the layout a browser shows at that size.

Add `&present=1` (or `?present=1` on a live page) for the frame to record: the smallest type on the
page goes from 15-16px to 19-20px and the gate's own technical lines come off the provenance foot.
Nothing load-bearing is lost — a derived time is still explained beside the time itself.

### The visual review, 2026-10-21

The owner reviewed the page from browser screenshots at about 1326×683 and found it clipping. What
changed, in the order it was asked for:

1. **Nothing clips.** The grid rows were fixed pixel heights and the panels had `overflow: hidden`,
   so a longer act silently cut "your transaction history", the window's last line and "and 1 more".
   Rows are `minmax(auto, 1fr)` now and can only grow; panels clip nothing; `.frame` scrolls if a
   page ever outgrows the stage, so an overflow would be visible rather than hidden.
2. **The verdict is the hero** of any act that has one, at `--type-verdict`, and the door rule is
   one compact line above it. Before a door has answered anything the rule leads, as it did.
3. **"markers shared between rooms" appears only when two rooms hold markers.** Until then the page
   says "nothing to compare yet" and why, because a zero drawn from one room reads as a finding.
4. **The window is a timeline**, not a bar: h0 and h1 are named points on an axis that runs past
   both, and the band between them carries the rule in words. A room state may carry one `holder`
   (label, from, to), drawn as a second line: one that covers the band is `--admit`, one that does
   not is `--refuse` and says so. That is how act 4's newcomer will read. No recorded act draws one.
5. **The picker's cards size to their content**, each with an "Open room" link, and the rule and the
   cards are one block centred in what is left, so there is no hole under them.
6. **Act 3 says "unchanged: the refusal wrote nothing"** under what the door wrote down. It is
   computed: any verdict that is not an admission says the store was not changed.
7. **`?present=1`**, on a live page or any act, documented in `ui/frame.md`.
8. The badge no longer says "live from this gate · gate".
9. The bot counter names the window's length: "held unmoved for 720 blocks (about 5 hours at ~25 s
   per block)", computed from the manifest. The 25 s is testnet's NU7 target from
   `briefing/CLAUDE.md`, which is why the line says "about" and names it.

The page state gained two fields for this (`holder` per room, `present` per page), so the four acts
were regenerated by `scripts/replay-from-evidence.py` with `holder: null` and `present: false`.
`scripts/page-layout-check.mjs` is how the first point is held: it drives a headless Chromium over
the DevTools protocol across every page variant at both sizes and fails if anything is hidden. It
is **not** a gate step — `scripts/gate.sh` needs neither a browser nor a node — so it is run by
hand and its run is recorded in `evidence/2026-10-21-page-layout-check.md`, which is the first
`MEASURED` row in `CLAIMS.md`.

## 10. M4: act 4, built and recorded 2026-10-22, committed as `2e24559` and `527f871`

Act 4 of the demo script (PRD §9) is the newcomer: a holder whose coins are newer than the gate's
h0. **Both of its halves are now recorded against the real testnet**, and the act is a page the
gate serves. A second testnet wallet
(`/home/kingghidorah/Documents/projects/bounties/zecathon/bob-wallet`), whose two Ironwood coins
were mined at 4,526,839 and 4,526,842 — both after h0 4,505,790 — knocked `holders-room`: the
room's own rule refused it `COINS_TOO_NEW`, with no challenge requested and nothing sent
(`evidence/2026-10-22-bob-knock.txt`), and the forced submission was refused by the gate itself,
REFUSED / ROOT_MISMATCH, writing nothing (`evidence/2026-10-22-bob-forced-knock.txt`, header
`evidence/2026-10-22-bob-forced-knock.meta.md`). `act4-newcomer` is generated from those runs and is the
fifth act the gate serves. **Both steps are committed**: step 1 as `2e24559` (with the
2026-10-21 evidence as `62a0b8d`), step 2 — the act file, the axis rule it needed, the 413 drain
and the docs — as `527f871`.

What was built, in two steps on the same day — step 1 the code, `2e24559` (gate run
`evidence/2026-10-22-gate-m4-step1.md`); step 2 the recorded act, the window axis it needed and the
413 fix, `527f871` (gate run `evidence/2026-10-22-gate-m4-step2.md`):

- **`knock` applies the room's rule before it asks for a challenge.** `crates/unstirred-cli/src/proving/`
  is two steps now: `prepare` reads the wallet and applies the room's published rule, touching no
  network; `Prepared::prove` is the half that needs the challenge, the tree state and the PIR
  lookup. So a holder the rule refuses asks the gate for nothing but its manifest, and the command
  ends with "nothing was sent to the gate" and "no challenge was asked for, so this gate has no
  record of this attempt". "proving now" is printed only once a proof is actually being built, and
  it now names the note count and the root height it is about. A45 runs the real binary against a
  gate that records every path it is asked for (`crates/unstirred-cli/tests/knock.rs`, the crate's
  first test target) and asserts the order: the manifest is fetched, the wallet is read, and the
  run ends with both sentences and no challenge request — with a challenge request of the test's
  own afterwards, to show the recorder would have seen one. What that target cannot do is reach the
  rule's own refusals, because no wallet a test can build is one the knock will open (the synthetic
  wallet's keys are regtest; see `briefing/BUILD_NOTES.md`); the refusals themselves are the
  prover's tests (A03, A17, A18), and the real COINS_TOO_NEW refusal is the recorded Bob run.
- **`knock --force-unqualified-demo`**, the forced submission (`src/proving/demo.rs`). It builds a
  real proof — real notes, real witnesses, a real private lookup, a real binding signature —
  against the latest note-commitment root the holder's coins appear in, which is the height their
  wallet is scanned through, and posts it, so the gate refuses it `ROOT_MISMATCH` on the roots it
  pinned itself. Nothing in the package is faked: it carries the room's own published window, the
  nullifier-set root the room pinned at h1, and the real note-commitment root the proof is about.
  It cannot be reached by accident: it is in no usage text, it refuses to run for a wallet the room
  would admit ("there is nothing to force"), and it prints
  `DEMO: submitting a proof that does not meet this room's rule` with the code the room refused it
  with, before any proving. A46 holds the first two halves (a unit test in `demo.rs` for the guard,
  the binary's own output for the hiding).
- **A03's verifier half**, which had only ever been tested on the prover's side:
  `crates/unstirred-verifier/tests/adversarial/forced_newcomer.rs` admits a real proof of a note
  that did not exist at its room's h0, built against the later root it does appear in, and the gate
  answers REFUSED / ROOT_MISMATCH, consumes nothing and writes nothing. The fourth package of the
  store fixture's corpus is that proof.
- **One prover change:** `ProofRequest` carries `notes_unspent_at`, the height whose unspent notes a
  nullifier is read from. The honest path passes the gate's h1 — "witnessed at h0, unspent at h1" is
  the whole statement — and the forced demo passes its own witness height, because a newcomer has no
  unspent note at the room's h1 to read. `DECISIONS.md` has the row; `briefing/BUILD_NOTES.md` has
  why the PIR snapshot pins the other half of this.
- **Two page fixes from the screenshot review:** the room picker aligns its block to the top of the
  stage instead of centring it (no empty band above the cards), and a derived time's explanation
  moved out of the verdict card into the provenance foot, which `?present=1` drops — the card keeps
  "(time derived)" beside the time itself, so a derived number is still never shown as an observed
  one.
- **The replay acts' registry is generated**: `scripts/replay-from-evidence.py` now writes
  `crates/unstirred-gate/ui/replay/acts.rs` beside the act files, and `src/page/replay.rs` includes
  it. An act that a run produces is served with no code change. A44 checks the names against the
  demo script's five instead of counting four, and `scripts/page-layout-check.mjs` reads the acts
  from disk too.

Step 1's gate run is `evidence/2026-10-22-gate-m4-step1.md`: **GATE: PASS**, exit 0, 102 tests
(97 before it), at `rustc 1.91.1`, with its page fixes measured in a real browser
(`evidence/2026-10-22-page-layout-check.md`). Two attempts preceded it — one `cargo fmt` failure,
and one A37 failure that only reproduces under load. Step 2's gate run is
`evidence/2026-10-22-gate-m4-step2.md`, and act 4's own layout check, twenty-eight page variants
now, is `evidence/2026-10-22-page-layout-check-act4.md`.

Step 2 also closed A37's flakiness, at the server rather than in the test: an admit body refused
on its declared `Content-Length` is now read and thrown away up to `DRAIN_LIMIT_BYTES` (four times
the limit the gate accepts) before the 413 is written, so the client is not reset mid-send and the
answer the gate made can be read. Nothing drained is kept, parsed, verified or stored, and past
that cap the client is still left to the close. The refusal's own text changed with it: "nothing
was read" is now "nothing was parsed, verified or stored", because the gate does now read it.
A37 itself is unchanged and still strict — it retries nothing (`DECISIONS.md`, 2026-10-22).

### The newcomer's two knocks, as they were run, 2026-10-22

Recorded in `evidence/2026-10-22-bob-knock.txt` and `evidence/2026-10-22-bob-forced-knock.txt`,
with the header beside them. The ordinary knock asked for the room's manifest and stopped: no
challenge line, no "proving now", `COINS_TOO_NEW`, "nothing was sent to the gate". The forced one
announced itself, proved one note (mined 4,526,842, unspent through 4,528,029) against the
note-commitment root at 4,528,029, posted it, and the gate answered REFUSED / ROOT_MISMATCH; the
room's log afterwards still shows the same five markers and the same one `ADMITTED` as the
two-room run of 2026-10-20 left it. Both root sources were live, so no answer carried the
single-source label. `$ID` is the age identity for that wallet. The gate's **two root sources must
both be live**, or the admit route answers INCONCLUSIVE / `ROOT_SOURCE_UNAVAILABLE` instead of
refusing the holder, and act 4 would show the wrong thing.

```bash
# 1. build the binary this run uses; scripts/gate.sh does not refresh it
cd /home/kingghidorah/Documents/projects/bounties/zecathon/unstirred
cargo build --release

# 2. nf-server, exactly as in section 3 above (the private lookup at h1 needs it)

# 3. the gate: the same two rooms and the same database as the 2026-10-20 run, so the room still
#    shows its five markers and its one admission, which is what act 4's page is about
cd /home/kingghidorah/Documents/projects/bounties/zecathon/gate
/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred \
    gate serve --config gate.toml

# 4. in a second terminal
cd /home/kingghidorah/Documents/projects/bounties/zecathon
U=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/target/release/unstirred
E=/home/kingghidorah/Documents/projects/bounties/zecathon/unstirred/evidence
B=/home/kingghidorah/Documents/projects/bounties/zecathon/bob-wallet
ID=<the age identity for that wallet>
H1=8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26

# 4a. the room's own rule, with the fix in place: no challenge, nothing sent (exit 1)
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$B" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1" \
    2>&1 | tee "$E/2026-10-22-bob-knock.txt"

# 4b. the forced submission: expect REFUSED / ROOT_MISMATCH from the gate (exit 1)
$U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$B" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1" --force-unqualified-demo \
    2>&1 | tee "$E/2026-10-22-bob-forced-knock.txt"

# 4c. the room afterwards: still five markers and one ADMITTED, nothing new
curl -s http://127.0.0.1:8080/rooms/holders-room/log | tee -a "$E/2026-10-22-bob-forced-knock.txt"
```

Three things can stop 4b on the holder's own side, and each of them refuses rather than inventing a
package: those coins total under 0.125 ZEC at the latest root (`INSUFFICIENT_HELD_BALANCE`), the
wallet is not scanned past them (`COINS_TOO_NEW` again, now at the demo height — re-sync and
retry), or the witness rebuild at that height fails because the wallet's shards do not reach it,
which is also a re-sync. The demo height *is* the height that wallet is fully scanned through, so
all three move with its sync. None of them happened on 2026-10-22. If the gate answers INCONCLUSIVE
instead of REFUSED, a root source was unreachable, not the holder's fault — fix the source and run
4b again (nothing was consumed).

A knock prints no address, no note value and no tree position, so those two files are publishable
as they stand. The forced knock's own lines are what act 4 is built from: the challenge expiry, the
block its coins were first mined in, the block they are still unspent through, and the block whose
root the proof was built against.

### What followed the run

All of it is done:

1. The header beside the two recorded files is `evidence/2026-10-22-bob-forced-knock.meta.md`,
   covering both and naming the date, the network, the window, the binary, the gate and its
   database, the rooms, the root sources, the h1 root, the newcomer's wallet and the nf-server
   dataset. The two `.txt` files are exactly what the terminal held. The run of 2026-10-21 stays as
   recorded too: it is the one that showed the old order, and its `CLAIMS.md` row says so.
2. `python3 -I scripts/replay-from-evidence.py` wrote `act4-newcomer.json` and `acts.rs: 5 acts`.
   Nothing else about the other four acts changed.
3. `CLAIMS.md` has a row for each new evidence file — both knocks and the header — and the A44 row
   now says the acts are built "from a recorded run", naming which run each comes from, since act 4
   comes from the other one.
4. `./scripts/gate.sh` is recorded as `evidence/2026-10-22-gate-m4-step2.md` — **GATE: PASS**,
   exit 0, 102 tests — and the layout check was run again with the fifth act on it, twenty-eight
   page variants: `evidence/2026-10-22-page-layout-check-act4.md`. Act 4 failed that check three
   times first; what it cost to fit is in that file and in `briefing/BUILD_NOTES.md`.
5. A37's flakiness is closed at the server, not in the test: a body refused on its declared length
   is drained up to `DRAIN_LIMIT_BYTES` before the 413 is written. The test retries nothing.

### What act 4's page shows

`/room/holders-room?replay=act4-newcomer`: the room as the two-room run left it — five markers, one
ADMITTED — with the newcomer's REFUSED / ROOT_MISMATCH as the newest verdict, the closed code set's
own words for it, and one `holder` line on the timeline labelled "the newcomer's coins, from the
holder's own wallet (the gate never sees this)", running from block 4,526,842 to block 4,528,029.
Both heights come from the holder's wallet, are printed on the holder's own terminal by the knock,
and are never sent to the gate; the line is drawn in the refusal colour and says it does not cover
the window. Note what it does **not** show: a line "from the coins' mined height through h1", which
is impossible — they are 21,519 blocks *newer* than h1. The axis stays on the window, and the line
is drawn as a stub off the right-hand end with "20,332 blocks after h1, never inside this window"
under it: stretching the axis to reach it (which is what the page did at first) left the window
three per cent of the rail with its caption wrapped into a column, 128px past the stage's 1080.

All three of those are done in §11 and committed as `3d2bf57`: act 4's visual fix is the broken axis, the docs
are `SPEC.md`, the rebuilt README and the finalised LEAKS, and the release is prepared as far as
it can be without committing, tagging or publishing. PRD §9's act 7, the chain itself, still has
no run behind it.

## 11. Release prep, 2026-10-22 and 2026-10-23, committed as `3d2bf57`

Three pieces, all committed as `3d2bf57`: act 4's visual fix, the docs, and everything the
release needs short of doing it. The gate run that covers all of it is
`evidence/2026-10-23-gate-release-prep.md` — GATE: PASS, exit 0, 104 tests.

### Act 4's broken axis

The newcomer used to be a short faded stub drawn at the right-hand end of the window's axis, with
the distance only in the red sentence underneath. It fitted and hid nothing, and on a screenshot
it read as an edge effect. Now the rail is **broken**: the window keeps its own scale over the
first 66% of the rail, a `//` mark sits at 75%, and the holder is a dot in the refusal colour at
89% carrying `+20,332 blocks`, with the sentence under the rail reading "the newcomer's coins
arrive here: 20,332 blocks after h1, never inside the window" and a dimmer line under that saying
both blocks were read from the holder's own wallet. Every number comes from the act's state.

- `ui/render.js`: `axisOf` became `viewOf`, which carries the axis plus where it sits on the rail
  and where the break goes; `holderOnAxis` now returns either a line or a mark past the break; the
  rail is drawn in two segments when it is broken. The mark is two `/` spans, not the string
  `"//"`, because that string is a protocol-relative URL as far as A02c's scan is concerned.
- `ui/tokens.css`: `.axis-break`, `.axis-beyond`, `.axis-beyond-dot`, `.axis-beyond-badge`,
  `.holder-aside`, and `.thirds-holder`, which widens the window panel to `1.35fr 1.05fr 0.6fr`
  whenever a holder is drawn, because the window is then what the act is explaining. The old
  `.holder-beyond-*` gradients are gone.
- The page state's `holder` gained an optional `aside` (`src/page/contract.rs`), the generator
  writes it (`scripts/replay-from-evidence.py`), act 4 was regenerated, and the holder test checks
  it. `DECISIONS.md` has rows for both the break and the field.
- Measured again, twenty-eight page variants at 1920x1080 and 1326x683, nothing hidden and
  nothing clipped: `evidence/2026-10-23-page-layout-check-act4-broken-axis.md`. Presentation mode
  at 1920x1080 is the frame the fix was made for — it went 22px past the stage with its last
  provenance line cut before the panel was widened.

### The docs

- **`SPEC.md`** (new, repository root): the statement; what is reused unchanged and credited and
  what is added here; the split-height soundness argument, tightened from
  `briefing/knowledge/HOLDING_DURATION_PAPER_CHECK.md` and stated as **ours**, deviating from
  Valar's same-height rule, tested but not externally reviewed; markers per context and why
  `round_id` excludes heights; the spend-key binding and the viewing-key attack it closes; the
  root authority; the wire formats; the admission order; the closed code set; the limits. Every
  claim cites an acceptance ID or a file in `evidence/`.
- **`README.md`** rebuilt top to bottom: two sentences, a clearly-marked placeholder for the demo
  video, the 60-second judge path (release binary → `gate init` → `unstirred verify` on
  `evidence/2026-10-17-first-proof.json` against one public lightwalletd, with the two-source form
  beside it), the five replay pages, the status table, what is reused and credited, the limits and
  the build. The verify command was run while writing it and took 35 seconds to print ADMITTED.
- **The status table is generated.** `scripts/status-table.py` renders every row of `CLAIMS.md`
  between markers in the README, and `scripts/gate.sh` runs it with `--check`, so the table cannot
  drift from the ledger the way `ui/frame.md` once drifted from the tokens.
- **`briefing/LEAKS.md` is finalised**: every row either names an acceptance ID or is written as a
  stated non-defence. No row says "planned". Two IDs were registered in PRD §10 with
  `DECISIONS.md` rows and then written, because two rows had a real defence and no test: **A47**
  (L7) and **A48** (L5b).
- **A47 found a documentation bug.** It was written to assert that two proofs of the same coins
  carry identical markers; only one of the five matched. The circuit pads the bundle to five notes
  with padding nullifiers, so a proof of one real note publishes one real marker and four fresh
  ones. Single use is unaffected — one collision is enough — but `SPEC.md` §4 and LEAKS L4 had to
  be corrected, and the test now asserts the shared count equals the number of real notes.

### The release, prepared

- **`.github/workflows/release.yml`** (new): triggered by a `v*` tag (or `workflow_dispatch` with
  an existing tag), builds `unstirred` for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` and
  `x86_64-apple-darwin`, packages each as `unstirred-<tag>-<target>.tar.gz` with the binary, the
  README, SPEC, CLAIMS and both licences, writes a `.sha256` beside it and attaches both to the
  release. Actions are pinned by commit SHA — `actions/checkout@11bd719` (v4.2.2) and
  `softprops/action-gh-release@efb3536` (v3.0.3), each resolved from the GitHub API rather than
  from memory, the second dereferenced from its annotated tag to the commit a `uses:` can take.
  **It creates no tag and publishes nothing to crates.io.**
  **It has never run**, because running it means tagging. Two things in it are therefore
  unproven: the macOS x86_64 build is a cross-compile from the arm64 runner, which has to build
  `rusqlite`'s bundled SQLite for the other architecture, and the `--version` step runs an
  x86_64 binary on that runner, which is why it ends in `|| true`. If the cross-build fails on
  the tag, drop that one matrix entry and release the other two rather than holding the release.
- **`LICENSE-MIT` and `LICENSE-APACHE`** added at the root; the repository claimed
  `MIT OR Apache-2.0` and carried neither text. The MIT copyright line reads
  "Copyright (c) 2026 Aghaduno Kenelumchukwu"; `LICENSE-APACHE` is the canonical Apache-2.0 text
  with its appendix left as the licence prints it, which is what every Rust crate ships.
- **crates.io metadata**, on the workspace and inherited by every crate: `repository`, `homepage`,
  `documentation`, `keywords`, `categories`, plus a per-crate `description`, `readme` and
  `include`. Each crate has its own short `README.md`. `include` lists only `src/**/*`, the
  README, and `ui/**/*` for the gate, so `evidence/` and `briefing/` cannot be shipped — confirmed
  with `cargo package --list` on all five (14, 27, 24, 42 and 28 files, none of them from either
  directory).
- **No path-only dependencies are left**: every internal dependency is
  `{ path = "…", version = "=0.1.0" }`. There are no git dependencies.
- **The CLI package is renamed** `unstirred-cli` → `unstirred`, so `cargo install unstirred`
  works. The binary's name is unchanged, and the directory is still `crates/unstirred-cli`. The
  rename rewrote `Cargo.lock`, which is committed with it — CI's `cargo metadata --locked` fails
  otherwise.
- **The publish dry run passes for the whole workspace**, in dependency order, each crate packed
  and then compiled from its own tarball: `evidence/2026-10-23-publish-dry-run.md`.

### What is left: two acts, neither done

The commit is done (`3d2bf57`). Its gate run was made with
`CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1` and with zebrad, the nf-server and every gate stopped,
because the parallel run was killed for memory on this 7 GB machine first; the settings change
what runs at once and nothing about what is asserted. **Neither of the two below has been done.**

1. **Publish to crates.io**, from the tagged commit, with a clean tree. One version, 0.1.0, five
   crates, in dependency order. Cargo does the ordering and the index waits itself:

   ```
   cargo login                  # once, with a crates.io token
   cargo publish --workspace --dry-run    # the recorded run, repeated on the final tree
   cargo publish --workspace
   ```

   If anything makes `--workspace` unhappy, the same thing one crate at a time, in this order,
   waiting for each to appear in the index before the next:

   ```
   cargo publish -p unstirred-core
   cargo publish -p unstirred-prover
   cargo publish -p unstirred-verifier
   cargo publish -p unstirred-gate
   cargo publish -p unstirred
   ```

   Publishing is permanent, and `=0.1.0` in every internal dependency means a later 0.1.1 of one
   crate will not be picked up by the others without their own release. That is deliberate for a
   single coordinated publish. `DECISIONS.md` still has the one open row — whether to reserve the
   names first with a 0.0.1 placeholder — and a straight 0.1.0 publish closes it either way.

2. **Tag and release.** The workflow runs on the tag, not on the commit:

   ```
   git tag -a v0.1.0 -m "Unstirred 0.1.0"
   git push origin v0.1.0
   ```

   That builds the three binaries and attaches them with their checksums. Check the run, then
   check one tarball's `.sha256` before telling anyone the link. The README's "Watch the demo"
   block is a marked placeholder until the video exists; it is the only placeholder in the README
   and it says so in words.


## 12. The demo video: a 2:27 silent cut, 2026-10-23, rebuilt and rendered as a 2:00 narrated walkthrough, 2026-10-26 — uncommitted

`video/` is a HyperFrames project (CLI 0.8.186) that now renders **2:00** at 1920x1080, 30 fps —
120.000 s, 3,600 frames — to `video/out/unstirred-demo.mp4`, with narration, captions and a music
bed. It was a 2:27 silent cut until 2026-10-26. **The 2:00 cut is rendered**: the file on disk is
that render, sha256 `9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399`, and
`video/out/stills/` holds one still per scene cut out of it — see "what ran" below.

**What the film is.** Twelve scenes of two kinds, each labelled on screen:

- **Demo** scenes re-enact a recorded run and carry `re-enactment of a recorded testnet run ·
  evidence/<file>` in the window's title bar: the five acts of the Holders' Room, and Alice's and
  Bob's knocks behind them.
- **How-to** scenes carry `how to use it` plus where their text came from: installing the binary
  (release tarball and checksum, or clone and `cargo build --release`, then `unstirred --version`),
  running a door (`gate init`, `gate.toml`, `gate serve`), the gate's HTTP API (the whole admit
  round trip in four curl calls), and the two crates as libraries.

The arc: hook → install → run a door → knock → refused → the newcomer → a second room → the API →
the crates → uses → what faking it costs → close. `video/STORYBOARD.md` has the timings, the
narration line and the caption for every scene, and a section naming the three beats of the 2:27
cut that were dropped to pay for the four how-to beats. The two retired compositions are kept as
`compositions/*.html.retired` rather than deleted.

**The honesty rule the film is built under**, stated in `video/BRIEF.md` and `video/STORYBOARD.md`:
every command and output line in a demo scene is copied verbatim from the file its label names.
Two edits are made and both are visible: the owner's home path is elided to `…/`, and a line over
about 120 characters is shortened with a `…`. A compressed wait is marked `— cut —` and **no
invented duration appears anywhere**. A how-to scene never shows a value no run produced: the
`/rooms` body is the recorded one, `unstirred 0.1.0` is this binary's own answer, and
`{"verdict":"ADMITTED"}` is what `verdict_object` writes — while the challenge response, which no
run recorded as a pair, is drawn as a *shape* in the film's "not settled" colour with that said on
the line beside it. The four use cases are labelled `uses · illustrative, not measured`. No
address, seed, key or note value is on screen anywhere.

**The sound.** Narration is Kokoro-82M, voice **`af_heart`** at speed 1.0, run locally through
`npx hyperframes tts` — 32 lines, 250 words, one `.wav` per line in `video/assets/vo/`, covering
81.7 s of the 120 s. **Each caption is its narration line word for word and carries the same
`data-start`**, so the words on screen and the words in the ear cannot drift; a caption holds
until the next line speaks. The bed is the owner's `video/assets/music.mp3`, which is 105.0 s
against a 120 s film, so it plays as two clips of the same file: clip A from its head, clip B from
45 s in, crossfading over 1.5 s at 1:00 under the voice, positioned so the track's own ending lands
on the last frame. Both carry a `data-automation` volume lane that holds the bed at 0.30 between
lines and ducks it to 0.11 while the voice speaks; clip B's lane carries the final fade. The two
clips are on track indices 12 and 13 because they overlap, which is what `duplicate_audio_track`
asks for. `assets/icon.png` appears in the hook and the close and nowhere else.

**What is in the repo and what is generated.** Committed: `video/BRIEF.md`,
`video/STORYBOARD.md`, `video/index.html`, `video/compositions/*.html` (twelve scenes, plus two
`.retired`), `video/assets/film.css`, `video/assets/film.js`, `video/assets/icon.png`,
`video/assets/music.mp3`, `video/assets/vo/*.wav`, `video/hyperframes.json`, `video/package.json`,
`video/.gitignore`. The narration `.wav` files are kept rather than gitignored **because
`index.html` hardcodes each line's measured duration**: regenerating them would need `index.html`
regenerated with it. Generated and gitignored: `video/node_modules/`, `video/out/`,
`video/assets/site/` (written by `scripts/export-replay-site.sh video/assets/site`) and
`video/assets/vo/narration-full.wav`. A fresh clone has to run that export before the browser
scenes have anything to show.

**The browser scenes show the real page**, unchanged from the 2:27 cut: each holds an `<iframe>`
of `video/assets/site/<act>.present.html` at exactly 1920x1080 inside a viewport that scales the
whole block by 0.875, so the act's own renderer computes `--scale` as 1 and lays out the page
exactly as the gate serves it. It works because a replay page fetches nothing.

**`scripts/export-replay-site.sh`** writes the acts and the picker as static HTML, default
`site/`, via `crates/unstirred-gate/examples/export-replay-site.rs`, which calls the gate's own
`room_document` and `picker_document`. Eleven files. The picker's room cards open the first
recorded act of each room, because `/room/<context>` is a gate route with nothing behind it in a
static export (A51, §14). `scripts/build-site.sh` is what assembles the published site around
them, and it renames the exported picker to `picker.html` so the site's own index of the acts can
take that directory's `index.html`.

**What ran.** `evidence/2026-10-26-demo-video-walkthrough.md` records it.
`npm run check` passes every pass against the delivered tree: Lint `0 error(s), 1 warning(s)`,
Runtime `0/0`, Layout `0 issues across 9 sample(s)`, Motion `0/0`, Contrast
`9/9 text checks pass WCAG AA`, Craft `0/0`. The one warning is `composition_file_too_large` on
`index.html`, which is deliberate — the caption track and both sound tracks live there so the whole
pass reads in one place. `film_silent` is gone, because the film now has sound.

The render ran with `--workers 2` and exited 0 in **30 m 41.3 s**: 120.000000 s, 3,600 frames at
`30/1`, 1920x1080, **two** streams where the 2:27 cut had one — h264 High at 873,680 b/s and aac LC
48 kHz stereo at 200,486 b/s — 16,218,341 bytes. Two workers rather than three because free memory
was 188 MiB at launch and 130 MiB during. The renderer fell back to `screenshot capture · software
gpu` (`BeginFrame did not run. Needs chrome-headless-shell`), which is the whole reason it took half
an hour; on a machine with memory to spare and that binary present the same project renders in
minutes, and the frames are deterministic either way. **Do not read half an hour as a fault in the
composition.**

The twelve stills were cut out of the delivered `.mp4` with `ffmpeg -ss <t> … -frames:v 1` rather
than captured separately, so each is a frame of the film that shipped. All twelve were looked at.
The 2:27 cut's stills are kept at `out/stills/pre-render-review/`.

**The transcription step is closed as not needed** — owner's decision, 2026-10-26. It is not an
open item and no later session should pick it up. The captions never depended on it: each caption is
the exact string handed to the TTS engine, on the same `data-start` as the `.wav` that string
produced, so the words on screen and the words in the ear are **in sync by construction**, not by
alignment, and a transcript could only re-measure what the build already guarantees. For the record
of why it could not have run anyway: `hyperframes transcribe` reports `Captions skipped —
whisper-cpp not found`, and `hyperframes models install parakeet` fails with npm `EALLOWSCRIPTS`,
because the sherpa-onnx runtime needs install scripts that this npm refuses for a project-scoped
install unless `allowScripts` is added to `package.json` or `.npmrc`. `video/transcript.json` does
not exist and is not wanted. `video/tools/build_narration_track.py`, written to feed that pass, is
kept only as a read-back of what the voice says and when; nothing in the film reads its output.

**The CLI pin stays 0.8.186 and the bump to 0.8.200 is declined** — owner's decision, 2026-10-26,
and closed, not deferred. The delivered render stays on the version it was recorded under: a pin the
delivered `.mp4` has not been made under is not verified for delivery, and taking the bump would
mean re-rendering to re-earn what is already green. `video/package.json` and
`video/tools/tts.sh` both name `hyperframes@0.8.186`.

**Not done, by instruction:** nothing is committed and GitHub Pages is not set up. Nothing was run
against either wallet.

### Session note — 2026-10-26, end of session

Everything below is **uncommitted**. `git log` still ends at `d2dfc12`.

**What changed this session**

- `video/` rebuilt from the 2:27 silent cut into the 2:00 narrated walkthrough: twelve scenes
  (`compositions/s1-hook` … `s12-close`), two retired as `*.html.retired`. New this session:
  `s2-install`, `s8-api`, `s9-library`. The rest were renamed and retimed.
- `video/index.html` regenerated; `video/STORYBOARD.md` and `video/BRIEF.md` rewritten.
- **Rendered.** `video/out/unstirred-demo.mp4` — 120.000000 s, 3,600 frames, 1920x1080, h264 High +
  aac LC, 16,218,341 bytes, sha256 `9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399`.
- `npm run check` passes every pass on the delivered tree (Contrast 9/9 WCAG AA, Craft 0/0). Seven
  findings were fixed before the render; the evidence file lists them with their codes.
- `video/out/stills/` — twelve stills, one per scene, cut out of the delivered `.mp4`. The earlier
  review stills moved to `out/stills/pre-render-review/`.
- `evidence/2026-10-26-demo-video-walkthrough.md` rewritten as the render record; `CLAIMS.md` row
  replaced with the measured numbers; `README.md` status table regenerated (50 claims, checker
  passes); three bullets added to `briefing/BUILD_NOTES.md`.
- `video/tools/` added — `index.html` is **generated** from `narration.json` plus the `ANCHOR`
  table in `build_index.py`, and those generators had been living outside the repo. Do not
  hand-edit the caption, audio or music clips in `index.html`; `video/tools/README.md` says how to
  regenerate.

**Machine caveat.** The render took 30 m 41 s because it fell back to `screenshot capture ·
software gpu` at ~150 MiB free. Frames are deterministic; the wall clock is this machine. Do not
read it as a regression.

### Session note — 2026-10-26, the generators verified and two decisions closed

Still **uncommitted**; `git log` still ends at `d2dfc12`. Nothing was re-rendered and the delivered
`.mp4` is untouched (sha256 still `9d1d7acb…22399`).

**`video/tools/` is complete and reproduces the film's `index.html` byte for byte.** The directory
holds `build_index.py`, `narration.json`, `tts.sh`, `build_narration_track.py` and `README.md` —
everything `index.html` is built from. Run from the repo root, from `tools/` and from `/tmp`, the
regenerated `index.html` is identical to the delivered file, sha256
`3fcc55ede0cc6d58be6f77545742bdbce61c5f2b16716ef2a17fd467380af926`, with no change to the video.

Three path faults were found and fixed; they would all have bitten the next session:

- `tts.sh` read its line table from **the previous session's scratchpad**
  (`/tmp/claude-1000/…/32342fdd-…/scratchpad/narration.json`), a directory that no longer exists.
  It now reads `tools/narration.json` and uses a `mktemp` for the table. It also called the
  `hyperframes` CLI through a hard-coded `~/.npm/_npx` cache path; it now calls
  `hyperframes@0.8.186`, the same pin as `video/package.json`, overridable with `HYPERFRAMES_PIN`.
  `HYPERFRAMES_PYTHON` keeps its recorded default and stays overridable. Re-run as a check: all 32
  lines skipped, nothing regenerated.
- `build_index.py` and `build_narration_track.py` each hard-coded the absolute path of this
  checkout. Both now derive the project root from `__file__`, so they run from any working
  directory and survive a move or a fresh clone.

No scratchpad or absolute repo path is left anywhere in `video/tools/`, or in the compositions,
`film.js` or `film.css`.

**One generator `index.html` needs is outside `video/tools/`, by design:** the browser scenes hold
an `<iframe>` of `video/assets/site/<act>.present.html`, and `assets/site/` is **gitignored** and
written by `scripts/export-replay-site.sh video/assets/site` at the repo root, via
`crates/unstirred-gate/examples/export-replay-site.rs`. A fresh clone must run that export before
the browser scenes have anything to show. Re-exported to a scratch directory and diffed this
session: **all eleven files identical.** `video/tools/README.md` now names this dependency.

**Two decisions from the owner, both closed rather than deferred:**

1. **The transcription step is closed as not needed.** Captions are synced by construction — each
   is the exact string handed to the TTS engine, on the same `data-start` as the `.wav` it
   produced. No later session should pick it up.
2. **The HyperFrames CLI pin bump to 0.8.200 is declined.** The delivered render stays on the
   version it was recorded under, 0.8.186.

**Open, in the order they matter**

1. Nothing is committed, by instruction.
2. GitHub Pages not set up; the film's close says `replay site: — (not published yet)`. The
   workflow and the site build landed in part B (§14); the setting is the owner's.

**The video is done.** The next step is **the final audit, parts A and B** (§2 item 2). Both are
done; see §13 and §14.

## 13. Final audit part A: the simpler knock, the guides, the examples, the rustdoc — committed as `456d19a`

Part A is done and committed as a checkpoint. **No CLAIMS row says GATED for it**: the gate was
not run for part A on its own, by instruction — one gate run at the end covers A and B together.

**1. A simpler knock (A49, A50).** A gate may now advertise the two public service addresses a
holder has no way to discover. `pir_url` and `lightwalletd_url` are optional top-level keys in
`gate.toml`; a value that is not http(s) with a host is a **startup refusal**, not a holder's
problem. `GET /rooms` carries a `services` object only when the gate sets at least one of them.
`knock` asks the gate for them only when `--pir` or `--lightwalletd` is missing, a flag the holder
passes always wins, and the report says which address came from the gate. A gate that advertises
nothing is read as two `None`s, not as an error, and a missing flag with nothing to fall back on is
a usage error before any challenge is asked for. Checked end to end against a serving gate:
`unstirred gate serve --config examples/gate.toml` and `curl /rooms` carries both addresses.
Registered first in `briefing/PRD.md` §10 as **A49** and **A50**, with the non-leak stated as
**L18** in `briefing/LEAKS.md` and the decision in `DECISIONS.md`.

**2–3. The two guides.** `docs/GETTING_STARTED.md` has the three paths step by step — a holder with
a wallet, an owner standing a gate up, a developer reading the code. `docs/INTEGRATION.md` has the
HTTP flow with curl and `fetch` examples, the Rust crates by git tag now and crates.io later,
embedding `unstirred-prover` in a wallet, one context per purpose, and the known limit that an app
cannot yet bind its own data (a vote, a payout address) into the signed message. Both are linked
from README's Docs section.

**4. Two runnable examples**, in `crates/unstirred-verifier/examples/`: `verify_package.rs` (164
lines) and `allowlist.rs` (187). They use nothing but the crate's public API, compile in the
workspace, and were **run and recorded** in `evidence/2026-10-26-examples-run.md`: the recorded
first proof is still ADMITTED against an h0 root fetched from the live testnet at tip 4,544,493,
and the same package is REFUSED / CHALLENGE_UNKNOWN_OR_EXPIRED by the allowlist, which then holds
0 admissions and 0 markers. Two PROVEN rows in `CLAIMS.md`.

**5. The manifests are in the repository.** `examples/holders-room.json`, `examples/second-room.json`
and a commented `examples/gate.toml`. README's 60-second path now points at the committed manifest
and shows that `unstirred gate init` writes the same file byte for byte (checked), and the demo
section is one line: `unstirred gate serve --config examples/gate.toml`. One fault found and fixed
while testing it: `examples/gate.toml` listed its rooms as `examples/holders-room.json`, but a
config's paths resolve against **its own** directory, so the gate looked for
`examples/examples/…`. The paths are now relative to the file, and the gate was started from it.

**6. Rustdoc on every public item** in core, verifier, prover and gate. `#![warn(missing_docs)]` is
on all four crates and the workspace is at **zero** missing-docs warnings, from 479 at the start.
**24 doc tests, all passing**: 12 in core, 5 in the verifier, 4 in the prover, 3 in the gate. One
is compile-only and says so in its own text — the prover's crate-level example, the holder's whole
path, because running it would build a real Halo2 proof and needs a wallet, a lightwalletd and an
nf-server. Everything else runs. Clippy with `-D warnings` is clean over `--all-targets
--all-features`, so a missing doc now fails the gate as an error rather than scrolling past as a
warning; `missing_docs` is left at `warn` so an ordinary local build is not blocked by it.

**Also run, and clean:** `cargo fmt --all`, the leak scan, the claims check (52 claims) and the
status table. **Not run:** `scripts/gate.sh`.

Three things were on disk uncommitted when part A was committed, from the session that began part
B before the two halves were untangled, and they are part of `456d19a`: `assets/icon.png`,
`scripts/make-favicon.sh` with the inline `data:` favicon in both pages (which is why
`scripts/leak-scan.sh` and A02c were narrowed from "no `<link>` tag" to "no reference to any
origin", recorded in `DECISIONS.md`), and a first draft of `scripts/render-md.py`.

**What followed:** part B, in §14.

## 14. Final audit part B: the published site, the clean-clone audit, the release prepared — uncommitted

Part B is the publish-ready pass. Nothing in it was tagged, published or uploaded, and nothing was
run against either wallet.

**1. The site, and the one setting the owner switches.** `scripts/build-site.sh` builds the whole
thing into one directory and `.github/workflows/pages.yml` runs that same script on every push to
`master`, so there is no second renderer: the five recorded acts (each in both readings) and the
room picker, straight out of the gate's own page module; the three guides rendered by
`scripts/render-md.py` into the site's own design tokens, which are the gate's `ui/tokens.css`
followed by `scripts/site/site.css`; `cargo doc` under `/api` behind a hand-written index naming
the four libraries; and a landing page. **869 files, 556 pages, 17 MB**, recorded in
`evidence/2026-10-26-site-build.md` along with the six planted faults each check was confirmed
against. All five actions are SHA-pinned (`checkout` v4.2.2, `cache` v6.1.0,
`upload-pages-artifact` v4.0.0, `deploy-pages` v4.0.5, and in `release.yml`
`softprops/action-gh-release` v3.0.3), each SHA re-checked against the GitHub tag API on
2026-10-26. The **one repository setting**: *Settings → Pages → Build and deployment → Source =
"GitHub Actions"*. No branch, no directory.

`scripts/site-pages.py` ends in two checks the build fails on — every local link resolves inside
the site, and no page fetches anything that is not a file beside it — plus one that compares
`scripts/site/acts.json` against `REPLAY_ACTS` in `crates/unstirred-gate/ui/replay/acts.rs` and
refuses to write the index if the acts described are not the acts the gate serves.

**2. The logo.** `assets/icon.png` is the README header, the site's header and the site's favicon
(decoded from the gate's own `ui/favicon.b64`, which is also the inline `data:` favicon of every
page the gate serves). **The owner uploads the social preview by hand**: *Settings → General →
Social preview → Edit → Upload an image*, with `assets/icon.png`, which is already 1280×640.

**3. README.** The site comes first, then the walkthrough video as a **clearly marked placeholder**
— the only place in the repository that needs the upload URL, with an HTML comment above it saying
so — then the 60-second path, which now uses the committed `examples/holders-room.json`.

**4. The clean-clone audit** is `evidence/2026-10-26-clean-clone-audit.md`: both README paths
followed as a stranger. **Three faults found and fixed.** The release tarball carried neither
file README's 60-second path names as an argument (`release.yml` now ships `examples/` and
`evidence/`); `docs/GETTING_STARTED.md` quoted a build time no run had produced (now 27 m 34 s,
350 crates, from this audit); and the guides named `v0.1.0-rc.1` while README named `v0.1.0`
(all `v0.1.0` now, with `docs/RELEASING.md` step 1 stating plainly that four links do not resolve
until that tag is pushed). One rough edge was left alone and written down: `unstirred codes | head`
ends in std's `SIGPIPE` panic.

**5. The release is prepared and not performed.** Versions were already `0.1.0` everywhere.
`CHANGELOG.md` has the 0.1.0 entry; `docs/RELEASING.md` has every command the owner runs, in
order — the gate, the two repository settings, `git tag`, the release notes (written out in full,
ready to paste), `gh release upload` for the video with its sha256, and `cargo publish` in
dependency order: **core, prover, verifier, gate, unstirred**. **Nothing was tagged, uploaded or
published.**

**6. Hygiene.** `CLAIMS.md` is at 55 claims and the README status table is generated from it; the
leak scan, the claims check and the status-table check all pass; §6 of this file has had its stale
lines corrected; and the ~180-line rule's ten exceptions are listed with a reason each in the
`DECISIONS.md` row of 2026-10-26, with every other tracked Rust file at or under 195 lines.

**The gate run at the end of part B covers A and B together, and it passed**:
`CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh`, exit 0, **132 tests** (108 ordinary,
24 doc tests), recorded in `evidence/2026-10-26-gate-final-audit.md`. The four GATED rows that
follow from it — the audit itself, A49/A50, A51 and the documented public API — are in `CLAIMS.md`,
which is now at **59 claims**.

**Committed as `c2751fd` and pushed.** Both workflows passed on it, `gate` and `pages`, and the
site went live at **https://drapkfile.github.io/Unstirred/** — every page checked for a 200: the
landing page, the five acts in both readings, the room picker, the three guides, the API reference
and the two icons. The two repository settings §14 asks the owner for are both done: the Pages
source and the social preview. **Nothing is tagged or published**: that is step 1 of §2.
