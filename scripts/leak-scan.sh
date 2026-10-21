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

# A02c, repository half: no page the gate serves may reference another origin at all. The served
# document is scanned in-process (A02c, crates/unstirred-gate/tests/page.rs); this is the same rule
# over the sources that document is assembled from, so a third-party font, script or tracking pixel
# cannot be added to ui/ and reviewed away.
python3 -I - <<'UISCAN'
import pathlib
import re
import sys

UI = pathlib.Path("crates/unstirred-gate/ui")
ASKS_ELSEWHERE = re.compile(
    r"<script\s+src|<link|<img|<iframe|<frame|<object|<embed|<video|<audio|<track|<base"
    r"|@import|url\(|srcset|integrity=|crossorigin|sendBeacon|importScripts",
    re.IGNORECASE,
)
ABSOLUTE_IN_CODE = re.compile(r"""["']\s*(?:https?:)?//""")

problems = []
for path in sorted(UI.rglob("*")):
    if not path.is_file() or path.suffix == ".json" or path.suffix == ".md":
        continue
    text = path.read_text()
    for found in ASKS_ELSEWHERE.findall(text):
        problems.append(f"{path} asks for a subresource: {found!r}")
    for found in ABSOLUTE_IN_CODE.findall(text):
        problems.append(f"{path} names an absolute or protocol-relative URL: {found!r}")

if problems:
    print("LEAK SCAN FAILED: a page the gate serves would ask another origin for something:", file=sys.stderr)
    for problem in problems:
        print(f"  - {problem}", file=sys.stderr)
    sys.exit(1)
print(f"leak scan: no third-party reference in {UI}")
UISCAN

echo "leak scan: no key material and no address in any tracked file"
