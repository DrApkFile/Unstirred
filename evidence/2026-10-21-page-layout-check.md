# Layout check: no page the gate serves hides text to fit

Commit `bf3fe87` (M3 step 2) plus the uncommitted fixes for the visual review of 2026-10-21.
Date: 2026-10-21T16:13:41Z. Browser: Chromium 153.0.8010.52 (Arch Linux), headless, driven over the
DevTools protocol by `scripts/page-layout-check.mjs` on Node v24.19.0.
Command: `node scripts/page-layout-check.mjs http://127.0.0.1:8099 holders-room second-room`
against a gate started with `unstirred gate serve` over two `gate init` manifests
(`test` / h0 4,505,790 / h1 4,506,510) and an empty database. Exit: 0.

```
LAYOUT CHECK: nothing clips, nothing is hidden
```

Every page the gate serves, at the video's own 1920x1080 and at the laptop size the review was
made at, 1326x683. For each: how far the 1080px stage outgrew itself, every box that hides what
does not fit, every piece of text whose box has left the stage, and the smallest type size on the
page. The browser is doing the measuring, so this is the real layout and not an estimate of it.

Nothing on this page concerns the Zcash network: the gate under test has an empty database and its
root sources are never read by a page. The act pages carry the recorded two-room run's own values,
which `evidence/2026-10-20-two-room-knock.txt` settles.

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
| 1326x683 | `act5-second-room` | 0px | none | none | 15px |
| 1326x683 | `act5-second-room + present` | 0px | none | none | 19px |

LAYOUT CHECK: nothing clips, nothing is hidden

## What this check does not prove

- **Not that the page is well designed.** It proves that no text is hidden and that the stage does
  not outgrow its frame. Whether the result reads well is a judgement, and it was made by looking:
  the screenshots behind the 2026-10-21 review are not in `evidence/` because the claims ledger
  cites text files, and a picture is not a claim this repo knows how to check.
- **Not that it holds at every size.** Two viewports were measured. The stage is a fixed
  1920x1080 scaled by `--scale`, so the layout is the same at every size and only the scale
  changes — which is why these two agree exactly — but a browser that lays text out differently
  could still differ.
- **Not in any browser but this one.** One Chromium build measured it. No other engine was run.
- **It is not a gate step.** `scripts/gate.sh` needs neither a browser nor a node, and gains
  neither here. This check is run by hand and recorded, which is why the command and the versions
  are written above.
- The check was confirmed against a planted canary before this run: `max-height: 140px;
  overflow: hidden` on `.panel` made it report every clipped panel on all 24 pages and exit 1.
