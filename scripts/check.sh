#!/usr/bin/env bash
# Single verification gate (AUTONOMY.md §2 step 5, INF-05).
# Local runs and CI use this same script. Judge it by its exit code.
#
# The quick sequential steps stop at the first failure. After the one build,
# the test executables and the bench checks are independent and run
# concurrently (backlog INF-08, scripts/gate_jobs.py): each runs to the end,
# a failing job's output is printed with its name, and any failure makes the
# exit code non-zero.
set -euo pipefail
cd "$(dirname "$0")/.."

step() { printf '== %s\n' "$1"; }

step "format";  cargo fmt --all --check
step "lint";    cargo clippy --all-targets --quiet -- -D warnings
step "build"   # every test executable and target/debug/bmb, which the benches use
built=$(mktemp)
trap 'rm -f "$built"' EXIT
cargo test --no-run --quiet --message-format=json-render-diagnostics > "$built"
step "backlog"; python3 scripts/check_backlog.py
step "docs";    target/debug/bmb codes --md | diff -u docs/diagnostics.md - > /dev/null || { echo "docs/diagnostics.md is stale: run cargo run -q -- codes --md > docs/diagnostics.md"; exit 1; }

step "test, bench m1, bench m1b, bench m3, bench bnb, bench brb (concurrent)"
# Bench sessions are independent (ADR-0025, backlog 2A-04): m1b runs two at a time per agent.
python3 scripts/gate_jobs.py --cargo-json "$built" \
  --job "test doc=cargo test --doc --quiet" \
  --job 'bench m1 oracle=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py run --agent oracle --expect all-pass --out "$tmp" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$tmp")
grep -qF "| 작성 1회 성공률 | 10/10 |" <<< "$table" || { echo "harness summarize: unexpected table"; exit 1; }' \
  --job "bench m1 weak=python3 bench/m1/harness.py run --agent weak --expect weak-fail > /dev/null" \
  --job "bench m1 anthropic agent=python3 bench/m1/test_anthropic_agent.py" \
  --job "bench m1 harness tests=python3 bench/m1/test_harness.py" \
  --job 'bench m1b oracle=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/m1b run --agent oracle --expect all-pass --jobs 2 --out "$tmp" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$tmp")
grep -qF "| 작성 1회 성공률 | 25/25 |" <<< "$table" || { echo "m1b summarize: unexpected table"; exit 1; }' \
  --job "bench m1b weak=python3 bench/m1/harness.py --set bench/m1b run --agent weak --expect weak-fail --jobs 2 > /dev/null" \
  --job "bench m3 generator=python3 bench/m3/test_gen.py > /dev/null" \
  --job "bench bnb=python3 bench/bnb/test_bnb.py > /dev/null" \
  --job 'bench brb oracle=b=$(mktemp); r=$(mktemp); trap "rm -f $b $r" EXIT
python3 bench/brb/harness.py run --arm bmb --agent oracle --expect all-pass --jobs 2 --out "$b" > /dev/null
python3 bench/brb/harness.py run --arm rust --agent oracle --proptest off --expect all-pass --jobs 2 --out "$r" > /dev/null
table=$(python3 bench/brb/harness.py summarize "$b" "$r")
grep -qF "| 성공(숨은 테스트 전부 통과) | 24/24 | 24/24 |" <<< "$table" || { echo "brb summarize: unexpected table"; exit 1; }
grep -qF "판정 불가(Rust 결함 0)" <<< "$table" || { echo "brb summarize: unexpected G5 line"; exit 1; }' \
  --job 'bench brb weak=python3 bench/brb/harness.py run --arm bmb --agent weak --expect weak-fail --jobs 2 > /dev/null
python3 bench/brb/harness.py run --arm rust --agent weak --proptest off --expect weak-fail --jobs 2 > /dev/null' \
  --job "bench brb adequacy=python3 bench/brb/harness.py adequacy --jobs 2 > /dev/null" \
  --job "bench brb tests=python3 bench/brb/test_brb.py"
echo "all checks passed"
