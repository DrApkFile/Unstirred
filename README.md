# Unstirred

**Proof your coins stayed put. Nothing else.**

Unstirred lets a Zcash holder prove to a gate — a community, an allowlist, a faucet — that they
held at least 0.125 ZEC in the Ironwood pool, unmoved, from one block to another, without
revealing their address, their balance, which coins, or anything they have ever spent. Each gate
admits the same coins exactly once, and two gates cannot tell they saw the same holder.

It is built on [Valar Group's audited delegation circuit](https://github.com/valargroup/voting-circuits),
used unchanged and pinned, with the two roots pinned to **two different heights** — which is what
turns a snapshot proof into a duration proof. [`SPEC.md`](SPEC.md) is the protocol: the statement,
the soundness argument for the split height, and every limit.

## Watch the demo

> **Not recorded yet.** The two-minute demo is cut from the pages in
> ["See the demo without running anything"](#see-the-demo-without-running-anything) below, which
> you can open today. This line becomes the link when the video exists; there is no video at any
> other URL.

## Check it yourself in 60 seconds

No wallet, no node, no database, nothing to configure. You will re-verify a **real proof made from
a real Zcash testnet wallet** against the **real testnet root**, in your own process.

**1. Get the binary.** From [the releases page](https://github.com/DrApkFile/Unstirred/releases),
download the tarball for your platform, check it against the `.sha256` beside it and unpack it.
Binaries are attached to a `v*` tag by CI, so until 0.1.0 is tagged that page is empty and the
path is [build from source](#build-from-source) — two commands, and a cold release build of the
Halo2 and Zcash dependency graph is not quick.

**2. The gate's manifest** is in the repository at
[`examples/holders-room.json`](examples/holders-room.json) — the room this proof was made for, four
numbers and nothing to configure. Nothing about it has to be taken on trust either: `unstirred`
writes the same file from the same four numbers, and the two are byte-for-byte identical.

```
unstirred gate init --network test --context holders-room --h0 4505790 --h1 4506510 \
    --out holders-room.json

diff holders-room.json examples/holders-room.json    # no output
```

**3. Verify the recorded proof** ([evidence/2026-10-17-first-proof.json](evidence/2026-10-17-first-proof.json))
against the live testnet, through one public lightwalletd:

```
unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

Expect, after about 35 seconds of Halo2 verification:

```
root sources for the h0 Ironwood note-commitment root (network test, h0 4505790):
  lwd:https://testnet.zec.rocks:443  root a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d  tip …
agreement: not checked (one source)
ROOT FROM ONE SOURCE: not independently checked
ADMITTED
verify does not record markers; use unstirred admit to enter a gate
```

Exit code 0. **One source is deliberately labelled as unchecked**: Unstirred's own default is two
independent sources that must agree, and `--single-root-source` is the explicit opt-in that says
so in every line of output. If you run a testnet `zebrad`, drop that flag and use the real thing —
this is the form that was recorded in
[evidence/2026-10-20-verify-two-source.txt](evidence/2026-10-20-verify-two-source.txt):

```
unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 \
    --root-source zebra:http://127.0.0.1:18232 \
    --h1-root 8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26 \
    --challenge 17eeb99835dd5136c986f0204445ac8e8e0d556367752f73b3b22fdb8e1d4e57
```

Both sources must agree on the h0 root or the answer is INCONCLUSIVE, never ADMITTED.

Two more things you can do without a chain: `unstirred codes` prints the whole closed verdict set,
and `unstirred verify` on a package with one byte changed is `PROOF_INVALID`, exit 1.

## See the demo without running anything

The gate serves the demo itself. Five acts, each one a recorded run rendered as a page — no chain,
no wallet, no nf-server, no network call of any kind from the page. The config and both room
manifests are in the repository, so there is nothing to write:

```
unstirred gate serve --config examples/gate.toml
```

[`examples/gate.toml`](examples/gate.toml) is commented line by line — it is the shortest
explanation of what a gate owner actually decides.

Then open:

| Act | Page | What it shows | Built from |
|---|---|---|---|
| 1 | `/room/holders-room?replay=act1-rule` | the door rule, before anyone has knocked | [two-room knock](evidence/2026-10-20-two-room-knock.txt) |
| 2 | `/room/holders-room?replay=act2-admitted` | ADMITTED, and the five markers the gate wrote down | [two-room knock](evidence/2026-10-20-two-room-knock.txt) |
| 3 | `/room/holders-room?replay=act3-refused` | the same coins again: ALREADY_USED_IN_CONTEXT, nothing written | [two-room knock](evidence/2026-10-20-two-room-knock.txt) |
| 4 | `/room/holders-room?replay=act4-newcomer` | a newcomer's coins, 20,332 blocks too new, marked past a break in the window's rail: ROOT_MISMATCH | [the forced knock](evidence/2026-10-22-bob-forced-knock.txt) |
| 5 | `/room/second-room?replay=act5-second-room` | the same coins admitted at a second room, with a disjoint marker set | [two-room knock](evidence/2026-10-20-two-room-knock.txt) |

`/` is the room picker; `/room/<context>` is the live page for a room this gate hosts. Add
`&present=1` to any of them for the larger type the video is rendered in. The root sources in that
config are only read when something is admitted — the pages never touch them, and no page asks any
other origin for anything ([layout check](evidence/2026-10-23-page-layout-check-act4-broken-axis.md),
A02c).

## Status

Nothing is claimed here that does not link to a file in `evidence/`. This table is generated from
[`CLAIMS.md`](CLAIMS.md) by `scripts/status-table.py`, which `scripts/gate.sh` re-runs with
`--check`, so it cannot drift from the ledger.

<!-- status table: generated by scripts/status-table.py from CLAIMS.md; do not edit -->

| Status | Claim | Evidence |
|---|---|---|
| **PROVEN** | A wallet's Ironwood notes held across a gate's window, and the selection, can be listed read-only | [evidence/2026-10-16-notes-real-testnet.txt](evidence/2026-10-16-notes-real-testnet.txt) |
| **PROVEN** | Each selected note's witness can be rebuilt at h0 and checked against the h0 root | [evidence/2026-10-16-witness-real-testnet.txt](evidence/2026-10-16-witness-real-testnet.txt) |
| **PROVEN** | Each selected note's nullifier can be shown absent from the h1 nullifier set through PIR, with no plaintext query | [evidence/2026-10-16-lookup-real-testnet.txt](evidence/2026-10-16-lookup-real-testnet.txt) |
| **PROVEN** | A real holding-duration proof and its binding signature can be made from the real testnet wallet over h0 4,505,790 through h1 4,506,510 | [evidence/2026-10-17-first-proof.json](evidence/2026-10-17-first-proof.json) |
| **PROVEN** | That package is ADMITTED in a fresh process against the real testnet roots | [evidence/2026-10-17-verify-first-proof.txt](evidence/2026-10-17-verify-first-proof.txt) |
| **PROVEN** | The gate database admits a package once, refuses the same package as ALREADY_USED_IN_CONTEXT, and admits the same coins at a second gate with a different marker set | [evidence/2026-10-17-admit-two-rooms.txt](evidence/2026-10-17-admit-two-rooms.txt) |
| **PROVEN** | That two-room run used one lightwalletd source, because the owner's Zebra node was still behind h0, and every output said so | [evidence/2026-10-17-two-rooms-summary.md](evidence/2026-10-17-two-rooms-summary.md) |
| **PROVEN** | The holder's wallet file was unchanged by that run (same sha256 before and after) | [evidence/2026-10-17-two-rooms-summary.md](evidence/2026-10-17-two-rooms-summary.md) |
| **PROVEN** | Two independent root sources agree on the h0 root on the real testnet (zec.rocks lightwalletd and the owner's zebrad 7.0.0-rc.0, both reporting `a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d` at h0 4,505,790), and the governing tip is the lower of the answers | [evidence/2026-10-20-roots-two-source.txt](evidence/2026-10-20-roots-two-source.txt) |
| **PROVEN** | The first real proof is ADMITTED against two agreeing root sources, with no single-source label in the output | [evidence/2026-10-20-verify-two-source.txt](evidence/2026-10-20-verify-two-source.txt) |
| **PROVEN** | A gate serving the real testnet wallet admits a knock at one room over HTTP, refuses the same coins there as ALREADY_USED_IN_CONTEXT, and admits them at a second room, with two agreeing root sources (zec.rocks lightwalletd and the owner's zebrad) and no single-source label; each room's log shows five markers and the two rooms share none | [evidence/2026-10-20-two-room-knock.txt](evidence/2026-10-20-two-room-knock.txt), header [evidence/2026-10-20-two-room-knock.meta.md](evidence/2026-10-20-two-room-knock.meta.md) |
| **PROVEN** | A second real testnet wallet, funded after h0 4,505,790, is refused by the room's own rule on the holder's side — COINS_TOO_NEW, no proof made — when it knocks `holders-room`. That run predates the eligibility-before-challenge fix: its output shows the gate issuing a challenge and the knock printing "proving now" before the local refusal, which is the order A45 now forbids | [evidence/2026-10-21-bob-knock.txt](evidence/2026-10-21-bob-knock.txt) |
| **PROVEN** | A gate serving the real Zcash testnet refuses the newcomer's forced submission REFUSED / ROOT_MISMATCH and writes nothing for it: the package is a real proof of a coin mined at block 4,526,842, built against the latest note-commitment root it appears in (block 4,528,029), and the room's log afterwards still shows the same five markers and the same one ADMITTED as before | [evidence/2026-10-22-bob-forced-knock.txt](evidence/2026-10-22-bob-forced-knock.txt), header [evidence/2026-10-22-bob-forced-knock.meta.md](evidence/2026-10-22-bob-forced-knock.meta.md) |
| **PROVEN** | With the eligibility check in place, the newcomer's ordinary knock asks that gate for nothing but the room's manifest: no challenge line, no "proving now", and the run ends "nothing was sent to the gate" / "no challenge was asked for" (A45, on the real testnet wallet this time) | [evidence/2026-10-22-bob-knock.txt](evidence/2026-10-22-bob-knock.txt), header [evidence/2026-10-22-bob-forced-knock.meta.md](evidence/2026-10-22-bob-forced-knock.meta.md) |
| **PROVEN** | The recorded first proof is still ADMITTED today by a program that uses nothing but `unstirred-verifier`'s public API, against an h0 root fetched from the live testnet at tip 4,544,493 — about 38,000 blocks past h1 — and reported as `a16809847bf51d60ac6d9bb3f71cbbf27483050868b94b242ca831fe14bf692d`, the same root the runs of 2026-10-17 and 2026-10-20 reported for h0 4,505,790. One source, opted into, labelled in the output | [the two examples, run](evidence/2026-10-26-examples-run.md) |
| **PROVEN** | That same package is REFUSED / CHALLENGE_UNKNOWN_OR_EXPIRED by the 187-line allowlist example, which then holds 0 admissions and 0 markers: a package cannot be lifted out of one gate's records and replayed into another's, and the refusal writes nothing | [the two examples, run](evidence/2026-10-26-examples-run.md) |
| **MEASURED** | No page the gate serves hides text to make itself fit: on every page, live, replayed and presented, at 1920x1080 and at 1326x683, the stage does not outgrow its 1080px, no box hides what does not fit, and no text has left the stage — measured in a real browser | [layout check](evidence/2026-10-21-page-layout-check.md) |
| **MEASURED** | `?present=1` raises the smallest type on any page from 15-16px to 19-20px and takes the gate's own technical lines off the provenance foot, on a live page and on a replayed act alike | [layout check](evidence/2026-10-21-page-layout-check.md) |
| **MEASURED** | The two page fixes of 2026-10-22 — the picker aligned to the top, and a derived time's explanation moved to the provenance foot, which `?present=1` drops while the card keeps "(time derived)" — hide nothing: every page variant measured again at 1920x1080 and 1326x683, and three frames looked at | [layout check](evidence/2026-10-22-page-layout-check.md) |
| **MEASURED** | Act 4 is a page the gate serves, built from the newcomer's two recorded knocks: the gate's REFUSED / ROOT_MISMATCH, and one holder whose two block heights were read from that holder's own wallet by the knock itself. The window keeps its own scale and the rail is broken after h1, with those coins marked past the break in the refusal colour and the real distance on the mark. All twenty-eight page variants hide nothing, in presentation mode too | [layout check after the break](evidence/2026-10-23-page-layout-check-act4-broken-axis.md), and [the same twenty-eight variants before it](evidence/2026-10-22-page-layout-check-act4.md), when the holder was a faded stub at the end of the axis |
| **MEASURED** | All five crates pack into the tarballs crates.io would receive and compile from them, in dependency order — core, prover, verifier, gate, unstirred — with every internal dependency carrying a version beside its path, and no file from `evidence/`, `briefing/` or any test directory in any of them. Nothing was uploaded and no token was used | [publish dry run](evidence/2026-10-23-publish-dry-run.md) |
| **MEASURED** | Every recorded act, in normal and in presentation mode, and the room picker, export to static HTML that opens with no gate, no chain, no wallet and no nf-server: eleven pages, written by the gate's own `room_document` and `picker_document` rather than by a second renderer | [the export and the render](evidence/2026-10-23-demo-video-render.md) |
| **MEASURED** | The 2:27 silent cut of the demo video was a recorded render, not a description of one: 2:27.0 at 1920x1080 and 30 fps, 4,410 frames, 12.1 MB, no audio stream, produced by HyperFrames 0.8.186 from `video/` as it stood on 2026-10-23. `video/` has since been rebuilt as a 2:00 narrated walkthrough, so that render is the record of a film this project no longer produces | [the export and the render](evidence/2026-10-23-demo-video-render.md) |
| **MEASURED** | The 2:00 narrated walkthrough is a recorded render, not a description of one: 120.000000 s at 1920x1080 and 30 fps, 3,600 frames, two streams (h264 High and aac LC 48 kHz stereo), 16,218,341 bytes, sha256 `9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399`, produced by HyperFrames 0.8.186 from `video/` in 30 m 41.3 s. Twelve scenes planned in `video/STORYBOARD.md` before they were built; thirty-two narration lines generated locally by `hyperframes tts` with Kokoro-82M `af_heart`, covering 81.7 s of the 120 s, each caption the exact string handed to the engine on the same `data-start` (no `transcript.json`: no transcription backend is installed and the npm install of one is refused by this project's script policy); `npm run check` passes every pass, including `9/9 text checks pass WCAG AA`, with the old `film_silent` warning gone; twelve stills in `video/out/stills/`, one per scene, cut out of the delivered file itself | [the rebuild and the render](evidence/2026-10-26-demo-video-walkthrough.md) |
| **GATED** | M1 step 1 (read-only wallet, held notes, selection) passed the gate | [evidence/2026-10-16-gate-m1-step1.md](evidence/2026-10-16-gate-m1-step1.md) |
| **GATED** | M1 step 2 (h0 tree state, historical witnesses) passed the gate | [evidence/2026-10-16-gate-m1-step2.md](evidence/2026-10-16-gate-m1-step2.md) |
| **GATED** | M1 step 3 (private nullifier lookup) passed the gate | [evidence/2026-10-16-gate-m1-step3.md](evidence/2026-10-16-gate-m1-step3.md) |
| **GATED** | M1 step 4 (proof package, identity path, prove) passed the gate, with core's manifest, window, context and code tests | [evidence/2026-10-17-gate-m1-step4.md](evidence/2026-10-17-gate-m1-step4.md) |
| **GATED** | M2 step 1 (two-source root authority, `unstirred roots`) passed the gate | [evidence/2026-10-17-gate-m2-step1.md](evidence/2026-10-17-gate-m2-step1.md) |
| **GATED** | The quality pass passes the gate: fmt, clippy with `-D warnings`, 76 tests including the property, oracle and adversarial suites, the leak scan and the claims check | [evidence/2026-10-20-gate-quality-pass.md](evidence/2026-10-20-gate-quality-pass.md) |
| **GATED** | The gate rejects an admit body over its limit before parsing or storing it and claims no verdict for it (A37), answers 404 for a room it does not host without naming anything it holds (A38), and refuses to start on an invalid manifest, a missing setting, duplicate contexts, the wrong number of root sources or a bad h1 root (A41) | [gate log](evidence/2026-10-20-gate-m3-step1.md) |
| **GATED** | `unstirred gate init` writes only a manifest `unstirred-core` parses, refusing h0 >= h1, an h1 off the nullifier step and a context outside the charset (A40) | [gate log](evidence/2026-10-20-gate-m3-step1.md) |
| **GATED** | The gate serves the Holders' Room page itself: a room picker at `GET /`, a page per hosted room at `GET /room/<context>`, 404 for any other room, each one self-contained HTML rendered by one pure function from the gate's own state (A42) | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) |
| **GATED** | No page the gate serves asks another origin for anything — no script, stylesheet, font, image, frame or beacon, no absolute URL in the page's own code, and every page carries `default-src 'none'` — checked over every page the gate serves and over `crates/unstirred-gate/ui/` as a gate step, both confirmed against a planted third-party stylesheet and a planted beacon (A02c) | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) |
| **GATED** | The gate's recent-verdict feed shows a refusal the store never wrote, keeps only a verdict, a code and a time, is bounded per room, writes nothing, and is empty again after a restart on the same database (A43) | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) |
| **GATED** | Each demo act is a replay state built from a recorded run by `scripts/replay-from-evidence.py` — the two-room knock of 2026-10-20 for acts 1, 2, 3 and 5, the newcomer's forced knock of 2026-10-22 for act 4 — restates the closed code set's own words, and renders in a document with no network call site and `connect-src 'none'` (A44). The script writes the list of the acts the gate serves beside them, so an act exists exactly when a run produced it | [gate log](evidence/2026-10-21-gate-m3-step2.md), [after the review](evidence/2026-10-21-gate-page-review.md) |
| **GATED** | A room page may draw one holder's own line across the window, drawn short of the band and in the refusal colour when it does not cover it; a live gate draws none, because it knows no holder's block range, and a range that is not a range is refused by the state contract | [after the review](evidence/2026-10-21-gate-page-review.md) |
| **GATED** | `unstirred knock` applies the room's published rule to the wallet before it asks that gate for anything but the manifest: no challenge is requested, nothing is posted, and the command says both that nothing was sent and that the gate has no record of the attempt (A45) | [gate log](evidence/2026-10-22-gate-m4-step1.md) |
| **GATED** | The demo's forced submission, `knock --force-unqualified-demo`, is in no usage text the CLI prints and refuses to run for a wallet the room would admit, so it cannot be reached by accident (A46) | [gate log](evidence/2026-10-22-gate-m4-step1.md) |
| **GATED** | M4 step 1 (act 4's gate half: the local eligibility check, the forced submission, A03's verifier half, the generated act registry and the two page fixes) passes the gate: 102 tests, the leak scan and the claims check | [gate log](evidence/2026-10-22-gate-m4-step1.md) |
| **GATED** | M4 step 2 (act 4 generated from its recorded runs, the window's axis rule for a holder line the axis does not reach, and the oversized-body drain) passes the gate: 102 tests, the leak scan and the claims check | [gate log](evidence/2026-10-22-gate-m4-step2.md) |
| **GATED** | An admit body refused on its declared length is read and thrown away, up to four times the limit the gate accepts, before the 413 is written, so the refusal reaches the client instead of being lost to a reset; nothing drained is kept, parsed, verified or stored (A37) | [gate log](evidence/2026-10-22-gate-m4-step2.md) |
| **GATED** | A private non-membership answer shows neither the IMT path nor the leaf bounds that bracket the holder's nullifier — not in its own `Debug`, nor in that of the lookup that carries it (A48) | [gate log](evidence/2026-10-23-gate-release-prep.md) |
| **GATED** | The release-prep pass passes the gate: act 4's broken axis, `SPEC.md`, the README rebuilt around a 60-second judge path with a status table the gate checks against this ledger, `briefing/LEAKS.md` with no untested row left, and the crates.io metadata and release workflow — 104 tests, the leak scan, the claims check and the status-table check | [gate log](evidence/2026-10-23-gate-release-prep.md) |
| **SYNTHETIC** | M1 step 5 (`unstirred verify`) passed the gate, with the prover's tests over a real K=12 proof on a synthetic wallet | [evidence/2026-10-17-gate-m1-step5.md](evidence/2026-10-17-gate-m1-step5.md) |
| **SYNTHETIC** | M2 step 2 (challenge and marker store, `admit`, `gate markers`) passed the gate: A02b, A05, A05c, A05d, A07, A08 and A11 on synthetic packages | [evidence/2026-10-17-gate-m2-step2.md](evidence/2026-10-17-gate-m2-step2.md) |
| **SYNTHETIC** | A single root source is an explicit opt-in, labelled "ROOT FROM ONE SOURCE: not independently checked" in human output and in the JSON verdict | [evidence/2026-10-17-gate-m2-step1.md](evidence/2026-10-17-gate-m2-step1.md) |
| **SYNTHETIC** | A gate hosts one room per manifest over HTTP, and `unstirred knock` fetches a room's manifest and a gate-issued challenge, proves, posts the package and reads the PRD §3.5 verdict: admitted once (A34), the same coins at that room refused ALREADY_USED_IN_CONTEXT (A35), and a challenge the gate never issued refused with nothing written (A36) | [gate log](evidence/2026-10-20-gate-m3-step1.md) |
| **SYNTHETIC** | The gate's log lines carry no address but the one it bound itself, and neither its log nor any response carries a proof-package body, a raw challenge or a session key (A39) | [gate log](evidence/2026-10-20-gate-m3-step1.md) |
| **SYNTHETIC** | A real proof against a note-commitment root a gate did not pin — which is what a newcomer's forced submission is — is refused ROOT_MISMATCH and writes nothing: no marker, no admission, and the challenge it answered is still unconsumed. The verifier refuses on its own pinned root, not on the window the package claims (A03, verifier half) | [gate log](evidence/2026-10-22-gate-m4-step1.md) |
| **SYNTHETIC** | Two proofs of the same coins at the same gate carry a different `rk`, session key and `van_comm`, because `alpha` and the value-commitment blinding are drawn fresh per proof; exactly one marker per real note repeats, which is the collision single use rests on, and the other markers in a proof are the circuit's own padding nullifiers (A47) | [gate log](evidence/2026-10-23-gate-release-prep.md) |
| **NOT BUILT** | The h1 nullifier-set root checked by the verifier itself (nf-server `verify-root`) | — |

52 claims: 16 PROVEN, 8 MEASURED, 20 GATED, 7 SYNTHETIC, 1 NOT BUILT. **PROVEN** — a recorded run against the real Zcash testnet; **MEASURED** — a recorded run of a named tool, against something this repo serves; **GATED** — settled by a recorded `scripts/gate.sh` run: fmt, clippy and the whole suite; **SYNTHETIC** — true of a wallet, nullifier set or root built inside the test process; **NOT BUILT** — named here so it cannot be read as working.

<!-- end status table -->

## What it reuses, and what is ours

| Reused, unchanged | Version | For |
|---|---|---|
| [valargroup/voting-circuits](https://github.com/valargroup/voting-circuits) — the delegation circuit, audited | `=0.12.2`, `--no-default-features --features lrz` | the whole zero-knowledge statement |
| [valargroup/vote-nullifier-pir](https://github.com/valargroup/vote-nullifier-pir) — nullifier IMT and YPIR server | tag `v0.12.3` | the private "is my nullifier absent at h1?" lookup |
| [zcash_client_sqlite / zcash_client_backend](https://github.com/zcash/librustzcash) | `=0.22.0` / `=0.24.0` | read-only wallet access, historical witnesses |
| [Zebra](https://github.com/ZcashFoundation/zebra) JSON-RPC, lightwalletd | zebrad `7.0.0-rc.0` | the two independent root sources |

Ours, and therefore ours to be wrong about: pinning the two roots to two different heights and the
argument that this is sound ([`SPEC.md` §3](SPEC.md)); markers scoped per context so two gates
cannot link a holder; the binding signature that stops a viewing-key holder from proving as you;
and a verifier that takes every root from its own authority and never from the package.

The circuit's audit covers the circuit. It does not cover any of that.

## Known limits

Every one of these is stated in full in [`SPEC.md` §10](SPEC.md), with the reason:

- **Testnet only.** Every recorded run is Zcash testnet, Ironwood, over blocks 4,505,790 to
  4,506,510.
- **The threshold is fixed at 0.125 ZEC**, because that is one ballot in the circuit, and a proof
  covers **at most 5 notes**.
- **The h1 nullifier-set root is operator-pinned.** Checking it independently needs `nf-server
  verify-root` against a full archive node, which is [not built](CLAIMS.md).
- **The split-height soundness argument has had no external review.** It is tested (A03, A04); it
  has not been read by a cryptographer outside this project.
- **Network-level metadata is not defended.** A gate sees your connection; the code drops your
  address where it accepts it (A39) but cannot hide that you connected. Use Tor.
  [`briefing/LEAKS.md`](briefing/LEAKS.md) lists every channel with its test or its stated
  non-defence.
- **Not post-quantum**, and **point-in-time**: the statement says nothing about after h1.

## Build from source

Rust 1.91 (pinned by `rust-toolchain.toml`) and python3, which the gate's leak scan, claims check
and status table use. The same gate runs in CI on every push and pull request.

```
cargo build --release
./scripts/gate.sh
```

`scripts/gate.sh` is fmt, clippy with `-D warnings`, the whole test suite, the repository leak
scan, the claims check and the status-table check. It prints `GATE: PASS` or fails.

**It wants about 7 GB of free memory.** Several targets build real K=12 Halo2 proofs, and by
default cargo builds and runs them in parallel, which is what makes the suite tolerable (about
six minutes) and also what makes it the heaviest thing in this repository. On a smaller machine,
or one with other work on it, run it serialised instead — slower, same assertions:

```
CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1 ./scripts/gate.sh
```

That is how [the recorded run](evidence/2026-10-23-gate-release-prep.md) was made, on a 7 GB
machine with everything else stopped. CI uses neither setting.

## Docs

- [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md) — three paths, step by step: a holder with
  a wallet, an owner standing a gate up, a developer reading the code
- [`docs/INTEGRATION.md`](docs/INTEGRATION.md) — building on it: the HTTP flow in any language,
  the Rust crates, embedding the prover in a wallet, and one context per purpose
- [`SPEC.md`](SPEC.md) — what is proved, why it holds, the wire formats, the limits
- [`CLAIMS.md`](CLAIMS.md) — every claim, its status, the file that settles it
- [`briefing/LEAKS.md`](briefing/LEAKS.md) — every privacy channel, with its test or stated limit
- [`briefing/PRD.md`](briefing/PRD.md) — the requirements, §10 being the test list
- [`DECISIONS.md`](DECISIONS.md) — every decision that changed a frozen rule, dated

Two runnable programs use nothing but the library's public API, and are the shortest way to read
it: [`verify_package.rs`](crates/unstirred-verifier/examples/verify_package.rs) is `unstirred
verify` in 164 lines, and [`allowlist.rs`](crates/unstirred-verifier/examples/allowlist.rs) is a
whole one-admission-per-holder allowlist in 187. Both were
[run and recorded](evidence/2026-10-26-examples-run.md).

```
cargo run --release -p unstirred-verifier --example verify_package -- --help
```

## License

MIT OR Apache-2.0
