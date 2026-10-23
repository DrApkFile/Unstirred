# Layout check: every page, after act 4's broken axis

Commit `3f79c4e` plus the uncommitted release-prep tree: a holder the window's axis does not reach
is no longer a faded stub at the end of the rail. The rail is broken instead — the window keeps
its own scale, a `//` mark sits after h1, and past it the holder is a dot in the refusal colour
carrying the real distance (`+20,332 blocks`), with the sentence under the rail reading "the
newcomer's coins arrive here: 20,332 blocks after h1, never inside the window" and a line under
that saying both blocks were read from the holder's own wallet. The window panel takes more of the
row whenever a holder is drawn on it.
Date: 2026-10-23T00:16:46Z. Browser: Chromium 153.0.8010.52 (Arch Linux), headless, driven over
the DevTools protocol by `scripts/page-layout-check.mjs` on Node v24.19.0.
Command: `node scripts/page-layout-check.mjs http://127.0.0.1:8123 holders-room second-room`
against a gate started with `unstirred gate serve` over two `gate init` manifests
(`test` / h0 4,505,790 / h1 4,506,510), an empty database and two root sources no page ever reads.
Exit: 0.

```
LAYOUT CHECK: nothing clips, nothing is hidden
```

Twenty-eight page variants: the picker, a live room page and all five acts, each of them again
with `?present=1`, at the video's own 1920x1080 and at 1326x683. Act 4 at 1920x1080 with
`?present=1` is the frame this fix was made for. It went 22px past the stage with its last
provenance line cut when the marker and its two lines were first added, and it is 0px over here
with the foot whole — the three columns of that row are `1.35fr 1.05fr 0.6fr` when a holder is
drawn, which was chosen by sweeping the ratio in the browser and reading the overflow back.

Nothing here concerns the Zcash network: the gate under test has an empty database and its root
sources are never read by a page. The act pages carry recorded runs' own values — the two-room
knock of 2026-10-20 for acts 1, 2, 3 and 5, the newcomer's two knocks of 2026-10-22 for act 4.

## The run

| Viewport | Page | Stage overflow | Boxes that hide text | Text outside the stage | Smallest type |
|---|---|---|---|---|---|
| 1920x1080 | `picker` | 0px | none | none | 16px |
| 1920x1080 | `picker + present` | 0px | none | none | 20px |
| 1920x1080 | `live holders-room` | 0px | none | none | 15px |
| 1920x1080 | `live holders-room + present` | 0px | none | none | 19px |
| 1920x1080 | `act1-rule` | 0px | none | none | 15px |
| 1920x1080 | `act1-rule + present` | 0px | none | none | 19px |
| 1920x1080 | `act2-admitted` | 0px | none | none | 15px |
| 1920x1080 | `act2-admitted + present` | 0px | none | none | 19px |
| 1920x1080 | `act3-refused` | 0px | none | none | 15px |
| 1920x1080 | `act3-refused + present` | 0px | none | none | 19px |
| 1920x1080 | `act4-newcomer` | 0px | none | none | 15px |
| 1920x1080 | `act4-newcomer + present` | 0px | none | none | 19px |
| 1920x1080 | `act5-second-room` | 0px | none | none | 15px |
| 1920x1080 | `act5-second-room + present` | 0px | none | none | 19px |
| 1326x683 | `picker` | 0px | none | none | 16px |
| 1326x683 | `picker + present` | 0px | none | none | 20px |
| 1326x683 | `live holders-room` | 0px | none | none | 15px |
| 1326x683 | `live holders-room + present` | 0px | none | none | 19px |
| 1326x683 | `act1-rule` | 0px | none | none | 15px |
| 1326x683 | `act1-rule + present` | 0px | none | none | 19px |
| 1326x683 | `act2-admitted` | 0px | none | none | 15px |
| 1326x683 | `act2-admitted + present` | 0px | none | none | 19px |
| 1326x683 | `act3-refused` | 0px | none | none | 15px |
| 1326x683 | `act3-refused + present` | 0px | none | none | 19px |
| 1326x683 | `act4-newcomer` | 0px | none | none | 15px |
| 1326x683 | `act4-newcomer + present` | 0px | none | none | 19px |
| 1326x683 | `act5-second-room` | 0px | none | none | 15px |
| 1326x683 | `act5-second-room + present` | 0px | none | none | 19px |


## What this run does not prove

- Anything about any other browser or viewport. The stage is a fixed 1920x1080 scaled to the
  window, so the layout is the same at every size, but another engine could lay text out
  differently.
- That act 4 reads well to someone seeing it for the first time. It fits and hides nothing, and
  the numbers on it come from the recorded runs; whether the break reads as a break is a review,
  not a measurement. The screenshots behind this change were looked at, which is what the stub it
  replaced had going against it: that one measured clean too.
- Anything about the Zcash network: no wallet, no node and no nf-server was involved, and the
  gate's database was empty.
