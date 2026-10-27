# The published site, redesigned around a background video — 2026-10-27

What this file settles: that the site in `site/` was rebuilt on 2026-10-27 from one script, that
the landing page, the new Start here page, the three guides, the API index and the index of acts
share one design, that the five recorded act pages are untouched and still ask no other origin for
anything, that every link in the site resolves, that the five third-party origins the new design
uses are the only ones any page reaches, and that every run of text on four page kinds clears
WCAG AA at two viewport sizes **measured against the real background video**.

It says nothing about the Zcash network. Nothing was run against either wallet to make this file.

| | |
|---|---|
| Date | 2026-10-27 |
| Tree | the working tree at `c86ba4c` plus the video work and this, **uncommitted** |
| Built by | `scripts/build-site.sh` |
| Checked by | `scripts/site-pages.py` (links, origins, commands) and `scripts/site-browser-check.mjs` (browser) |
| Browser | Chromium 153.0.8010.52, headless, driven over the DevTools Protocol |
| Node | v24.19.0 |

## What was built

```
$ scripts/build-site.sh
```

| | |
| --- | --- |
| files | 899 |
| HTML pages | 557 |
| size | 18 MB |
| pages this repository writes | 18 (17 of its own, plus `api/index.html`) |
| the rest | rustdoc's, from `cargo doc --no-deps --workspace --all-features` |

The five recorded act pages, their five presentation readings and the room picker are **unchanged**:
they are exported by `scripts/export-replay-site.sh` from the gate's own page code, they carry
their CSS and their icon inside themselves, and nothing in this change touches them. They are the
product UI the walkthrough films, and the brief for this redesign was explicit that they stay as
they are.

## The three checks the build runs

```
links: every local link in 18 page(s) resolves inside the site
origins: 11 page(s) fetch nothing but files beside them (the five acts, the picker and rustdoc's own)
origins: cdnjs.cloudflare.com — on 7 page(s), allowed on the pages this repository writes
origins: d8j0ntlcm91z4.cloudfront.net — on 1 page(s), allowed on the landing page
origins: db.onlinewebfonts.com — on 7 page(s), allowed on the pages this repository writes
origins: fonts.googleapis.com — on 7 page(s), allowed on the pages this repository writes
origins: www.youtube-nocookie.com — on 1 page(s), allowed on the landing page and Start here
origins: allowed but unused: fonts.gstatic.com
start: 2 command block(s) are character-identical to README's
```

`fonts.gstatic.com` is listed and reached at view time by the stylesheet Google Fonts serves, not
by a tag in any page here, so the build sees it only as a `preconnect`. It stays on the list
because that is where the font files actually come from.

The commands on Start here are not retyped: `scripts/site-pages.py` lifts them out of README's
60-second section, and the check asserts the rendered page still carries them character for
character. A judge copying from the site and a judge copying from README get the same two
commands because there is only one source for them.

## The browser check

```
$ node scripts/site-browser-check.mjs site
```

Four page kinds — landing, Start here, a guide, the API index — at **1920x1080** and **390x844**.
The site is served over HTTP by a threaded `python3` server, which is how GitHub Pages serves it.

| page | viewport | text runs | worst contrast | floor |
| --- | --- | --- | --- | --- |
| landing | 1920x1080 | 24 | **6.41:1** | 4.5 |
| Start here | 1920x1080 | 23 | 8.30:1 | 4.5 |
| guide | 1920x1080 | 31 | 8.79:1 | 4.5 |
| API index | 1920x1080 | 23 | 6.41:1 | 4.5 |
| landing | 390x844 | 18 | **6.33:1** | 4.5 |
| Start here | 390x844 | 11 | 8.30:1 | 4.5 |
| guide | 390x844 | 16 | 10.31:1 | 4.5 |
| API index | 390x844 | 10 | 10.31:1 | 4.5 |

Also checked, at both sizes: every link on every page resolves to a file in the site; the five act
links on Start here open `act1-rule` … `act5-second-room` **in that order**; both copy buttons
carry their command verbatim in `data-command`; and the three dots marking the active page in the
nav clear that page's own label, measured as the distance from the bottom of the label's line box
to the top of the `::after` box — **2.5px under "Home", 3.0px under "Start", "Docs" and "API"**.
Below 720px the nav pill is replaced by the burger menu, so there is nothing to measure there.

## The walkthrough player, and proving it is not loaded until it is wanted

The film is published at `https://youtu.be/J3R-K1KfnpQ` and plays from
`https://www.youtube-nocookie.com/embed/J3R-K1KfnpQ`, checked 2026-10-27:

| checked | how | result |
| --- | --- | --- |
| the watch URL | `curl -sIL https://youtu.be/J3R-K1KfnpQ` | 200 |
| the embed URL | `curl -s -o /dev/null -w '%{http_code}' …/embed/J3R-K1KfnpQ` | 200 |
| that it is the right film | YouTube's oEmbed endpoint | `"title":"Unstirred Demo For Zecathon by Zcash"` |

It sits in two places, and only one of them is in the markup:

- **Start here, step 1** (`#watch`) carries the `<iframe>` itself, in a 16:9 box.
- **the landing page** has a "Watch the Demo" button and an empty lightbox. The `<iframe>` is
  **built in JavaScript when the button is pressed** and the container is emptied again on close,
  so a visitor who never asks for the film never makes a request to Google, and closing the box
  stops playback instead of leaving it running behind a hidden overlay.

The browser check asserts all of that rather than taking it on trust. At both viewports it reports:

```
landing @ 1920x1080: lightbox empty until clicked, opens on youtube-nocookie, closes and destroys the frame
landing @ 1920x1080: Escape closes the lightbox
landing @ 1920x1080: the backdrop closes the lightbox, the film itself does not
start   @ 1920x1080: the walkthrough embed is 16:9, inside its box, on youtube-nocookie
start   @ 390x844:   the walkthrough embed is 16:9, inside its box, on youtube-nocookie
```

It counts the iframes in the lightbox before the click (**must be zero**), reads the created
frame's `src` (must be on `www.youtube-nocookie.com`), and then closes the box three separate
times — by the close button, by Escape, and by clicking the dark backdrop — counting the iframes
again after each (**zero again**). The backdrop case also clicks the film itself first and asserts
the box is **still open**, because a handler on the whole overlay that does not check
`event.target` closes the box the moment a visitor clicks the player.

The landing-page lightbox is checked at 1920x1080 only, and that is deliberate rather than an
omission: at 720px and under the "Watch the Demo" control is left as a plain link to
`/start/#watch`, because a lightbox is the wrong shape for a 16:9 film on a phone. The check
asserts the landing page ships **no embed in its markup** at either size, and the Start here embed
at both.

The Start here embed is measured against its own container: width within a pixel of the box,
height within a pixel of 9/16 of the width, and the container itself within 0.03 of 16:9 and not
past the viewport.

**One thing had to be undone to make the player work at all.** The pages carried a page-level
`<meta name="referrer" content="no-referrer">`, and YouTube answers a referrer-less embed with
*"Video player configuration error — Error 153"*: it will not serve a frame it cannot attribute to
a host. The meta is still there for the page; the player iframe alone carries
`referrerpolicy="strict-origin-when-cross-origin"`, which sends the origin and no path. This was
found by loading the embed, not by reading about it.

## How the contrast is measured, and three ways it lied first

This number is the one most easily faked, so the method matters.

The text is measured against **what is actually painted behind it**, not against the CSS
background it inherits: the page is screenshotted twice, once as published and once with every
glyph made transparent, and the second image is the background. For each run of text the darkest
case is the brightest background pixel inside the **line boxes its own text nodes occupy**, taken
from a `Range`, not from the element's bounding rectangle.

Three separate faults each produced a clean pass that meant nothing, and each is why a piece of
the check exists:

1. **Measuring the text against itself.** The first version sampled the brightest pixel in the
   text's box with the text still in it, so white type on black reported 1.00:1 and everything
   "failed" for the wrong reason. Hence the transparent-ink plate.
2. **Measuring the element instead of the glyphs.** The trust pill's rectangle overlaps the white
   avatar rings beside it, and the time pills' rectangles include their own 40%-white border, so
   both reported failures that were not there. Hence the `Range` line boxes.
3. **Measuring a video that never painted.** This is the serious one. Headless Chromium here
   decodes no H.264, so the background video renders as a black rectangle, and every text run
   passed comfortably against a backdrop no visitor will ever see. The video is **not** dark
   footage: sampled over its own ten-second loop it averages **90 of 255 and peaks at 249**, which
   is near-white. White text straight onto that is about 2:1.

So the check now pulls twelve real frames out of the video with `ffmpeg`, serves them beside the
page, paints each one into the background in the video's place, and measures every run of text
against all twelve. It also proves they painted: each frame's plate is compared pixel by pixel
against a plate with no backdrop at all, and a frame that changes nothing is reported as a failure
rather than silently measured as black. A brightness threshold cannot do this job, because dark
footage is dark too.

The scrim over the video is sized from that measurement rather than by eye: 0.74 through the hero,
0.84–0.88 across the stats where `#8e8e8e` labels need a background under roughly 40/255, 0.80 at
the top. The footage still reads as motion behind it.

## Three real bugs this found, two of them invisible without it

**The background video would never have appeared.** `.page > * { position: relative }` has the same
specificity as `.bg { position: absolute }` and came later in the stylesheet, so the background
container became a static flex item and collapsed to **0×0** — with the video inside it. The page
rendered pure black, looked deliberate, and no check that did not measure the backdrop would have
said a word. The three regions are now named one by one instead of selected as a group.

**A step's last paragraph fell out of its card.** `.step .number` spanned a fixed three rows, so
step 2, which has five, wrapped its closing paragraph into a one-word-wide column underneath the
number and outside the card. Every child but the number is now placed in column two explicitly.

**The active page's three dots sat on top of its own label.** `.nav-pill a` had no height, so its
box was the line box of the text, and `bottom: 5px` put the dots **inside the glyphs** rather than
under them — visible in the first screenshots as a smudge through "Home" and "Start". The link is
now a full-height flex item, so the dots have the pill's own padding to sit in. The check measures
the gap between the bottom of the text's line box and the top of the `::after` rather than trusting
the stylesheet, and fails if it is under a pixel.

All three were found by looking at the rendered screenshots, which is why they are kept in
`evidence/2026-10-27-site-shots/`.

## The origin policy, which is narrower than it was

The old promise was that **no page of the site asked any other origin for anything**. That is no
longer true, and saying so would be the easy thing to leave alone, so it was changed everywhere it
appeared: `README.md`, the `CLAIMS.md` row, and the header comment of `scripts/build-site.sh`.

What replaced it is enforced per page rather than site-wide:

- the **five act pages, their five presentation readings and the room picker** ask no other origin
  for anything at all, and carry no `integrity=` or `crossorigin` attribute either — this is LEAKS
  L10 and A02c, and it is what lets an act open from a file with nothing behind it;
- the **seven pages this repository writes** — landing, Start here, three guides, API index,
  index of acts — may reach the font and icon services, and **only those**:
  `fonts.googleapis.com`, `fonts.gstatic.com`, `db.onlinewebfonts.com`, `cdnjs.cloudflare.com`;
- `d8j0ntlcm91z4.cloudfront.net`, the background video, is allowed on **the landing page alone**;
- `www.youtube-nocookie.com`, the walkthrough player, is allowed on **the landing page and Start
  here alone**;
- any other origin, on any page, fails the build; so does an allowed origin appearing on a page its
  own rule does not cover, and so does any off-origin fetch from an act page.

The allowlist stopped being one flat list when the player arrived. A flat list would have made the
YouTube frame legal on the three guides and the API reference, where nothing should ever embed it,
so each origin now carries a predicate saying which pages may hold it and the check runs that
predicate per page.

No page of either half carries analytics, a beacon or a tracker.

While this was being written the origin check was found to be **weaker than it claimed**: its
stylesheet pattern required `rel="stylesheet"` to appear before `href`, so a `<link>` written the
other way round was not seen at all — which is exactly how the OnlineWebFonts stylesheet on every
page went unreported. It now reads each `<link>` as a whole tag and picks the attributes apart in
either order.

The decision itself — that the landing page uses a hosted template video and hosted fonts — is the
owner's, and is recorded in `DECISIONS.md` with its reasoning.

## The numbers on the landing page

All four are read out of this repository at build time by `scripts/site-pages.py`. None is written
into the page, and a source that cannot be read fails the build rather than producing a plausible
number.

| shown | value | read from |
| --- | --- | --- |
| Blocks Held Unmoved | **720** | `examples/holders-room.json`, h1 − h0 (4,506,510 − 4,505,790) |
| Markers Shared Between Rooms | **0** | `evidence/2026-10-20-two-room-knock.txt`, the intersection of the two rooms' marker sets, counted |
| Claims Proven On Testnet | **17** | `CLAIMS.md`, rows with status PROVEN |
| Tests Passing | **132** | the most recent gate log in `evidence/`, which is `2026-10-26-gate-final-audit.md` |

The second is counted rather than asserted: both rooms' marker sets are parsed out of the recorded
run and intersected. The fourth refuses a gate log that records any failing test.

## What this file does not say

It does not say this particular build is published — this is a local build of `site/`, which is
gitignored, and the Pages workflow publishes its own from the same script. It does not say anything
about how the walkthrough plays once YouTube has it: the check proves the frame is created, points
at the `nocookie` host and sits inside its box, not that the film decodes. And it says nothing about
how the page behaves in a browser that is not Chromium 153.
