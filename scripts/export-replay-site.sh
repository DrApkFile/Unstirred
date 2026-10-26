#!/usr/bin/env bash
# Writes the recorded acts and the room picker into site/ as static HTML.
#
# The pages come out of the gate's own page module (crates/unstirred-gate/examples/export-replay-site.rs),
# so a file in site/ is byte for byte what `unstirred gate serve` answers at
# /room/<context>?replay=<act>, in both readings: <act>.html and <act>.present.html. A replay page
# polls nothing and its Content-Security-Policy forbids it to connect anywhere, so site/ needs no
# gate, no chain, no wallet and no nf-server to open.
#
# The demo video's browser scenes embed these files, and a published replay site would serve the
# same ones. Usage: scripts/export-replay-site.sh [directory]   (default: site)
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="${1:-$root/site}"

cd "$root"
cargo run --release --quiet -p unstirred-gate --example export-replay-site -- "$out"

echo
echo "The picker's room cards open the first recorded act of each room, because /room/<context> is"
echo "a gate route with no file behind it here (A51). Every link in $out resolves inside $out."
