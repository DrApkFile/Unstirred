# frame.md: the design system of the Unstirred demo video

The video is rendered from the Holders' Room page itself, not from a separate design. `crates/unstirred-gate/ui/tokens.css` is the source of every value below and this file is its mirror; a unit test fails if the two drift. Change a value there, then update the table here.

## The frame

- **Stage:** exactly 1920 by 1080 CSS pixels, scaled to the viewport by `--scale` and nothing else. A render at 1920 by 1080 is scale 1, so a frame of the video and the page a judge opens are the same layout, pixel for pixel.
- **Rows (room page):** the head, the door rule and the provenance foot take the height they need; the two rows of panels take what they need and then share whatever is left. No row can be smaller than its own content, so nothing is ever clipped to make one fit — if a page ever outgrew the stage, `.frame` would scroll to it rather than hide it.
- **Rows (picker):** head, one centred block holding the rule and the room cards, provenance foot.
- **Columns:** the hero row is `2fr` (the verdict) and `1fr` (what the door learned); the row below it is `1fr` (the window), `1.15fr` (the two doors) and `0.85fr` (the bot counter).
- **Hierarchy:** the verdict is the hero of any act that has one, at `--type-verdict`, and the door rule shrinks to one line above it. Before a door has answered anything, the rule is the hero at `--type-hero` and the verdict card reads "Nobody has knocked yet".
- **Motion:** none. There is no CSS transition and no keyframe anywhere in the page, and a replayed act runs no timer. Everything that changes between acts is a different state, so the video steps from one act to the next rather than animating within one.
- **Type:** system families only (`--sans`, `--mono`); no font is fetched, so a render needs no network and cannot differ by what is installed in a CDN.

## Presentation mode

`?present=1` on any page, live or replayed, is the frame to record: the smallest type on the page grows, and the provenance foot keeps only the one line saying where the numbers came from. Nothing load-bearing is lost — a derived time is explained beside the time itself in the verdict card, and a refusal that wrote nothing says so where it would have been written. It changes these tokens and nothing else:

| Token | Normal | `?present=1` |
|---|---|---|
| `--type-small` | `16px` | `20px` |
| `--type-mono` | `18px` | `21px` |
| `--type-mono-small` | `15px` | `19px` |
| `--type-body` | `20px` | `23px` |

## The window, and a holder on it

The window is a timeline, not a bar: `h0` and `h1` are labelled points on an axis that runs past both of them, and the band between them carries the rule in words — *coins must exist here and stay unspent through here*. A room state may also carry one `holder` (`label`, `from`, `to`), drawn as a second line below the band. A holder line that covers the whole band is `--admit`; one that does not is `--refuse` and says so, which is how an act shows a newcomer whose coins arrived after `h0` as plainly outside the rule rather than describing them. None of the four recorded acts draws one: there is no recorded run of a newcomer yet.

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

Each act is a page state in `ui/replay/`, generated from the recorded two-room knock by `scripts/replay-from-evidence.py`. Render one by opening the gate at `/room/<context>?replay=<act>`: that document polls nothing, reads no clock and carries the act inside it, so two renders of it are the same frames.

| Act | Page | What the frame shows |
|---|---|---|
| `act1-rule` | `/room/holders-room?replay=act1-rule` | the door rule as the hero, nothing admitted yet |
| `act2-admitted` | `/room/holders-room?replay=act2-admitted` | ADMITTED in green, the five markers the gate wrote down |
| `act3-refused` | `/room/holders-room?replay=act3-refused` | REFUSED in red, ALREADY_USED_IN_CONTEXT, the same five markers and "unchanged: the refusal wrote nothing" |
| `act5-second-room` | `/room/second-room?replay=act5-second-room` | both rooms side by side, markers shared between rooms: 0 |

PRD section 9 also has an act 4 (a holder whose coins are newer than h0) and an act 7 (the chain itself). Neither has a recorded run behind it yet, so neither has an act file: the page would have to be given numbers nobody produced. Act 4 is what the `holder` line is for, and is milestone M4.
