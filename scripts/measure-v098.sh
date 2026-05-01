#!/bin/bash
# Measure 15 historic benches: BMB v0.98 vs C (-O3) vs Rust (-O)
# Per-bench 60s hard timeout, 1 warmup + adaptive runs (P-5), take min.
#
# Noise gate (P-5, Cycle 2527):
#   Benches with warmup < NOISE_THRESHOLD ms get bumped to NOISE_MIN_RUNS.
#   Override via env: NOISE_THRESHOLD=100 NOISE_MIN_RUNS=10 NOISE_GATE=1.

set +e
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"
[ -f "$BMB" ] || BMB="${PROJECT_ROOT}/target/release/bmb.exe"
BENCHDIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb/benches"
OUT="${PROJECT_ROOT}/target/benchmarks/v098-historic.json"
mkdir -p "$(dirname "$OUT")"

export BMB_RUNTIME_PATH="${PROJECT_ROOT}/bmb/runtime"

NOISE_GATE="${NOISE_GATE:-1}"
NOISE_THRESHOLD="${NOISE_THRESHOLD:-100}"
NOISE_MIN_RUNS="${NOISE_MIN_RUNS:-10}"
BASE_RUNS="${BASE_RUNS:-3}"

# Tier 1 (compute) historic 15
T1=(mandelbrot n_body fannkuch fibonacci hash_table binary_trees fasta spectral_norm string_hash)
# Tier 3 (real_world) historic
T3=(brainfuck csv_parse http_parse json_parse json_serialize lexer sorting)

now_ms() { date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))'; }

time_run() {
  local exe=$1 to=${2:-60}
  local s=$(now_ms)
  timeout "$to" "$exe" > /dev/null 2>&1
  local rc=$?
  local e=$(now_ms)
  if [ $rc -eq 124 ]; then echo "TIMEOUT"; else echo $((e - s)); fi
}

best_of() {
  local exe=$1
  local best=""
  # warmup — also probes noise gate
  local warmup=$(time_run "$exe" 60)
  local runs=$BASE_RUNS
  if [ "$NOISE_GATE" = "1" ] && [ "$warmup" != "TIMEOUT" ] && [ "$warmup" -lt "$NOISE_THRESHOLD" ] && [ "$runs" -lt "$NOISE_MIN_RUNS" ]; then
    echo "    [noise-gate] warmup=${warmup}ms < ${NOISE_THRESHOLD}ms → ${runs} → ${NOISE_MIN_RUNS} runs" >&2
    runs=$NOISE_MIN_RUNS
  fi
  for ((i=1; i<=runs; i++)); do
    local t=$(time_run "$exe" 60)
    [ "$t" = "TIMEOUT" ] && { echo "TIMEOUT"; return; }
    if [ -z "$best" ] || [ "$t" -lt "$best" ]; then best=$t; fi
  done
  echo "$best"
}

results="["
first=true

bench_one() {
  local tier=$1 name=$2 dir=$3
  local bmb_src="${dir}/bmb/main.bmb"
  local c_src="${dir}/c/main.c"
  local rust_src="${dir}/rust/main.rs"
  local bmb_exe="${PROJECT_ROOT}/target/benchmarks/${name}_${tier}_bmb"
  local c_exe="${PROJECT_ROOT}/target/benchmarks/${name}_${tier}_c"
  local rust_exe="${PROJECT_ROOT}/target/benchmarks/${name}_${tier}_rust"

  local bmb_t="null" c_t="null" rust_t="null"

  echo ">>> $name"
  if [ -f "$bmb_src" ]; then
    "$BMB" build "$bmb_src" -o "$bmb_exe" > /dev/null 2>&1
    [ -f "${bmb_exe}.exe" ] && bmb_exe="${bmb_exe}.exe"
    if [ -x "$bmb_exe" ] || [ -f "$bmb_exe" ]; then
      bmb_t=$(best_of "$bmb_exe")
    fi
  fi

  if [ -f "$c_src" ]; then
    clang -O3 -march=native -o "${c_exe}.exe" "$c_src" -lm > /dev/null 2>&1
    if [ -f "${c_exe}.exe" ]; then
      c_t=$(best_of "${c_exe}.exe")
    fi
  fi

  if [ -f "$rust_src" ]; then
    rustc -O -o "${rust_exe}.exe" "$rust_src" > /dev/null 2>&1
    if [ -f "${rust_exe}.exe" ]; then
      rust_t=$(best_of "${rust_exe}.exe")
    fi
  fi

  echo "    BMB=${bmb_t}ms  C=${c_t}ms  Rust=${rust_t}ms"

  # Append JSON
  $first || results="${results},"
  results="${results}{\"tier\":${tier},\"name\":\"${name}\",\"bmb\":\"${bmb_t}\",\"c\":\"${c_t}\",\"rust\":\"${rust_t}\"}"
  first=false
}

for n in "${T1[@]}"; do
  d="${BENCHDIR}/compute/${n}"
  [ -d "$d" ] && bench_one 1 "$n" "$d"
done

for n in "${T3[@]}"; do
  d="${BENCHDIR}/real_world/${n}"
  [ -d "$d" ] && bench_one 3 "$n" "$d"
done

results="${results}]"
echo "$results" > "$OUT"
echo ""
echo "=== JSON written to $OUT ==="
