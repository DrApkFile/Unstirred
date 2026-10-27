# How `index.html` is generated

`video/index.html` is **generated**, not hand-written. Edit these and re-run; do not hand-edit the
caption, audio or music clips in `index.html`, because the next run overwrites them.

One row in `narration.json` — `[id, scene, anchor, text]` — produces three things that must agree:
the caption, the `<audio>` clip, and the music's duck window. That is why they come from one table.

```
python3 video/tools/build_index.py   # rewrites video/index.html from narration.json + the ANCHOR dict
cd video && npm run check
```

Every path in this directory is derived from the script's own location, so these run from any
working directory. Verified 2026-10-26 for the 2:00 cut: run from the repo root, from `tools/` and from `/tmp`,
`build_index.py` reproduced the delivered `index.html` byte for byte. The 1:30 re-cut of
2026-10-27 changed `TOTAL`, `SCENES`, `ANCHOR` and the music lane (one clip, not two), so that
hash no longer applies; the generator is still the only thing that writes `index.html`.

`build_index.py` holds the nine scene mounts (`SCENES`), the film length (`TOTAL`) and the measured
in-point of every line (`ANCHOR`). It reads each `.wav` in `assets/vo/` for its true length and **refuses to write** if a
line runs into the next one or past the end of its scene — that check caught four bad anchors
before anything was rendered. The last line only has to end by `TOTAL`.

## Regenerating the voice

`tts.sh` loops `narration.json` through `hyperframes tts` (Kokoro-82M, voice `af_heart`, speed
1.0), skipping lines whose `.wav` already exists.

**It imports `kokoro_onnx` in the same shell immediately before each call on purpose.** The CLI
probes for the package with a ten-second `import`, and on a swapping machine a cold import takes
minutes, so the probe times out and the CLI silently falls back to system python and reports the
package as missing. Keeping the pages warm is what makes `HYPERFRAMES_PYTHON` actually take.

Changing a line's text changes its measured duration, so **re-run `build_index.py` after any
`tts.sh` run** — `index.html` hardcodes each line's length. That is also why `assets/vo/*.wav` is
committed rather than gitignored.

`build_narration_track.py` lays all the lines onto one `assets/vo/narration-full.wav` the length of
the film, with ffmpeg. It was written to feed a transcription pass, and **that pass is closed as not needed**
(2026-10-26, owner's decision): a caption is the exact string handed to the TTS engine, on the same
`data-start` as the `.wav` that string produced, so the words on screen and the words in the ear are
in sync by construction and need no alignment to confirm it. Nothing in the film reads
`narration-full.wav`; it is gitignored, and the script is kept only as the read-back of what the
voice says and when.

## What `index.html` needs that is not generated here

- `compositions/s1-hook.html` … `s9-close.html`, `assets/film.css`, `assets/film.js`,
  `assets/icon.png`, `assets/music.mp3` — hand-written and committed. No generator.
- `assets/vo/n01.wav` … `n23.wav` — `tts.sh`, and committed, because `index.html` hardcodes each
  line's measured length. **`tts.sh` passes `--json` to `hyperframes tts` and that is load-bearing**:
  without it the CLI's interactive output reads this loop's own line list from stdin and every line
  after the first is silently skipped.
- `assets/site/<act>.present.html`, which the gate browser scenes hold in an `<iframe>` —
  **`scripts/export-replay-site.sh video/assets/site`** at the repo root, not a tool in here. It
  runs `crates/unstirred-gate/examples/export-replay-site.rs`, the gate's own page module, and
  `assets/site/` is gitignored, so a fresh clone must run that export before the browser scenes
  have anything to show. Verified 2026-10-26: a fresh export matches all eleven files byte for
  byte.

- `assets/site-pages/**` — the published site's own pages, which **scene 8**'s browser frames hold
  in an `<iframe>`: copied out of `site/` after `scripts/build-site.sh` at the repo root
  (`index.html`, `styles.css`, `icon.png`, `favicon.png`, `replay/act2-admitted.html`,
  `guides/getting-started.html`, `api/index.html`). Gitignored, so a fresh clone must build the
  site and copy them again before scene 8 has anything to show.
