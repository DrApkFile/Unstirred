#!/usr/bin/env python3
"""Writes the pages of the published site that are not a rendered markdown file, and then checks
that every local link in the finished site resolves and that no page asks an origin it may not.

Three pages are written here: the landing page, the "Start here" page a judge is pointed at, and
the index of recorded acts. The acts come from `scripts/site/acts.json`, and a mismatch between
that file and the acts the gate actually serves is an error rather than a page with a hole in it —
so a newly recorded act fails the build until somebody says what it shows.

Every number on the landing page is read out of this repository at build time. None of them is
written here, and a source that cannot be read is a failed build rather than a page with a
plausible number on it.

    scripts/site-pages.py site --video-url https://...
"""
import argparse
import html
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
REPO = "https://github.com/DrApkFile/Unstirred"
BLOB = f"{REPO}/blob/master/"

# The background video and the three font services. These are the only origins any page of this
# site is allowed to reach, they are reached only by the four page kinds written or styled by this
# repository, and the five act pages reach none of them. DECISIONS.md carries the choice.
VIDEO_URL = (
    "https://d8j0ntlcm91z4.cloudfront.net/user_38xzZboKViGWJOttwIXH07lWA1P/"
    "hf_20260809_012548_ef22562c-c0ae-4816-ad9d-f8922af4e6a7.mp4"
)
ALLOWED_ORIGINS = (
    "fonts.googleapis.com",
    "fonts.gstatic.com",
    "db.onlinewebfonts.com",
    "cdnjs.cloudflare.com",
    "d8j0ntlcm91z4.cloudfront.net",
)

FONT_AWESOME = (
    "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css",
    "sha512-SnH5WK+bZxgPHs44uWIX+LLJAJ9/2PkPKZ5QiAj6Ta86w+fsb2TkcmfRyVX3pBnMFcV7oQPJkl9QevSCWr3W6A==",
)

NAV = (
    ("Home", "index.html"),
    ("Start", "start/index.html"),
    ("Demo", "replay/index.html"),
    ("Docs", "guides/getting-started.html"),
    ("API", "api/index.html"),
)


def head(title, description, up, extra=""):
    """The <head> every page of this site shares: the three font services, the stylesheet, and
    nothing that is not named in ALLOWED_ORIGINS."""
    fa_href, fa_integrity = FONT_AWESOME
    return f"""    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="referrer" content="no-referrer" />
    <meta name="description" content="{html.escape(description)}" />
    <title>{html.escape(title)}</title>
    <link rel="icon" type="image/png" href="{up}favicon.png" />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link
      rel="stylesheet"
      href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap"
    />
    <link href="https://db.onlinewebfonts.com/c/8cb707a9b8a73f8a7403336b861c3074?family=BubbledotICG-FinePos" rel="stylesheet" />
    <link
      rel="stylesheet"
      href="{fa_href}"
      integrity="{fa_integrity}"
      crossorigin="anonymous"
      referrerpolicy="no-referrer"
    />
    <link rel="stylesheet" href="{up}styles.css" />
{extra}"""


def header(active, up):
    """The same header on every page: the mark, the pill of links, and the repository. Only the
    `active` class differs, so a reader always knows which of the five they are on."""
    links = "\n".join(
        f'          <a href="{up}{href}"{" class=\"active\"" if name == active else ""}>{name}</a>'
        for name, href in NAV
    )
    menu_links = "\n".join(
        f'        <a href="{up}{href}"{" class=\"active\"" if name == active else ""} style="--d: {0.04 * i:.2f}s">{name}</a>'
        for i, (name, href) in enumerate(NAV)
    )
    return f"""      <header class="site-header">
        <a class="logo" href="{up}index.html" aria-label="Unstirred, home">
          <img src="{up}icon-mark.png" alt="" width="52" height="52" />
        </a>
        <nav class="nav-pill">
{links}
        </nav>
        <a class="ghpill" href="{REPO}">GitHub</a>
        <button class="burger" type="button" aria-expanded="false" aria-controls="menu" aria-label="Menu">
          <span class="bar"></span>
          <span class="bar"></span>
          <span class="bar"></span>
        </button>
      </header>
      <div class="overlay" hidden></div>
      <nav class="menu" id="menu" hidden>
{menu_links}
        <a class="ghpill-full" href="{REPO}">GitHub</a>
      </nav>"""


MENU_JS = """    <script>
      (function () {
        var burger = document.querySelector(".burger");
        var menu = document.getElementById("menu");
        var overlay = document.querySelector(".overlay");
        if (!burger || !menu || !overlay) return;
        function setOpen(open) {
          burger.setAttribute("aria-expanded", open ? "true" : "false");
          menu.hidden = !open;
          overlay.hidden = !open;
          document.body.classList.toggle("menu-open", open);
        }
        burger.addEventListener("click", function () {
          setOpen(burger.getAttribute("aria-expanded") !== "true");
        });
        overlay.addEventListener("click", function () { setOpen(false); });
        menu.addEventListener("click", function (event) {
          if (event.target.tagName === "A") setOpen(false);
        });
        document.addEventListener("keydown", function (event) {
          if (event.key === "Escape") setOpen(false);
        });
        window.addEventListener("resize", function () {
          if (window.innerWidth > 720) setOpen(false);
        });
      })();
    </script>"""

FOOTER = f"""      <footer>
        <p>
          <a href="{REPO}">The repository</a> · MIT OR Apache-2.0 ·
          Zcash <strong>testnet</strong> only; nothing here has been exercised against mainnet.
        </p>
      </footer>"""


def doc_page(out, title, description, active, body, depth, scripts=""):
    """One of the scrolling pages: Start here, the index of acts, the API index."""
    up = "../" * depth
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        f"""<!doctype html>
<html lang="en">
  <head>
{head(title, description, up)}
  </head>
  <body class="doc">
{header(active, up)}
    <main>
{body}
    </main>
{FOOTER}
{MENU_JS}
{scripts}
  </body>
</html>
"""
    )
    print(f"{out} ({len(body)} bytes of body)")


# ------------------------------------------------------------------ the numbers on the landing


def window_blocks():
    """The demo window, in blocks: the room the recorded proof was made for."""
    manifest = json.loads((ROOT / "examples/holders-room.json").read_text())
    return manifest["window"]["h1"] - manifest["window"]["h0"]


def markers_shared_between_rooms():
    """How many markers the two rooms of the recorded two-room knock have in common.

    This is the whole unlinkability claim as one number, and it is counted rather than asserted:
    the run's own output carries both rooms' marker sets, and this intersects them."""
    text = (ROOT / "evidence/2026-10-20-two-room-knock.txt").read_text()
    rooms = {}
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("{") or '"markers"' not in line:
            continue
        row = json.loads(line)
        rooms[row["context"]] = {m["marker"] for m in row["markers"]}
    if len(rooms) < 2:
        raise SystemExit(
            "evidence/2026-10-20-two-room-knock.txt no longer carries two rooms' marker sets; "
            "the landing page's unlinkability number is counted from them and cannot be guessed"
        )
    shared = set.intersection(*rooms.values())
    return len(shared)


def claims_proven():
    """Rows of CLAIMS.md settled by a run against the real testnet."""
    return len(re.findall(r"\|\s*PROVEN\s*\|", (ROOT / "CLAIMS.md").read_text()))


def latest_gate_log():
    """The most recent recorded gate run, by the date in its file name."""
    logs = sorted((ROOT / "evidence").glob("*-gate-*.md"))
    if not logs:
        raise SystemExit("no gate log in evidence/: the landing page's test count comes from one")
    return logs[-1]


def tests_passing(log):
    """The test count out of that run's own summary line."""
    found = re.search(r"\*\*(\d+) passed, (\d+) failed", log.read_text())
    if not found:
        raise SystemExit(f"{log} does not state 'N passed, N failed': the test count comes from it")
    if int(found.group(2)):
        raise SystemExit(f"{log} records {found.group(2)} failing test(s); it is not a passing run")
    return int(found.group(1))


def stats():
    """The four numbers on the landing page, each read from the file it links to."""
    log = latest_gate_log()
    return [
        ("<", window_blocks(), "Blocks Held Unmoved", f"{BLOB}examples/holders-room.json"),
        ("%", markers_shared_between_rooms(), "Markers Shared Between Rooms",
         f"{BLOB}evidence/2026-10-20-two-room-knock.txt"),
        ("*", claims_proven(), "Claims Proven On Testnet", f"{BLOB}CLAIMS.md"),
        ("#", tests_passing(log), "Tests Passing", f"{BLOB}evidence/{log.name}"),
    ]


# ------------------------------------------------------------------------------ the landing page

COUNTUP_JS = """    <script>
      (function () {
        var values = Array.prototype.slice.call(document.querySelectorAll(".stat .value"));
        if (!values.length) return;
        var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        function settle(el) { el.textContent = el.getAttribute("data-to"); }
        if (reduced || !("IntersectionObserver" in window)) {
          values.forEach(settle);
          return;
        }
        function run(el, index) {
          var to = Number(el.getAttribute("data-to"));
          var duration = 1500 + index * 80;
          var started = null;
          function frame(now) {
            if (started === null) started = now;
            var t = Math.min((now - started) / duration, 1);
            var eased = 1 - Math.pow(1 - t, 3);
            el.textContent = String(Math.round(to * eased));
            if (t < 1) requestAnimationFrame(frame);
            else settle(el);
          }
          setTimeout(function () { requestAnimationFrame(frame); }, 480 + index * 90);
        }
        var seen = new WeakSet();
        var observer = new IntersectionObserver(function (entries) {
          entries.forEach(function (entry) {
            if (!entry.isIntersecting || seen.has(entry.target)) return;
            seen.add(entry.target);
            run(entry.target, values.indexOf(entry.target));
            observer.unobserve(entry.target);
          });
        }, { threshold: 0.25 });
        values.forEach(function (el) { observer.observe(el); });
      })();
    </script>"""


def landing(out, video_url):
    """One viewport: the header, the claim, the two things to do, and four numbers.

    `video_url` is the published walkthrough. While there is none, the second call to action opens
    the index of acts instead — the same material, as pages — rather than a link that does not
    work."""
    watch_href = video_url or "replay/index.html"
    rows = "\n".join(
        f"""          <a class="stat anim" style="--d: {delay}s" href="{html.escape(source, quote=True)}">
            <span class="glyph">{html.escape(glyph)}</span>
            <span class="value" data-to="{value}">0</span>
            <span class="label">{html.escape(label)}</span>
          </a>"""
        for (glyph, value, label, source), delay in zip(stats(), ("0.5", "0.58", "0.66", "0.74"))
    )
    body = f"""      <div class="bg">
        <video class="bg-video" autoplay muted loop playsinline>
          <source src="{VIDEO_URL}" type="video/mp4" />
        </video>
      </div>
{header("Home", "")}
      <section class="hero">
        <div class="trust anim" style="--d: 0.05s">
          <span class="ring"><span><i class="fa-solid fa-shield-halved"></i></span></span>
          <span class="ring"><span><i class="fa-solid fa-lock"></i></span></span>
          <span class="ring"><span><i class="fa-solid fa-code"></i></span></span>
          <span class="pill">Built on Valar Group's audited circuit</span>
        </div>
        <h1 class="headline">
          <span>Prove You Held</span>
          <span>Not Who You Are</span>
        </h1>
        <p class="subhead anim" style="--d: 0.28s">
          A zero-knowledge proof that you held shielded ZEC since a given block, without revealing
          your address, balance or transactions.
        </p>
        <div class="ctas">
          <a class="cta primary anim" style="--d: 0.4s" href="start/index.html">Start Here</a>
          <a class="cta secondary anim" style="--d: 0.46s" href="{html.escape(watch_href, quote=True)}">Watch the Demo</a>
        </div>
      </section>
      <section class="stats">
{rows}
      </section>"""
    up = ""
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        f"""<!doctype html>
<html lang="en">
  <head>
{head("Unstirred — Prove You Held. Not Who You Are.", "A zero-knowledge proof that you held shielded ZEC since a given block, without revealing your address, balance or transactions.", up)}
  </head>
  <body class="landing">
    <div class="page">
{body}
    </div>
{MENU_JS}
{COUNTUP_JS}
  </body>
</html>
"""
    )
    print(f"{out} (landing, {len(body)} bytes of body)")


# ------------------------------------------------------------------------ the Start here page

SIXTY_SECOND_HEADING = "## Check it yourself in 60 seconds"


def readme_commands():
    """The two commands of README's 60-second path, lifted out of README itself.

    They are not retyped here. A judge who copies the command off this site and a judge who copies
    it out of README must get the same characters, and the only way to promise that is to have one
    source."""
    readme = (ROOT / "README.md").read_text()
    start = readme.index(SIXTY_SECOND_HEADING)
    section = readme[start : readme.index("\n## ", start + 1)]
    blocks = re.findall(r"```\n(.*?)```", section, re.DOTALL)
    chosen = [b.rstrip("\n") for b in blocks if b.startswith("unstirred ")]
    if len(chosen) < 2:
        raise SystemExit(
            "README's 60-second path no longer has two `unstirred` command blocks; the Start here "
            f"page copies them from it and found {len(chosen)}"
        )
    return chosen[0], chosen[1]


def command_block(command, label):
    return f"""        <div class="command">
          <pre><code>{html.escape(command)}</code></pre>
          <button class="copy" type="button" data-command="{html.escape(command, quote=True)}" aria-label="Copy {label}">Copy</button>
        </div>"""


COPY_JS = """    <script>
      document.querySelectorAll(".command .copy").forEach(function (button) {
        button.addEventListener("click", function () {
          var text = button.getAttribute("data-command");
          function done() {
            button.textContent = "Copied";
            button.setAttribute("data-copied", "yes");
            setTimeout(function () {
              button.textContent = "Copy";
              button.removeAttribute("data-copied");
            }, 1600);
          }
          if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).then(done, done);
          } else {
            var area = document.createElement("textarea");
            area.value = text;
            document.body.appendChild(area);
            area.select();
            try { document.execCommand("copy"); } catch (error) {}
            document.body.removeChild(area);
            done();
          }
        });
      });
    </script>"""

# What to notice in each act, in the order a visitor should click them. The act ids are checked
# against the acts the gate serves, the same way the index of acts is.
ACT_NOTES = (
    ("act1-rule", "the door's rule, before anyone knocks"),
    ("act2-admitted", "Alice admitted, and what the door never saw"),
    ("act3-refused", "the same coins refused, and the refusal wrote nothing"),
    ("act4-newcomer", "Bob's coins are too new, shown past the break in the timeline"),
    ("act5-second-room", "a second room admits the same coins with no shared markers"),
)


def start_page(out, video_url, described):
    """The page a judge is sent to: four numbered steps, each with what it costs in minutes."""
    gate_init, verify = readme_commands()

    watch = (
        f'<a class="go" href="{html.escape(video_url, quote=True)}">Watch the walkthrough</a>'
        if video_url
        else '<a class="go quiet" href="../replay/index.html">Not published yet — open the acts instead</a>'
    )
    watch_text = (
        "What Unstirred is, and how it is used, in two minutes."
        if video_url
        else "The walkthrough is rendered but not published yet, so there is no link to it here "
        "rather than one that does not work. Step 2 is the same material as pages."
    )

    acts = "\n".join(
        f'            <li><a href="../replay/{act}.html">Act {i}</a> — {html.escape(note)}</li>'
        for i, (act, note) in enumerate(ACT_NOTES, start=1)
    )

    deeper = (
        ("../guides/getting-started.html", "Getting started"),
        ("../guides/integration.html", "Integration — how apps use it"),
        ("../api/index.html", "API reference"),
        ("../guides/spec.html", "SPEC — the soundness argument"),
        (f"{BLOB}CLAIMS.md", "CLAIMS — every claim and its evidence"),
    )
    deeper_html = "\n".join(
        f'            <li><a href="{html.escape(href, quote=True)}">{html.escape(text)}</a></li>'
        for href, text in deeper
    )

    body = f"""      <div class="page-head">
        <h1>Start here</h1>
        <p>Four steps, about six minutes, in the order that makes the rest make sense.</p>
      </div>
      <div class="steps">
        <article class="step">
          <span class="number">1</span>
          <h2>Watch <span class="time">2 min</span></h2>
          <p>{watch_text}</p>
          {watch}
        </article>
        <article class="step">
          <span class="number">2</span>
          <h2>Click through the demo <span class="time">3 min</span></h2>
          <p>Five acts, in order. What to notice in each:</p>
          <ol class="acts-in-order">
{acts}
          </ol>
          <p>These are replays of real testnet runs, not mock-ups. Each act links to the evidence
            file the run was recorded in.</p>
          <a class="go" href="../replay/index.html">Open the five acts</a>
        </article>
        <article class="step">
          <span class="number">3</span>
          <h2>Verify a real proof yourself <span class="time">~1 min after download</span></h2>
          <p>Download the release binary for your platform, then run these two commands from
            inside the unpacked directory. The first writes the gate's manifest and shows it is
            identical to the one in the repository; the second re-verifies a recorded proof
            against the live Zcash testnet, in your own process. Expect
            <strong>ADMITTED</strong>.</p>
{command_block(gate_init, "the gate init command")}
{command_block(verify, "the verify command")}
          <a class="go" href="{REPO}/releases">Get the binary</a>
        </article>
        <article class="step">
          <span class="number">4</span>
          <h2>Go deeper <span class="time">optional</span></h2>
          <p>Everything else, once the three steps above have shown you what this is.</p>
          <ul class="deeper">
{deeper_html}
          </ul>
        </article>
      </div>
      <p class="note">
        <strong>What is real, and what is replayed.</strong> The five acts are recordings: real
        runs against the Zcash testnet, re-rendered as pages so they need no chain, wallet or gate
        behind them. Step 3 is not a recording — it checks a real proof against the live testnet,
        on your own machine, in your own process.
      </p>"""
    doc_page(
        out,
        "Start here — Unstirred",
        "Four steps, about six minutes: watch it, click through the demo, verify a real proof yourself.",
        "Start",
        body,
        1,
        scripts=COPY_JS,
    )

    served = acts_the_gate_serves()
    listed = [act for act, _ in ACT_NOTES]
    if listed != served:
        raise SystemExit(
            "the Start here page's act order and the acts the gate serves do not match:\n"
            f"  start page: {listed}\n"
            f"  served:     {served}"
        )


# ------------------------------------------------------------------------ the index of acts


def acts_the_gate_serves():
    """The act names compiled into the gate, read from the list the replay generator writes."""
    listed = (ROOT / "crates/unstirred-gate/ui/replay/acts.rs").read_text()
    return re.findall(r'\("([a-z0-9-]+)",\s*include_str!', listed)


def replay_index(out, described):
    serves = acts_the_gate_serves()
    if list(described) != serves:
        raise SystemExit(
            "scripts/site/acts.json and the acts the gate serves do not match:\n"
            f"  described: {list(described)}\n"
            f"  served:    {serves}\n"
            "An act exists when a run produced it; say what a new one shows before publishing it."
        )
    body = [
        '      <div class="page-head">',
        "        <h1>The demo</h1>",
        "        <p>",
        "          Five recorded runs against the Zcash testnet, rendered by the gate's own page",
        "          code. A page carries its state inside it and its own policy forbids it to",
        "          connect anywhere, so none of this needs a gate, a chain, a wallet or a",
        "          nullifier server. <strong>Presented</strong> is the same page in the larger",
        "          type the walkthrough is filmed in.",
        "        </p>",
        "      </div>",
        '      <section class="acts">',
    ]
    for index, (act, facts) in enumerate(described.items(), start=1):
        body += [
            '        <article class="act">',
            f'          <span class="number">{index}</span>',
            f"          <h2>{html.escape(facts['title'])}</h2>",
            '          <span class="readings">',
            f'            <a href="{act}.html">Open</a>',
            f'            <a href="{act}.present.html">Presented</a>',
            "          </span>",
            f"          <p>{html.escape(facts['shows'])} <em>From {html.escape(facts['from'])}.</em></p>",
            "        </article>",
        ]
    body += [
        "      </section>",
        '      <p class="note">',
        '        <a href="picker.html">The room picker</a> is the page a gate serves at its root:',
        "        both rooms side by side, as they stood before anyone had knocked. Its cards open",
        "        the first act of each room, because a gate route is not a file here.",
        "      </p>",
    ]
    doc_page(
        out,
        "The demo — Unstirred",
        "Five recorded runs against the Zcash testnet, as pages.",
        "Demo",
        "\n".join(body),
        1,
    )


# ------------------------------------------------------------------------------- the two checks

LOCAL_LINK = re.compile(r'(?:href|src)="([^"#:]+?)(?:#[^"]*)?"')

# Every way a page could make the browser fetch something, with what it fetches. A link a
# reader clicks is not one of these: it is a navigation, and the site does link to GitHub.
SUBRESOURCE = re.compile(
    r'<(?:script|img|iframe|frame|object|embed|video|audio|track|source|base)\b[^>]*?'
    r'\s(?:src|data|href)="([^"]*)"',
    re.IGNORECASE,
)
# Any <link> whose rel makes the browser go and get something. Read as a whole tag and then
# picked apart, because `rel` and `href` are written in either order and a pattern that assumes
# one of them lets the other through: an off-origin stylesheet written `href=… rel=stylesheet`
# went unseen by exactly that bug while this check was being written.
LINK_TAG = re.compile(r"<link\b[^>]*>", re.IGNORECASE)
ATTR = re.compile(r'(\w[\w-]*)\s*=\s*"([^"]*)"')
FETCHING_REL = {"stylesheet", "icon", "shortcut", "preload", "prefetch", "apple-touch-icon", "manifest"}
PHONES_HOME = re.compile(r'@import|sendBeacon|importScripts', re.IGNORECASE)
HOST = re.compile(r"^(?:https?:)?//([^/]+)")

# The pages this repository writes or styles, and which are therefore allowed to fetch from the
# five origins in ALLOWED_ORIGINS. Everything else the site publishes — the five act pages, the
# room picker, and every page rustdoc generates — must fetch nothing but files beside it.
def may_reach_out(where):
    return (
        where in ("index.html", "start/index.html", "api/index.html", "replay/index.html")
        or where.startswith("guides/")
    )


def check_origins(site, pages):
    """What each page is allowed to ask for.

    The act pages and the room picker are the gate's own bytes and must ask no other origin for
    anything (LEAKS L10, A02c): that is what lets an act be opened from a file with nothing behind
    it. The four kinds of page this repository writes around them — landing, Start here, the
    guides, the API index — reach five named origins for fonts and the landing's background video,
    which is a deliberate choice recorded in DECISIONS.md. Any other origin, on any page, and any
    off-origin fetch at all from an act page, fails the build."""
    problems = []
    reaching = {}
    for page_path in pages:
        text = page_path.read_text()
        where = page_path.relative_to(site).as_posix()
        allowed_here = may_reach_out(where)
        targets = list(SUBRESOURCE.findall(text))
        for tag in LINK_TAG.findall(text):
            attrs = {k.lower(): v for k, v in ATTR.findall(tag)}
            rels = set(attrs.get("rel", "").lower().split())
            if rels & FETCHING_REL and attrs.get("href"):
                targets.append(attrs["href"])
        for target in targets:
            if target.startswith("data:"):
                continue
            if not (target.startswith("/") or "//" in target or ":" in target):
                continue
            host = HOST.match(target)
            if not host:
                problems.append(f"{where} fetches {target!r}, which is not a file beside it")
            elif not allowed_here:
                problems.append(
                    f"{where} fetches {host.group(1)}, and this page must ask no other origin"
                )
            elif host.group(1) not in ALLOWED_ORIGINS:
                problems.append(
                    f"{where} fetches {host.group(1)}, which is not one of the five allowed origins"
                )
            else:
                reaching.setdefault(host.group(1), set()).add(where)
        for found in PHONES_HOME.findall(text):
            problems.append(f"{where} carries {found!r}")
        if not allowed_here:
            for found in re.findall(r"integrity=|crossorigin", text):
                problems.append(f"{where} carries {found!r}")
    if problems:
        raise SystemExit("this site asks an origin it may not:\n  " + "\n  ".join(problems))

    closed = [
        p.relative_to(site).as_posix()
        for p in pages
        if not may_reach_out(p.relative_to(site).as_posix())
    ]
    print(
        f"origins: {len(closed)} page(s) fetch nothing but files beside them "
        f"(the five acts, the picker and rustdoc's own)"
    )
    for host in sorted(reaching):
        print(f"origins: {host} — on {len(reaching[host])} page(s) of this repository's own")
    missing = set(ALLOWED_ORIGINS) - set(reaching)
    if missing:
        print(f"origins: allowed but unused: {', '.join(sorted(missing))}")


def check_links(site):
    """Every local link in the site must resolve to a file in it. A published page that 404s on
    its own navigation is the one failure a visitor is guaranteed to find."""
    broken = []
    # Every page of this site, and the one page of the API reference this repository writes.
    # Rustdoc's own cross-links are cargo doc's to keep, and there are hundreds of them.
    pages = [
        p
        for p in sorted(site.rglob("*.html"))
        if "api" not in p.relative_to(site).parts or p.relative_to(site).as_posix() == "api/index.html"
    ]
    for page_path in pages:
        for target in LOCAL_LINK.findall(page_path.read_text()):
            if target.startswith(("//", "data:", "mailto:")):
                continue
            if not (page_path.parent / target).exists():
                broken.append(f"{page_path.relative_to(site)} -> {target}")
    if broken:
        raise SystemExit("broken local links in the site:\n  " + "\n  ".join(broken))
    print(f"links: every local link in {len(pages)} page(s) resolves inside the site")
    check_origins(site, pages)


# --------------------------------------------------------------------------- the API index

# Rustdoc writes one directory per crate and no page above them, so the four libraries get a page
# that names them. Without it /api is a directory listing or a 404.
LIBRARIES = (
    ("unstirred_core", "unstirred-core",
     "The shared contract: the manifest, the window and context rules, the package schema, the "
     "binding message, and the closed set of verdict codes."),
    ("unstirred_prover", "unstirred-prover",
     "The holder's half: read-only wallet access, selection, witnesses at h0, the private "
     "nullifier lookup at h1, the proof and its binding signature."),
    ("unstirred_verifier", "unstirred-verifier",
     "The gate's half: the engine over roots it establishes itself, and the store that admits one "
     "set of coins per context exactly once."),
    ("unstirred_gate", "unstirred-gate",
     "The HTTP service, the client the command line knocks with, and the page the gate serves "
     "itself."),
)


def api_index(out):
    cards = "\n".join(
        f"""        <article class="card">
          <h2>{name}</h2>
          <p>{text}</p>
          <a class="more" href="{module}/index.html">{module} →</a>
        </article>"""
        for module, name, text in LIBRARIES
    )
    body = f"""      <div class="page-head">
        <h1>API reference</h1>
        <p>Four libraries. The command line is a binary and has no page here.</p>
      </div>
      <section class="cards">
{cards}
      </section>"""
    doc_page(
        out,
        "API reference — Unstirred",
        "Rustdoc for the four libraries: core, prover, verifier and gate.",
        "API",
        body,
        1,
    )


def check_start_commands(site):
    """The commands on Start here must be, character for character, README's.

    They are generated from README, so this cannot normally fail — which is the point of running
    it anyway: it is the assertion that the one-source rule still holds after any change to either
    the page or the renderer's escaping."""
    page = (site / "start/index.html").read_text()
    on_page = [html.unescape(c) for c in re.findall(r"<pre><code>(.*?)</code></pre>", page, re.DOTALL)]
    expected = list(readme_commands())
    if on_page != expected:
        raise SystemExit(
            "the commands on the Start here page are not README's 60-second path:\n"
            f"  on the page: {on_page}\n"
            f"  in README:   {expected}"
        )
    print(f"start: {len(expected)} command block(s) are character-identical to README's")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("site")
    parser.add_argument("--video-url", default="", help="the published walkthrough; omitted while there is none")
    parser.add_argument("--check-links-only", action="store_true")
    args = parser.parse_args()

    site = pathlib.Path(args.site)
    if not args.check_links_only:
        described = json.loads((HERE / "site/acts.json").read_text())
        landing(site / "index.html", args.video_url)
        start_page(site / "start/index.html", args.video_url, described)
        replay_index(site / "replay/index.html", described)
        api_index(site / "api/index.html")
    check_links(site)
    if not args.check_links_only:
        check_start_commands(site)


if __name__ == "__main__":
    sys.exit(main())
