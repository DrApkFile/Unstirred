# Layout check: the page after act 4's two fixes

Commit `7c95671` plus the uncommitted M4 step 1 work, of which two changes are this page's: the
room picker aligns its block to the top of the stage instead of centring it, and a derived time's
explanation moved out of the verdict card into the provenance foot, which `?present=1` drops — the
card keeps "(time derived)" beside the time itself.
Date: 2026-10-22T05:45:12Z. Browser: Chromium 153.0.8010.52 (Arch Linux), headless, driven over the
DevTools protocol by `scripts/page-layout-check.mjs` on Node v24.19.0.
Command: `node scripts/page-layout-check.mjs http://127.0.0.1:8098 holders-room second-room`
against a gate started with `unstirred gate serve` over two `gate init` manifests
(`test` / h0 4,505,790 / h1 4,506,510), an empty database and two root sources nothing listens on.
Exit: 0.

```
LAYOUT CHECK: nothing clips, nothing is hidden
```

Every page the gate serves, at the video's own 1920x1080 and at the laptop size the first review
was made at, 1326x683. For each: how far the 1080px stage outgrew itself, every box that hides what
does not fit, every piece of text whose box has left the stage, and the smallest type size on the
page. The browser is doing the measuring, so this is the real layout and not an estimate of it.
The script now reads the acts from `crates/unstirred-gate/ui/replay/`, so act 4 will be in this
table the moment its run exists; today there are four acts and twenty-four page variants.

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

The numbers are identical to the run of 2026-10-21: neither fix changes how much room anything
takes, only where it sits.

## Looked at, not only measured

Three frames were captured at 1920x1080 with the same browser
(`chromium --headless --screenshot --window-size=1920,1080`) and read, because measuring is not the
same as looking:

- **the picker** — the rule and the two room cards sit directly under the head with no band above
  them, which is what the review asked for. What is left over is below the cards, where a gate with
  more rooms would put them.
- **`act3-refused`** — the verdict card reads "answered 2026-10-20 14:48:47 UTC (time derived) ·
  retryable: no", and the sentence saying how that time was derived is the third line of the
  provenance foot.
- **`act3-refused&present=1`** — the foot is the one line naming the recorded run, the derivation
  sentence is gone with the rest of the technical lines, and "(time derived)" is still in the card,
  so nothing on a projector shows a derived number as an observed one.

The frames themselves are not kept: they are a look at a page this repo serves, reproducible from
the command above, not a measurement.

## What this run does not prove

- Anything about any other browser, or any other viewport. The stage is fixed and scaled, so the
  layout is the same at every size, but an engine that lays text out differently could still
  differ.
- That the page reads well. Nothing a tool can measure says that; the owner's review is what moved
  these two things.
- Anything about the Zcash network, this gate's roots, or a proof: no wallet, no node and no
  nf-server was involved, and the gate's database was empty.
