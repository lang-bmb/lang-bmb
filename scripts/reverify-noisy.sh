#!/bin/bash
# Re-verify lexer + http_parse with 10 runs each (best-of) for noise reduction.
set +e
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"

now_ms() { date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))'; }

best_of_10() {
  local exe=$1 best=""
  # 2 warmups
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

for n in lexer http_parse; do
  bmb_exe="${PROJECT_ROOT}/target/benchmarks/${n}_3_bmb.exe"
  c_exe="${PROJECT_ROOT}/target/benchmarks/${n}_3_c.exe"
  [ -f "$bmb_exe" ] || { echo "$n: bmb_exe missing"; continue; }
  [ -f "$c_exe" ] || { echo "$n: c_exe missing"; continue; }
  bmb=$(best_of_10 "$bmb_exe")
  c=$(best_of_10 "$c_exe")
  ratio=$(python3 -c "print(f'{$bmb / $c:.3f}')")
  echo "$n: BMB=${bmb}ms C=${c}ms ratio=${ratio}"
done
