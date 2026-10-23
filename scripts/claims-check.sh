#!/usr/bin/env bash
# Every claim this repo makes must link to a file that settles it, and every recorded run must be
# accounted for. CLAIMS.md is the ledger; this is the check that keeps it honest.
set -euo pipefail
cd "$(dirname "$0")/.."

python3 -I - <<'PY'
import pathlib
import re
import sys

STATUSES = {"PROVEN", "SYNTHETIC", "GATED", "MEASURED", "NOT BUILT"}
ledger = pathlib.Path("CLAIMS.md")
problems = []

rows = [
    [cell.strip() for cell in line.strip().strip("|").split("|")]
    for line in ledger.read_text().splitlines()
    if line.startswith("|") and not set(line) <= set("|- ")
]
header, rows = rows[0], rows[1:]
if header[:4] != ["Claim", "Status", "Evidence", "Claimed in"]:
    sys.exit(f"CLAIMS CHECK FAILED: the ledger's columns changed: {header}")

cited = set()
for claim, status, evidence, _where in (row[:4] for row in rows):
    if status not in STATUSES:
        problems.append(f"{claim!r} has status {status!r}, which is not one of {sorted(STATUSES)}")
    links = re.findall(r"\(([^)]+\.(?:md|txt|json))\)", evidence)
    if status != "NOT BUILT" and not links:
        problems.append(f"{claim!r} is {status} but cites no evidence file")
    if status == "NOT BUILT" and links:
        problems.append(f"{claim!r} is NOT BUILT but cites evidence")
    for link in links:
        cited.add(link)
        if not pathlib.Path(link).is_file():
            problems.append(f"{claim!r} cites {link}, which does not exist")

# Every document that cites a recorded run is held to it, each link read from where that
# document sits, so LEAKS.md's ../evidence/… resolves the same way a reader's click does.
for document in ("README.md", "CLAIMS.md", "SPEC.md", "briefing/LEAKS.md"):
    here = pathlib.Path(document)
    for link in re.findall(r"\]\(((?:\.\./)?(?:evidence|briefing)/[^)]+)\)", here.read_text()):
        if not (here.parent / link).is_file():
            problems.append(f"{document} links {link}, which does not exist")

recorded = {
    str(path)
    for path in pathlib.Path("evidence").iterdir()
    if path.is_file() and path.name != "README.md"
}
for run in sorted(recorded - cited):
    problems.append(f"{run} is a recorded run no claim cites; add it to CLAIMS.md or remove it")

if problems:
    print("CLAIMS CHECK FAILED:", file=sys.stderr)
    for problem in problems:
        print(f"  - {problem}", file=sys.stderr)
    sys.exit(1)

proven = sum(1 for row in rows if row[1] == "PROVEN")
synthetic = sum(1 for row in rows if row[1] == "SYNTHETIC")
gated = sum(1 for row in rows if row[1] == "GATED")
measured = sum(1 for row in rows if row[1] == "MEASURED")
not_built = sum(1 for row in rows if row[1] == "NOT BUILT")
print(
    f"claims check: {len(rows)} claims, {proven} proven on the real testnet, {synthetic} synthetic, "
    f"{gated} settled by a recorded gate run, {measured} measured by a recorded tool run, "
    f"{not_built} named as not built"
)
PY
