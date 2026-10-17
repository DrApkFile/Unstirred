#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== fmt";    cargo fmt --all -- --check
echo "== clippy"; cargo clippy --release --all-targets --all-features -- -D warnings
echo "== tests";  cargo test --release --workspace --all-features

echo
echo "GATE: PARTIAL"
echo "Implemented checks passed. Not yet implemented (see briefing/BUILD_CONTRACT.md):"
echo "  - leak scanner (A02)"
echo "  - claims check (every README claim links to evidence/)"
