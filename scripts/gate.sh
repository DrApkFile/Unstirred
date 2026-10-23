#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== fmt";    cargo fmt --all -- --check
echo "== clippy"; cargo clippy --release --all-targets --all-features -- -D warnings
echo "== tests";  cargo test --release --workspace --all-features
echo "== leaks";  ./scripts/leak-scan.sh
echo "== claims"; ./scripts/claims-check.sh
echo "== status"; python3 -I scripts/status-table.py --check

echo
echo "GATE: PASS"
echo "fmt, clippy, the whole test suite, the leak scan (A02, repository half), the claims check"
echo "and the README status table against CLAIMS.md."
echo "What the gate still cannot see (see briefing/BUILD_CONTRACT.md):"
echo "  - a leak through a channel rather than a string, in a running process"
echo "  - whether a recorded run in evidence/ says what this ledger says it says"
