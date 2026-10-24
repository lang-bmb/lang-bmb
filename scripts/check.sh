#!/usr/bin/env bash
# Single verification gate (AUTONOMY.md §2 step 5, INF-05).
# Local runs and CI use this same script. Exits non-zero on the first failure.
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '== %s\n' "$1"; }

step "format";  cargo fmt --all --check
step "lint";    cargo clippy --all-targets --quiet -- -D warnings
step "test";    cargo test --quiet
step "backlog"; python3 scripts/check_backlog.py
step "docs";    cargo run -q -- codes --md | diff -u docs/diagnostics.md - > /dev/null || { echo "docs/diagnostics.md is stale: run cargo run -q -- codes --md > docs/diagnostics.md"; exit 1; }
step "bench m1"; tmp=$(mktemp)
python3 bench/m1/harness.py run --agent oracle --expect all-pass --out "$tmp" > /dev/null
python3 bench/m1/harness.py summarize "$tmp" | grep -q "작성 1회 성공률 | 10/10 |" || { echo "harness summarize: unexpected table"; exit 1; }
rm -f "$tmp"
python3 bench/m1/harness.py run --agent weak --expect weak-fail > /dev/null
python3 bench/m1/test_anthropic_agent.py 2> /dev/null
out=$(python3 bench/m1/test_harness.py 2>&1) || { printf '%s\n' "$out"; exit 1; }
echo "all checks passed"
