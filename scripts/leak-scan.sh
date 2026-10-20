#!/usr/bin/env bash
# A02, repository half: no key material and no holder identifier may be committed to this repo.
#
# What it proves: no tracked file contains an age secret key, an extended or unified viewing or
# spending key, a PEM private key, or a shielded address, and every proof package recorded in
# evidence/ carries exactly the nine fields of the package schema and nothing else.
#
# What it does not prove: anything about a running process, a log this repo never saw, or a leak
# through a channel rather than a string. Those are the in-process scans (A02 over the package,
# A02b over the gate database) and briefing/LEAKS.md.
set -euo pipefail
cd "$(dirname "$0")/.."

scan() {
    local what="$1" pattern="$2" found=0
    git grep --line-number -I --extended-regexp -e "$pattern" -- . || found=$?
    case "$found" in
        0) echo "LEAK SCAN FAILED: $what (the lines above)" >&2; exit 1 ;;
        1) ;;
        *) echo "LEAK SCAN FAILED: the scan for \"$what\" could not run (git grep exit $found)" >&2; exit 1 ;;
    esac
}

scan "an age secret key is committed" 'AGE-SECRET-KEY-1[0-9A-Z]{10,}'
scan "an age plugin identity is committed" 'AGE-PLUGIN-[A-Z]+-1[0-9A-Z]{10,}'
scan "a sapling or unified spending or viewing key is committed" \
    'secret-extended-key[-a-z0-9]*1[0-9a-z]{20,}|zxview[a-z]*1[0-9a-z]{20,}|uview[a-z]*1[0-9a-z]{20,}'
scan "a PEM private key is committed" 'BEGIN [A-Z ]*PRIVATE KEY'
scan "a shielded or unified address is committed" \
    '\bu(test)?1[0-9a-z]{70,}|\bzs1[0-9a-z]{70,}|\bztestsapling1[0-9a-z]{70,}'

python3 -I - <<'PY'
import json
import pathlib
import sys

FIELDS = {
    "version",
    "context",
    "network",
    "window",
    "instance",
    "proof",
    "session_pubkey",
    "challenge",
    "binding_signature",
}

for path in sorted(pathlib.Path("evidence").glob("*.json")):
    body = json.loads(path.read_text())
    if not isinstance(body, dict) or "proof" not in body:
        continue
    extra = set(body) - FIELDS
    if extra:
        sys.exit(f"LEAK SCAN FAILED: {path} carries fields outside the package schema: {sorted(extra)}")
    window = body.get("window", {})
    if set(window) - {"h0", "h1"}:
        sys.exit(f"LEAK SCAN FAILED: {path} carries a window with extra fields: {sorted(window)}")
    print(f"leak scan: {path} carries only the package schema's fields")
PY

echo "leak scan: no key material and no address in any tracked file"
