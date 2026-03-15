#!/bin/bash
# FASTER Category Classifier — distinguishes BMB metadata advantages from pipeline artifacts
#
# Method: Build C through the same pipeline as BMB (opt -O3 → clang -fno-unroll-loops)
# If C-same-pipeline is also faster than C-standard, the speedup is a pipeline artifact.
# If BMB is still faster than C-same-pipeline, the speedup is a genuine metadata advantage.
#
# Usage:
#   ./scripts/classify_faster.sh [--filter NAME] [--runs N] [--json FILE]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BMB="${PROJECT_ROOT}/target/release/bmb"
RUNTIME_OBJ="${PROJECT_ROOT}/bmb/runtime/bmb_runtime.o"
RUNTIME_EVT="${PROJECT_ROOT}/bmb/runtime/bmb_event_loop.o"
BENCH_DIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb/benches/compute"
BUILD_DIR="/tmp/bmb-classify"

RUNS=11
WARMUP=3
FILTER=""
JSON_FILE=""

# Colors
if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BLUE=''; CYAN=''; BOLD=''; NC=''
fi

while [[ $# -gt 0 ]]; do
    case $1 in
        --runs)    RUNS="$2"; shift 2 ;;
        --warmup)  WARMUP="$2"; shift 2 ;;
        --filter)  FILTER="$2"; shift 2 ;;
        --json)    JSON_FILE="$2"; shift 2 ;;
        --help)    head -10 "$0" | tail -8; exit 0 ;;
        *)         echo "Unknown: $1"; exit 1 ;;
    esac
done

# Check tools
BMB_ACTUAL="$BMB"
[ -f "${BMB}.exe" ] && BMB_ACTUAL="${BMB}.exe"
[ ! -f "$BMB_ACTUAL" ] && { echo "BMB not found"; exit 1; }
command -v opt &>/dev/null || { echo "opt not found"; exit 1; }
command -v clang &>/dev/null || { echo "clang not found"; exit 1; }

mkdir -p "$BUILD_DIR"

get_ms() { date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'; }
time_run() { local s; s=$(get_ms); "$1" > /dev/null 2>&1 || true; echo $(( $(get_ms) - s )); }

median_of() {
    python3 -c "
import math
vals = sorted([$1])
n = len(vals)
if n >= 5:
    q1, q3 = vals[n//4], vals[3*n//4]
    iqr = q3 - q1
    vals = [v for v in vals if q1-1.5*iqr <= v <= q3+1.5*iqr] or vals
print(vals[len(vals)//2])
"
}

run_bench() {
    local exe=$1
    local -a times=()
    for ((i=1; i<=WARMUP; i++)); do time_run "$exe" > /dev/null; done
    for ((i=1; i<=RUNS; i++)); do times+=($(time_run "$exe")); done
    local csv; csv=$(IFS=,; echo "${times[*]}")
    median_of "$csv"
}

# Build C through BMB's pipeline: C → LLVM IR → opt -O3 → clang -fno-unroll-loops
build_c_bmb_pipeline() {
    local src=$1 out=$2 name=$3
    local ir="${BUILD_DIR}/${name}_c.ll"
    local ir_opt="${BUILD_DIR}/${name}_c_opt.ll"
    local link_flags="-lm"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* ]] && link_flags="$link_flags -lws2_32"

    # Step 1: C → LLVM IR
    if ! clang -O3 -march=native -S -emit-llvm -o "$ir" "$src" 2>/dev/null; then
        echo "FAIL:emit-ir"; return 1
    fi
    # Step 2: opt -O3 (same as BMB pipeline)
    if ! opt -O3 --mcpu=native "$ir" -S -o "$ir_opt" 2>/dev/null; then
        echo "FAIL:opt"; return 1
    fi
    # Step 3: clang -O3 -fno-unroll-loops (same as BMB pipeline)
    local lld_flag=""
    command -v ld.lld &>/dev/null && lld_flag="-fuse-ld=lld -Wl,--gc-sections"
    if ! clang -w -O3 -fno-unroll-loops -march=native -ffunction-sections $lld_flag "$ir_opt" -o "$out" $link_flags 2>/dev/null; then
        echo "FAIL:link"; return 1
    fi
    echo "OK"
}

# Build C standard
build_c_standard() {
    local src=$1 out=$2
    local link_flags="-lm"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* ]] && link_flags="$link_flags -lws2_32"
    if ! clang -O3 -march=native -o "$out" "$src" $link_flags 2>/dev/null; then
        echo "FAIL"; return 1
    fi
    echo "OK"
}

# Build BMB
build_bmb() {
    local src=$1 out=$2 name=$3
    local ir="${BUILD_DIR}/${name}.ll"
    local ir_opt="${BUILD_DIR}/${name}_opt.ll"
    local link_flags="-lm"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* ]] && link_flags="$link_flags -lws2_32"
    "$BMB_ACTUAL" build "$src" --emit-ir -o "$ir" > /dev/null 2>&1 || { echo "FAIL:emit-ir"; return 1; }
    opt -O3 --mcpu=native "$ir" -S -o "$ir_opt" 2>/dev/null || { echo "FAIL:opt"; return 1; }
    local lld_flag=""
    command -v ld.lld &>/dev/null && lld_flag="-fuse-ld=lld -Wl,--gc-sections"
    clang -w -O3 -fno-unroll-loops -march=native -ffunction-sections $lld_flag "$ir_opt" "$RUNTIME_OBJ" "$RUNTIME_EVT" -o "$out" $link_flags 2>/dev/null || { echo "FAIL:link"; return 1; }
    echo "OK"
}

echo -e "${BOLD}FASTER Category Classifier${NC}"
echo -e "Method: Compare BMB vs C-standard vs C-same-pipeline"
echo -e "Config: ${RUNS} runs + ${WARMUP} warmup"
echo ""

# Collect benchmarks
declare -a BENCHMARKS=()
for d in "${BENCH_DIR}"/*/; do
    name=$(basename "$d")
    [ -n "$FILTER" ] && [[ "$name" != *"$FILTER"* ]] && continue
    [ -f "$d/bmb/main.bmb" ] && [ -f "$d/c/main.c" ] && BENCHMARKS+=("$name")
done
echo -e "Benchmarks: ${#BENCHMARKS[@]}"
echo ""

# Build phase
echo -e "${BLUE}Building (3 variants per benchmark)...${NC}"
declare -a RUNNABLE=()
for name in "${BENCHMARKS[@]}"; do
    src_bmb="${BENCH_DIR}/${name}/bmb/main.bmb"
    src_c="${BENCH_DIR}/${name}/c/main.c"
    bmb_exe="${BUILD_DIR}/${name}_bmb"
    c_std="${BUILD_DIR}/${name}_c_std"
    c_pipe="${BUILD_DIR}/${name}_c_pipe"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* ]] && {
        bmb_exe="${bmb_exe}.exe"; c_std="${c_std}.exe"; c_pipe="${c_pipe}.exe"
    }

    printf "  %-25s " "$name"
    s1=$(build_bmb "$src_bmb" "$bmb_exe" "${name}_bmb")
    s2=$(build_c_standard "$src_c" "$c_std")
    s3=$(build_c_bmb_pipeline "$src_c" "$c_pipe" "${name}")
    if [ "$s1" = "OK" ] && [ "$s2" = "OK" ] && [ "$s3" = "OK" ]; then
        echo -e "${GREEN}OK${NC}"
        RUNNABLE+=("$name")
    else
        echo -e "${RED}BMB:${s1} C-std:${s2} C-pipe:${s3}${NC}"
    fi
done
echo ""

# Measure phase
echo -e "${BLUE}Measuring...${NC}"
echo ""
printf "${BOLD}%-20s %8s %8s %8s %7s %7s %12s${NC}\n" \
    "Benchmark" "BMB(ms)" "C-std" "C-pipe" "BMB/Cst" "BMB/Cpipe" "Category"
printf "%s\n" "$(printf '=%.0s' {1..90})"

META_COUNT=0
PIPE_COUNT=0
BOTH_COUNT=0
NONE_COUNT=0
declare -a JSON_ENTRIES=()

for name in "${RUNNABLE[@]}"; do
    bmb_exe="${BUILD_DIR}/${name}_bmb"
    c_std="${BUILD_DIR}/${name}_c_std"
    c_pipe="${BUILD_DIR}/${name}_c_pipe"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* ]] && {
        bmb_exe="${bmb_exe}.exe"; c_std="${c_std}.exe"; c_pipe="${c_pipe}.exe"
    }

    t_bmb=$(run_bench "$bmb_exe")
    t_cstd=$(run_bench "$c_std")
    t_cpipe=$(run_bench "$c_pipe")

    r_std=$(python3 -c "print(f'{${t_bmb}/${t_cstd}:.2f}')")
    r_pipe=$(python3 -c "print(f'{${t_bmb}/${t_cpipe}:.2f}')")

    # Classification logic
    is_faster_std=$(python3 -c "print('y' if ${t_bmb}/${t_cstd} < 0.95 else 'n')")
    is_faster_pipe=$(python3 -c "print('y' if ${t_bmb}/${t_cpipe} < 0.95 else 'n')")
    cpipe_faster_std=$(python3 -c "print('y' if ${t_cpipe}/${t_cstd} < 0.95 else 'n')")

    if [ "$is_faster_std" = "y" ] && [ "$is_faster_pipe" = "y" ]; then
        category="METADATA"
        color=$GREEN
        META_COUNT=$((META_COUNT+1))
    elif [ "$is_faster_std" = "y" ] && [ "$cpipe_faster_std" = "y" ]; then
        category="PIPELINE"
        color=$YELLOW
        PIPE_COUNT=$((PIPE_COUNT+1))
    elif [ "$is_faster_std" = "y" ]; then
        category="MIXED"
        color=$CYAN
        BOTH_COUNT=$((BOTH_COUNT+1))
    else
        category="PASS"
        color=$NC
        NONE_COUNT=$((NONE_COUNT+1))
    fi

    printf "%-20s %8s %8s %8s %7sx %9sx ${color}%12s${NC}\n" \
        "$name" "$t_bmb" "$t_cstd" "$t_cpipe" "$r_std" "$r_pipe" "$category"

    JSON_ENTRIES+=("{\"name\":\"${name}\",\"bmb_ms\":${t_bmb},\"c_std_ms\":${t_cstd},\"c_pipe_ms\":${t_cpipe},\"ratio_std\":${r_std},\"ratio_pipe\":${r_pipe},\"category\":\"${category}\"}")
done

echo ""
printf "%s\n" "$(printf '=%.0s' {1..90})"
echo -e "${BOLD}Summary:${NC}"
echo -e "  ${GREEN}METADATA${NC}:  ${META_COUNT}  (BMB faster than C even with same pipeline — genuine advantage)"
echo -e "  ${YELLOW}PIPELINE${NC}:  ${PIPE_COUNT}  (C-same-pipeline also faster — pipeline artifact)"
echo -e "  ${CYAN}MIXED${NC}:     ${BOTH_COUNT}  (Partial pipeline effect)"
echo -e "  PASS:     ${NONE_COUNT}  (Not FASTER in this run)"

# JSON output
if [ -n "$JSON_FILE" ]; then
    {
        echo "{"
        echo "  \"date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
        echo "  \"method\": \"3-way: BMB vs C-standard vs C-same-pipeline\","
        echo "  \"summary\": {\"metadata\": ${META_COUNT}, \"pipeline\": ${PIPE_COUNT}, \"mixed\": ${BOTH_COUNT}, \"pass\": ${NONE_COUNT}},"
        echo "  \"results\": ["
        first=true
        for e in "${JSON_ENTRIES[@]}"; do
            [ "$first" = true ] && first=false || echo ","
            echo -n "    $e"
        done
        echo ""
        echo "  ]"
        echo "}"
    } > "$JSON_FILE"
    echo ""
    echo "JSON written to: $JSON_FILE"
fi
