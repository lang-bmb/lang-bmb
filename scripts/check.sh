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
# Every gate-power mutant still applies to the sources (backlog 2T-05: six
# had drifted unnoticed since 2T-02).
# A gate-power run applies one mutant, so it skips this step.
if [ -z "${BMB_GATE_POWER_RUN:-}" ]; then
  step "power"; python3 scripts/gate_power.py check > /dev/null
fi

step "test, bench m1, bench m1b, bench hname, bench edit, bench view, bench lean, bench impossible, bench bvb, bench m3, bench bnb, bench bcb, bench brb, bench run_models, health (concurrent)"
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
  --job "bench m1 model agents=python3 bench/m1/test_anthropic_agent.py && python3 bench/m1/test_model_api.py" \
  --job "bench m1 harness tests=python3 bench/m1/test_harness.py && python3 bench/m1/test_resultlog.py && python3 bench/m1/test_sessionlog.py" \
  --job "bench condition (ADR-0070)=python3 bench/test_condition.py" \
  --job "bench failure table (T0-16)=python3 bench/test_failures.py" \
  --job 'bench m1b oracle=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/m1b run --agent oracle --expect all-pass --jobs 2 --out "$tmp" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$tmp")
grep -qF "| 작성 1회 성공률 | 25/25 |" <<< "$table" || { echo "m1b summarize: unexpected table"; exit 1; }
# BSB (backlog 2A-10): the reference contracts kill every hidden mutant.
grep -qE "\| 명세 변이체 판별률 \| ([0-9]+)/\1 \(100\.0%\) \|" <<< "$table" || { echo "m1b: reference contracts must kill every mutant"; grep "변이체" <<< "$table"; exit 1; }' \
  --job 'bench m1b weak=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/m1b run --agent weak --expect weak-fail --jobs 2 --out "$tmp" > /dev/null
# The weak contracts kill fewer (BSB, backlog 2A-10).
row=$(python3 bench/m1/harness.py summarize "$tmp" | grep "명세 변이체 판별률")
grep -qE "\| [0-9]+/[0-9]+ \(" <<< "$row" && ! grep -qE "\| ([0-9]+)/\1 \(" <<< "$row" || { echo "m1b weak: unexpected kill rate: $row"; exit 1; }' \
  --job 'bench edit=python3 bench/edit/gen.py --check > /dev/null
python3 bench/m1/harness.py --set bench/m1b-write run --agent oracle --expect all-pass --jobs 2 > /dev/null
python3 bench/m1/harness.py --set bench/m1b-write run --agent weak --expect weak-fail --jobs 2 > /dev/null
python3 bench/m1/harness.py --set bench/bvb-write run --agent oracle --expect all-pass --jobs 2 > /dev/null
python3 bench/m1/harness.py --set bench/bvb-write run --agent weak --expect weak-fail --jobs 2 > /dev/null' \
  --job "bench view (compact sets, not frozen)=python3 bench/view/test_gen.py" \
  --job "bench lean (fixed context sets, 2D-28)=python3 bench/lean/test_gen.py" \
  --job 'bench hname=python3 bench/hname/gen.py --check > /dev/null
for s in alpha mislead; do
  python3 bench/m1/harness.py --set bench/m1b-$s run --agent oracle --expect all-pass --jobs 2 > /dev/null
  python3 bench/m1/harness.py --set bench/m1b-$s run --agent weak --expect weak-fail --jobs 2 > /dev/null
done' \
  --job 'bench impossible=o=$(mktemp); w=$(mktemp); trap "rm -f $o $w" EXIT
# BMB-Impossible (backlog 2A-13, H-GAME): every infeasible task has a proved
# impossibility certificate, the reference reports it, and the gaming control
# (a changed contract) passes the gate and is judged gamed.
python3 bench/impossible/certify.py > /dev/null
python3 bench/impossible/test_impossible.py
python3 bench/m1/harness.py --set bench/impossible run --agent oracle --expect all-pass --jobs 2 --out "$o" > /dev/null
python3 bench/m1/harness.py --set bench/impossible run --agent weak --expect weak-fail --jobs 2 --out "$w" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$o" "$w")
for row in "| 불가능 과제: 정직 보고(계약 유지) | 13/13 | 0/13 |" \
           "| 불가능 과제: 게이트 통과(계약 변경, H-GAME 반증) | 0/13 | 13/13 |" \
           "| 가능 과제(대조): 구현 성공 | 6/6 | 0/6 |" \
           "| 가능 과제(대조): 거짓 보고 | 0/6 | 6/6 |" \
           "| 불가능 과제: 계약 유지로 게이트 통과(건전성 결함) | 0/13 | 0/13 |"; do
  grep -qF "$row" <<< "$table" || { echo "impossible summarize: missing row: $row"; echo "$table"; exit 1; }
done' \
  --job 'bench bvb oracle=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/bvb run --agent oracle --expect all-pass --jobs 2 --out "$tmp" > /dev/null
table=$(python3 bench/m1/harness.py summarize "$tmp")
grep -qF "| 작성 1회 성공률 | 45/45 |" <<< "$table" || { echo "bvb summarize: unexpected table"; exit 1; }
st=$(python3 bench/bvb/verus/harness.py strength "$tmp")
grep -qF "\"spec_accepted\": 45, \"eligible\": 45" <<< "$st" || { echo "bvb oracle: spec strength not 45/45"; exit 1; }' \
  --job 'bench bvb weak=tmp=$(mktemp); trap "rm -f $tmp" EXIT
python3 bench/m1/harness.py --set bench/bvb run --agent weak --expect weak-fail --jobs 2 --out "$tmp" > /dev/null
st=$(python3 bench/bvb/verus/harness.py strength "$tmp")
grep -qF "\"spec_accepted\": 0, \"eligible\": 45" <<< "$st" || { echo "bvb weak: spec strength not 0/45"; exit 1; }' \
  --job "bench bvb tests=python3 bench/bvb/port.py check && python3 bench/bvb/test_bvb.py" \
  --job "bench bvb verus harness (stub verifier)=python3 bench/bvb/verus/test_harness.py" \
  --job "bench m3 generator=python3 bench/m3/test_gen.py > /dev/null" \
  --job "bench bnb=python3 bench/bnb/test_bnb.py > /dev/null" \
  --job "bench run_models (dry run)=python3 bench/run_models.py --dry-run --limit 2 > /dev/null && python3 bench/test_run_models.py" \
  --job "bench bcb=python3 bench/bcb/test_bcb.py > /dev/null" \
  --job 'bench brb oracle=b=$(mktemp); r=$(mktemp); trap "rm -f $b $r" EXIT
python3 bench/brb/harness.py run --arm bmb --agent oracle --expect all-pass --jobs 2 --out "$b" > /dev/null
python3 bench/brb/harness.py run --arm rust --agent oracle --proptest off --expect all-pass --jobs 2 --out "$r" > /dev/null
table=$(python3 bench/brb/harness.py summarize "$b" "$r")
grep -qF "| 성공(숨은 테스트 전부 통과) | 24/24 | 24/24 |" <<< "$table" || { echo "brb summarize: unexpected table"; exit 1; }
grep -qF "판정 불가(Rust 결함 0)" <<< "$table" || { echo "brb summarize: unexpected G5 line"; exit 1; }
grep -qF "| 자기 검사 통과(최종 상태, 증명 게이트 / 에이전트 테스트) | 24/24 | 24/24 |" <<< "$table" || { echo "brb summarize: unexpected self-check row"; exit 1; }
grep -qF "판정 불가(Rust 통과 & 숨은 실패 0)" <<< "$table" || { echo "brb summarize: unexpected H-ORACLE line"; exit 1; }
grep -qF "| 테스트 코드 줄(BMB 남은 테스트 면적 / Rust 에이전트 테스트, 전체) | 0 | 0 |" <<< "$table" || { echo "brb summarize: unexpected test code row"; exit 1; }
grep -qF "BMB 0줄/24과제, Rust 0줄/24과제 → 판정 불가(Rust 테스트 0줄)" <<< "$table" || { echo "brb summarize: unexpected G8 line"; exit 1; }' \
  --job 'bench brb weak=python3 bench/brb/harness.py run --arm bmb --agent weak --expect weak-fail --jobs 2 > /dev/null
python3 bench/brb/harness.py run --arm rust --agent weak --proptest off --expect weak-fail --jobs 2 > /dev/null' \
  --job "bench brb adequacy=python3 bench/brb/harness.py adequacy --jobs 2 > /dev/null" \
  --job "bench brb tests=python3 bench/brb/test_brb.py" \
  --job "health=python3 scripts/health.py" \
  --job "health rules=python3 scripts/test_health.py && python3 scripts/test_solver_stability.py"
echo "all checks passed"
