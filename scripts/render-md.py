#!/usr/bin/env python3
"""Renders one of this repository's markdown files as a page of the published site.

Deliberately small and dependency-free: the site is built by a workflow that installs nothing but
the Rust toolchain, and a markdown library is a supply-chain dependency for a document. What it
covers is the markdown these files actually use — headings, paragraphs, fenced code, lists,
tables, block quotes, rules, links, emphasis and inline code — and it escapes everything else
rather than passing raw HTML through.

A link to another file in the repository is rewritten: to its own page when that file is published
as one, and to the file on GitHub when it is not, so no link on the site dead-ends.

    scripts/render-md.py SPEC.md site/guides/spec.html --title "The protocol" --depth 1
"""
import argparse
import html
import pathlib
import re
import sys

REPO = "https://github.com/DrApkFile/Unstirred/blob/master/"

# Files published as pages of this site, and the page each one becomes.
AS_PAGE = {
    "GETTING_STARTED.md": "getting-started.html",
    "INTEGRATION.md": "integration.html",
    "SPEC.md": "spec.html",
}

INLINE_CODE = re.compile(r"`([^`]+)`")
LINK = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
BOLD = re.compile(r"\*\*([^*]+)\*\*")
ITALIC = re.compile(r"(?<![*\w])\*([^*\n]+)\*(?!\*)")
HEADING = re.compile(r"^(#{1,6})\s+(.*)$")
ORDERED = re.compile(r"^(\d+)\.\s+(.*)$")
UNORDERED = re.compile(r"^[-*]\s+(.*)$")
TABLE_RULE = re.compile(r"^\|[\s:|-]+\|$")
COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)


def anchor(text):
    """GitHub's heading anchor, so a link written for the repository still lands on the page."""
    slug = re.sub(r"[^\w\s-]", "", text.lower())
    return re.sub(r"[-\s]+", "-", slug).strip("-")


def destination(href, depth):
    """Where a link in the markdown should point once the file is a page of the site."""
    if href.startswith(("http://", "https://", "#", "mailto:")):
        return href
    path, _, fragment = href.partition("#")
    fragment = f"#{fragment}" if fragment else ""
    if path in AS_PAGE:
        return AS_PAGE[path] + fragment
    if not path:
        return fragment
    return REPO + path + fragment


def inline(text, depth):
    """One line of markdown as HTML. Code spans are lifted out first so nothing is applied inside
    them: a `--root-source` in backticks is not emphasis, and a `<div>` is not a tag."""
    spans = []

    def park(match):
        spans.append(match.group(1))
        return f"\x00{len(spans) - 1}\x00"

    text = INLINE_CODE.sub(park, text)
    text = html.escape(text, quote=False)
    text = LINK.sub(
        lambda m: f'<a href="{html.escape(destination(m.group(2), depth), quote=True)}">{m.group(1)}</a>',
        text,
    )
    text = BOLD.sub(r"<strong>\1</strong>", text)
    text = ITALIC.sub(r"<em>\1</em>", text)
    return re.sub(
        r"\x00(\d+)\x00",
        lambda m: f"<code>{html.escape(spans[int(m.group(1))], quote=False)}</code>",
        text,
    )


def render(markdown, depth):
    out, contents = [], []
    lines = COMMENT.sub("", markdown).split("\n")
    index, paragraph, list_kind = 0, [], None

    def close_paragraph():
        if paragraph:
            out.append(f"<p>{inline(' '.join(paragraph), depth)}</p>")
            paragraph.clear()

    def close_list():
        nonlocal list_kind
        if list_kind:
            out.append(f"</{list_kind}>")
            list_kind = None

    while index < len(lines):
        line = lines[index]
        stripped = line.strip()

        if stripped.startswith("```"):
            close_paragraph()
            close_list()
            index += 1
            body = []
            while index < len(lines) and not lines[index].strip().startswith("```"):
                body.append(lines[index])
                index += 1
            out.append(f"<pre><code>{html.escape(chr(10).join(body), quote=False)}</code></pre>")
            index += 1
            continue

        if not stripped:
            close_paragraph()
            close_list()
            index += 1
            continue

        if stripped in ("---", "***", "___"):
            close_paragraph()
            close_list()
            out.append("<hr />")
            index += 1
            continue

        heading = HEADING.match(stripped)
        if heading:
            close_paragraph()
            close_list()
            level, text = len(heading.group(1)), heading.group(2)
            slug = anchor(text)
            if level == 2:
                contents.append((slug, text))
            out.append(f'<h{level} id="{slug}">{inline(text, depth)}</h{level}>')
            index += 1
            continue

        if stripped.startswith("|") and index + 1 < len(lines) and TABLE_RULE.match(lines[index + 1].strip()):
            close_paragraph()
            close_list()
            cells = [c.strip() for c in stripped.strip("|").split("|")]
            rows = []
            index += 2
            while index < len(lines) and lines[index].strip().startswith("|"):
                rows.append([c.strip() for c in lines[index].strip().strip("|").split("|")])
                index += 1
            head = "".join(f"<th>{inline(c, depth)}</th>" for c in cells)
            body = "".join(
                "<tr>" + "".join(f"<td>{inline(c, depth)}</td>" for c in row) + "</tr>" for row in rows
            )
            out.append(f"<div class=\"scroll\"><table><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>")
            continue

        if stripped.startswith(">"):
            close_paragraph()
            close_list()
            quote = []
            while index < len(lines) and lines[index].strip().startswith(">"):
                quote.append(lines[index].strip().lstrip(">").strip())
                index += 1
            out.append(f"<blockquote><p>{inline(' '.join(quote), depth)}</p></blockquote>")
            continue

        item = UNORDERED.match(stripped) or ORDERED.match(stripped)
        if item:
            close_paragraph()
            kind = "ul" if UNORDERED.match(stripped) else "ol"
            if list_kind != kind:
                close_list()
                out.append(f"<{kind}>")
                list_kind = kind
            text = item.group(1) if kind == "ul" else item.group(2)
            index += 1
            # A wrapped list item: the continuation lines are indented and belong to the same item.
            while index < len(lines) and lines[index].startswith("  ") and lines[index].strip():
                text += " " + lines[index].strip()
                index += 1
            out.append(f"<li>{inline(text, depth)}</li>")
            continue

        if list_kind:
            close_list()
        paragraph.append(stripped)
        index += 1

    close_paragraph()
    close_list()
    return "\n".join(out), contents


PAGE = """<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="referrer" content="no-referrer" />
    <meta name="description" content="{description}" />
    <title>{title} — Unstirred</title>
    <link rel="icon" type="image/png" href="{up}favicon.png" />
    <link rel="stylesheet" href="{up}styles.css" />
  </head>
  <body>
    <header class="masthead">
      <a class="wordmark" href="{up}index.html"><img src="{up}icon.png" alt="Unstirred" /></a>
      <nav>
        <a href="{up}guides/getting-started.html">Getting started</a>
        <a href="{up}guides/integration.html">Integration</a>
        <a href="{up}guides/spec.html">Spec</a>
        <a href="{up}replay/index.html">Demo</a>
        <a href="{up}api/unstirred_core/index.html">API</a>
      </nav>
    </header>
    <main class="prose">
{contents}
{body}
    </main>
    <footer>
      <p>
        <a href="https://github.com/DrApkFile/Unstirred/blob/master/{source}">This page's source</a>
        in the repository. MIT OR Apache-2.0.
      </p>
    </footer>
  </body>
</html>
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source")
    parser.add_argument("out")
    parser.add_argument("--title", required=True)
    parser.add_argument("--description", default="")
    parser.add_argument("--depth", type=int, default=1, help="how many directories below the site root")
    args = parser.parse_args()

    source = pathlib.Path(args.source)
    body, contents = render(source.read_text(), args.depth)
    up = "../" * args.depth

    listed = "".join(f'<li><a href="#{slug}">{html.escape(text)}</a></li>' for slug, text in contents)
    nav = f'<nav class="contents"><p>On this page</p><ul>{listed}</ul></nav>' if listed else ""

    out = pathlib.Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        PAGE.format(
            title=html.escape(args.title),
            description=html.escape(args.description or args.title),
            body=body,
            contents=nav,
            up=up,
            source=source.name,
        )
    )
    print(f"{args.source} -> {args.out} ({len(body)} bytes of body, {len(contents)} sections)")


if __name__ == "__main__":
    sys.exit(main())
