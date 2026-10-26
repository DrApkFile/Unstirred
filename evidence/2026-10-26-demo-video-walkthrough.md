# The demo video, rebuilt as a 2:00 walkthrough: what ran

What this file settles: that the film in `video/` was rebuilt on 2026-10-26 from a 2:27 silent cut
into a 2:00 narrated walkthrough, that its narration is a recorded run of a named tool, that its
composition check is a recorded clean pass, and that **the render ran** and produced the file whose
hash is below.

It says nothing about the Zcash network. The chain claims in the film are the recorded runs it
re-enacts, each one named on screen and each one already in this ledger. The earlier cut's render
is [evidence/2026-10-23-demo-video-render.md](2026-10-23-demo-video-render.md), which stays as the
record of that run; `video/` no longer produces that film.

| | |
|---|---|
| Date | 2026-10-26 |
| Tree | the working tree at `d2dfc12` plus the video work, **uncommitted** |
| Tool | HyperFrames CLI **0.8.186** (the project's pin; `npx hyperframes --version` reports 0.8.200 as latest) |
| Node | v24.19.0 |
| Machine | this one; 7.5 GiB of memory, **130–276 MiB free** and 5.6–8.4 GiB in swap throughout |

Nothing was run against either wallet to make this file.

**The project's CLI pin was left at 0.8.186 deliberately.** `npx hyperframes@latest upgrade
--project . --check` reports that the pins would move 0.8.186 → 0.8.200, and the rule for taking a
bump is to apply it and then verify with `hyperframes check`. At the time the bump was offered a
full check could not run on this machine, so the bump could not be verified and was not taken; the
project stays on the version its earlier render was made with. A later check did pass (below), so
this is a bump that can now be revisited — on a machine with room to re-render afterwards, since a
pin change the render has not been re-run against is not verified for delivery.

## What the film is now

2:00 — 120.000 s, 3,600 frames at 30 fps — at 1920x1080, planned in `video/STORYBOARD.md` before
it was built. Twelve scenes of two kinds, each labelled on screen:

- **Demo** scenes re-enact a recorded run and carry `re-enactment of a recorded testnet run ·
  evidence/<file>`: acts 1 to 5 of the Holders' Room, and the knocks behind them.
- **How-to** scenes carry `how to use it` and say where their text came from: installing the
  binary, running a door, the gate's HTTP API, and the two crates as libraries.

Every command in a how-to scene is the real one from `README.md`, `.github/workflows/release.yml`
or the CLI's own usage text. Every output line is what the program actually prints: the `/rooms`
body is the recorded one from `evidence/2026-10-20-two-room-knock.txt`, `unstirred 0.1.0` is this
binary's own answer to `--version`, and `{"verdict":"ADMITTED"}` is exactly what
`unstirred_verifier::verdict_object` writes. **The one block with no recorded value behind it —
the challenge response — is drawn as a shape**, in the film's "not settled" colour, with
`the shape (PRD §3.3) — no recorded value is pasted here` on the line beside it, rather than a
challenge hex from one run set next to an expiry from another. No address, seed, key or note value
appears anywhere in the film.

Three beats of the 2:27 cut were dropped to pay for the four how-to beats, and `STORYBOARD.md`
says which and why. The two retired compositions are kept beside the live ones as
`compositions/*.html.retired`.

## The narration — a recorded run

```
cd video
HYPERFRAMES_PYTHON=~/.venv/bin/python \
  npx hyperframes@0.8.186 tts "<line>" --voice af_heart --speed 1.0 --output assets/vo/<id>.wav --json
```

Thirty-two lines, 250 words, one `.wav` each in `video/assets/vo/`, all `{"ok":true}`, voice
**`af_heart`** (Kokoro-82M, run locally, `"lang":"en-us"`, `"langApplied":true`). The voice covers
**81.7 s of the 120 s**. Each line's caption is that line word for word and carries the same
`data-start`, so a caption cannot drift from the voice; a caption holds until the next line speaks,
so text never leaves the frame.

`video/assets/vo/narration-full.wav` is those 32 files laid onto one 120 s track at their film
times, assembled with `ffmpeg` (one `adelay` per line into one `amix`) and meant to be transcribed.

**The transcription did not run.** `npx hyperframes@0.8.186 transcribe
assets/vo/narration-full.wav --output transcript.json` reports `Captions skipped — whisper-cpp not
found`, and the suggested `hyperframes models install parakeet` fails with npm
`EALLOWSCRIPTS`: the sherpa-onnx runtime needs install scripts, which this npm refuses for a
project-scoped install unless `allowScripts` is added to `package.json` or `.npmrc`. Granting a
native package permission to run install scripts in this project is the owner's decision, so it
was left alone, and `video/transcript.json` does not exist. **The captions do not rest on it**:
each caption is the exact string that was handed to the TTS engine, carrying the same `data-start`
as the `.wav` that string produced, so they are in sync by construction rather than by alignment.
`video/STORYBOARD.md` prints all 32 lines with their in-points and measured lengths.

**`HYPERFRAMES_PYTHON` is load-bearing on this machine.** `kokoro-onnx` is installed in
`~/.venv`, not in the system python, and the CLI probes it with a 10-second `import kokoro_onnx`.
Under this machine's swap that import took up to 2 m 28 s cold and about 3.8 s warm, so the probe
timed out unless `kokoro_onnx` had just been imported. The generator script imports it immediately
before each `tts` call for that reason. Two of the thirty-two lines failed the probe anyway and
were re-run.

## The composition check — a clean pass

```
cd video && npm run check
```

Run against the delivered tree, after the five findings below were fixed:

| Pass | Result |
|---|---|
| Lint | `0 error(s), 1 warning(s), 0 info(s)` |
| Runtime | `0 errors, 0 warnings` |
| Layout | `0 issues across 9 sample(s)` |
| Motion | `0 errors, 0 warnings` |
| Contrast | `9/9 text checks pass WCAG AA` |
| Craft | `0 errors, 0 warnings` |

`Check passed`. The one remaining warning is `composition_file_too_large` on `index.html`
(336 lines): the caption track and both sound tracks live in that one file deliberately, so the
whole caption pass and the voice can be read against each other in one place. **The `film_silent`
warning the 2:27 cut carried on every run is gone**, because the film now has sound.

Seven findings were fixed rather than carried:

- `duplicate_audio_track` — the two music clips overlap for their crossfade, so they are on two
  track indices (12 and 13) instead of one.
- `film_silent` — resolved by the narration and the bed.
- `clip_media_fit` x32 — each voice clip's slot had been padded 0.06 s past its `.wav`. The render
  shortens the slot to the media length anyway, so the padding bought nothing and was removed.
- `container_overflow` on `#s12-logo > img` — the close logo is cropped on purpose; the two logo
  images now carry `data-layout-allow-overflow`, which says so rather than silencing it.
- `text_below_floor` on `#s2-caveat` — the releases-page caveat is reading copy, so it was raised
  from 26px to the 28px floor.
- `line_unread` on the prover line in `s9-library` — 16 words on screen for 3.07 s against a
  3.97 s floor. The line was cut to "Read-only; no transaction." and brought in 0.9 s earlier.
- `text_below_floor` on `#s11-source` — a provenance line, like every `window-prov` in the film,
  so it was marked `data-layout-ignore` and given 0.9 s more on screen.

## The render — a recorded run

```
cd video
npx hyperframes@0.8.186 render --output out/unstirred-demo.mp4 --fps 30 --quality delivery \
    --workers 2 --skill general-video
```

Exit 0. `--workers 2`, not 3: free memory was 188 MiB at launch and fell to 130 MiB during the run,
so wall time was traded for headroom.

| | |
|---|---|
| File | `video/out/unstirred-demo.mp4` |
| Duration | **120.000000 s** — the 2:00 limit, exactly |
| Frames | 3,600 at `30/1` fps |
| Picture | h264, profile High, 1920x1080, 873,680 b/s |
| Sound | aac, profile LC, 48,000 Hz, 2 channels, 200,486 b/s |
| Size | 16,218,341 bytes, 1,081,222 b/s overall |
| sha256 | `9d1d7acb820e2343119fa8d818f3859da78c976d3a92f3522f4a86c081522399` |
| Wall | 30 m 41.3 s |

Two streams, h264 and aac. The 2:27 cut had one, because it was silent; the second stream is this
cut's narration and music bed, in the file.

Stage times, as the renderer reported them: compile 17.2 s, audio 40.5 s, probe 3.7 s, setup
3 m 20.7 s, capture 13 m 19.0 s, encode 12 m 15.2 s, assemble 23.5 s.

**This was a slow path, and the slowness is the machine, not the film.** The renderer fell back to
`screenshot capture · software gpu` and said why: `BeginFrame did not run. Needs
chrome-headless-shell and no --resolution upscale.` On a machine with memory to spare and
`chrome-headless-shell` present, the same project renders in minutes. The frames are deterministic
either way, so the fallback changes the wall clock and nothing else — but anyone re-running this
should expect either half an hour or a few minutes depending on which path they get, and should not
read half an hour as a fault in the composition.

## The stills

`video/out/stills/` holds twelve PNGs, one per scene, named `NN-<scene>-at-<t>s.png`. They were cut
out of the delivered `.mp4` with `ffmpeg -ss <t> -i out/unstirred-demo.mp4 -frames:v 1`, so each
still is a frame of the film that shipped rather than a separate browser capture of the same
moment. The times are one per scene, each late enough that the scene has resolved: 4.8, 19.2, 30.5,
44.0, 58.0, 76.8, 84.5, 95.0, 101.0, 106.0, 111.0, 118.0.

All twelve were looked at. Every scene carries the label it should, the captions are on screen, the
challenge block is the amber shape with its disclaimer beside it, the uses panel carries
`uses · illustrative, not measured`, and the close carries `replay site: — (not published yet)`
rather than a link to a site that does not exist.

The eleven stills of the 2:27 cut, and the two contact sheets made while reviewing this one before
the render, are kept at `video/out/stills/pre-render-review/`.
