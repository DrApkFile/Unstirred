# frame.md: the design system of the Unstirred demo video

The video is rendered from the Holders' Room page itself, not from a separate design. `crates/unstirred-gate/ui/tokens.css` is the source of every value below and this file is its mirror: if the two disagree, the stylesheet is right and this file is stale. Change a value there, then regenerate the table here.

## The frame

- **Stage:** exactly 1920 by 1080 CSS pixels, scaled to the viewport by `--scale` and nothing else. A render at 1920 by 1080 is scale 1, so a frame of the video and the page a judge opens are the same layout, pixel for pixel.
- **Rows (room page):** the head, the door rule and the provenance lines take the height they need; the two rows of panels between them share what is left, equally, and can only shrink. Four `--gap-3` gaps and `--gap-4` of padding on each side. Nothing can push the stage past 1080px, and a panel whose content does not fit clips inside its own border rather than breaking the frame.
- **Rows (picker):** head, the rule panel, the room cards, provenance.
- **Columns:** the stretching row is three equal columns (verdict, what the door learned, the window); the row below it is `2fr` (the rooms) and `1fr` (the bot counter).
- **Motion:** none. There is no CSS transition and no keyframe anywhere in the page, and a replayed act runs no timer. Everything that changes between acts is a different state, so the video steps from one act to the next rather than animating within one.
- **Type:** system families only (`--sans`, `--mono`); no font is fetched, so a render needs no network and cannot differ by what is installed in a CDN.

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
| `--type-hero` | `46px` | `—` |
| `--type-title` | `42px` | `—` |
| `--type-head` | `27px` | `—` |
| `--type-body` | `21px` | `—` |
| `--type-small` | `17px` | `—` |
| `--type-mono` | `18px` | `—` |
| `--type-mono-small` | `15px` | `—` |
| `--leading-tight` | `1.1` | `—` |
| `--leading-body` | `1.45` | `—` |
| `--gap-1` | `8px` | `—` |
| `--gap-2` | `16px` | `—` |
| `--gap-3` | `24px` | `—` |
| `--gap-4` | `36px` | `—` |
| `--gap-5` | `56px` | `—` |
| `--radius-1` | `10px` | `—` |
| `--radius-2` | `18px` | `—` |
| `--edge` | `1px` | `—` |
| `--sans` | `ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` | `—` |
| `--mono` | `ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace` | `—` |

## The acts

Each act is a page state in `ui/replay/`, generated from the recorded two-room knock by `scripts/replay-from-evidence.py`. Render one by opening the gate at `/room/<context>?replay=<act>`: that document polls nothing, reads no clock and carries the act inside it, so two renders of it are the same frames.

| Act | Page | What the frame shows |
|---|---|---|
| `act1-rule` | `/room/holders-room?replay=act1-rule` | the door rule, nothing admitted yet |
| `act2-admitted` | `/room/holders-room?replay=act2-admitted` | ADMITTED in green, the five markers the gate wrote down |
| `act3-refused` | `/room/holders-room?replay=act3-refused` | REFUSED in red, ALREADY_USED_IN_CONTEXT, the same five markers unchanged |
| `act5-second-room` | `/room/second-room?replay=act5-second-room` | both rooms side by side, markers shared between rooms: 0 |

PRD section 9 also has an act 4 (a holder whose coins are newer than h0) and an act 7 (the chain itself). Neither has a recorded run behind it yet, so neither has an act file: the page would have to be given numbers nobody produced.
