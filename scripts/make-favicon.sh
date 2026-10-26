#!/usr/bin/env bash
# Derives the favicon every Unstirred page carries from the one logo file, assets/icon.png.
#
# The logo is a wide wordmark; only its circular mark reads at 32 px, so this cuts that mark out
# (it occupies exactly 200x200 at +180+200), pads it with the design system's own --paper, and
# scales it down. The result is written as base64 text rather than a binary file, for two reasons:
# crates/unstirred-gate/ui/ is scanned line by line by the leak scan, which reads every file as
# text, and the page embeds the bytes as a `data:` URI anyway, so base64 is the form it is used in.
#
# The pages that carry it fetch nothing: a `data:` URI is bytes in the document, not a request to
# an origin (A02c, LEAKS L10). Run this only when the logo changes; the output is committed.
set -euo pipefail
cd "$(dirname "$0")/.."

source=assets/icon.png
out=crates/unstirred-gate/ui/favicon.b64
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

magick "$source" -crop 200x200+180+200 +repage \
    -background '#f4f3ef' -gravity center -extent 224x224 \
    -filter Lanczos -resize 32x32 \
    -strip -define png:exclude-chunk=date,time,tIME \
    "$work/favicon.png"

base64 -w 76 < "$work/favicon.png" > "$out"

echo "$out: $(wc -c < "$work/favicon.png") bytes of PNG, $(wc -c < "$out") bytes of base64"
echo "source sha256: $(sha256sum "$source" | cut -d' ' -f1)"
