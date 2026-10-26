#!/usr/bin/env python3
"""Writes the pages of the published site that are not a rendered markdown file, and then checks
that every local link in the finished site resolves.

Two pages are written here: the landing page, and the index of recorded acts. The acts come from
`scripts/site/acts.json`, and a mismatch between that file and the acts the gate actually serves
is an error rather than a page with a hole in it — so a newly recorded act fails the build until
somebody says what it shows.

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

NAV = """      <nav>
        <a href="{up}replay/index.html">Demo</a>
        <a href="{up}guides/getting-started.html">Getting started</a>
        <a href="{up}guides/integration.html">Integration</a>
        <a href="{up}guides/spec.html">Spec</a>
        <a href="{up}api/unstirred_core/index.html">API</a>
      </nav>"""

SHELL = """<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="referrer" content="no-referrer" />
    <meta name="description" content="{description}" />
    <title>{title}</title>
    <link rel="icon" type="image/png" href="{up}favicon.png" />
    <link rel="stylesheet" href="{up}styles.css" />
  </head>
  <body>
    <header class="masthead">
      <a class="wordmark" href="{up}index.html"><img src="{up}icon.png" alt="Unstirred" /></a>
{nav}
    </header>
    <main>
{body}
    </main>
    <footer>
      <p>
        <a href="{repo}">The repository</a> · MIT OR Apache-2.0 ·
        Zcash <strong>testnet</strong> only; nothing here has been exercised against mainnet.
      </p>
    </footer>
  </body>
</html>
"""


def page(out, title, description, body, depth):
    up = "../" * depth
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        SHELL.format(
            title=html.escape(title),
            description=html.escape(description),
            body=body,
            up=up,
            nav=NAV.format(up=up),
            repo=REPO,
        )
    )
    print(f"{out} ({len(body)} bytes of body)")


def acts_the_gate_serves():
    """The act names compiled into the gate, read from the list the replay generator writes."""
    listed = (ROOT / "crates/unstirred-gate/ui/replay/acts.rs").read_text()
    return re.findall(r'\("([a-z0-9-]+)",\s*include_str!', listed)


def video_line(video_url):
    if video_url:
        return (
            f'<p class="note">The two-minute walkthrough is at '
            f'<a href="{html.escape(video_url, quote=True)}">{html.escape(video_url)}</a>. '
            "Every page it shows is one of the acts below, opened in a browser.</p>"
        )
    return (
        '<p class="note"><strong>The walkthrough video is not published yet</strong>, so there is '
        "no link to it here rather than a link that does not work. Every page it is cut from is "
        "one of the acts below, and you can open those now.</p>"
    )


def landing(out, video_url):
    cards = [
        (
            "replay/index.html",
            "See it without running anything",
            "Five recorded runs, each rendered as a page that polls nothing and reaches no "
            "network. No chain, no wallet, no gate.",
            "Open the demo",
        ),
        (
            "guides/getting-started.html",
            "Do it yourself",
            "Three paths, step by step: a holder with a wallet, an owner standing a gate up, a "
            "developer reading the code.",
            "Getting started",
        ),
        (
            "guides/spec.html",
            "Read what is actually proved",
            "The statement, the soundness argument for pinning the two roots at two different "
            "heights, the wire formats, and every limit.",
            "The protocol",
        ),
        (
            "guides/integration.html",
            "Build on it",
            "The HTTP flow in any language, the Rust crates, embedding the prover in a wallet, "
            "and one context per purpose.",
            "Integration",
        ),
        (
            "api/unstirred_core/index.html",
            "The API reference",
            "Rustdoc for the four libraries: core, prover, verifier and gate. Every public item, "
            "with examples that are compiled.",
            "Browse the API",
        ),
        (
            REPO,
            "The repository",
            "Every claim with the file that settles it, every privacy channel with its test, and "
            "the gate that has to pass before any of it is said.",
            "On GitHub",
        ),
    ]
    body = [
        '      <section class="hero">',
        '        <img src="icon.png" alt="Unstirred" />',
        "        <h1>Proof your coins stayed put. Nothing else.</h1>",
        "        <p>",
        "          A Zcash holder proves to a gate that they held at least 0.125 ZEC in the",
        "          Ironwood pool, <strong>unmoved</strong>, from one block to another — without",
        "          revealing their address, their balance, which coins, or anything they have ever",
        "          spent. Each gate admits the same coins exactly once, and two gates cannot tell",
        "          they saw the same holder.",
        "        </p>",
        "      </section>",
        f"      {video_line(video_url)}",
        '      <section class="cards">',
    ]
    for href, title, text, more in cards:
        body += [
            '        <article class="card">',
            f"          <h2>{title}</h2>",
            f"          <p>{text}</p>",
            f'          <a class="more" href="{html.escape(href, quote=True)}">{more} →</a>',
            "        </article>",
        ]
    body += [
        "      </section>",
        '      <p class="note">',
        "        It is built on",
        f'        <a href="{REPO}/blob/master/SPEC.md">Valar Group\'s audited delegation circuit</a>,',
        "        used unchanged and pinned, with the note-commitment root pinned at the first",
        "        block and the nullifier-set root at the second — which is what turns a snapshot",
        "        proof into a proof about a span of time.",
        "      </p>",
    ]
    page(out, "Unstirred", "Prove your Zcash coins stayed put. Nothing else.", "\n".join(body), 0)


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
        '      <section class="hero">',
        "        <h1>Five recorded runs, as pages</h1>",
        "        <p>",
        "          Each act is a real run against the Zcash testnet, rendered by the gate's own",
        "          page code. A page carries its state inside it and its own policy forbids it to",
        "          connect anywhere, so none of this needs a gate, a chain, a wallet or a",
        "          nullifier server. <strong>Presented</strong> is the same page in the larger type",
        "          the walkthrough is filmed in.",
        "        </p>",
        "      </section>",
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
    page(out, "The demo — Unstirred", "Five recorded runs against the Zcash testnet, as pages.", "\n".join(body), 1)


LOCAL_LINK = re.compile(r'(?:href|src)="([^"#:]+?)(?:#[^"]*)?"')

# Every way a page could make the browser fetch something, with what it fetches. A link a
# reader clicks is not one of these: it is a navigation, and the site does link to GitHub.
SUBRESOURCE = re.compile(
    r'<(?:script|img|iframe|frame|object|embed|video|audio|track|source|base)\b[^>]*?'
    r'\s(?:src|data|href)="([^"]*)"',
    re.IGNORECASE,
)
STYLESHEET = re.compile(r'<link\b[^>]*?rel="stylesheet"[^>]*?href="([^"]*)"', re.IGNORECASE)
ICON = re.compile(r'<link\b[^>]*?rel="icon"[^>]*?href="([^"]*)"', re.IGNORECASE)
PHONES_HOME = re.compile(r'@import|integrity=|crossorigin|sendBeacon|importScripts', re.IGNORECASE)


def check_origins(site, pages):
    """No page of this site may ask another origin for anything (LEAKS L10). A `data:` icon is
    bytes already in the document, and an `<a href>` to GitHub is a link a reader clicks, not a
    request the browser makes on its own."""
    problems = []
    for page_path in pages:
        text = page_path.read_text()
        where = page_path.relative_to(site)
        for pattern in (SUBRESOURCE, STYLESHEET, ICON):
            for target in pattern.findall(text):
                if target.startswith("data:"):
                    continue
                if target.startswith("/") or "//" in target or ":" in target:
                    problems.append(f"{where} fetches {target!r}, which is not a file beside it")
        for found in PHONES_HOME.findall(text):
            problems.append(f"{where} carries {found!r}")
    if problems:
        raise SystemExit("this site would ask another origin for something:\n  " + "\n  ".join(problems))
    print(f"origins: no page of this site fetches anything but files inside it")


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
        replay_index(site / "replay/index.html", described)
    check_links(site)


if __name__ == "__main__":
    sys.exit(main())
