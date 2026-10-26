---
workflow: general-video
flow: automation
storyboard: yes
format: 1920x1080
fps: 30
length: 2m00s
message: "A door can learn that you held, long enough, without learning who you are — and here is how to run one."
audience: "Zcathon judges, and anyone deciding whether to trust a holder-only door"
destination: "repository README, submission page"
audio: narration (local Kokoro, af_heart) + music bed (assets/music.mp3)
---

# Brief — the Unstirred demo video

A motion-design walkthrough of how Unstirred is used: designed terminal scenes where the real
commands type themselves and their real output appears, and designed browser scenes of the real
Holders' Room page. Two minutes, narrated, with captions that are the narration word for word and
a music bed under it.

**Two kinds of scene, each labelled on screen.** *Demo* scenes re-enact a recorded run and carry
`re-enactment of a recorded testnet run · evidence/<file>`. *How-to* scenes show a judge or a
developer how to run it and carry `how to use it`, plus where their text came from: the commands
are the real ones from `README.md`, the release workflow or the CLI's own usage text, and any
output is what the program actually prints — from `evidence/` where a run recorded it, otherwise
the exact shape the code or `briefing/PRD.md` defines, marked as a shape and never given an
invented value.

## The honesty rule

This is a **re-enactment of recorded runs**, not an invention.

- Every command and every output line on screen is copied verbatim from a file in `evidence/`.
- Only timing is compressed. A compressed wait is marked on screen with `— cut —`; nothing is
  retimed silently.
- Long output is shortened only with a visible `…`.
- Every scene that shows a run carries a corner label:
  `re-enactment of a recorded testnet run · evidence/<file>`.
- The four use cases are labelled **uses** and are illustrative, not evidence. They get no fake UI.
- A how-to scene never shows a value no run produced. The challenge response is drawn as a shape,
  in the film's "not settled" colour, with that said on the line beside it.
- No address, seed, key or note value appears. Nothing is taken from wallet history output. The
  owner's home path is elided as `…/` wherever a recorded line contains it, which is the one
  edit made to a recorded line and is visible as an elision.

One deviation from the instructions, deliberately: the brief asked for long waits to read
`proving… (~60 s)`. No recorded run holds a measured proving time, so the terminal keeps the
recorded line (`proving now; this takes seconds and reveals nothing to the gate but the package`)
and the cut is marked `— cut —` with no invented number.

## Assets

- `assets/icon.png` — the lockup, in the hook and the close only. Its own background is the
  design system's light `--paper` (`#f4f3ef`), so it sits on the dark stage as a light plate.
- `assets/music.mp3` — the owner's bed, ducked under the voice and faded out at the end.
- `assets/vo/*.wav` — 32 narration lines, Kokoro-82M (`af_heart`, speed 1.0), generated locally by
  `npx hyperframes tts`. Generated; not carried in the repo.
- `assets/site/*.html` — the recorded acts as static pages, written by
  `scripts/export-replay-site.sh video/assets/site` from the gate's own page module. The browser
  scenes embed these files in an iframe at exactly 1920×1080, so the page inside the browser frame
  is the page the gate serves, at its own scale 1, not a redrawing of it. Generated, gitignored.
- No fonts are fetched, and no third-party asset of any kind is used.

## Design

`crates/unstirred-gate/ui/frame.md` is the design truth, as it is for the page. The film uses that
system's **dark** values for its own chrome — the desk, the terminal window, the cards — and shows
the light page inside a browser frame, so both palettes on screen are the page's own.

## Customizations

- One designed terminal component and one designed browser-frame component, reused by every scene.
- One caption track and two sound tracks in `index.html`, so the voice and the words on screen can
  be read against each other in one file.
- An animated cursor in the browser scenes.
- All motion on one paused GSAP timeline per composition, seekable, nothing timer-driven.
