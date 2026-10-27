---
format: 1920x1080
fps: 30
duration: 90s
message: "A door can learn that you held, long enough, without learning who you are — and here is how to run one."
arc: Hook → install it → run a door → knock → the newcomer → in your code → uses → try it yourself → close
audience: "Zcathon judges, and anyone deciding whether to trust a holder-only door"
mode: autonomous
voice: Kokoro af_heart, speed 1.0, local (`hyperframes tts`)
music: assets/music.mp3, ducked under the voice, faded out at the end
---

The cut is 90 s = **1:30** at 30 fps, 2,700 frames. Every scene below is one sub-composition in
`compositions/`, mounted by `index.html` at the `data-start` in its row. Captions and narration
live on two tracks in `index.html`, not in the scenes, so the caption pass and the voice can be
read against each other in one place. **Each caption is the narration line, word for word**, so
the words on screen and the words in the ear are the same sentence; a caption appears when its
narration clip starts and stays until the next one does.

`index.html` is **generated**: `python3 tools/build_index.py` writes it from `tools/narration.json`
plus the `SCENES` and `ANCHOR` tables in that script, and refuses to write if a line runs into the
next one or past the end of its scene. Do not hand-edit the caption, voice or music clips.

This cut is the 2:00 walkthrough re-cut to 1:30. It keeps the design, both window components, the
voice, the caption style, the music handling and every provenance label; nine scenes carry what
twelve carried, because three of the 2:00 scenes are merged into two and one is dropped. Two kinds
of scene, each labelled:

- **Demo scenes** — a re-enactment of a recorded run. Label: `re-enactment of a recorded testnet
  run · evidence/<file>`. Every command and output line is copied verbatim from that file.
- **How-to scenes** — how a judge or a developer runs it. Label: `how to use it`, plus where the
  text came from. Commands are the real ones from `README.md`, `docs/GETTING_STARTED.md`,
  `docs/INTEGRATION.md`, the release workflow, crates.io or the CLI's own usage text. Output is
  what the program actually prints: from `evidence/` where a run recorded it, otherwise the exact
  shape the code or `briefing/PRD.md` defines, marked as a shape and never given an invented value.

| # | Scene | Kind | In | Out | Length | src |
|---|---|---|---|---|---|---|
| 1 | Hook | title | 0:00 | 0:05 | 5 s | `s1-hook.html` |
| 2 | Install it | how-to | 0:05 | 0:15 | 10 s | `s2-install.html` |
| 3 | Run a door | how-to + demo page | 0:15 | 0:25 | 10 s | `s3-door.html` |
| 4 | The knock | demo | 0:25 | 0:47 | 22 s | `s4-knock.html` |
| 5 | The newcomer | demo | 0:47 | 0:58 | 11 s | `s5-newcomer.html` |
| 6 | In your code | how-to | 0:58 | 1:10 | 12 s | `s6-code.html` |
| 7 | Uses | uses | 1:10 | 1:17 | 7 s | `s7-uses.html` |
| 8 | Try it yourself | how-to | 1:17 | 1:25 | 8 s | `s8-tryit.html` |
| 9 | Close | title | 1:25 | 1:30 | 5 s | `s9-close.html` |

## What the 2:00 cut had and this one does not

Nothing was retimed silently, and no recorded line was dropped without a visible marker saying how
many lines went and which file holds them.

- **Three scenes became one.** "Alice is admitted" (16 s), "Alice again" (12 s) and "A second
  room" (8 s) are scene 4 here, 22 s instead of 36. Every recorded line of the first knock
  survives. The second knock loses the five lines in which the gate reads its own rule back — the
  first knock already read all five out, and the elision is on screen as
  `…   the same flags, the same coins, the same room`. The third knock loses its terminal
  altogether: the beat is now the page's own two marker sets and the line that counts what they
  share, which is what the narration says.
- **Two scenes became one.** "The gate's HTTP API" (10 s) and "The crates as libraries" (6 s) are
  scene 6 here, 12 s instead of 16. Both keep all of their text, and scene 6 adds two things the
  2:00 cut could not show: `cargo add unstirred-verifier`, which the published 0.1.0 makes a real
  command, and the API reference's own URL.
- **"What faking it costs" is dropped** (5 s: the bot panel counting 1,000 fake members against
  125 ZEC). It is the one scene that argued rather than demonstrated, and act 4's page still
  carries the same panel for anyone who opens the site. `compositions/s11-cost.html.retired` keeps
  it.
- **"Try it yourself" is new** (8 s). The 2:00 cut closed on `replay site: — (not published yet)`.
  The site is published now, so the film shows it: the landing page, a replay act, a guide, the API
  reference, and then the README's own 60-second verify command with its recorded output.
- **The newcomer loses 7 s.** The ordinary knock keeps every recorded line. The forced attempt
  elides five of its diagnostic lines behind one marker that says so —
  `…   five recorded lines: the room's own refusal, the root it proved against, the challenge` —
  and keeps the one that carries the point, `expect the gate to refuse this with ROOT_MISMATCH`.
- **The hook, the door and the close lose hold time only.** No line of any of them was cut.

Scenes absorbed or dropped are kept beside the live ones as `compositions/*.html.retired`:
`s4-admitted`, `s5-again`, `s7-second-room`, `s8-api`, `s9-library`, `s11-cost`, and the two the
2:00 cut had already retired, `s7c-readonly` and `s8-verify`.

## The links on screen, and that they resolve

Every URL the film shows was requested on the day of this cut and answers. None is a placeholder.

| On screen | Checked |
|---|---|
| `github.com/DrApkFile/Unstirred` | 200 |
| `drapkfile.github.io/Unstirred/` | 200 |
| `drapkfile.github.io/Unstirred/replay/act2-admitted.html` | 200 |
| `drapkfile.github.io/Unstirred/guides/getting-started.html` | 200 |
| `drapkfile.github.io/Unstirred/api/index.html` | 200 |
| `github.com/DrApkFile/Unstirred/releases/download/v0.1.0/unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz` and its `.sha256` | 200 each |
| `crates.io/crates/unstirred` | **the crate answers 200 at the crates.io API** (`/api/v1/crates/unstirred`, `max_version` 0.1.0, `bin_names` `["unstirred"]`), and `docs.rs/unstirred` and the other four crates answer 200. A bare `curl -sI` of the HTML page answers 403, which is crates.io refusing a non-browser request, not a missing crate |

`v0.1.0` is tagged, with three platform tarballs and a `.sha256` beside each, and all five crates
are on crates.io at 0.1.0 — which is why **the 2:00 cut's caveat is gone**: it said the releases
page was empty until the tag existed, and that is no longer true.

## What is verbatim, and the three edits

Two edits carried over from the 2:00 cut, and one is new. All three are visible on screen:

- **The owner's home path is elided**, `/home/kingghidorah/Documents/projects/bounties/zecathon/`
  → `…/`.
- **Lines over about 120 characters are shortened with a visible `…`**, which is the terminal's
  width at 22 px mono inside a 1740 px window. Every elision is marked; nothing is paraphrased.
- **New in this cut: whole lines are elided, never silently.** Where a recorded block does not fit
  the shorter cut, the lines that go are replaced by one `…` line that says **how many** went and
  names the file that holds them. There are two, both listed above.

Compressed waits are marked `— cut —` in the terminal's own dim colour, and no wait is given an
invented duration. **No address, seed, key or note value appears anywhere in the film.**

The gate scenes show the gate route in the URL bar
(`127.0.0.1:8080/room/<context>?replay=<act>&present=1`) because that is the real URL of the page
in the frame. The bytes in the frame come from `assets/site/<act>.present.html`, which
`scripts/export-replay-site.sh` writes with the gate's own `room_document`. Scene 8's browser
frames show the published site's real addresses, and their bytes are this repository's own build of
those pages, copied out of `site/` into `assets/site-pages/` — the label on the frame says so:
`how to use it · the published site · these bytes: scripts/build-site.sh`.

## Design

`crates/unstirred-gate/ui/frame.md` is the design truth. The film's own chrome uses that system's
**dark** values and the page inside the browser frame is the light one. Shared tokens and the two
window components live in `assets/film.css`; the typing, cursor and camera helpers live in
`assets/film.js`, loaded once by `index.html`. Nothing was changed in either file for this cut.

Geometry held by every scene: the terminal window is 1740 px wide at x 90, the browser window
1680 px at x 120 — the page inside it is its own 1920×1080 scaled by one block factor of 0.875.
The provenance label rides in the window's own title bar, right-aligned, which keeps the foot of
the frame for the caption band at y 1000–1064. Terminal type is 22 px mono; the terminal bodies
and the window chrome carry `data-layout-ignore`, a deliberate exception to the 28 px reading
floor, taken because the alternative is cutting words out of recorded output.

`assets/icon.png` appears in **two scenes only**: the hook and the close.

## Sound

- **Voice.** Kokoro-82M, run locally through `npx hyperframes tts`, voice **`af_heart`** at speed
  1.0 — the same voice as the 2:00 cut. 23 lines, 191 words, one `.wav` per line in `assets/vo/`.
  Each line is placed at its own `data-start` and **its caption is the text that was fed to the
  engine, at the same second**, so the caption cannot drift from the voice by construction rather
  than by alignment. `n01` and `n02` are the 2:00 cut's files unchanged, because their text is
  unchanged; the other 21 were generated for this cut. The 2:00 cut's surplus lines, `n24`–`n32`,
  are no longer mounted and were removed from `assets/vo/`; they are in git at `c86ba4c`.
- **Music.** The bed is now **one clip, not two.** `assets/music.mp3` is 105.038 s and the film is
  90 s, so a single clip covers the whole film: it starts 15.038 s into the file so the track's
  **own ending lands on the film's last frame**, and the 2:00 cut's 1.5 s crossfade is gone with
  the second clip it existed to join. The volume lane is unchanged in kind — it holds the bed at
  0.30 between lines and ducks it to 0.11 whenever the voice is speaking, with 0.2 s ramps, and
  fades out over the closing 3 s — and it is still derived by `build_index.py` from where the voice
  actually is, rather than written by hand.

---

## Frame 1 — Hook (0:00–0:05)

- status: animated
- src: compositions/s1-hook.html
- duration: 5s
- kind: title card — no provenance label, because nothing here is a re-enactment
- rules: waterfall-entry, kinetic-beat-slam
- narration / caption: "Fake accounts are free. History isn't." · "Prove you held. Not who you are."

Black-paper stage, no window. Line one is already composed at frame 0 (frame 0 is the thumbnail);
line two replaces it at the same optical centre at 2.68 s. "History" and "held" carry `--admit`.
`assets/icon.png` sits above the first line, fading up over the first half second and leaving with
it. Every beat of the 2:00 hook is here, 1 s earlier.

## Frame 2 — Install it (0:05–0:15)

- status: animated
- src: compositions/s2-install.html
- duration: 10s
- kind: **how to use it** · release URL from .github/workflows/release.yml; `cargo install` from crates.io; output from `unstirred --version`
- rules: discrete-text-sequence
- narration / caption: "Take the release and check its checksum." · "Or install it with cargo." · "Then ask it what it is."

One terminal, whole scene. Two paths, then the one line the binary prints:

```
$ R=https://github.com/DrApkFile/Unstirred/releases/download/v0.1.0
$ T=unstirred-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
$ curl -LO $R/$T  &&  curl -LO $R/$T.sha256
$ sha256sum -c $T.sha256
$ tar -xzf $T
                              — or from crates.io —
$ cargo install unstirred
                              — cut — a cold release build is not quick (README.md)
$ unstirred --version
unstirred 0.1.0
```

The two shell variables keep the release URL and the tarball name on one readable line each, the
way `evidence/2026-10-20-two-room-knock.meta.md` sets `$U` for the binary path. The tarball and
checksum names are the ones `.github/workflows/release.yml` builds and the v0.1.0 release actually
carries.

**`cargo install unstirred` replaces the 2:00 cut's build-from-source path**, which cost four lines
to say what one line now says: the crate is published, so a judge does not have to clone anything.
No `sha256sum -c` output and no `cargo install` output is shown, because no recorded run holds
either; `unstirred 0.1.0` is this binary's own answer, run for the 2:00 cut and unchanged.
The wait for the build is marked `— cut —` with no number on it, as every compressed wait in this
film is, and the separator between the two paths is a `line-sep` rather than a `line-cut`, so the
one marker keeps its one meaning.

Under the window, in place of the 2:00 cut's caveat: **v0.1.0 is tagged: three platform tarballs,
each with its .sha256 — and all five crates are on crates.io at 0.1.0.**

## Frame 3 — Run a door (0:15–0:25)

- status: animated
- src: compositions/s3-door.html
- duration: 10s
- kind: **how to use it** (terminal, commands from README.md) + **demo** (browser, act1-rule)
- rules: discrete-text-sequence, context-sensitive-cursor
- narration / caption: "A gate owner writes one manifest per room, and one config file." · "Serve it, and the door publishes its rule in advance."

Terminal, 0–5.4 s — the README's own block, in full; nothing in it needs eliding at 22 px:

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
frame shows what the file holds rather than how it was typed. `gate init` and `gate serve` print
nothing in any recorded run, so the terminal shows no output for them. The result is the page.

Browser, 5.4–10 s. `act1-rule.present.html` fades up inside the browser frame and the cursor comes
to rest beside the rule panel, which is that act's hero. This half carries the demo label. The
2:00 cut's third narration line, "The same rule for everyone who knocks", is gone; the page says
it and the scene is 2 s shorter.

## Frame 4 — The knock (0:25–0:47)

- status: animated
- src: compositions/s4-knock.html
- duration: 22s
- kind: demo · evidence/2026-10-20-two-room-knock.txt
- rules: discrete-text-sequence, context-sensitive-cursor, cursor-click-ripple
- narration / caption: "A holder knocks with one command, proving it on her own machine." · "Admitted." · "The door learned that she held, and nothing else." · "No address, no balance, no idea which coins." · "The same coins knock again: already used in this context." · "The refusal wrote nothing." · "A second door admits them, sharing no markers with the first."

Three of the 2:00 cut's scenes in one shot, with five cuts inside it.

**Terminal A, 0–8.4 s.** The command types, verbatim from the meta file, then the recorded output
appears line by line, down to the two closing lines:

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

`ADMITTED` lands in `--admit` with a single scale beat at 5.45 s, which is the second the voice
says the word.

**Browser A, 8.4–12.4 s.** `act2-admitted.present.html`, with the pointer travelling up the page
and a spotlight holding the "What it never saw" column from 9.9 s — the six lines it lists are the
page's own, and they are what the two captions over it read out.

**Terminal B, 12.4–16.4 s.** The same command with its middle flags elided to one visible `…`,
then the recorded second block: `REFUSED` / `ALREADY_USED_IN_CONTEXT` / the code's own message /
`retryable: false`. The five lines in which the gate reads its own rule back are elided, with the
marker saying so on screen; terminal A printed all five.

**Browser B, 16.4–18.8 s.** `act3-refused.present.html`, whose own panel says "unchanged: the
refusal wrote nothing"; the spotlight holds that panel. A short beat — the page is the evidence for
one sentence, not a scene of its own any more.

**Browser C, 18.8–22 s.** `act5-second-room.present.html`, both rooms side by side, the pointer at
rest on the page's own line "markers shared between rooms: 0" with the spotlight over both marker
sets. The third knock's terminal is not in this cut; the page carries the beat.

## Frame 5 — The newcomer (0:47–0:58)

- status: animated
- src: compositions/s5-newcomer.html
- duration: 11s
- kind: demo · evidence/2026-10-22-bob-knock.txt, evidence/2026-10-22-bob-forced-knock.txt
- rules: discrete-text-sequence, viewport-change
- narration / caption: "A newcomer's coins are newer than the window, so nothing is sent." · "Forced through, the gate refuses it itself: root mismatch."

**Terminal A, 0–4.7 s**, the ordinary knock, every recorded line: `REFUSED  COINS_TOO_NEW`, the
reason, the `why:` and `next:` lines, then "nothing was sent to the gate" / "no challenge was asked
for, so this gate has no record of this attempt". Those last two slide in as their own beat,
because they are the point of the run.

**Terminal B, 4.7–8.8 s**, the forced attempt: `--force-unqualified-demo`, the demo banner, one
marker eliding five recorded lines, the line that says what to expect
(`expect the gate to refuse this with ROOT_MISMATCH: it decides from the roots it pinned, …`), the
`— cut —`, then `REFUSED` / `ROOT_MISMATCH` / the message / `retryable: false`.

**Browser, 8.8–11 s.** `act4-newcomer.present.html`, one continuous `viewport-change` to zoom 2.18
on page point (440, 783) — the broken rail, the `+20,332 blocks` mark and the sentence under it.

The 2:00 cut's fourth narration line, "It decides from the roots it pinned, never from the
package", is gone from the voice; the terminal prints that sentence itself.

## Frame 6 — In your code (0:58–1:10)

- status: animated
- src: compositions/s6-code.html
- duration: 12s
- kind: **how to use it** · routes SPEC.md; `/rooms` body evidence/2026-10-20-two-room-knock.txt; shapes briefing/PRD.md §3.3, §3.5; API crates/unstirred-verifier/src/lib.rs
- rules: discrete-text-sequence, waterfall-entry
- narration / caption: "A gate is plain HTTP: a challenge, a package, a verdict." · "Or add the verifier crate and check a package in Rust."

**Terminal, 0–6.2 s** — the whole round trip a client makes:

```
$ curl -s http://127.0.0.1:8080/rooms
{"rooms":[{"context":"holders-room","h1_root":"8af0c16b…be26","h1_root_provenance":"operator-…
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
usage text. `{"verdict":"ADMITTED"}` is exactly what `verdict_object` writes, and it turns the
colour of an admission as it lands.

**Code card, 6.2–12 s** — the same decision taken in-process, over the real signatures:

```
$ cargo add unstirred-verifier
                              — then, in your own service —
use unstirred_verifier::{verify_package, PinnedRoots, Verdict};

let verdict = verify_package(&manifest_json, &package_json, &roots, chain_tip, &challenge);
assert_eq!(verdict, Verdict::Admitted);
```

`cargo add unstirred-verifier` is new in this cut and is a real command: the crate is published at
0.1.0. `verify_package`, `PinnedRoots` and `Verdict` are the items
`crates/unstirred-verifier/src/lib.rs` re-exports, in that signature's own argument order. Under
it, the prover's half — **`unstirred_prover::prove(ProofRequest { … })` → the package. Read-only;
no transaction.** — and then the API reference's own URL,
`drapkfile.github.io/Unstirred/api/index.html`, which is the link the storyboard calls for on
screen. Nothing in this half is output, so the label says where the API is defined rather than
naming a run.

## Frame 7 — Uses (1:10–1:17)

- status: animated
- src: compositions/s7-uses.html
- duration: 7s
- kind: **uses · illustrative, not measured** — on screen from the scene's first frame
- rules: spring-pop-entrance
- narration / caption: "Holder-only rooms, airdrops, polls, private markets."

Four cards on the dark stage, arriving as one staggered beat, each a title and one short line, no
screenshots and no invented product: holder-only communities and allowlists; airdrops and faucets;
polls and feedback; private markets. The band above them reads `uses · illustrative, not
measured`, so the frame cannot be read as a claim about anything built. Unchanged from the 2:00
cut except for 2 s more on screen, which is what the four cards wanted.

## Frame 8 — Try it yourself (1:17–1:25)

- status: animated
- src: compositions/s8-tryit.html
- duration: 8s
- kind: **how to use it** · the published site, bytes from scripts/build-site.sh; command README.md; output evidence/2026-10-26-clean-clone-audit.md
- rules: discrete-text-sequence, context-sensitive-cursor, cursor-click-ripple
- narration / caption: "The runs, the guides and the API reference are on the site." · "And one command re-verifies a real proof yourself."

New in this cut. One browser frame, four pages, then one terminal. The URL bar is the thread: it
carries the real published address of the page in the frame, and each page's bytes are this
repository's own build of that page.

| Local | URL in the bar | Page |
|---|---|---|
| 0–1.7 s | `drapkfile.github.io/Unstirred/` | the landing page; the pointer crosses to "Open the demo" and clicks |
| 1.7–2.8 s | `drapkfile.github.io/Unstirred/replay/act2-admitted.html` | the act the click opens |
| 2.8–3.9 s | `drapkfile.github.io/Unstirred/guides/getting-started.html` | a guide |
| 3.9–4.9 s | `drapkfile.github.io/Unstirred/api/index.html` | the API reference |

**Terminal, 4.9–8 s** — README's 60-second check, with the output a recorded run of it produced:

```
$ unstirred verify --package evidence/2026-10-17-first-proof.json \
    --manifest examples/holders-room.json \
    --root-source lwd:https://testnet.zec.rocks:443 --single-root-source \
    --h1-root 8af0c16b…be26  --challenge 17eeb998…4e57
root sources for the h0 Ironwood note-commitment root (network test, h0 4505790):
  lwd:https://testnet.zec.rocks:443  root a16809847bf…bf692d  tip 4545089
agreement: not checked (one source)
    …                       two lines: the tip, and the operator-pinned h1 root
ROOT FROM ONE SOURCE: not independently checked
ADMITTED
verify does not record markers; use unstirred admit to enter a gate
```

The command is README's, with the two 64-character hexes shortened with a visible `…`. The output
is `evidence/2026-10-26-clean-clone-audit.md`'s recording of that exact command, with two of its
lines elided behind the marker that says how many. **`ROOT FROM ONE SOURCE: not independently
checked` stays, in the film's "not settled" colour**, because that is the program's own label on a
single-source answer and dropping it would flatter the result.

## Frame 9 — Close (1:25–1:30)

- status: animated
- src: compositions/s9-close.html
- duration: 5s
- kind: title card
- rules: waterfall-entry
- narration / caption: "A private proof of how long you've held." · "Built on Valar Group's audited circuit."

`assets/icon.png`, which carries the mark and the name **unstirred** together, then the statement
"A private proof of how long you've held." as the hero, then, settling under it:

- Built on **Valar Group's** audited delegation circuit, reused unchanged.
- Zcash **testnet**, Ironwood pool. Window h0 4,505,790 → h1 4,506,510.
- `github.com/DrApkFile/Unstirred` · `drapkfile.github.io/Unstirred` · `crates.io/crates/unstirred`

**The 2:00 cut's `replay site: — (not published yet)` placeholder is gone**, and so is the rule it
was drawn with: all three addresses answer, and the film ends on three live links instead of two
and an apology.

---

## The narration, as measured

Kokoro-82M, voice **`af_heart`**, speed 1.0, generated line by line by `tools/tts.sh`, which runs
`npx hyperframes tts "<line>" --voice af_heart --speed 1.0 --output assets/vo/<id>.wav`.
Twenty-three lines, 191 words, **62.2 s of voice in 90 s**. Every caption is the line in this table
word for word and carries the same `data-start` as its `.wav`, and holds until the next line
speaks. `build_index.py` reads each `.wav` for its true length and refuses to write `index.html` if
a line runs into the next one or past the end of its scene.

| # | Scene | In | Spoken | Narration line — and the caption, word for word |
|---|---|---|---|---|
| `n01` | s1-hook | 0:00.25 | 2.33 s | Fake accounts are free. History isn't. |
| `n02` | s1-hook | 0:02.75 | 1.77 s | Prove you held. Not who you are. |
| `n03` | s2-install | 0:05.50 | 2.05 s | Take the release and check its checksum. |
| `n04` | s2-install | 0:08.40 | 1.51 s | Or install it with cargo. |
| `n05` | s2-install | 0:12.00 | 1.39 s | Then ask it what it is. |
| `n06` | s3-door | 0:15.50 | 3.90 s | A gate owner writes one manifest per room, and one config file. |
| `n07` | s3-door | 0:20.00 | 2.97 s | Serve it, and the door publishes its rule in advance. |
| `n08` | s4-knock | 0:25.40 | 3.54 s | A holder knocks with one command, proving it on her own machine. |
| `n09` | s4-knock | 0:30.40 | 0.79 s | Admitted. |
| `n10` | s4-knock | 0:32.00 | 2.37 s | The door learned that she held, and nothing else. |
| `n11` | s4-knock | 0:34.80 | 2.82 s | No address, no balance, no idea which coins. |
| `n12` | s4-knock | 0:37.80 | 3.43 s | The same coins knock again: already used in this context. |
| `n13` | s4-knock | 0:41.60 | 1.51 s | The refusal wrote nothing. |
| `n14` | s4-knock | 0:43.90 | 3.48 s | A second door admits them, sharing no markers with the first. |
| `n15` | s5-newcomer | 0:48.00 | 3.46 s | A newcomer's coins are newer than the window, so nothing is sent. |
| `n16` | s5-newcomer | 0:53.60 | 3.46 s | Forced through, the gate refuses it itself: root mismatch. |
| `n17` | s6-code | 0:58.50 | 4.10 s | A gate is plain HTTP: a challenge, a package, a verdict. |
| `n18` | s6-code | 1:04.60 | 3.11 s | Or add the verifier crate and check a package in Rust. |
| `n19` | s7-uses | 1:10.60 | 3.26 s | Holder-only rooms, airdrops, polls, private markets. |
| `n20` | s8-tryit | 1:17.50 | 3.54 s | The runs, the guides and the API reference are on the site. |
| `n21` | s8-tryit | 1:21.60 | 3.20 s | And one command re-verifies a real proof yourself. |
| `n22` | s9-close | 1:25.30 | 2.05 s | A private proof of how long you've held. |
| `n23` | s9-close | 1:27.60 | 2.18 s | Built on Valar Group's audited circuit. |

---

## The render, and what ran

`evidence/2026-10-27-demo-video-90s.md` records this build, and
`evidence/2026-10-26-demo-video-walkthrough.md` stays as the record of the 2:00 cut.

`video/out/` is gitignored, so the file and the stills are produced by the commands in that
evidence file rather than carried here.
