# frame.md: the design system of the Unstirred demo video

The video is rendered from the Holders' Room page itself, not from a separate design. `crates/unstirred-gate/ui/tokens.css` is the source of every value below and this file is its mirror; a unit test fails if the two drift. Change a value there, then update the table here.

## The frame

- **Stage:** exactly 1920 by 1080 CSS pixels, scaled to the viewport by `--scale` and nothing else. A render at 1920 by 1080 is scale 1, so a frame of the video and the page a judge opens are the same layout, pixel for pixel.
- **Rows (room page):** the head, the door rule and the provenance foot take the height they need; the two rows of panels take what they need and then share whatever is left. No row can be smaller than its own content, so nothing is ever clipped to make one fit — if a page ever outgrew the stage, `.frame` would scroll to it rather than hide it.
- **Rows (picker):** head, then one block holding the rule and the room cards at the top of what is left, provenance foot. The cards are the height of their own content and there is no gap above them.
- **Columns:** the hero row is `2fr` (the verdict) and `1fr` (what the door learned); the row below it is `1fr` (the window), `1.15fr` (the two doors) and `0.85fr` (the bot counter).
- **Hierarchy:** the verdict is the hero of any act that has one, at `--type-verdict`, and the door rule shrinks to one line above it. Before a door has answered anything, the rule is the hero at `--type-hero` and the verdict card reads "Nobody has knocked yet".
- **Motion:** none. There is no CSS transition and no keyframe anywhere in the page, and a replayed act runs no timer. Everything that changes between acts is a different state, so the video steps from one act to the next rather than animating within one.
- **Type:** system families only (`--sans`, `--mono`); no font is fetched, so a render needs no network and cannot differ by what is installed in a CDN.

## Presentation mode

`?present=1` on any page, live or replayed, is the frame to record: the smallest type on the page grows, and the provenance foot keeps only the one line saying where the numbers came from. Nothing load-bearing is lost — the verdict card marks a derived time as derived where the time itself is, the sentence saying *how* it was derived is one of the provenance lines, and a refusal that wrote nothing says so where it would have been written. It changes these tokens and nothing else:

| Token | Normal | `?present=1` |
|---|---|---|
| `--type-small` | `16px` | `20px` |
| `--type-mono` | `18px` | `21px` |
| `--type-mono-small` | `15px` | `19px` |
| `--type-body` | `20px` | `23px` |

## The window, and a holder on it

The window is a timeline, not a bar: `h0` and `h1` are labelled points on an axis that runs past both of them, and the band between them carries the rule in words — *coins must exist here and stay unspent through here*. A room state may also carry one `holder` (`label`, `from`, `to`, and an optional `aside`), drawn as a second line below the band. A holder line that covers the whole band is `--admit`; one that only overlaps it is `--refuse` and says so.

**The window always keeps its own scale.** When a holder's blocks lie wholly outside the axis — act 4's newcomer is 20,332 blocks past `h1` — the rail is *broken* rather than stretched: the window's axis keeps 66% of the rail, a `//` break mark sits at 75%, and the holder is a dot in `--refuse` at 89% carrying the real distance in mono (`+20,332 blocks`), with the sentence under the rail reading *the newcomer's coins arrive here: 20,332 blocks after h1, never inside the window*. Stretching the axis to reach it instead left the window three per cent of the rail with its marks overlapping and its caption wrapped into a column; a break with the distance printed on it says the same thing and stays readable at a glance. The panel itself widens whenever a holder is drawn (`.thirds-holder`), because the window is then what the act is explaining.

A live page never draws a holder line: the gate knows no holder's block range. The two heights act 4 draws are read from the holder's own wallet by the knock itself and never sent, which is what the `aside` under the rail says — it stays on the page in presentation mode, where the provenance foot comes off.

## Tokens

| Token | Light | Dark |
|---|---|---|
| `--stage-w` | `1920px` | `—` |
| `--stage-h` | `1080px` | `—` |
| `--paper` | `#f4f3ef` | `#0d0f13` |
| `--panel` | `#ffffff` | `#171a20` |
| `--panel-edge` | `#d8d5cc` | `#2e333c` |
| `--ink` | `#14161b` | `#edeff3` |
| `--ink-dim` | `#4a4f59` | `#b2b9c6` |
| `--ink-faint` | `#616874` | `#868e9d` |
| `--admit` | `#0c6b3d` | `#5ed39b` |
| `--admit-wash` | `#e0f2e7` | `#10301f` |
| `--admit-edge` | `#9bd3b3` | `#2b6b48` |
| `--refuse` | `#a0152a` | `#ff8e9c` |
| `--refuse-wash` | `#fce9ec` | `#341319` |
| `--refuse-edge` | `#efb3bd` | `#7c333f` |
| `--unknown` | `#8a5600` | `#e8bb6b` |
| `--unknown-wash` | `#fdf1dc` | `#2f2512` |
| `--unknown-edge` | `#e4c48a` | `#6e5725` |
| `--accent` | `#1b4f8f` | `#8fbcf2` |
| `--accent-wash` | `#e6eef8` | `#15263c` |
| `--type-verdict` | `104px` | `—` |
| `--type-hero` | `46px` | `—` |
| `--type-title` | `38px` | `—` |
| `--type-head` | `26px` | `—` |
| `--type-body` | `20px` | `—` |
| `--type-small` | `16px` | `—` |
| `--type-mono` | `18px` | `—` |
| `--type-mono-small` | `15px` | `—` |
| `--leading-tight` | `1.08` | `—` |
| `--leading-body` | `1.4` | `—` |
| `--gap-1` | `8px` | `—` |
| `--gap-2` | `14px` | `—` |
| `--gap-3` | `20px` | `—` |
| `--gap-4` | `32px` | `—` |
| `--gap-5` | `56px` | `—` |
| `--radius-1` | `10px` | `—` |
| `--radius-2` | `18px` | `—` |
| `--edge` | `1px` | `—` |
| `--rail` | `10px` | `—` |
| `--sans` | `ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` | `—` |
| `--mono` | `ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace` | `—` |

## The acts

A room card carries where it opens only when the page is not being served by a gate: live, the card opens `/room/<context>` and the state says nothing about it; in the static export, each card's `page` is the first recorded act of that room, so an exported picker links to files beside it rather than to a gate route with nothing behind it. A `page` that is not a path beside the page it is on — anything with a scheme, a host or a leading slash — is refused by the state contract (A51).

Each act is a page state in `ui/replay/`, generated from the recorded runs in `evidence/` by `scripts/replay-from-evidence.py`, which also writes the list of the acts the gate serves beside them as `ui/replay/acts.rs`: an act exists when a run produced it, and never before. Render one by opening the gate at `/room/<context>?replay=<act>`: that document polls nothing, reads no clock and carries the act inside it, so two renders of it are the same frames.

| Act | Page | What the frame shows |
|---|---|---|
| `act1-rule` | `/room/holders-room?replay=act1-rule` | the door rule as the hero, nothing admitted yet |
| `act2-admitted` | `/room/holders-room?replay=act2-admitted` | ADMITTED in green, the five markers the gate wrote down |
| `act3-refused` | `/room/holders-room?replay=act3-refused` | REFUSED in red, ALREADY_USED_IN_CONTEXT, the same five markers and "unchanged: the refusal wrote nothing" |
| `act4-newcomer` | `/room/holders-room?replay=act4-newcomer` | REFUSED in red, ROOT_MISMATCH, and the newcomer's coins — blocks 4,526,842 to 4,528,029 — marked past a break in the rail, `+20,332 blocks` in the refusal colour, with the distance in words under it |
| `act5-second-room` | `/room/second-room?replay=act5-second-room` | both rooms side by side, markers shared between rooms: 0 |

Act 4 is built from two runs of 2026-10-22 (`evidence/2026-10-22-bob-knock.txt` for the room's own refusal, `evidence/2026-10-22-bob-forced-knock.txt` for the gate's) and says both on the page. PRD section 9's act 7, the chain itself, has no run behind it and so has no act file: the page would have to be given numbers nobody produced.
