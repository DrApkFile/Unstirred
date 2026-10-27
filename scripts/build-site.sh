#!/usr/bin/env bash
# Builds the published site into one directory: the recorded acts, the API reference, the guides
# and a landing page.
#
# Everything here is produced from this repository by this script, so the site a workflow
# publishes and the site you can build locally are the same site. Nothing is fetched from anywhere
# at build time but the crates the workspace already pins.
#
# At view time the site is in two halves. The five recorded act pages and the room picker are the
# gate's own bytes and ask no other origin for anything (LEAKS L10, A02c) — that is what lets an
# act be opened from a file with nothing behind it. The pages this repository writes around them —
# the landing page, Start here, the three guides, the API index and the index of acts — fetch
# fonts from three services, the landing's background video from one bucket and the walkthrough
# player from www.youtube-nocookie.com: six origins, each with a rule saying which pages may hold
# it. That is a deliberate choice, recorded in DECISIONS.md, and scripts/site-pages.py fails the
# build if any page reaches an origin outside the list, if an allowed origin turns up on a page its
# own rule does not cover, or if an act page reaches one at all.
#
#     scripts/build-site.sh [directory]        (default: site)
#     SKIP_API=1 scripts/build-site.sh         (everything but cargo doc, which is the slow part)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/site}"
cd "$root"

rm -rf "$out"
mkdir -p "$out"

echo "== the recorded acts"
scripts/export-replay-site.sh "$out/replay" > /dev/null
# The exporter writes the gate's own room picker as the directory's index; the site's index of the
# acts takes that name, so the picker becomes a page of its own.
mv "$out/replay/index.html" "$out/replay/picker.html"

echo "== the guides"
scripts/render-md.py docs/GETTING_STARTED.md "$out/guides/getting-started.html" \
    --title "Getting started" --source-path docs/GETTING_STARTED.md \
    --description "Three paths, step by step: a holder, a gate owner, a developer."
scripts/render-md.py docs/INTEGRATION.md "$out/guides/integration.html" \
    --title "Integration" --source-path docs/INTEGRATION.md \
    --description "Building on Unstirred: the HTTP flow, the Rust crates, and one context per purpose."
scripts/render-md.py SPEC.md "$out/guides/spec.html" \
    --title "The protocol" --source-path SPEC.md \
    --description "What is proved, why it holds, the wire formats, and every limit."

echo "== the site's own stylesheet"
# The gate's tokens.css is deliberately NOT concatenated here any more. It is the light theme of
# the product UI, and the act pages carry it inside themselves; the site around them is its own
# dark design, and mixing the two left every variable defined twice.
cp scripts/site/site.css "$out/styles.css"

echo "== the display font, if it is in the tree"
# The fallback display face. BubbledotICG-FinePos is hosted; this is the local face named after it
# in the stack, and it is optional: no file, no @font-face rule, and the stack falls through to
# monospace rather than the page asking for a font that is not there.
if [ -f assets/fonts/GeistPixel-Circle.woff2 ]; then
    mkdir -p "$out/fonts"
    cp assets/fonts/GeistPixel-Circle.woff2 "$out/fonts/GeistPixel-Circle.woff2"
    cat >> "$out/styles.css" <<'FONT'

@font-face {
  font-family: "Geist Pixel Circle";
  src: url("fonts/GeistPixel-Circle.woff2") format("woff2");
  font-weight: 400;
  font-display: swap;
}
FONT
    echo "   fonts/GeistPixel-Circle.woff2"
else
    echo "   assets/fonts/GeistPixel-Circle.woff2 is not in the tree: no @font-face written"
fi

echo "== the icons"
cp assets/icon.png "$out/icon.png"
# The header's mark is the emblem on its own, cropped out of the wordmark so the two cannot drift.
# Regenerate with:
#   ffmpeg -y -i assets/icon.png -vf "crop=216:216:172:189" assets/icon-mark.png
cp assets/icon-mark.png "$out/icon-mark.png"
# python3 rather than base64(1), whose decode flag is spelled differently on macOS, and which is
# the one dependency the gate already has beyond the toolchain.
python3 -I -c 'import base64,pathlib,sys; pathlib.Path(sys.argv[2]).write_bytes(base64.b64decode(pathlib.Path(sys.argv[1]).read_text()))' \
    crates/unstirred-gate/ui/favicon.b64 "$out/favicon.png"

if [ "${SKIP_API:-}" = "1" ]; then
    echo "== the API reference: skipped (SKIP_API=1)"
    # One stub per library, because the API index names all four and a skipped build must still
    # be a site whose links resolve.
    for crate in unstirred_core unstirred_prover unstirred_verifier unstirred_gate; do
        mkdir -p "$out/api/$crate"
        printf '<!doctype html><title>Not built</title><p>cargo doc was skipped for this build.\n' \
            > "$out/api/$crate/index.html"
    done
else
    echo "== the API reference"
    cargo doc --no-deps --workspace --all-features --quiet
    cp -R target/doc "$out/api"
fi

echo "== the landing page, Start here, the index of acts and the API index"
scripts/site-pages.py "$out" ${VIDEO_URL:+--video-url "$VIDEO_URL"}

# GitHub Pages serves the artifact as-is, but a .nojekyll costs nothing and keeps rustdoc's
# underscore-prefixed files safe under any other static host.
touch "$out/.nojekyll"

echo
du -sh "$out"
echo "site built in $out — open $out/index.html"
