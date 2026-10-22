# Layout check: every page, with act 4 on it

Commit `2e24559` (M4 step 1) plus the uncommitted M4 step 2 work, after act 4 was generated from
its recorded runs and
the window's axis was changed for it: the axis is now always the window plus its margin, and a
holder line that runs past an end is drawn to that end as a stub faded off it with the distance in
blocks beside it.
Date: 2026-10-22T06:39:33Z. Browser: Chromium 153.0.8010.52 (Arch Linux), headless, driven over the
DevTools protocol by `scripts/page-layout-check.mjs` on Node v24.19.0.
Command: `node scripts/page-layout-check.mjs http://127.0.0.1:8098 holders-room second-room`
against a gate started with `unstirred gate serve` over two `gate init` manifests
(`test` / h0 4,505,790 / h1 4,506,510), an empty database and two root sources nothing listens on.
Exit: 0.

```
LAYOUT CHECK: nothing clips, nothing is hidden
```

Twenty-eight page variants now: the picker, a live room page and all five acts, each of them again
with `?present=1`, at the video's own 1920x1080 and at 1326x683. The script reads the acts from
`crates/unstirred-gate/ui/replay/`, so act 4 joined this table by being generated, not by an edit.

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

## What act 4 cost to fit

Act 4 failed this check three times before it passed, and each failure was measured and then
looked at:

1. **128px over, 148px presented.** The axis widened to take in a holder line 21,519 blocks past
   h1, which left the 720-block window three per cent of the rail: its `h0` and `h1` marks
   overlapped into one string and the caption pinned to the band's span wrapped into a column of
   single words. Only the screenshot showed why the number was what it was. The axis keeps to the
   window now.
2. **2px over, 53px presented.** The act's provenance foot was five entries where the other acts
   have two, and its first line — the one presentation mode keeps — ran to three lines.
3. **3px over, 25px presented.** The holder's own note under the rail ran to four lines in
   presentation mode.

What fixed each: the axis rule, a foot the length of the other acts', a shorter holder label, and
the room log's row cap dropped from a replayed page, where it describes a route that page never
calls. Nothing was hidden to make any of it fit — no box on any of these pages hides what does not
fit, which is the column this table exists for.

## Looked at, not only measured

Two frames of act 4 were captured at 1920x1080 with the same browser and read: the window keeps its
shape with `h0 4,505,790` and `h1 4,506,510` legible at either end of the band, and the newcomer's
line is a short faded stub off the right-hand end of the rail with "the newcomer's coins — from
their wallet, not the gate's: blocks 4,526,842 to 4,528,029 — 20,332 blocks after h1, never inside
this window" under it in the refusal colour. In presentation mode the foot is the one line naming
the recorded run and the stub is still there. The frames are not kept: they are a look at a page
this repo serves, reproducible from the command above.

## What this run does not prove

- Anything about any other browser or viewport, as before.
- That act 4 reads well to someone seeing it for the first time. It fits, it hides nothing, and the
  numbers on it come from the recorded runs; whether it lands is a review, not a measurement.
- Anything about the Zcash network: no wallet, no node and no nf-server was involved, and the
  gate's database was empty.
