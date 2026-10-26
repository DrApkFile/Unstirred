#!/usr/bin/env bash
# Builds the published site into one directory: the recorded acts, the API reference, the guides
# and a landing page.
#
# Everything here is produced from this repository by this script, so the site a workflow
# publishes and the site you can build locally are the same site. Nothing is fetched from anywhere
# at build time but the crates the workspace already pins, and no page the site serves asks any
# other origin for anything (LEAKS L10): the stylesheet, the icons and the acts are all files in
# the output.
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

echo "== the design tokens the gate's pages use, plus the site's own layout"
cat crates/unstirred-gate/ui/tokens.css scripts/site/site.css > "$out/styles.css"

echo "== the icons"
cp assets/icon.png "$out/icon.png"
# python3 rather than base64(1), whose decode flag is spelled differently on macOS, and which is
# the one dependency the gate already has beyond the toolchain.
python3 -I -c 'import base64,pathlib,sys; pathlib.Path(sys.argv[2]).write_bytes(base64.b64decode(pathlib.Path(sys.argv[1]).read_text()))' \
    crates/unstirred-gate/ui/favicon.b64 "$out/favicon.png"

if [ "${SKIP_API:-}" = "1" ]; then
    echo "== the API reference: skipped (SKIP_API=1)"
    mkdir -p "$out/api/unstirred_core"
    printf '<!doctype html><title>Not built</title><p>cargo doc was skipped for this build.\n' \
        > "$out/api/unstirred_core/index.html"
else
    echo "== the API reference"
    cargo doc --no-deps --workspace --all-features --quiet
    cp -R target/doc "$out/api"
fi

# Rustdoc writes one directory per crate and no page above them, so the four libraries get a page
# that names them. Without it /api is a directory listing or a 404.
cat > "$out/api/index.html" <<'HTML'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>API reference — Unstirred</title>
    <link rel="icon" type="image/png" href="../favicon.png" />
    <link rel="stylesheet" href="../styles.css" />
  </head>
  <body>
    <header class="masthead">
      <a class="wordmark" href="../index.html"><img src="../icon.png" alt="Unstirred" /></a>
      <nav>
        <a href="../replay/index.html">Demo</a>
        <a href="../guides/getting-started.html">Getting started</a>
        <a href="../guides/integration.html">Integration</a>
        <a href="../guides/spec.html">Spec</a>
      </nav>
    </header>
    <main>
      <section class="hero">
        <h1>API reference</h1>
        <p>Four libraries. The command line is a binary and has no page here.</p>
      </section>
      <section class="cards">
        <article class="card">
          <h2>unstirred-core</h2>
          <p>The shared contract: the manifest, the window and context rules, the package schema,
            the binding message, and the closed set of verdict codes.</p>
          <a class="more" href="unstirred_core/index.html">unstirred_core →</a>
        </article>
        <article class="card">
          <h2>unstirred-prover</h2>
          <p>The holder's half: read-only wallet access, selection, witnesses at h0, the private
            nullifier lookup at h1, the proof and its binding signature.</p>
          <a class="more" href="unstirred_prover/index.html">unstirred_prover →</a>
        </article>
        <article class="card">
          <h2>unstirred-verifier</h2>
          <p>The gate's half: the engine over roots it establishes itself, and the store that
            admits one set of coins per context exactly once.</p>
          <a class="more" href="unstirred_verifier/index.html">unstirred_verifier →</a>
        </article>
        <article class="card">
          <h2>unstirred-gate</h2>
          <p>The HTTP service, the client the command line knocks with, and the page the gate
            serves itself.</p>
          <a class="more" href="unstirred_gate/index.html">unstirred_gate →</a>
        </article>
      </section>
    </main>
    <footer>
      <p>
        <a href="https://github.com/DrApkFile/Unstirred">The repository</a> · MIT OR Apache-2.0 ·
        Zcash <strong>testnet</strong> only; nothing here has been exercised against mainnet.
      </p>
    </footer>
  </body>
</html>
HTML

echo "== the landing page and the index of acts"
scripts/site-pages.py "$out" ${VIDEO_URL:+--video-url "$VIDEO_URL"}

# GitHub Pages serves the artifact as-is, but a .nojekyll costs nothing and keeps rustdoc's
# underscore-prefixed files safe under any other static host.
touch "$out/.nojekyll"

echo
du -sh "$out"
echo "site built in $out — open $out/index.html"
