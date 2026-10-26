---
format: 1920x1080
fps: 30
duration: 120s
message: "A door can learn that you held, long enough, without learning who you are — and here is how to run one."
arc: Hook → install it → run a door → knock → the newcomer → a second room → integrate it → why it matters → close
audience: "Zcathon judges, and anyone deciding whether to trust a holder-only door"
mode: autonomous
voice: Kokoro af_heart, speed 1.0, local (`hyperframes tts`)
music: assets/music.mp3, ducked under the voice, faded out at the end
---

The cut is 120 s = **2:00** at 30 fps, 3,600 frames. Every scene below is one sub-composition in
`compositions/`, mounted by `index.html` at the `data-start` in its row. Captions and narration
live on two tracks in `index.html`, not in the scenes, so the caption pass and the voice can be
read against each other in one place. **Each caption is the narration line, word for word**, so
the words on screen and the words in the ear are the same sentence; a caption appears when its
narration clip starts and stays until the next one does.

This cut is the 2:27 film rewritten as a **walkthrough**: it keeps every design decision, both
window components, all five page scenes, the captions and the provenance labels, and spends the
time it saves on showing a judge how to use Unstirred. Two kinds of scene, each labelled:

- **Demo scenes** — a re-enactment of a recorded run. Label: `re-enactment of a recorded testnet
  run · evidence/<file>`. Every command and output line is copied verbatim from that file.
- **How-to scenes** — how a judge or a developer runs it. Label: `how to use it`, plus where the
  text came from. Commands are the real ones from `README.md`, `HANDOFF.md`, the release workflow
  or the CLI's own usage text. Output is what the program actually prints: from `evidence/` where a
  run recorded it, otherwise the exact shape the code or `briefing/PRD.md` defines, marked as a
  shape and never given an invented value.

| # | Scene | Kind | In | Out | Length |
|---|---|---|---|---|---|
| 1 | Hook | title | 0:00 | 0:06 | 6 s |
| 2 | How to install it | how-to | 0:06 | 0:20 | 14 s |
| 3 | How to run a door | how-to + demo page | 0:20 | 0:32 | 12 s |
| 4 | Alice is admitted | demo | 0:32 | 0:48 | 16 s |
| 5 | Alice again | demo | 0:48 | 1:00 | 12 s |
| 6 | Bob, the newcomer | demo | 1:00 | 1:18 | 18 s |
| 7 | A second room | demo | 1:18 | 1:26 | 8 s |
| 8 | The gate's HTTP API | how-to | 1:26 | 1:36 | 10 s |
| 9 | The crates as libraries | how-to | 1:36 | 1:42 | 6 s |
| 10 | Where it fits | uses | 1:42 | 1:47 | 5 s |
| 11 | What faking it costs | demo panel | 1:47 | 1:52 | 5 s |
| 12 | Close | title | 1:52 | 2:00 | 8 s |

## What the 2:27 cut had and this one does not

Three beats were dropped to pay for the four how-to beats, and nothing was retimed silently:

- **"Check it yourself"** (`unstirred verify` over two root sources, 15 s). The README's 60-second
  judge path is the thing a judge should run, not watch; scene 2 gets them a binary and scene 8
  shows the whole admit round trip, which includes a verdict from the same verifier.
- **"What the holder gave up"** (the read-only closing lines, 6 s). Those two lines are still on
  screen in scene 4, where the run prints them.
- The **newcomer** beat loses 9 s, **a second room** 7 s and **Alice again** 1 s. In each case the
  terminal keeps every recorded line; what goes is hold time after the last line has landed.

The two retired compositions are kept beside the live ones as `*.html.retired`, so the dropped
beats can be read and remounted without being rebuilt.

## What is verbatim, and the one edit

Unchanged from the 2:27 cut. Two edits are made to recorded lines, both visible on screen:

- **The owner's home path is elided**, `/home/kingghidorah/Documents/projects/bounties/zecathon/`
  → `…/`.
- **Lines over about 120 characters are shortened with a visible `…`**, which is the terminal's
  width at 22 px mono inside a 1740 px window. Every elision is marked; nothing is paraphrased.

Compressed waits are marked `— cut —` in the terminal's own dim colour, and no wait is given an
invented duration. **No address, seed, key or note value appears anywhere in the film.**

The browser scenes show the gate route in the URL bar
(`127.0.0.1:8080/room/<context>?replay=<act>&present=1`) because that is the real URL of the page
in the frame. The bytes in the frame come from `assets/site/<act>.present.html`, which
`scripts/export-replay-site.sh` writes with the gate's own `room_document`.

## Design

`crates/unstirred-gate/ui/frame.md` is the design truth. The film's own chrome uses that system's
**dark** values and the page inside the browser frame is the light one. Shared tokens and the two
window components live in `assets/film.css`; the typing, cursor and camera helpers live in
`assets/film.js`, loaded once by `index.html`.

Geometry held by every scene: the terminal window is 1740 px wide at x 90, the browser window
1680 px at x 120 — the page inside it is its own 1920×1080 scaled by one block factor of 0.875.
The provenance label rides in the window's own title bar, right-aligned, which keeps the foot of
the frame for the caption band at y 1000–1064. Terminal type is 22 px mono; the terminal bodies
and the window chrome carry `data-layout-ignore`, a deliberate exception to the 28 px reading
floor, taken because the alternative is cutting words out of recorded output.

`assets/icon.png` appears in **two scenes only**: the hook and the close.

## Sound

- **Voice.** Kokoro-82M, run locally through `npx hyperframes tts`, voice **`af_heart`** at speed
  1.0 — the clearest of the twelve on a technical script. 32 lines, about 250 words, one `.wav`
  per line in `assets/vo/`. Each line is placed at its own `data-start` and **its caption is the
  text that was fed to the engine, at the same second**, so the caption cannot drift from the
  voice by construction rather than by alignment. A word-level `transcript.json` was attempted and
  could not be made on this machine; what that needs is under the render.
- **Music.** `assets/music.mp3` is 105.0 s and the film is 120 s, so the bed is two clips of the
  same file crossfaded over 1.5 s at 1:00, under the voice, where the join is inaudible; the
  second clip is positioned so the track's **own ending lands on the film's last frame**, with a
  3 s fade over it. A volume lane holds the bed at 0.30 between lines and ducks it to 0.11
  whenever the voice is speaking, with 0.2 s ramps.

---

## Frame 1 — Hook (0:00–0:06)

- status: animated
- src: compositions/s1-hook.html
- duration: 6s
- kind: title card — no provenance label, because nothing here is a re-enactment
- scene: "Fake accounts are free. History isn't." → "Prove you held. Not who you are.", with the logo
- rules: waterfall-entry, kinetic-beat-slam
- narration / caption: "Fake accounts are free. History isn't." · "Prove you held. Not who you are."

Black-paper stage, no window. Line one is already composed at frame 0 (frame 0 is the thumbnail);
line two replaces it at the same optical centre at 2.9 s. "History" and "held" carry `--admit`.
`assets/icon.png` sits above the first line, fading up over the first half second and leaving with
it.

## Frame 2 — How to install it (0:06–0:20)

- status: animated
- src: compositions/s2-install.html
- duration: 14s
- kind: **how to use it** · commands from README.md and .github/workflows/release.yml; output from `unstirred --version`
- rules: discrete-text-sequence
- narration / caption: "First, get the binary." · "Download the release and check it against the checksum beside it." · "Or clone the repository and build it from source." · "Then ask it what it is."

One terminal, whole scene. Two paths, then the one line the binary prints:

```
$ R=https://github.com/DrApkFile/Unstirred/releases/download/v0.1.0
$ T=unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
$ curl -LO $R/$T  &&  curl -LO $R/$T.sha256
$ sha256sum -c $T.sha256
$ tar -xzf $T
                              — or build it yourself —
$ git clone https://github.com/DrApkFile/Unstirred && cd Unstirred
$ cargo build --release
                              — cut — a cold release build is not quick (README.md)
$ unstirred --version
unstirred 0.1.0
```

The two shell variables keep the release URL and the tarball name on one readable line each, the
way `evidence/2026-10-20-two-room-knock.meta.md` sets `$U` for the binary path.

The tarball and checksum names are the ones `.github/workflows/release.yml` builds
(`unstirred-${tag}-${target}.tar.gz` and `.sha256`). A line under the window says, in
`--unknown`, what the README says: **the releases page is empty until v0.1.0 is tagged — until
then the path is building from source.** No `sha256sum -c` output and no `cargo build` output is
shown, because no recorded run holds either; `unstirred 0.1.0` is this binary's own answer, run
for this film. The wait for the build is marked `— cut —` with no number on it, as every
compressed wait in this film is, and the separator between the two paths is a `line-sep` rather
than a `line-cut`, so the one marker keeps its one meaning.

## Frame 3 — How to run a door (0:20–0:32)

- status: animated
- src: compositions/s3-door.html
- duration: 12s
- kind: **how to use it** (terminal, commands from README.md) + **demo** (browser, act1-rule)
- rules: discrete-text-sequence, context-sensitive-cursor
- narration / caption: "A gate owner writes one manifest per room, and one config file." · "Serve it, and the door publishes its rule in advance." · "The same rule for everyone who knocks."

Terminal, 0–5.8 s — the README's own block, in full; nothing in it needs eliding at 22 px:

```
$ unstirred gate init --network test --context holders-room --h0 4505790 --h1 4506510 --out holders-room.json
$ unstirred gate init --network test --context second-room  --h0 4505790 --h1 4506510 --out second-room.json
$ cat gate.toml
listen = "127.0.0.1:8080"
database = "gate.sqlite"
root_sources = ["lwd:https://testnet.zec.rocks:443", "zebra:http://127.0.0.1:18232"]
h1_root = "8af0c16b82ff1c9170de5dc87b3c06ef3a7144befdba71fc40959effe9f5be26"

[[rooms]]
manifest = "holders-room.json"

[[rooms]]
manifest = "second-room.json"
$ unstirred gate serve --config gate.toml
```

The config is the README's own file, line for line, and `cat` replaces the README's heredoc so the
frame shows what the file holds rather than how it was typed.

`gate init` and `gate serve` print nothing in any recorded run, so the terminal shows no output
for them. The result is the page.

Browser, 6.6–12 s. `act1-rule.present.html` fades up inside the browser frame and the cursor comes
to rest beside the rule panel, which is that act's hero. This half carries the demo label.

## Frame 4 — Alice is admitted (0:32–0:48)

- status: animated
- src: compositions/s4-admitted.html
- duration: 16s
- kind: demo · evidence/2026-10-20-two-room-knock.txt
- rules: discrete-text-sequence, context-sensitive-cursor, depth-of-field-blur
- narration / caption: "A holder knocks with one command." · "The proof is made on her own machine." · "Admitted." · "The door learned that she held, and nothing else." · "No address, no balance, no idea which coins."

Terminal, 0–9.0 s. The command types, verbatim from the meta file, then the recorded output
appears line by line, down to the two closing lines — the read-only ones that were their own scene
in the 2:27 cut:

```
$ $U knock --gate http://127.0.0.1:8080 --room holders-room --wallet "$W" \
    --pir http://localhost:3000 --lightwalletd https://testnet.zec.rocks:443 \
    --identity "$ID" --expected-h1-root "$H1"
gate          http://127.0.0.1:8080
room          holders-room
…
challenge     issued by the gate, expires 2026-10-20T14:41:14Z
proving now; this takes seconds and reveals nothing to the gate but the package
                                                                      — cut —
ADMITTED
the gate was sent one package proving 1 note(s) held unmoved across the window, and nothing else
wallet …/wallet/data.sqlite was opened read-only; no transaction was created
```

`ADMITTED` lands in `--admit` with a single scale beat. Browser, 9.5–16 s:
`act2-admitted.present.html`, with a rack focus holding the "What it never saw" column — the six
lines it lists are the page's own, and they are what the last two captions read out.

## Frame 5 — Alice again (0:48–1:00)

- status: animated
- src: compositions/s5-again.html
- duration: 12s
- kind: demo · evidence/2026-10-20-two-room-knock.txt
- rules: discrete-text-sequence, context-sensitive-cursor
- narration / caption: "The same coins knock again." · "Refused: already used in this context." · "The refusal wrote nothing. The log is unchanged."

Terminal, 0–5.2 s. The same command with its middle flags elided to one visible `…`, then the
recorded second block: `REFUSED` / `ALREADY_USED_IN_CONTEXT` / the code's own message /
`retryable: false`. Browser, 5.8–12 s: `act3-refused.present.html`, whose own panel says
"unchanged: the refusal wrote nothing"; the cursor rests on that line.

## Frame 6 — Bob, the newcomer (1:00–1:18)

- status: animated
- src: compositions/s6-newcomer.html
- duration: 18s
- kind: demo · evidence/2026-10-22-bob-knock.txt, evidence/2026-10-22-bob-forced-knock.txt
- rules: discrete-text-sequence, viewport-change, context-sensitive-cursor
- narration / caption: "A newcomer's coins are newer than the window." · "The room's own rule refuses them before anything is sent." · "Forced through, the gate refuses it itself: root mismatch." · "It decides from the roots it pinned, never from the package."

Terminal A, 0–6.2 s, the ordinary knock, verbatim: `REFUSED  COINS_TOO_NEW`, the reason, then
"nothing was sent to the gate" / "no challenge was asked for, so this gate has no record of this
attempt". Terminal B, 6.6–13.9 s, the forced attempt: `--force-unqualified-demo`, the demo banner,
the `— cut —`, then `REFUSED` / `ROOT_MISMATCH`. Browser, 14.2–18 s:
`act4-newcomer.present.html`, one continuous `viewport-change` to zoom 2.18 on page point
(440, 783) — the broken rail, the `+20,332 blocks` mark and the sentence under it.

## Frame 7 — A second room (1:18–1:26)

- status: animated
- src: compositions/s7-second-room.html
- duration: 8s
- kind: demo · evidence/2026-10-20-two-room-knock.txt
- rules: discrete-text-sequence, context-sensitive-cursor
- narration / caption: "A different door, the same coins: admitted." · "Markers shared between the two rooms: zero."

Terminal, 0–4.3 s: the third recorded knock, `--room second-room`, down to `ADMITTED`. Browser,
4.6–8 s: `act5-second-room.present.html`, both rooms side by side, the cursor at rest on the
page's own line "markers shared between rooms: 0".

## Frame 8 — The gate's HTTP API (1:26–1:36)

- status: animated
- src: compositions/s8-api.html
- duration: 10s
- kind: **how to use it** · routes from SPEC.md; `/rooms` body from evidence/2026-10-20-two-room-knock.txt; challenge and verdict shapes from briefing/PRD.md §3.3 and §3.5
- rules: discrete-text-sequence
- narration / caption: "A gate is plain HTTP." · "Read the rooms, take a challenge, prove, post the package." · "The verdict comes back as JSON."

One terminal, the whole round trip a client makes:

```
$ curl -s http://127.0.0.1:8080/rooms
{"rooms":[{"context":"holders-room","h1_root":"8af0c16b…be26","h1_root_provenance":"operator-pinned", …
$ curl -s -X POST http://127.0.0.1:8080/rooms/holders-room/challenge
{ "challenge": "<32 bytes hex>", "expires_at": "<RFC3339>" }        the shape (PRD §3.3), not a value
$ unstirred prove --wallet "$W" --manifest holders-room.json --lightwalletd … \
    --pir … --identity "$ID" --challenge <32 bytes hex> --out package.json
$ curl -s -X POST --data-binary @package.json \
    http://127.0.0.1:8080/rooms/holders-room/admit
{"verdict":"ADMITTED"}
```

The `/rooms` body is the recorded one, shortened with a visible `…`. The challenge response is the
**shape**, in `--unknown` with that said on the line beside it: no recorded challenge value is
pasted in next to an expiry from a different run. `--challenge <32 bytes hex>` is the CLI's own
usage text. `{"verdict":"ADMITTED"}` is exactly what `verdict_object` writes.

## Frame 9 — The crates as libraries (1:36–1:42)

- status: animated
- src: compositions/s9-library.html
- duration: 6s
- kind: **how to use it** · API from crates/unstirred-verifier/src/lib.rs and crates/unstirred-prover/src/lib.rs
- rules: waterfall-entry
- narration / caption: "Or call the verifier directly, in a few lines of Rust." · "A wallet can embed the prover the same way."

A code card, in the terminal's own surface, over the real signatures:

```rust
use unstirred_verifier::{verify_package, PinnedRoots, Verdict};

let verdict = verify_package(&manifest_json, &package_json, &roots, chain_tip, &challenge);
assert_eq!(verdict, Verdict::Admitted);
```

and one line under it: **a wallet embeds `unstirred-prover`: `prove(ProofRequest { … })` returns
the package. The wallet is opened read-only and no transaction is created.** Both are the crates'
own public items, named in their `lib.rs`; nothing here is output, so the label says where the API
is defined rather than naming a run.

## Frame 10 — Where it fits (1:42–1:47)

- status: animated
- src: compositions/s10-uses.html
- duration: 5s
- kind: **uses · illustrative, not measured** — on screen from the scene's first frame
- rules: spring-pop-entrance
- narration / caption: "Holder-only rooms, airdrops, polls, private markets."

Four cards on the dark stage, arriving as one staggered beat, each a title and one short line, no
screenshots and no invented product: holder-only communities and allowlists; airdrops and faucets;
polls and feedback; private markets. The band above them reads `uses · illustrative, not
measured`, so the frame cannot be read as a claim about anything built.

## Frame 11 — What faking it costs (1:47–1:52)

- status: animated
- src: compositions/s11-cost.html
- duration: 5s
- kind: the page's own bot panel over the manifest window, h0 4,505,790 through h1 4,506,510
- rules: counting-dynamic-scale
- narration / caption: "A thousand fake members would cost 125 ZEC, held still."

The page's own panel at hero size: the counter runs 0 → 1,000 fake members while the cost counts
to **125 ZEC**, then the page's own sentence, shortened with a visible `…` to fit five seconds:
"1,000 members at 0.125 ZEC each, held unmoved for 720 blocks …". Every number is the manifest's:
12,500,000 zatoshis is 0.125 ZEC, and h1 − h0 is 720 blocks.

## Frame 12 — Close (1:52–2:00)

- status: animated
- src: compositions/s12-close.html
- duration: 8s
- kind: title card
- rules: waterfall-entry
- narration / caption: "A private proof of how long you've held." · "Built on Valar Group's audited delegation circuit, on Zcash testnet."

`assets/icon.png` with the name **unstirred** beside it, then the statement "A private proof of
how long you've held." as the hero, then, settling under it:

- Built on **Valar Group's** audited delegation circuit, reused unchanged.
- Zcash **testnet**, Ironwood pool. Window h0 4,505,790 → h1 4,506,510.
- `github.com/DrApkFile/Unstirred`
- `replay site: —` — a placeholder, marked as one in `--unknown`, until the site has a URL.

---

## The narration, as measured

Kokoro-82M, voice **`af_heart`**, speed 1.0, generated line by line with
`npx hyperframes tts "<line>" --voice af_heart --speed 1.0 --output assets/vo/<id>.wav`.
Thirty-two lines, 250 words, **81.7 s of voice in 120 s**. Every caption is the line in this
table word for word and carries the same `data-start` as its `.wav`, and holds until the next
line speaks. `assets/vo/narration-full.wav` is the 32 files laid onto one 120 s track at these
times, ready to transcribe; **the transcription itself did not run** — see the note under the
render below — so `transcript.json` does not exist and this table is the script, not a read-back
of the voice.

| # | Scene | In | Spoken | Narration line — and the caption, word for word |
|---|---|---|---|---|
| `n01` | s1-hook | 0:00.30 | 2.33 s | Fake accounts are free. History isn't. |
| `n02` | s1-hook | 0:03.10 | 1.77 s | Prove you held. Not who you are. |
| `n03` | s2-install | 0:06.40 | 1.49 s | First, get the binary. |
| `n04` | s2-install | 0:08.20 | 3.26 s | Download the release and check it against the checksum beside it. |
| `n05` | s2-install | 0:11.90 | 2.92 s | Or clone the repository and build it from source. |
| `n06` | s2-install | 0:15.20 | 1.39 s | Then ask it what it is. |
| `n07` | s3-door | 0:20.40 | 3.90 s | A gate owner writes one manifest per room, and one config file. |
| `n08` | s3-door | 0:24.80 | 2.97 s | Serve it, and the door publishes its rule in advance. |
| `n09` | s3-door | 0:28.20 | 2.13 s | The same rule for everyone who knocks. |
| `n10` | s4-admitted | 0:32.40 | 1.83 s | A holder knocks with one command. |
| `n11` | s4-admitted | 0:36.60 | 2.05 s | The proof is made on her own machine. |
| `n12` | s4-admitted | 0:39.30 | 0.79 s | Admitted. |
| `n13` | s4-admitted | 0:41.60 | 2.37 s | The door learned that she held, and nothing else. |
| `n14` | s4-admitted | 0:45.40 | 2.82 s | No address, no balance, no idea which coins. |
| `n15` | s5-again | 0:48.40 | 1.62 s | The same coins knock again. |
| `n16` | s5-again | 0:51.80 | 2.43 s | Refused: already used in this context. |
| `n17` | s5-again | 0:55.60 | 2.86 s | The refusal wrote nothing. The log is unchanged. |
| `n18` | s6-newcomer | 1:00.40 | 2.41 s | A newcomer's coins are newer than the window. |
| `n19` | s6-newcomer | 1:04.60 | 3.35 s | The room's own rule refuses them before anything is sent. |
| `n20` | s6-newcomer | 1:08.80 | 3.46 s | Forced through, the gate refuses it itself: root mismatch. |
| `n21` | s6-newcomer | 1:13.20 | 3.22 s | It decides from the roots it pinned, never from the package. |
| `n22` | s7-second-room | 1:18.40 | 2.47 s | A different door, the same coins: admitted. |
| `n23` | s7-second-room | 1:23.00 | 2.60 s | Markers shared between the two rooms: zero. |
| `n24` | s8-api | 1:26.40 | 1.83 s | A gate is plain HTTP. |
| `n25` | s8-api | 1:29.00 | 3.24 s | Read the rooms, take a challenge, prove, post the package. |
| `n26` | s8-api | 1:34.00 | 1.92 s | The verdict comes back as JSON. |
| `n27` | s9-library | 1:36.80 | 2.43 s | Or call the verifier directly, in Rust. |
| `n28` | s9-library | 1:39.90 | 1.71 s | A wallet can embed the prover. |
| `n29` | s10-uses | 1:42.60 | 3.26 s | Holder-only rooms, airdrops, polls, private markets. |
| `n30` | s11-cost | 1:47.30 | 4.50 s | A thousand fake members would cost 125 ZEC, held still. |
| `n31` | s12-close | 1:52.80 | 2.05 s | A private proof of how long you've held. |
| `n32` | s12-close | 1:55.60 | 4.29 s | Built on Valar Group's audited delegation circuit, on Zcash testnet. |

---

## The render, and what ran

`evidence/2026-10-26-demo-video-walkthrough.md` records this build. **The check passed and the
render ran.**

```
cd video && npm run check
```

Lint `0 error(s), 1 warning(s)`; Runtime `0 errors, 0 warnings`; Layout `0 issues across 9
sample(s)`; Motion `0 errors, 0 warnings`; Contrast `9/9 text checks pass WCAG AA`; Craft
`0 errors, 0 warnings`. `Check passed`.

The one warning is `composition_file_too_large` on `index.html` (336 lines), which is deliberate:
the caption track and both sound tracks live there so the whole caption pass and the voice can be
read against each other in one place. The `film_silent` warning the 2:27 cut carried on every run
is gone, because the film now has sound. Six other findings were fixed rather than carried — the
two overlapping music clips went onto two track indices; the 0.06 s of padding on each voice clip's
slot came off; the two cropped logo images were marked `data-layout-allow-overflow`; the releases
caveat was raised to the 28px reading floor; the prover line was cut to "Read-only; no
transaction." and brought in earlier; and the cost panel's source line was marked
`data-layout-ignore` and given longer on screen. The evidence file lists them with their codes.

**The transcription did not run.** `hyperframes transcribe assets/vo/narration-full.wav`
reports `Captions skipped — whisper-cpp not found`, and `hyperframes models install parakeet`
fails in npm with `EALLOWSCRIPTS`: the sherpa-onnx runtime needs install scripts, which this npm
refuses for a project-scoped install unless `allowScripts` is added to `package.json` or
`.npmrc`. That is a permission to let a native package run install scripts in this project, so it
is the owner's call, not one to take quietly. The captions do not depend on it: each one is the
exact string handed to the TTS engine, placed at the same second as the `.wav` it produced.

```
cd video
npx hyperframes@0.8.186 render --output out/unstirred-demo.mp4 --fps 30 --quality delivery \
    --workers 2 --skill general-video
```

Exit 0, in 30 m 41.3 s. **120.000000 s, 3,600 frames at `30/1` fps, 1920x1080**, and **two**
streams where the 2:27 cut had one: h264 High at 873,680 b/s and aac LC 48 kHz stereo at
200,486 b/s. 16,218,341 bytes; sha256
`9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399`.

`--workers 2` rather than 3 because free memory was 188 MiB at launch and 130 MiB during. The
renderer fell back to `screenshot capture · software gpu` — `BeginFrame did not run. Needs
chrome-headless-shell` — which is why it took half an hour; with memory to spare and
`chrome-headless-shell` present the same project renders in minutes. The frames are deterministic
either way.

`video/out/stills/` holds twelve PNGs, one per scene, cut out of the delivered `.mp4` itself with
`ffmpeg -ss <t> -i out/unstirred-demo.mp4 -frames:v 1` at 4.8, 19.2, 30.5, 44.0, 58.0, 76.8, 84.5,
95.0, 101.0, 106.0, 111.0 and 118.0 s, so each still is a frame of the film that shipped. The 2:27
cut's stills and the pre-render contact sheets are kept at `out/stills/pre-render-review/`.

`video/out/` is gitignored, so the file and the stills are produced by those commands rather than
carried here.
