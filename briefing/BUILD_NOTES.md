# BUILD_NOTES

Things learned while building that the code and the other briefing files do not say. Decisions go
to `DECISIONS.md`; recorded runs go to `evidence/`. This file is for the facts that cost time.

## Final audit part A: advertised services, the guides, the examples, rustdoc, 2026-10-26

- **A config file's paths resolve against the config file, so a path that reads correctly from the
  repo root is usually wrong.** `examples/gate.toml` listed `manifest = "examples/holders-room.json"`
  — which is what you type on the command line, and which the loader turned into
  `examples/examples/holders-room.json`. Nothing caught it, because no test starts a gate from that
  file and reading the line in isolation looks right. **The only check that settles a committed
  config is starting the thing from it**, which is now done: `unstirred gate serve --config
  examples/gate.toml` serves both rooms, and `curl /rooms` shows the advertised services.
- **A stale `target/release` binary will lie to you about a feature you just added.** Testing the
  new `pir_url` key with `./target/release/unstirred` reported `unknown field pir_url` — the binary
  predated the field. `scripts/gate.sh` does not refresh it either (HANDOFF §10 already says so for
  a different run). Build before you test a config surface, or test it through the crate's own
  tests, which had already passed.
- **`missing_docs` is a per-crate lint, so turning it on is four edits and one number.** The
  workspace went from 479 missing items to zero. The bulk of it is one file: `ProverError` alone is
  57 variants and 52 named fields. Those were documented by a script that parses the enum and
  inserts a line above each variant and field, keyed by name — **never by retyping the
  `#[error(...)]` strings**, which are user-visible text and the one thing a bulk edit must not
  touch.
- **A doc comment inserted above `pub struct` lands below the `#[derive]`, which is a compile
  error.** Two scripted passes were needed: one to move doc blocks above their attributes, and one
  to delete the attribute block a replacement had duplicated. If a bulk doc edit anchors on
  `pub struct X {`, anchor on the attributes above it instead, or expect to clean up after.
- **Doc tests are the expensive kind of test in a Halo2 workspace.** Each one links its own binary
  against the whole dependency graph: five doc tests in `unstirred-verifier` took 145 s, almost all
  of it linking. That is the reason there are 24 and not 60 — a doc example earns its place by
  showing a rule that is hard to see in the signature (which codes a refusal uses, that a challenge
  cannot be replayed, that the stored challenge is not the one handed out), not by restating the
  types.
- **One doc example is `no_run` on purpose and says so in its own text**: the prover's crate-level
  example, the holder's whole path. It compiles — which is what keeps it honest when the API moves
  — but running it would build a real Halo2 proof and would need a wallet, a lightwalletd and an
  nf-server.
- **Advertising a service address is a startup decision, not a request-time one.** A gate that
  cannot say where its nf-server is should refuse to start, because the alternative is a holder
  reading the mistake in their own terminal, from a URL they did not choose. `advertisable()`
  checks scheme and host through `http::Uri` in `load_config`, so the refusal is `ConfigError` and
  the socket is never bound (A49).

## The video's generators, and two steps closed rather than deferred, 2026-10-26

- **A generator that lives in a scratchpad is a generator that is already lost.** `video/index.html`
  is written by `video/tools/build_index.py`, and the whole set was moved into the repo only at the
  end of the session that used it. By the next session the previous scratchpad was gone, taking
  `tts.sh`'s line table with it — the script still read `narration.json` from
  `/tmp/claude-1000/…/<old-session-id>/scratchpad`. Two more scripts hard-coded this checkout's
  absolute path. Nothing had noticed, because the file they produce was already on disk. **Move a
  generator into the repo when you first run it, and derive every path from `__file__` or
  `BASH_SOURCE`, not from where the session happens to be standing.**
- **"The generators are committed" and "the generators reproduce the artefact" are two claims.**
  The check that settles the second one is cheap and is the only one worth recording: copy the
  artefact aside, re-run the generator from a *different* working directory, and diff. Here all
  three runs — repo root, `tools/`, `/tmp` — reproduced `index.html` byte for byte, sha256
  `3fcc55ed…0af926`, and the same method confirmed `scripts/export-replay-site.sh` reproduces all
  eleven files of the exported replay site.
- **A video's inputs do not all live under the video.** The browser scenes `<iframe>` the exported
  replay pages, which are **gitignored** and written from the gate's own page module by
  `scripts/export-replay-site.sh video/assets/site` at the repo root. A fresh clone renders those
  scenes empty until that export runs. When one project's assets come out of another's build, the
  dependency belongs in writing next to the generators — `video/tools/README.md` now names it.
- **The transcription step is closed as not needed** (owner's decision). A caption that *is* the
  exact string handed to the TTS engine, placed on the same `data-start` as the `.wav` that string
  produced, is in sync **by construction**. Aligning a transcript against it could only re-measure
  what the build already guarantees, so the whole class of work — `whisper-cpp`, the `parakeet`
  model's npm `EALLOWSCRIPTS` failure, `transcript.json` — is not a blocked step but a step that was
  never needed. Build the artefact so the check is unnecessary and you do not have to pass it.
- **The HyperFrames CLI pin bump to 0.8.200 is declined** (owner's decision). The pin stays
  0.8.186, the version the delivered render was recorded under. A pin the delivered file has not
  been made under is not verified for delivery, and the rule is apply-then-verify: taking the bump
  means re-rendering to re-earn a result that is already green. **Once a deliverable is recorded,
  its toolchain pin stops being a maintenance question and becomes part of the record.**

## The demo video rebuilt as a narrated 2:00 walkthrough, 2026-10-26

- **`hyperframes tts` probes for `kokoro_onnx` with a ten-second `import`, and on a swapping
  machine that is the whole problem.** `kokoro-onnx` is in `~/.venv`, not the system python, so
  `HYPERFRAMES_PYTHON` has to point at it — but the override is only taken if
  `<python> --version` answers inside five seconds *and* `import kokoro_onnx` answers inside ten,
  and on failure the CLI silently falls back to `python3` and reports "the kokoro-onnx package is
  not installed". A cold `import kokoro_onnx` here took **2 m 28 s of wall time for 2 s of CPU**;
  warm it took 3.8 s. The fix is to import it in the same shell immediately before each `tts`
  call, which keeps the pages resident. Two of thirty-two lines still lost the race and were
  re-run. If that error appears and the package *is* installed, it is a timeout, not a missing
  dependency.
- **Kokoro `af_heart` speaks at about four words a second**, well above the 2.1 w/s that 250 words
  over two minutes implies. Writing to a word budget and then measuring was the right order: the
  voice covers 81.7 s of 120 s, which leaves the gaps the captions need. Plan the anchors, measure
  the files, then place — never the other way round.
- **Captions written as "the narration line, word for word, on the same `data-start`" cannot
  drift.** `index.html` is generated from one table of (id, scene, anchor, text): the caption, the
  `<audio>` and the music's duck window all come out of that one row, so a change to a line is one
  edit, not three. The generator refuses to write a file if a line runs into the next one or past
  its scene, which caught four anchors before anything was rendered.
- **A music bed shorter than the film is two clips, not a loop flag.** `music.mp3` is 105.0 s
  against 120 s and has its own fade-out, so clip B starts 45 s into the same file and is placed
  so that ending lands on the last frame, crossfading with clip A over 1.5 s at 1:00 — under the
  voice, where the join is inaudible. Two clips that overlap must be on two `data-track-index`
  values or `lint` reports `duplicate_audio_track`.
- **`film.terminal()` rewrites each line's `textContent`, so a line cannot carry markup.** The
  Rust snippet needed a keyword colour, so that scene reveals its lines with a stagger instead of
  typing them. Anything inside a `.line` that is not plain text is gone after the helper's first
  paint.
- **The browser is the floor under every visual check, and it has a thirty-second launch budget
  that no flag raises.** `LAUNCH_TIMEOUT_MS` is a constant in the CLI; `snapshot --timeout` is the
  runtime-init wait, not the launch. With 139-225 MiB free, Chrome took 60.9 s to start for
  `check` and never started for `snapshot`. Lint is pure static analysis and runs regardless, so a
  memory-starved machine can still prove the composition is well-formed — it just cannot prove
  anything about what the frames look like, and the two must not be reported as one result.
  **The failure is not deterministic, though**: once a 2.5 GiB browser extension process was gone,
  the same `snapshot` and the same `check` succeeded at 130-276 MiB free, and the render ran to
  completion. Treat a launch timeout as "this machine, this minute", and retry before recording it
  as a blocked step.
- **A render that falls back to `screenshot capture · software gpu` takes roughly ten times as
  long, and says so.** The line is `BeginFrame did not run. Needs chrome-headless-shell and no
  --resolution upscale.` This 120 s film took 30 m 41 s that way — 13 m 19 s capture plus
  12 m 15 s encode — against minutes on the fast path. The frames are identical either way, so
  this is a wall-clock fact about the machine and not a fault in the composition; record which
  path a render took, or the next person reads the duration as a regression. Dropping
  `--workers 3` to `--workers 2` is the trade to make when free memory is in the hundreds of MiB.
- **Stills for evidence are cheaper and truer out of the rendered file than out of the browser.**
  `ffmpeg -ss <t> -i out/film.mp4 -frames:v 1 out.png` costs nothing, needs no browser, and yields
  a frame of the film that actually shipped — whereas `hyperframes snapshot` re-renders the moment
  and pays the whole browser-launch cost per batch. Snapshot is for looking at a composition
  *before* a render exists; once one does, cut the stills from it.
- **The logo is a light-palette asset on a dark-palette stage.** `assets/icon.png` carries
  `#f4f3ef` as its own background, which is the design system's light `--paper`, so it needs no
  cut-out: a wrapper with `overflow: hidden` crops the PNG's generous margin and the result reads
  as the page's own light surface on the film's dark one.

## The demo video, 2026-10-23

- **A timed element that is not absolutely positioned stacks in normal flow, and every later
  scene renders off-canvas.** HyperFrames keys a clip's visibility off `data-start`; the
  `position: absolute; inset: 0` that makes a clip a *layer* is the author's own CSS, and the
  scaffold's `.clip` rule was not carried into our stylesheet. The first `hyperframes check` came
  back with six `canvas_overflow` and six `container_overflow` findings spread over five scenes,
  each of them really the same bug: scene 6's root was sitting a thousand pixels down the page
  because scenes 1 to 5 were blocks above it. One `.clip { position: absolute; inset: 0 }` cleared
  all twelve. A check finding that names a dozen elements in a dozen places is usually one
  structural mistake, not a dozen layout mistakes.
- **`white-space: pre` on a terminal's body renders the source's own indentation as blank lines.**
  Each recorded line is its own `<span class="line">`, and the newline and leading spaces *between*
  those spans in the HTML are text nodes. With `pre` on the container they are drawn, so every
  terminal came out double-spaced and the long scenes clipped half their output. `pre` belongs on
  the line, which is what has to preserve its own columns; the container must not have it.
- **The page can be embedded as the page, not as a picture of it.** A browser scene holds an
  `<iframe>` of `assets/site/<act>.present.html` sized at exactly 1920×1080 inside a viewport that
  scales the whole block by 0.875. The act's own renderer then computes `--scale` as 1 and lays out
  the page exactly as the gate serves it, so the frame is the real page rather than a redrawing of
  it. It works because a replay page fetches nothing: its own CSP forbids it to connect anywhere,
  so there is no load that can fail mid-render.
- **The layout audit's 28 px reading floor and a verbatim terminal cannot both be satisfied.** The
  longest recorded line in the film is 117 characters; at 28 px monospace that is 1,960 px, wider
  than the frame. 22 px in a 1,740 px window is the largest that fits, so the terminal bodies carry
  `data-layout-ignore` and the storyboard says why. The alternative was eliding words out of
  recorded output to make the type bigger, which is the wrong trade for this repo.
- **No recorded run holds a proving time, so the film does not claim one.** The instruction for the
  video asked that long waits read `proving… (~60 s)`. Nothing in `evidence/` measures the wait, so
  the terminal keeps the recorded line and the cut is marked `— cut —` with no number. The one
  number in the film, "about 35 s of Halo2 verification" in act 8, is `README.md`'s, which recorded
  it. A placeholder duration is a placeholder shown as real.
- **`hyperframes skills update` fails closed when GitHub is unreachable**, and says so rather than
  reporting success. The skills were already installed, so the build went ahead on them; the CLI
  prints "Skills freshness unverified" and that is the honest state to report.
- **Chromium takes longer than thirty seconds to open its debugging port on this machine**, and a
  loop that starts several of them in one shell is killed. One screenshot per command works; the
  measuring harness in `scripts/page-layout-check.mjs` has the same constraint written down.

## Release prep: the broken axis, the docs, crates.io, 2026-10-22 and 2026-10-23

- **Four of a proof's five markers are the circuit's padding, and nothing said so.** A47 was
  written to assert that two proofs of the same coins carry identical markers, and it failed on
  the first run: only `gov_null[0]` matched. The delegation builder pads the bundle to five notes
  with padding nullifiers of its own (the same padding the PIR oracle has to stay live for), so a
  proof of one real note publishes one real marker and four fresh ones. Single use still works —
  `UNIQUE(context, marker)` needs one collision, not five — but `SPEC.md` §4 and LEAKS L4 had both
  been written as "the same coins produce the same markers", which is wrong in a way no previous
  test could catch, because A05 only ever needed the admission to be refused. The test now asserts
  the shared count equals the number of real notes, which is a statement about padding as well as
  about markers. A test written to confirm the documentation is how the documentation got fixed.
- **The leak scan refuses `"//"` in the page's own code, and it is right to.** The broken axis's
  mark is two slashes; written as one string it reads as a protocol-relative URL, which is exactly
  what A02c's absolute-URL scan is for. Building it from two `/` spans keeps both the mark and the
  scan. Do not widen a leak check to fit a glyph.
- **A stub at the end of an axis reads as an edge effect, not as information.** The measured
  layout said act 4 fitted and hid nothing, and it did; a screenshot said a non-engineer sees a
  faded smudge and reads the red sentence or nothing. Breaking the rail with a `//` and marking
  the holder past it with `+20,332 blocks` puts the fact on the drawing. Measuring answers "is it
  all there"; only looking answers "is it read".
- **Widening one column of a three-column row can make the row taller.** Giving the window panel
  more space (`.thirds-holder`) squeezed the rooms panel until its explanation wrapped to three
  lines, and act 4 in presentation mode went 22px past the stage with its last provenance line
  cut. Sweeping `grid-template-columns` live over the DevTools protocol — set the value, read
  `scrollHeight - clientHeight`, try the next — found `1.35fr 1.05fr 0.6fr` in one browser session
  instead of one rebuild per guess. A layout question can be asked of the running page.
- **`cargo publish --dry-run --workspace` is the only thing that finds the packaging errors.**
  `cargo build`, `cargo test` and the gate all pass on a workspace that cannot be published:
  path dependencies with no `version` are the obvious one, and `readme = "README.md"` pointing at
  a file that is not in the crate directory is the one that actually stopped the run. Each crate
  got its own README. The verify step compiles every crate from the packed tarball, so the whole
  run is about fifteen minutes.
- **Renaming a workspace member rewrites `Cargo.lock`.** `unstirred-cli` became `unstirred` (the
  binary's name was already `unstirred`), and the next `cargo metadata --locked` failed until the
  lock was regenerated. CI runs that check first, so the lock has to be committed with the rename.
- **Quoting apostrophes through a patch script.** A `.replace("'", '"')` meant to turn Python
  quotes into TOML quotes also turned "the holder's half" into `the holder"s half`, and every
  manifest in the workspace failed to parse. Build the string with the quotes it needs rather than
  translating quotes afterwards.
- **The gate suite needs about 7 GB free, and the machine it is run on is part of the run.** A
  full `./scripts/gate.sh` was killed for memory part-way through `tests/http.rs`, where several
  K=12 proofs build and run at once. It passes serialised —
  `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=1`, with zebrad, the nf-server and any running gate
  stopped — and the recorded run says so, because "GATE: PASS" means less without the conditions
  it passed under: `tests/http.rs` took 140.89s serialised against 111.43s in parallel, and that
  difference is the only thing the settings change.
- **Chromium needs more than fifteen seconds to open its debugging port on a loaded machine.**
  `scripts/page-layout-check.mjs` polls for 15 s; with a release build running alongside it, the
  browser was not up in time and the script reported "chromium never opened a debugging port"
  while the browser it started kept the port. Run the layout check on a quiet machine, and kill
  the chromium it left behind before retrying, by PID.

## M4 step 2 (act 4 recorded, and the 413 race closed), 2026-10-22

- **The forced proof used one note, not two.** The newcomer's wallet holds two coins, and the
  selection picked the fewest that reach the threshold, which was one (mined 4,526,842; the other,
  4,526,839, was not needed). So act 4's holder line is the span of the coin the proof is about —
  4,526,842 to 4,528,029 — and not the wallet's whole history, which the gate never learns either.
  A line drawn from a selection is a line about that selection; the act's provenance says which run
  printed it.
- **An axis that stretches to reach a holder 21,519 blocks away destroys the window.** `axisOf`
  used to widen to take in a holder line, which was right for a line a few hundred blocks off and
  wrong for act 4: the 720-block window became three per cent of the rail, its h0 and h1 marks
  overlapped into `h1 4,506,51h0 4,505,790`, and the caption pinned to the band's span wrapped into
  a column of one word per line that pushed the page 128px past its 1080. Measuring found the
  overflow; only the screenshot showed why. The axis is the window plus its margin now, whatever a
  holder does, and a line that runs past an end is drawn to that end as a short stub faded off it,
  with the note underneath saying in blocks how far it really goes ("20,332 blocks after h1, never
  inside this window"). An axis break with the real number beside it is honest; a window squeezed
  to a sliver is not readable, which is its own kind of dishonest.
- **The line could not be drawn "from the coins' mined height through h1" as first asked**, because
  those coins are *newer* than h1: `from` would have been greater than `to`, which the page state
  contract refuses outright. It runs from the first block the proof's coin was mined in to the
  block it is still unspent through, which is what the knock prints.
- **`?present=1` is where a long holder label bites.** The label is one line under the rail, and in
  presentation mode it is set in larger type; it wraps rather than clipping, because the rail is in
  normal flow (the lesson from the first review's absolutely positioned caption). Act 4 fits at
  1080 only after three trims measured one at a time: a shorter label, a provenance foot the length
  of the other acts', and the log route's row cap dropped from a replayed page, where it describes
  a route that page never calls. The browser is the only thing that can say which trim was enough.
- **Never kill a process by a pattern that can match the owner's.** Cleaning up one scratch gate,
  `pkill -f "gate serve --config gate.toml"` matched all three gates on this machine and stopped
  the owner's two as well — the real two-room gate on `:8080` and the page gate on `:8099`. Nothing
  durable was lost (the SQLite file keeps the markers and admissions; only the deliberately
  ephemeral recent feeds went), but a pattern kill does not know whose process it found. Kill by
  the PID you started, and confirm a process is yours by `readlink /proc/<pid>/cwd` before touching
  it. The older note above about `pkill -f` matching the shell's own command line is the same
  mistake from the other end.
- **A backgrounded compound command gives you the subshell's PID, not the server's.**
  `cd dir && nohup gate serve … &` then `$!` recorded a PID that was gone a moment later while the
  gate kept the port, so the restart failed to bind and the next measurement silently re-measured
  the old binary. Background the bare command, or read the PID back from the listener.
- **An evidence file is cited before the gate can pass.** Both Bob runs and their header had to
  have `CLAIMS.md` rows written *before* the gate run that covers them, because the claims check
  fails on a recorded run nothing cites. Only the gate log's own row can be added afterwards.

## M4 step 1 (act 4: the knock's local check, and the forced demo), 2026-10-22

- **The nullifier read height is not the witness height, and the PIR server pins the other one.**
  `note_inputs` read every nullifier from the notes the wallet shows unspent at `manifest.window.h1`,
  which a newcomer has none of: those coins did not exist then, so `NoteNotHeldAtThatHeight` stopped
  the forced proof before the circuit. Moving that height into `ProofRequest::notes_unspent_at` was
  the whole of it — and the h1 it reads *cannot* simply be moved forward, because the PIR snapshot
  is pinned: `snapshot::describe` refuses a server whose `/root` height is not the h1 it was asked
  for. That turns out to be exactly right for the demo: the nullifier-set root in the forced package
  stays the one the room pinned at h1, so the only root that differs is the note-commitment root,
  and the gate's refusal is `ROOT_MISMATCH` about that one thing.
- **No wallet a test can build is one the CLI will open.** `SyntheticWallet` is a `LocalNetwork`
  (regtest) wallet, and `zcash_client_sqlite` refuses it under testnet parameters: "Data DB is
  corrupted: UFVK is for network Regtest but we expected Test". A gate manifest can only say `test`
  or `main` (PRD §3.1), and the harness cannot be moved onto testnet's own activations either —
  Ironwood begins at testnet height 4,134,000, and `TestBuilder` generates blocks from activation
  upwards. So the CLI's own tests check the *order* of the steps (the wallet is read, and the rule
  applied, before a challenge is requested) and leave the rule's refusals to the prover's tests,
  which open the same wallet with the parameters it was built with.
- **A test that proves a request was not made needs a canary for the recorder.** "The gate was
  asked for the manifest and nothing else" passes just as well when the listener records nothing at
  all, so A45 makes one challenge request of its own afterwards and asserts it shows up.
- **`include!` resolves a nested `include_str!` relative to the included file.** A registry
  generated beside the act files can therefore name them by bare filename
  (`include_str!("act1-rule.json")`), which is what lets `scripts/replay-from-evidence.py` write
  both the acts and the list of them. rustfmt does not follow `include!`, only `mod`, so a
  generated `.rs` under `ui/` is never reformatted and never fails `cargo fmt --check`.
- **Clippy's `panic = "deny"` covers unit tests.** A `#[test]` that ends a match arm with `panic!`
  fails the gate; `assert!(matches!(…), "…")` says the same thing and passes.
- **The recorded Bob run shows the bug it motivated, and stays as it was.** Its output has the gate
  issuing a challenge and the knock printing "proving now" before the local refusal — the order A45
  now forbids. An evidence file is never edited, so the `CLAIMS.md` row says the run predates the
  fix. (The same session found the claims check already failing, because that run had been added to
  `evidence/` without a row: a recorded run nothing cites is a gate failure by design.)
- **A37 was load-dependent, and the race was the gate's own 413.** A full gate run failed on A37
  with `Unreachable { … /admit, "client error (SendRequest)" }`: the gate refused an over-long
  `Content-Length` without reading the body at all, hyper answered and closed, and the client —
  still writing its 256 KiB — was reset before it could read the answer. Whether it won that race
  was a matter of socket buffers and scheduling, so the target passed alone (7 passed, 97 s) and
  failed under load (139 s for the same target in the failing run). **The fix is at the server**
  (step 2, `handlers/mod.rs`): a body refused on its declared length is now read and thrown away up
  to `DRAIN_LIMIT_BYTES`, four times what the gate would accept, before the 413 is written, so the
  client is never reset mid-send; past that cap it is still left to the close, because a gate that
  reads an unbounded body to be polite is a gate anyone can make read. Nothing drained is kept,
  parsed, verified or stored. What the fix costs is one word of the refusal's own text: "nothing
  was read" became "nothing was parsed, verified or stored", because the gate does now read it —
  and reading a body is not reading it as a package, which is what A37's claim was always about.
  The test stayed strict: no retry, no tolerance of a send error.
- **"proving now" was printed before any proof was built.** The line came from the header, which
  was written when the header was the last thing printed before the proof. Anything that announces
  work belongs where the work starts, not where the output happens to be grouped.

## M3 step 2 (the Holders' Room page), 2026-10-21

- **`GET /` was already a route.** The router trimmed trailing slashes and treated an empty path as
  the room list, so `/` answered the room-list JSON. The picker needed `/` split from `/rooms`, and
  the room pages a prefix of their own (`/room/<context>`), so that no page path can ever be
  mistaken for a path under `/rooms` and no room name can collide with a route leaf.
- **A page that displays the gate's root sources contains an IP address.** A live page prints
  `zebra:http://127.0.0.1:18232` because that is what the operator configured, so A39's "no address
  in a response" scan must not be widened to the pages: it would fail on the gate's own
  configuration. This is also why the A02c scan separates the state block from the code. A root
  source the page *shows* is text; only the code can make a request. Scanning the whole document
  for `https://` would have flagged the gate's own published configuration as a leak.
- **The one `fetch` call takes a variable, so scanning its arguments proves nothing.** The way to
  make "a replayed act reaches no network" checkable was structural: a replay document is assembled
  without `ui/live.js` at all, so `fetch(`, `setInterval(` and the rest are absent from the document,
  and its own `Content-Security-Policy` is `connect-src 'none'`. A test can then assert absence
  rather than intent.
- **`--ink-faint` at `#767c88` is 3.78:1 on `--paper`.** It carries all of the small print (panel
  titles, provenance, the struck-through list), so that is below AA for body text. It is `#616874`
  now, 5.06:1. Contrast has to be computed; no palette that looks reasonable can be assumed to pass.
- **A helper in `tests/gate/` is compiled only into the targets that declare it, and every item in
  it must be used by each of them.** `tests/http.rs` and `tests/page.rs` both declare `mod gate`,
  but each lists only the files it needs, so a page-only scanner cannot live in `support.rs`: the
  HTTP target would fail `-D warnings` on dead code. Unused *fields* count too — a fixture struct
  carrying a `database` nothing reads is a dead-code failure.
- **The page tests build no wallet, so they cost no proof.** Everything about the routes, the pages
  and the feed is observable without a package that verifies: a body that is not a package is
  refused by the engine, which is a verdict, which is all A43 needs. `tests/gate/plain.rs` starts a
  gate with two rooms, injected roots and no wallet, and the whole target runs in about a tenth of a
  second.
- **`ui/` lives inside the gate crate, not at the repository root.** The replay acts and the
  templates are compiled in with `include_str!`, and `cargo publish` packages only the crate's own
  directory, so a repo-root `ui/` would build here and fail from the published crate.
- **The recorded run holds no timestamp for a refusal**, because the store writes nothing for one.
  Act 3's time is derived from that knock's recorded challenge expiry less the 600-second TTL, which
  is the second the challenge was issued. Rather than print a derived number as if it had been
  observed, the page state carries an `at_note` per verdict and the page prints it beside the time.
- **A shell heredoc can eat a single-backslash escape.** `"\u00b7"` written into `ui/render.js`
  arrived as the character itself. That is harmless in a UTF-8 JavaScript file, but a patch written
  against the text as typed then fails to match. Read a generated file back before patching it.

## M3 step 2, visual review in a real browser, 2026-10-21

- **A fixed grid row is a silent clipper, and nothing in Rust can see one.** The first layout gave
  `.stage-room` fixed pixel rows and `.panel` `overflow: hidden`, so a longer act pushed its row
  past its height and the browser quietly cut the rest: at 1326x683, act 3 lost half of "your
  transaction history", the window's last line and "and 1 more". Every Rust test passed, because a
  line box is not something the gate can measure. Rows are `minmax(auto, 1fr)` now, panels clip
  nothing, and `.frame` scrolls if a page ever outgrows the stage, so an overflow is visible rather
  than hidden.
- **`scrollHeight > clientHeight` is not clipping.** Nearly every text node reports two to five
  pixels of it, because the font's line box is taller than `line-height: 1.08`. The first probe
  flagged fifteen elements on a page where nothing was wrong. What matters is overflow on a box
  whose computed `overflow` is not `visible`, plus each leaf's bounding rect against the stage's.
- **This session can see the page: chromium is installed and node 24 has a built-in WebSocket.**
  `scripts/page-layout-check.mjs` drives the browser over the DevTools protocol with no dependency
  at all — `Runtime.evaluate` to measure, `Page.captureScreenshot` to look. A visual review is a
  look, not an estimate, and the estimates in this file's first draft were wrong by 100px.
- **An absolutely positioned element cannot clip, so it overlaps instead — and the probe cannot
  see it.** The window's caption was `position: absolute` over the band. In presentation mode the
  larger type wrapped it to two lines, which grew downwards straight through the band, and the
  layout check reported nothing: the element was inside the stage and its parent hid nothing. It
  is in normal flow now, pinned to the band's span by `margin-left` and `width`, so a wrap pushes
  the rail down. Only a screenshot found this one; measuring is not the same as looking.
- **A claim settled by a browser is not settled by the gate.** `CLAIMS.md` had four statuses and
  none of them fitted: `scripts/gate.sh` needs neither a browser nor a node and gains neither, so
  calling the layout check GATED would say the gate checked something it cannot, and PROVEN would
  claim the chain. Hence a fifth status, MEASURED — a recorded run of a named tool, with its
  version and its command in the evidence file. When a check does not fit the ledger's vocabulary,
  widen the vocabulary; do not borrow the nearest word.
- **`pkill -f "unstirred gate serve"` kills the shell that runs it.** The pattern matches the
  shell's own command line, which contains the whole script. `pkill -x unstirred` matches the
  process name instead.
- **The design-token mirror test paid for itself the same day it was written.** `--type-hero` moved
  from 52px to 46px and `ui/frame.md` still said 52px; the gate failed on exactly that. A document
  that restates a value drifts from it within the hour unless something fails when it does.
- **A `0` computed from one room reads as a finding.** "markers shared between rooms: 0" was true
  and meaningless with one room holding markers and the other empty. A count across a set needs the
  set to exist before it says anything; the page says "nothing to compare yet" until it does.

## M3 step 1 (the HTTP gate, `knock`, `gate init`), 2026-10-20

- **`&impl Clock` cannot take a trait object, and the gate only has one.** The service holds its
  clock as `Arc<dyn Clock + Send + Sync>` so a test can stop it, but `Store::issue` and
  `verifier::admit` took `clock: &impl Clock`, which requires `Sized`. Both now take
  `&dyn Clock`; every existing caller still compiles, because `&SystemClock` and `&FixedClock`
  coerce. A trait-object parameter is the right default for anything a service might inject.
- **`RootSource` is a boxed trait object with no `Send` bound, so it cannot cross into
  `spawn_blocking`.** Neither can `Store`, which holds a rusqlite connection. The gate therefore
  keeps its `--root-source` *specs* (plain `String`s), parses them once at startup to refuse an
  unusable one, and builds both the sources and a fresh `Store` inside each request's blocking
  task. The per-request connection is not a workaround: it is what makes two concurrent
  admissions race through SQLite's UNIQUE constraint rather than through a mutex in this process
  (A05c).
- **Verification has to run on a blocking task.** One admission is a Halo2 verify plus two
  network root reads plus SQLite, all blocking, and the lightwalletd source builds its own
  current-thread runtime and calls `block_on` inside. Called from a reactor thread that panics;
  called from `spawn_blocking` it is fine. The gate runs on a current-thread runtime and does all
  real work in `spawn_blocking`.
- **A non-`Sync` fixture cannot live in a `OnceLock`, so the HTTP tests cannot share one proof.**
  `SyntheticWallet` holds a wallet database, so the `OnceLock` corpus trick the store tests use
  (BUILD_NOTES: one real proof per target) does not apply: a `static OnceLock<T>` needs `T: Sync`.
  Each HTTP test builds its own wallet and pays its own ~50 s proof, and the target's four proofs
  run in parallel across test threads, so the whole target is about 136 s rather than 200.
- **`serde_json::json!` renders `Option::None` as `null`.** `"note": note` put `"note": null` in
  every two-source answer, which reads like a field the holder should look at. `reply::body_with_note`
  inserts the key only when there is a label.
- **A manifest body cannot carry the single-source label.** `parse_manifest` rejects unknown
  fields, so adding `note` to `GET manifest` would publish text this gate's own parser refuses.
  The label goes in the `unstirred-root-note` response header on every route, and in the body of
  every JSON object the gate owns.
- **An HTTP-level refusal must not borrow a code from the closed set.** A 413, 404, 405 or 400
  happens before any package is parsed, so there is no verdict and no code: those answers are
  `{"error": …}`, and the tests assert the body carries neither `"verdict"` nor `"code"`. The body
  limit is checked twice, once against `Content-Length` and once while reading, because a chunked
  body declares no length; `http_body_util::Limited` reports the second case as a
  `LengthLimitError` that has to be `downcast_ref`'d out of a boxed error.
- **Two clippy lints cost one gate run, and `cargo check` sees neither.** `fn serve_as(path:
  &PathBuf)` is `ptr_arg` (take `&Path`), and a helper returning `Result<(), Reply>` is
  `result_large_err`, because a `http::Response<Full<Bytes>>` is 144 bytes in the `Err` slot. The
  second one was the better design anyway: "the 404 for a room that is not here, or nothing"
  is an `Option<Reply>`, not a `Result` whose `Ok` carries no information. Run clippy, not
  `cargo check`, before claiming a crate is clean.
- **`cargo clippy -D warnings` applies the workspace's `expect_used = "deny"` to test code too.**
  A new test written with `.expect("…")` fails the gate even though every assertion passes. No
  test in this repo uses `unwrap` or `expect`; they return `Result<(), Box<dyn Error>>` and use
  `?`.
- **RFC 3339 is written out rather than taken from a date crate.** The challenge expiry is the
  only timestamp the gate puts on the wire, and `rfc3339.rs` converts unix seconds with Hinnant's
  `civil_from_days`, checked against six known instants and every day of a 400-year cycle.
- **A shell variable holding a relative path does not survive the `cd` two lines later.** The
  HANDOFF §8 command block set `U=./unstirred/target/release/unstirred` from `zecathon/` and then
  `cd gate`, so every `$U` in the block pointed at nothing. A path a command block carries across
  a `cd` is written absolute.
- **`git mv` of a command into a directory of the same name needs the module split too.**
  `src/gate.rs` became `src/gate/{mod,serve,init,store_commands}.rs`; the two old subcommands kept
  their text and moved to `store_commands.rs`, so `gate challenge` and `gate markers` behave
  exactly as the recorded runs show.

## Closing M2, and what this pass got wrong, 2026-10-20

- **Never amend a commit without asking the remote whether it has it.** `git rev-parse
  --abbrev-ref @{u}` answers "does this branch have an upstream configured", which is not the same
  question: a branch can have no upstream and still have had its commits pushed. `8398181` was
  reported here as safe to amend on the strength of an unset `@{u}`, and it was already on
  `origin`. The questions that actually answer it are `git branch -r --contains <sha>` and
  `git log <remote>/<branch>..HEAD`. The commit now stands as written, with a `DECISIONS.md` row
  saying what is really in it.
- **Piping `scripts/gate.sh` through `tail` reports the wrong exit code and hides whole targets.**
  `./scripts/gate.sh | tail -60` makes `$?` the exit status of `tail`, so a gate that failed looks
  like it passed, and the earlier test targets scroll out of the capture. CLAUDE.md already bans
  hiding a failure behind `tail`; the exit code is the second half of that rule. Redirect the
  whole run to a file and read the file.
- **A file directly in `tests/` is its own cargo target, including a helper.** A shared
  `tests/seeded.rs` compiles twice: once as a module of the suite that declares it, once as a test
  target of its own, where its `pub` items are dead code and the gate's `-D warnings` fails on
  them. Shared test helpers belong in a subdirectory (`tests/properties/seeded.rs`).
- **A property test needs a coverage assertion or it can test nothing and pass.** A32's first
  generator drew note values up to `u32::MAX` against thresholds around 0.125 ZEC, so one or two
  notes always reached the threshold and the "more than five notes needed" refusal was never
  produced: the oracle agreed on all 2,000 cases while one whole path went unexercised. The
  failure printed `the counts were [230, 599, 0, 1171]`. The fix is a deliberate dust shape, six
  to eight notes of about a sixth of `REQUIRED_MIN_ZATOSHIS` each, all held.
- **Clippy 1.91 rejects `x % n == 0` as `manual_is_multiple_of`.** The production rule in
  `window.rs` already used `is_multiple_of`; the property test restating it with `%` failed the
  gate under `-D warnings`. Restate a rule with the same call the rule uses, not an equivalent.
- **User-facing text that names a milestone goes stale in silence.** `unstirred verify` printed
  "markers not recorded (M2)" for the whole of M2 and kept printing it after M2 closed, because no
  test asserted that line. It is one constant now, printed by both the human and the JSON path,
  and A33 fails if the sentence stops naming `verify` and `admit` or starts naming a milestone
  again. When a milestone closes, grep the user-facing strings for its name.

## Quality pass: adversarial reorg, property tests, CI, 2026-10-20

- **rustfmt and rustc resolve a test target's nested modules differently.** For the target root
  `tests/store.rs`, rustc resolves `mod store { pub mod fixture; }` to `tests/store/fixture.rs`
  (the crate root's own directory, plus the inline module's name). rustfmt instead treats the
  target file's stem as the directory and ignores the inline wrapper, so the two agree only
  because that wrapper happens to be named after the file. Declaring `mod adversarial { ... }`
  inside `tests/verify.rs` compiles, but `cargo fmt` fails with "failed to resolve mod
  `foreign_gate`: tests/verify/foreign_gate.rs does not exist". `#[path = "adversarial"]` on the
  inline module makes both tools resolve `tests/adversarial/`, which is how one attack directory
  is shared between several targets.
- **One real proof per test target, not per file.** Each integration-test target is its own
  process, so a `OnceLock` fixture is built once per target, not once per run. Giving the attacks
  their own target would have rebuilt the verifier's four K=12 proofs (about 170 s) and the
  prover's one (about 55 s) on every gate run; instead each existing target declares the attack
  modules whose fixture it already pays for.
- **`rust-toolchain.toml` pins 1.91, and that pin rebuilds everything once.** The next cargo call
  after adding it makes rustup install 1.91.1 and invalidates every artifact built with the
  previous default toolchain, so the first gate run after the pin rebuilds the whole dependency
  graph in release.
- **Property tests are seeded by case number.** `StdRng::seed_from_u64(seed)` per case, every
  failure collected as `(seed, reason)` and the list asserted empty, so a failure names the case
  and nothing depends on OS entropy. The generators are biased on purpose (70% of windows put h1
  on the nullifier step, 80% put h0 below h1): uniformly random triples almost never make a valid
  window, and a property that only ever exercises refusals proves little. A28 and A32 assert that
  every outcome was actually reached.
- **`git grep`, not `git ls-files | xargs grep`, in the leak scan.** A pattern beginning with `-`
  is read as an option by grep, and with `|| true` on the end the check passed silently while
  scanning nothing. `git grep -e <pattern>` separates "no match" (exit 1) from "could not run"
  (exit above 1), which the scanner now reports as a failure, and it skips tracked files that have
  been deleted from the working tree. Both patterns were then confirmed against a planted canary.

## M2 step 2 (challenge and marker store), 2026-10-17

- **BLAKE2b personalisation is 16 bytes.** `unstirred/chal/v1` is 17 and `blake2b_simd` panics on
  a longer personalisation. The hash uses `UnstirredChal_v1`. The raw challenge is what the
  signature binds; the file stores only this hash.
- **A consumed challenge is allowed through the read-only check.** Markers are inserted before
  the consume, inside `BEGIN IMMEDIATE`. Replaying an admitted package therefore hits
  `UNIQUE(context, marker)` and comes back `ALREADY_USED_IN_CONTEXT`, and the transaction rolls
  back. Rejecting "consumed" before the proof would hide that replay behind
  `CHALLENGE_UNKNOWN_OR_EXPIRED`. A consumed challenge whose five markers are all new fails the
  `UPDATE` (zero rows) and is `CHALLENGE_UNKNOWN_OR_EXPIRED`, also rolled back. Unknown, expired
  and wrong-context challenges are refused before the proof. `expires_at > now`, so the instant
  of expiry is already expired. TTL is 600 seconds.
- **rusqlite 0.37 turns on extended result codes in `open`.** A unique violation is
  `SQLITE_CONSTRAINT_UNIQUE` (2067), not primary code 19. The store also treats
  `ConstraintViolation` whose message contains `UNIQUE` as the marker conflict, so a build that
  did not enable extended codes still maps to `ALREADY_USED_IN_CONTEXT`. A `NOT NULL` failure
  does not contain that word and stays a store error.
- **`busy_timeout` is per connection.** WAL persists in the file; the timeout does not. A second
  connection sees `journal_mode=wal` and rusqlite's own default (5000ms) until something sets it.
  The store sets 10s on the connection it opened. Read `PRAGMA busy_timeout` on that connection.
- **Dropping an uncommitted `Transaction` rolls it back.** A05d returns before `commit` and does
  not call `rollback` itself. The interrupted challenge stays unconsumed and its markers are absent.
- **The five markers of one proof are not a set you can compare across proofs blindly.** The real
  note's marker is stable for the same note and the same context, which is why a second proof of
  the same coins collides. The padding slots use fresh nullifiers, so the other four markers
  differ every proof. A different context changes the round id, so all five differ. A05's test
  asserts the two same-context proofs actually share a marker.
- **An integration test of the verifier cannot name `zcash_client_sqlite`.** It is not a direct
  dependency. The account type has to be inferred, which is why the three-proof fixture proves
  through a closure instead of a helper with a written signature.
- **The WAL sidecar is `<path>-wal`.** `PRAGMA wal_checkpoint(FULL)` folds it into the main file.
  A dump test has to read both, and must not search the bytes for a short note position: those
  integers show up inside unrelated encodings. Compare the integer columns with the position and
  the value, and search the bytes for the raw challenge, its hex, a long slice of the proof and
  the note value when that decimal has at least six digits. The marker blobs are supposed to be
  in the file.
- **On 2026-10-17 the owner's Zebra was still syncing, and the tip moved during the session.**
  `getblockcount` was 193,200 in the morning note, then 1,675,600, 1,791,200, and 1,836,400 at
  the start of the live admit (`evidence/2026-10-17-admit-two-rooms.txt`). All of those are
  short of h0 4,505,790, so that admit used one lightwalletd source.

## M2 step 1 (two-source root authority), 2026-10-17

- **The Zebra `z_gettreestate` shape was taken from source, not memory:** zebra-rpc 7.0.0-rc.0,
  `zebra-rpc/src/methods/trees.rs`, type `GetTreestateResponse` — `hash`/`height`/`time`, optional
  `sprout`, then `sapling`, `orchard` and optional `ironwood` (only present from NU6.3, omitted
  otherwise), each a `Treestate { commitments: Commitments }` whose `Commitments` serializes
  `finalRoot` and `finalState` as hex strings. The network name is the BIP70 `chain` field of
  `GetBlockchainInfoResponse` in `zebra-rpc/src/methods.rs`: z_gettreestate carries no network
  field, so the network check is a separate `getblockchaininfo` call.
- **The root is parsed from `finalState`, not `finalRoot`.** Both fields come from the same
  server, so they cannot cross-check each other; going through `finalState` (the zcashd-compatible
  `CommitmentTree` serialization) lets the Zebra source reuse the lightwalletd parsing path, so
  height, network and empty-tree rules stay one code path. Byte order also differs: Zebra's
  `finalRoot` is `bytes_in_display_order`, not the frontier `to_bytes()` order the lightwalletd
  path produces.
- **The owner's Zebra node (7.0.0-rc.0, `http://127.0.0.1:18232`, no cookie auth) answered
  `getblockcount` → 193200 on 2026-10-17** — short of h0 4,505,790, and Ironwood only exists from
  testnet height 4,134,000, so the live node cannot serve a fixture for h0 today. The Zebra-shape
  fixtures are derived from Zebra's own response types, plus one real snapshot body (Zebra's
  `z_get_treestate_by_hash@custom_testnet.snap`, height 0, no `ironwood` field).
- **An unusable source makes the whole root check INCONCLUSIVE, and "unusable" includes an honest
  tip below h0.** Unreachable, malformed, wrong height, or tip < h0 all yield
  `ROOT_SOURCE_UNAVAILABLE`; only answers for h0 with tip ≥ h0 are usable, those must agree on the
  root exactly, and the governing tip is the lower of the answers. This is why an unsynced Zebra
  produces INCONCLUSIVE rather than a misleading `REFUSED / WINDOW_INVALID`.
- **An empty `ironwood_tree` field is now refused.** It used to parse silently as the empty tree's
  root; both sources now report `ROOT_SOURCE_UNAVAILABLE` for it.
- **`CommitmentTree::from_frontier` needs incrementalmerkletree's `legacy-api` feature.** The
  verifier declares it as an explicit dev-dependency feature rather than relying on feature
  unification from another crate.
- **`&Vec<T>` → `&[T]` unsize coercion does not drive `.collect()` inference.** Passing `&vec`
  where `&[SourceOutcome]` was expected left `collect()` without an element type; annotate the
  `Vec` (`let per_source: Vec<SourceOutcome> = …`).
- **The single-source note and `verdict_object` live in the verifier library**, so the A27 test
  asserts the exact text the CLI prints ("ROOT FROM ONE SOURCE: not independently checked") in
  both the human output and the JSON verdict object.
- **`CommitmentTree` and `Frontier` are not re-exported by `zcash_primitives`**; import them from
  `incrementalmerkletree::frontier`. `Frontier::try_from(NonEmptyFrontier)` returns `FrontierError`,
  which implements neither `StdError` nor `Display`, so `map_err` has to format it with `{e:?}`.
- **The empty-`ironwood_tree` refusal fires in the conversion into the lightwalletd `TreeState`**
  (`proto_tree_state`), before `ironwood_root_at` runs — tests for it must assert on the
  conversion's outcome, not the later root check.
- **Zebra's JSON-RPC is plain HTTP POST via hyper-util's legacy client, not gRPC** (the lwd source
  is tonic); `ZebraSource` accepts `http://` only.
- **Clippy `useless_conversion` cost one gate run:** `.map_err(String::from)` on a
  `Result<_, String>` is a redundant conversion; use the error as-is.

## M1 step 5 (`unstirred verify`), 2026-10-17

- **A shared synthetic harness lives in the prover lib behind a `test-harness` feature.**
  `crates/unstirred-prover/src/testing/` is the old `tests/harness/` moved into the library so the
  verifier's tests build the same synthetic wallet instead of a copy. The feature is off by
  default, so `cargo test -p unstirred-prover` alone no longer compiles the integration tests:
  they need `--features test-harness` (the zcash crates' `test-dependencies` pattern), and
  `scripts/gate.sh` now runs clippy and tests with `--all-features`. The verifier enables it as a
  dev-dependency, which is ordinary feature unification and needs no workspace flags.
- **`redpallas::Signature` converts to bytes by reference only.** `[u8; 64]::from(signature)` does
  not exist; `[u8; 64]::from(&signature)` does (orchard 0.15.5).
- **`SpendAuthorizingKey::randomize` takes the randomizer by reference** (`&pallas::Scalar`).
- **A one-byte proof flip must reach step 8 to prove the ordering.** The binding message covers
  the instance, session and challenge but not the proof bytes, so a tampered proof passes
  signature verification and is refused at the circuit with `PROOF_INVALID`; a tampered *instance
  root* dies earlier at step 5 with `ROOT_MISMATCH`. Tests that flip one hex digit (keeping the
  field parseable) assert the code, not just the verdict kind, so ordering regressions show up.
- **The CLI parses the manifest before the root fetch, not the engine's `verify_package`.**
  Step 2 needs the tip and the fetch needs h0 and the network, so the command line runs core's
  `parse_manifest` first (a malformed manifest is refused before any network) and then calls
  `verify_parsed`; `verify_package(json, …)` exists for callers that have only text. A root
  source that fails is `INCONCLUSIVE / ROOT_SOURCE_UNAVAILABLE` and is reported before any
  verdict, which is the task's "any root-source failure" rule.
- **`unstirred verify` cannot test the real chain in unit tests.** The one network-touching
  test (`A26`) points at `127.0.0.1:1`, where nothing listens; every other check is pure over
  the injected roots and tip.
- **`PublicInstance::decode` can surface `BAD_SIGNATURE` on its own.** The rk in the instance is
  rebuilt as a curve point, so a package whose rk is not a valid point is refused at step 3's
  decode, before the signature check; the engine's ordering inherits this core behavior.
- **After a session interruption the shell's cwd can reset to the home directory.** A background
  `./scripts/gate.sh` started from there dies instantly (no such file) and the output never
  appears; check pwd or use absolute paths before long runs.

## M1 step 4 (prove, sign, package), 2026-10-16

- **The delegation builder queries the IMT provider for its own padding nullifiers.**
  `build_delegation_bundle` fills the unused slots of the five with synthetic zero-value notes and
  calls `imt_provider.non_membership_proof(padding.real_nf)` for each one, against the real root.
  A pre-fetched map of the selected notes' proofs is therefore not enough: the provider has to be
  live during the build, which is why `PirNullifierOracle` holds the client and caches answers.
  One note selected means four extra PIR queries per proof.
- **`pir_client::fetch_proof` returns `anyhow::Error`.** There is no typed variant, so "this
  nullifier is in the set" and "the server stopped answering" cannot be told apart from the error.
  The prover reports that case as a local error that claims no verdict, and keeps
  `PIR_UNAVAILABLE` for the failures it can identify (the `/root` handshake and the connection).
- **The `zcash_client_backend` test harness derives its account from 32 zero bytes**
  (`Secret::new(vec![0u8; 32])` in `data_api/testing.rs`), which is not a BIP 39 seed. A harness
  wallet can never be driven through the age-encrypted-mnemonic path, so the identity layer is
  tested against the published all-zero-entropy BIP 39 vector instead, and the synthetic proof
  test takes the harness's own spending key through `spend_authority_for`.
- **`orchard 0.15.5` key types do not zeroize on drop.** `SpendingKey`, `SpendAuthorizingKey` and
  `FullViewingKey` implement no `Drop` and no `Zeroize`, so "zeroize after use" can only mean the
  buffers this crate owns: the decrypted mnemonic and the seed are `secrecy::SecretVec` and are
  zeroized, and the spending key is derived inside one function, never copied to bytes, never
  printed and never written. A heap copy of the key may outlive the proof; that is the library's
  behaviour, not something this crate can fix without forking it.
- **`Instance::from_parts` rebuilds `rk` from `rk_x` plus the parity of `rk_y`.** A package that
  carried a wrong `rk_y` with the right parity is silently normalised to the real one, so the
  wire format effectively carries x and a sign bit. That is not a soundness hole (the proof pins
  both coordinates), but it means a round-trip test cannot detect a tampered `rk_y` of the same
  parity.
- **`bip0039::Mnemonic`'s `Debug` prints the phrase.** The struct itself holds `Zeroizing<String>`
  and `Zeroizing<Vec<u8>>`, so it does wipe itself on drop, but `#[derive(Debug)]` on anything that
  holds one would print the holder's mnemonic. Keep it a local, never a field.
- **`age 0.12.1` brings its own `secrecy 0.10.3`, which does not unify with the zcash cohort's
  `secrecy 0.8.0`.** Both are in the lock. Our own secret buffers use `secrecy` (0.8, the one the
  zcash crates use), and anything handed back by `age` has to be read through `age::secrecy`:
  `age::x25519::Identity::to_string()` returns a 0.10 `SecretString`, so `ExposeSecret` has to
  come from `age::secrecy` or it will not resolve. `age` also drags in `i18n-embed` and a second
  `toml` (0.5.11, through `find-crate`) as proc-macro-time dependencies.

## M1 step 3 (private nullifier lookup), 2026-10-16

- **A spent nullifier shows up two different ways, depending on parity.** In the K=2 punctured-range
  tree each nullifier is either an outer boundary of a leaf or that leaf's `nf_mid`. If it is a
  boundary, `pir_client::fetch_proof` *errors* ("nullifier not found in Tier 1 leaf scan"), because
  no leaf interval contains it. If it is `nf_mid`, a proof comes back and fails `verify`. So the
  lookup has to treat both a per-note fetch failure and `proven_unspent == false` as "spent"; only
  one of them is reachable for any given note, and which one is not predictable.
- **`voting-circuits 0.12.2` cannot verify an IMT proof out of circuit.** `protocol_hash` is a
  private module, `ImtProofData` has no `verify`, and `build_nullifier_list` is behind
  `unstable-internal-api`. The only public Poseidon is `gov_null_hash` (a `poseidon_hash_3` with a
  misleading name) and `derive_nullifier_domain` (fixed first argument). Out-of-circuit checking
  therefore goes through `imt-tree`'s `ImtProofData::verify`, which `pir-client` re-exports. Both
  crates hash with `halo2_gadgets` `P128Pow5T3`, so the two agree.
- **`pir-export` is not on crates.io and never will be from a release.** `cargo publish` strips
  path dev-dependencies, and it strips their feature references too: published `pir-client 0.7.4`
  has no `pir-export` dev-dep and its `upstream` feature lost the `pir-export/upstream` entry. So
  the fixture helpers `build_ranges_with_sentinels` / `build_pir_tree` / `export_for_layout` that
  `pir-client`'s own tests use are unreachable from a registry pin. That is why A04 builds its
  nullifier set with `voting_circuits::delegation::SpacedLeafImtProvider` instead of real tier
  blobs. Real tier blobs in a test would need a git or path dependency on the checkout.
- **`orchard::note::Nullifier::inner()` is only public under `unstable-voting-circuits`**, which
  `voting-crypto-deps/lrz-orchard` happens to enable for the whole graph. Relying on that is
  relying on feature unification from another crate; `to_bytes()` plus `Fp::from_repr` gets the
  same field element without it.
- **`pir-client` does not check the network or the height.** `with_transport` validates the pool,
  the dataset version, the tier layout and the YPIR shape, and nothing else. A server synced to
  another height, or on mainnet, would answer happily about the wrong `h1`.
- **The compiled layout is depth 19 = 12 public + 7 private, `poly_len` 4096**
  (`COMPILED_PIR_LAYOUT`), and the client refuses a server advertising anything else. Treat it as a
  protocol version, not a tuning knob.
- **Next step for a live lookup:** build `nf-server` from `v0.12.3` (`ac15ed2`), and re-export
  `pir-data` with `--features upstream`. The snapshot now in `../vote-nullifier-pir/pir-data/test`
  (height 4,506,510) was exported at HEAD with the default `zakura` features; whether its roots
  match the upstream cohort is untested.
- **Two concurrent `cargo test` runs stall `gate.sh` silently.** The second one waits on the build
  lock, and if its output is piped through `tail` nothing is printed at all, so the gate looks hung
  at `== clippy`. Kill stray cargo processes before running the gate.
