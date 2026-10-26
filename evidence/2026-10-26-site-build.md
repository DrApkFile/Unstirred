# The published site, built and checked — 2026-10-26

`scripts/build-site.sh` produces the whole site from this repository, and
`.github/workflows/pages.yml` runs that same script on every push to `master`. There is no second
renderer anywhere: what a workflow publishes is what the script gives you locally.

```
$ ./scripts/build-site.sh site
== the recorded acts
== the guides
docs/GETTING_STARTED.md -> site/guides/getting-started.html (15707 bytes of body, 5 sections)
docs/INTEGRATION.md -> site/guides/integration.html (21963 bytes of body, 5 sections)
SPEC.md -> site/guides/spec.html (29277 bytes of body, 11 sections)
== the design tokens the gate's pages use, plus the site's own layout
== the icons
== the API reference
== the landing page and the index of acts
site/index.html (3071 bytes of body)
site/replay/index.html (3335 bytes of body)
links: every local link in 17 page(s) resolves inside the site
origins: no page of this site fetches anything but files inside it

17M	site
```

**869 files, 556 HTML pages, 17 MB.** Of those 556, **17 are the site's own**: the landing page,
the index of acts, the eleven exported acts and picker, the three guides and the API index. The
other 539 are rustdoc's, which the link check skips because rustdoc's own cross-links are its
business and its search pages carry JavaScript templates that look like links and are not.

What is in it:

| | |
|---|---|
| `index.html` | the landing page, written by `scripts/site-pages.py` |
| `replay/` | the five recorded acts, each in normal and presentation mode, plus the room picker and an index — exported by the gate's own page module, so each file is byte for byte what `unstirred gate serve` answers |
| `guides/` | `GETTING_STARTED`, `INTEGRATION` and `SPEC`, rendered by `scripts/render-md.py` into the site's own design |
| `api/` | `cargo doc --no-deps --workspace --all-features`, under a hand-written index naming the four libraries |
| `styles.css` | the gate's own `ui/tokens.css` followed by `scripts/site/site.css`, so the site and the acts inside it use one set of tokens |
| `icon.png`, `favicon.png` | `assets/icon.png` and the gate's own `ui/favicon.b64`, decoded |

## The two checks, confirmed against planted faults

`scripts/site-pages.py` ends in two checks, and the build fails if either does. Each was confirmed
by planting the fault it is meant to catch into a copy of the finished site and re-running it.
Every one exits **1** and names the page:

| Planted | Caught as |
|---|---|
| `<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter">` in a guide | `guides/spec.html fetches 'https://fonts.googleapis.com/css2?family=Inter', which is not a file beside it` |
| A 1×1 tracking pixel, `<img src="https://metrics.example.com/px.gif?p=home">` | `guides/integration.html fetches 'https://metrics.example.com/px.gif?p=home', which is not a file beside it` |
| `<script src="x.js" integrity="…" crossorigin="anonymous">` with **no** `x.js` beside it | `broken local links in the site: guides/spec.html -> x.js` |
| the same, with `x.js` actually written beside it | `guides/spec.html carries 'integrity='` and `guides/spec.html carries 'crossorigin'` |
| an act file deleted after the export | `broken local links in the site: replay/index.html -> act3-refused.html` |
| an act added to `scripts/site/acts.json` that no run produced | `scripts/site/acts.json and the acts the gate serves do not match` — described six, served five |

The last one is the rule that keeps the site honest about itself: `site-pages.py` parses
`REPLAY_ACTS` out of `crates/unstirred-gate/ui/replay/acts.rs` and refuses to write the index if
the acts it describes are not exactly the acts the gate serves. An act exists when a run produced
it.

**One limit of the plant test, stated:** `index.html` and `replay/index.html` are *written* by
`site-pages.py`, so a fault planted in either is overwritten before it can be checked. Both were
planted in a rendered guide instead, which is a file the script only reads. The check itself reads
every page in the site, those two included.

## What this cannot say

- Nothing here was published. The site was built into `site/`, which is not tracked, and opened
  from the local filesystem.
- It says nothing about a third party a *browser* might reach that is not written in the HTML —
  there is no JavaScript on the site's own pages, and the acts' policy is `connect-src 'none'`
  (A44), but this check reads files, it does not watch a network.
