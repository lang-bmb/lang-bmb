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

step "test, bench m1, bench m1b, bench bvb, bench m3 (concurrent)"
# Bench sessions are independent (ADR-0025, backlog 2A-04): m1b and bvb run two at a time per agent.
# The original-Verus side of bvb (2A-09) needs the Verus toolchain: here only its harness logic
# runs (stub verifier); the real oracle/weak runs are the `bvb-verus` CI job.
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
  --job 'bench bvb oracle=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/bvb run --agent oracle --expect all-pass --jobs 2 --out "$tmp" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$tmp")
grep -qF "| 작성 1회 성공률 | 45/45 |" <<< "$table" || { echo "bvb summarize: unexpected table"; exit 1; }' \
  --job "bench bvb weak=python3 bench/m1/harness.py --set bench/bvb run --agent weak --expect weak-fail --jobs 2 > /dev/null" \
  --job "bench bvb tests=python3 bench/bvb/port.py check && python3 bench/bvb/test_bvb.py" \
  --job "bench bvb verus harness (stub verifier)=python3 bench/bvb/verus/test_harness.py" \
  --job "bench m3 generator=python3 bench/m3/test_gen.py > /dev/null"
echo "all checks passed"
