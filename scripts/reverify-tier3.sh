#!/bin/bash
# Re-verify all Tier 3 benches with 10 runs (small benches need noise reduction).
set +e
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"
BENCHDIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb/benches"
OUT="${PROJECT_ROOT}/target/benchmarks/v098-tier3-10runs.json"

export BMB_RUNTIME_PATH="${PROJECT_ROOT}/bmb/runtime"

now_ms() { date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))'; }

best_of_10() {
  local exe=$1 best=""
  "$exe" > /dev/null 2>&1
  "$exe" > /dev/null 2>&1
  for i in $(seq 1 10); do
    local s=$(now_ms)
    "$exe" > /dev/null 2>&1
    local e=$(now_ms)
    local t=$((e - s))
    if [ -z "$best" ] || [ "$t" -lt "$best" ]; then best=$t; fi
  done
  echo "$best"
}

T3=(brainfuck csv_parse http_parse json_parse json_serialize lexer sorting)

results="["
first=true
for n in "${T3[@]}"; do
  bmb_exe="${PROJECT_ROOT}/target/benchmarks/${n}_3_bmb.exe"
  c_exe="${PROJECT_ROOT}/target/benchmarks/${n}_3_c.exe"
  rust_exe="${PROJECT_ROOT}/target/benchmarks/${n}_3_rust.exe"
  [ -f "$bmb_exe" ] || continue
  bmb=$(best_of_10 "$bmb_exe")
  c="null"
  rust="null"
  [ -f "$c_exe" ] && c=$(best_of_10 "$c_exe")
  [ -f "$rust_exe" ] && rust=$(best_of_10 "$rust_exe")
  if [ "$c" != "null" ] && [ "$c" -gt 0 ]; then
    ratio=$(python3 -c "print(f'{$bmb / $c:.3f}')")
  else
    ratio="null"
  fi
  echo ">>> $n: BMB=${bmb}ms C=${c}ms Rust=${rust}ms ratio_c=${ratio}"
  $first || results="${results},"
  results="${results}{\"name\":\"${n}\",\"bmb\":${bmb},\"c\":${c},\"rust\":${rust},\"ratio_c\":\"${ratio}\"}"
  first=false
done
results="${results}]"
echo "$results" > "$OUT"
echo "=== JSON: $OUT ==="
