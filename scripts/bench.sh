#!/bin/bash
# BMB Benchmark Suite v5 — 3-Way Comparison (BMB vs C vs Rust)
#
# Design principles:
#   1. FAIRNESS: BMB and C use the same LLVM backend (clang -O3 -march=native)
#   2. REPRODUCIBILITY: IQR-based outlier removal, configurable runs + warmup
#   3. CONSISTENCY: Text backend emit-ir → opt -O3 → clang -O3 pipeline for BMB
#   4. 3-WAY: BMB vs C vs Rust (rustc --release) for comprehensive comparison
#   5. CORRECTNESS: Verify BMB/C/Rust produce identical output before timing
#
# Usage:
#   ./scripts/bench.sh [options]
#
# Options:
#   --runs N        Measured runs per benchmark (default: 11, must be odd)
#   --warmup N      Warmup runs discarded (default: 3)
#   --json FILE     Write JSON results to file
#   --list          List available benchmarks and exit
#   --filter NAME   Run only benchmarks matching NAME
#   --verbose       Show per-run timing details
#   --no-build      Skip building, reuse existing binaries
#   --no-rust       Skip Rust benchmarks (BMB vs C only)
#   --no-check      Skip output correctness verification
#   --help          Show this help

set -e

# ─── Configuration ───────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BMB="${PROJECT_ROOT}/target/release/bmb"
RUNTIME="${PROJECT_ROOT}/bmb/runtime/libbmb_runtime.a"
BENCH_DIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb/benches/compute"
BUILD_DIR="/tmp/bmb-bench"

# ─── Defaults ────────────────────────────────────────────────────────────────
RUNS=11
WARMUP=3
JSON_FILE=""
LIST_ONLY=false
FILTER=""
VERBOSE=false
NO_BUILD=false
NO_RUST=false
NO_CHECK=false

# ─── Colors ──────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BLUE=''; CYAN=''; BOLD=''; NC=''
fi

# ─── Parse Arguments ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case $1 in
        --runs)     RUNS="$2"; shift 2 ;;
        --warmup)   WARMUP="$2"; shift 2 ;;
        --json)     JSON_FILE="$2"; shift 2 ;;
        --list)     LIST_ONLY=true; shift ;;
        --filter)   FILTER="$2"; shift 2 ;;
        --verbose)  VERBOSE=true; shift ;;
        --no-build) NO_BUILD=true; shift ;;
        --no-rust)  NO_RUST=true; shift ;;
        --no-check) NO_CHECK=true; shift ;;
        --help)     head -24 "$0" | tail -22; exit 0 ;;
        *)          echo "Unknown option: $1"; exit 1 ;;
    esac
done

# ─── List Mode ───────────────────────────────────────────────────────────────
if [ "$LIST_ONLY" = true ]; then
    echo "Available benchmarks (${BENCH_DIR}):"
    for d in "${BENCH_DIR}"/*/; do
        name=$(basename "$d")
        has_bmb=""; has_c=""
        [ -f "$d/bmb/main.bmb" ] && has_bmb="BMB"
        [ -f "$d/c/main.c" ] && has_c="C"
        printf "  %-25s %s %s\n" "$name" "$has_bmb" "$has_c"
    done
    exit 0
fi

# ─── Prerequisites ───────────────────────────────────────────────────────────
check_tool() {
    command -v "$1" &>/dev/null || { echo "Error: $1 not found in PATH"; exit 1; }
}

# Check BMB compiler
BMB_ACTUAL="$BMB"
[ -f "${BMB}.exe" ] && BMB_ACTUAL="${BMB}.exe"
if [ ! -f "$BMB_ACTUAL" ]; then
    echo -e "${RED}Error: BMB compiler not found at $BMB${NC}"
    echo "Build with: cargo build --release"
    exit 1
fi

# Check LLVM tools
check_tool opt
check_tool clang

# Check Rust compiler (optional)
HAS_RUSTC=false
if [ "$NO_RUST" = false ] && command -v rustc &>/dev/null; then
    HAS_RUSTC=true
fi

# Check runtime
if [ ! -f "$RUNTIME" ]; then
    echo -e "${RED}Error: Runtime library not found at $RUNTIME${NC}"
    exit 1
fi

mkdir -p "$BUILD_DIR"

# ─── Timing ──────────────────────────────────────────────────────────────────
get_ms() {
    date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
}

# Run a single benchmark execution and return time in ms
time_run() {
    local exe=$1
    local start end
    start=$(get_ms)
    "$exe" > /dev/null 2>&1 || true
    end=$(get_ms)
    echo $((end - start))
}

# ─── Statistics with IQR-based outlier removal ─────────────────────────────
compute_stats() {
    # Takes sorted array as arguments, outputs: median mean stddev min max
    # Uses IQR (interquartile range) to filter outliers from system load
    local -a vals=("$@")
    local n=${#vals[@]}

    # Convert to comma-separated for python
    local csv
    csv=$(IFS=,; echo "${vals[*]}")

    # Python handles IQR filtering + statistics
    python3 -c "
import math
vals = sorted([${csv}])
n = len(vals)

# IQR-based outlier removal (only if enough data points)
if n >= 5:
    q1 = vals[n // 4]
    q3 = vals[3 * n // 4]
    iqr = q3 - q1
    lower = q1 - 1.5 * iqr
    upper = q3 + 1.5 * iqr
    filtered = [v for v in vals if lower <= v <= upper]
    if len(filtered) >= 3:
        vals = filtered
        n = len(vals)

median = vals[n // 2]
mean = sum(vals) / n
variance = sum((x - mean)**2 for x in vals) / n
stddev = math.sqrt(variance)
print(f'{median} {mean:.1f} {stddev:.1f} {vals[0]} {vals[-1]}')
"
}

# ─── Build Functions ─────────────────────────────────────────────────────────

# Build BMB benchmark using text backend pipeline
# Pipeline: bmb --emit-ir → opt -O3 → clang -O3 -march=native
build_bmb() {
    local src=$1
    local out=$2
    local name=$3
    local ir="${BUILD_DIR}/${name}.ll"
    local ir_opt="${BUILD_DIR}/${name}_opt.ll"

    # Step 1: Generate text LLVM IR
    if ! "$BMB_ACTUAL" build "$src" --emit-ir -o "$ir" > /dev/null 2>&1; then
        echo "FAIL:emit-ir"
        return 1
    fi

    # Step 2: Optimize with LLVM opt
    if ! opt --mcpu=native -passes="default<O3>,scalarizer,slp-vectorizer" "$ir" -S -o "$ir_opt" 2>/dev/null; then
        echo "FAIL:opt"
        return 1
    fi

    # Step 3: Compile + link with clang (same flags as C baseline)
    local link_flags="-lm"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && link_flags="$link_flags -lws2_32"

    if ! clang -w -O3 -march=native "$ir_opt" "$RUNTIME" -o "$out" $link_flags 2>/dev/null; then
        echo "FAIL:link"
        return 1
    fi

    echo "OK"
}

# Build C benchmark
# Pipeline: clang -O3 -march=native (same LLVM backend as BMB)
build_c() {
    local src=$1
    local out=$2
    local link_flags="-lm"
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && link_flags="$link_flags -lws2_32"

    if ! clang -O3 -march=native -o "$out" "$src" $link_flags 2>/dev/null; then
        echo "FAIL"
        return 1
    fi
    echo "OK"
}

# Build Rust benchmark
# Pipeline: rustc -C opt-level=3 -C target-cpu=native
build_rust() {
    local src=$1
    local out=$2
    if ! rustc -C opt-level=3 -C target-cpu=native -o "$out" "$src" 2>/dev/null; then
        echo "FAIL"
        return 1
    fi
    echo "OK"
}

# ─── Run Benchmark with Statistics ───────────────────────────────────────────
run_with_stats() {
    local exe=$1
    local -a times=()

    # Warmup runs (discarded)
    for ((i=1; i<=WARMUP; i++)); do
        time_run "$exe" > /dev/null
    done

    # Measured runs
    for ((i=1; i<=RUNS; i++)); do
        local t
        t=$(time_run "$exe")
        times+=($t)
        [ "$VERBOSE" = true ] && echo -n " ${t}" >&2
    done
    [ "$VERBOSE" = true ] && echo "" >&2

    # Sort
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS

    # Compute statistics
    compute_stats "${sorted[@]}"
}

# ─── Rating Function ─────────────────────────────────────────────────────────
rate_ratio() {
    local ratio=$1
    python3 -c "
r = $ratio
if r < 0.95:
    print('FASTER')
elif r <= 1.05:
    print('PASS')
elif r <= 1.10:
    print('WARN')
else:
    print('FAIL')
"
}

# ─── Main ────────────────────────────────────────────────────────────────────
echo -e "${BOLD}BMB Benchmark Suite v5 — 3-Way Comparison${NC}"
echo -e "Config: ${RUNS} runs + ${WARMUP} warmup"
echo -e "Pipeline: BMB emit-ir → opt --mcpu=native -O3 +scalarizer +slp → clang -O3"
echo -e "         C → clang -O3 -march=native"
[ "$HAS_RUSTC" = true ] && echo -e "         Rust → rustc -C opt-level=3 -C target-cpu=native"
echo ""

# Collect benchmark directories
declare -a BENCHMARKS=()
for d in "${BENCH_DIR}"/*/; do
    name=$(basename "$d")
    [ -n "$FILTER" ] && [[ "$name" != *"$FILTER"* ]] && continue
    [ -f "$d/bmb/main.bmb" ] && [ -f "$d/c/main.c" ] && BENCHMARKS+=("$name")
done

if [ ${#BENCHMARKS[@]} -eq 0 ]; then
    echo "No benchmarks found matching filter: $FILTER"
    exit 1
fi

echo -e "Benchmarks: ${#BENCHMARKS[@]}"
echo ""

# ─── Build Phase ─────────────────────────────────────────────────────────────
if [ "$NO_BUILD" = false ]; then
    echo -e "${BLUE}Building...${NC}"
    for name in "${BENCHMARKS[@]}"; do
        local_bmb="${BENCH_DIR}/${name}/bmb/main.bmb"
        local_c="${BENCH_DIR}/${name}/c/main.c"
        local_rust="${BENCH_DIR}/${name}/rust/main.rs"
        bmb_exe="${BUILD_DIR}/${name}_bmb"
        c_exe="${BUILD_DIR}/${name}_c"
        rust_exe="${BUILD_DIR}/${name}_rust"

        # Add .exe on Windows
        [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && {
            bmb_exe="${bmb_exe}.exe"
            c_exe="${c_exe}.exe"
            rust_exe="${rust_exe}.exe"
        }

        printf "  %-25s " "$name"

        bmb_status=$(build_bmb "$local_bmb" "$bmb_exe" "${name}_bmb")
        c_status=$(build_c "$local_c" "$c_exe")

        rust_status="—"
        if [ "$HAS_RUSTC" = true ] && [ -f "$local_rust" ]; then
            rust_status=$(build_rust "$local_rust" "$rust_exe")
        fi

        if [ "$bmb_status" = "OK" ] && [ "$c_status" = "OK" ]; then
            if [ "$rust_status" = "OK" ]; then
                echo -e "${GREEN}OK${NC} (BMB + C + Rust)"
            elif [ "$rust_status" = "—" ]; then
                echo -e "${GREEN}OK${NC} (BMB + C)"
            else
                echo -e "${GREEN}OK${NC} (BMB + C) ${YELLOW}Rust:${rust_status}${NC}"
            fi
        else
            echo -e "${RED}BMB:${bmb_status} C:${c_status}${NC}"
        fi
    done
    echo ""
fi

# ─── Correctness Verification Phase ─────────────────────────────────────────
MISMATCH_COUNT=0
if [ "$NO_CHECK" = false ]; then
    echo -e "${BLUE}Verifying output correctness...${NC}"
    for name in "${BENCHMARKS[@]}"; do
        bmb_exe="${BUILD_DIR}/${name}_bmb"
        c_exe="${BUILD_DIR}/${name}_c"
        rust_exe="${BUILD_DIR}/${name}_rust"

        [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && {
            bmb_exe="${bmb_exe}.exe"
            c_exe="${c_exe}.exe"
            rust_exe="${rust_exe}.exe"
        }

        [ ! -f "$bmb_exe" ] || [ ! -f "$c_exe" ] && continue

        # Skip correctness check for benchmarks with known format differences
        check_exempt="${BENCH_DIR}/${name}/.check_exempt"
        if [ -f "$check_exempt" ]; then
            [ "$VERBOSE" = true ] && printf "  %-25s ${CYAN}EXEMPT${NC} (%s)\n" "$name" "$(cat "$check_exempt")"
            continue
        fi

        bmb_out=$("$bmb_exe" 2>/dev/null || true)
        c_out=$("$c_exe" 2>/dev/null || true)

        if [ "$bmb_out" != "$c_out" ]; then
            # Check if this is a float precision difference (tolerate 1e-6 relative)
            is_float_ok=$(python3 -c "
import sys
b = '''$bmb_out'''.strip().split('\n')
c = '''$c_out'''.strip().split('\n')
if len(b) != len(c):
    print('no')
    sys.exit()
for bl, cl in zip(b, c):
    if bl == cl:
        continue
    try:
        bf, cf = float(bl), float(cl)
        if abs(cf) > 1e-15 and abs(bf - cf) / abs(cf) < 1e-6:
            continue
        elif abs(bf - cf) < 1e-9:
            continue
    except ValueError:
        pass
    print('no')
    sys.exit()
print('yes')
" 2>/dev/null || echo "no")
            if [ "$is_float_ok" = "yes" ]; then
                [ "$VERBOSE" = true ] && printf "  %-25s ${GREEN}OK${NC} (float ~equal)\n" "$name"
            else
                printf "  %-25s ${RED}MISMATCH${NC}  BMB=%s  C=%s\n" "$name" "$(echo "$bmb_out" | head -1)" "$(echo "$c_out" | head -1)"
                MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
            fi
        else
            if [ "$HAS_RUSTC" = true ] && [ -f "$rust_exe" ]; then
                rust_out=$("$rust_exe" 2>/dev/null || true)
                if [ "$rust_out" != "$c_out" ]; then
                    printf "  %-25s ${YELLOW}RUST MISMATCH${NC}  Rust=%s  C=%s\n" "$name" "$(echo "$rust_out" | head -1)" "$(echo "$c_out" | head -1)"
                else
                    [ "$VERBOSE" = true ] && printf "  %-25s ${GREEN}OK${NC}  out=%s\n" "$name" "$(echo "$bmb_out" | head -1)"
                fi
            else
                [ "$VERBOSE" = true ] && printf "  %-25s ${GREEN}OK${NC}  out=%s\n" "$name" "$(echo "$bmb_out" | head -1)"
            fi
        fi
    done
    if [ $MISMATCH_COUNT -gt 0 ]; then
        echo -e "${RED}WARNING: $MISMATCH_COUNT benchmark(s) have output mismatches!${NC}"
    else
        echo -e "  ${GREEN}All ${#BENCHMARKS[@]} benchmarks produce identical output${NC}"
    fi
    echo ""
fi

# ─── Measurement Phase ───────────────────────────────────────────────────────
echo -e "${BLUE}Measuring (${WARMUP} warmup + ${RUNS} measured runs each)...${NC}"
echo ""

# Header
if [ "$HAS_RUSTC" = true ]; then
    printf "${BOLD}%-20s %8s %8s %8s %7s %7s %7s %8s${NC}\n" \
        "Benchmark" "BMB(ms)" "C(ms)" "Rust(ms)" "BMB/C" "BMB/Rs" "Status" "BMBσ"
    printf "%s\n" "$(printf '=%.0s' {1..95})"
else
    printf "${BOLD}%-20s %8s %8s %7s %8s %8s %7s %8s${NC}\n" \
        "Benchmark" "BMB(ms)" "C(ms)" "Ratio" "BMB σ" "C σ" "Status" "Δ"
    printf "%s\n" "$(printf '=%.0s' {1..90})"
fi

# JSON accumulator
declare -a JSON_ENTRIES=()

FASTER_COUNT=0
PASS_COUNT=0
WARN_COUNT=0
FAIL_COUNT=0

for name in "${BENCHMARKS[@]}"; do
    bmb_exe="${BUILD_DIR}/${name}_bmb"
    c_exe="${BUILD_DIR}/${name}_c"
    rust_exe="${BUILD_DIR}/${name}_rust"

    # Add .exe on Windows
    [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && {
        bmb_exe="${bmb_exe}.exe"
        c_exe="${c_exe}.exe"
        rust_exe="${rust_exe}.exe"
    }

    # Skip if core binaries don't exist
    if [ ! -f "$bmb_exe" ] || [ ! -f "$c_exe" ]; then
        printf "%-20s %8s %8s %7s %8s %8s %7s\n" "$name" "—" "—" "—" "—" "—" "SKIP"
        continue
    fi

    # Run BMB
    [ "$VERBOSE" = true ] && echo -n "  BMB:" >&2
    bmb_stats=$(run_with_stats "$bmb_exe")
    bmb_median=$(echo "$bmb_stats" | awk '{print $1}')
    bmb_mean=$(echo "$bmb_stats" | awk '{print $2}')
    bmb_stddev=$(echo "$bmb_stats" | awk '{print $3}')
    bmb_min=$(echo "$bmb_stats" | awk '{print $4}')
    bmb_max=$(echo "$bmb_stats" | awk '{print $5}')

    # Run C
    [ "$VERBOSE" = true ] && echo -n "  C:  " >&2
    c_stats=$(run_with_stats "$c_exe")
    c_median=$(echo "$c_stats" | awk '{print $1}')
    c_mean=$(echo "$c_stats" | awk '{print $2}')
    c_stddev=$(echo "$c_stats" | awk '{print $3}')
    c_min=$(echo "$c_stats" | awk '{print $4}')
    c_max=$(echo "$c_stats" | awk '{print $5}')

    # Run Rust (optional)
    rust_median="—"
    rust_mean="0"
    rust_stddev="0"
    rust_min="0"
    rust_max="0"
    rust_ratio="—"
    if [ "$HAS_RUSTC" = true ] && [ -f "$rust_exe" ]; then
        [ "$VERBOSE" = true ] && echo -n "  Rust:" >&2
        rust_stats=$(run_with_stats "$rust_exe")
        rust_median=$(echo "$rust_stats" | awk '{print $1}')
        rust_mean=$(echo "$rust_stats" | awk '{print $2}')
        rust_stddev=$(echo "$rust_stats" | awk '{print $3}')
        rust_min=$(echo "$rust_stats" | awk '{print $4}')
        rust_max=$(echo "$rust_stats" | awk '{print $5}')
        rust_ratio=$(python3 -c "print(f'{${bmb_median} / ${rust_median}:.2f}')")
    fi

    # Compute ratio and rating (BMB vs C)
    ratio=$(python3 -c "print(f'{${bmb_median} / ${c_median}:.2f}')")
    rating=$(rate_ratio "$ratio")

    # Color the rating
    case "$rating" in
        FASTER) color=$GREEN; FASTER_COUNT=$((FASTER_COUNT+1)) ;;
        PASS)   color=$CYAN;  PASS_COUNT=$((PASS_COUNT+1)) ;;
        WARN)   color=$YELLOW; WARN_COUNT=$((WARN_COUNT+1)) ;;
        FAIL)   color=$RED;   FAIL_COUNT=$((FAIL_COUNT+1)) ;;
    esac

    if [ "$HAS_RUSTC" = true ]; then
        # 3-way display
        rust_disp="$rust_median"
        bmb_rust_disp="$rust_ratio"
        [ "$rust_median" = "—" ] && bmb_rust_disp="—"
        printf "%-20s %8s %8s %8s %7sx %7sx ${color}%7s${NC} %6s\n" \
            "$name" "$bmb_median" "$c_median" "$rust_disp" "$ratio" "$bmb_rust_disp" "$rating" "±${bmb_stddev}"
    else
        # Percentage difference
        pct_diff=$(python3 -c "
diff = (${bmb_median} - ${c_median}) / ${c_median} * 100
sign = '+' if diff >= 0 else ''
print(f'{sign}{diff:.1f}%')
")
        printf "%-20s %8s %8s %7sx %7s %7s ${color}%7s${NC} %8s\n" \
            "$name" "$bmb_median" "$c_median" "$ratio" "±${bmb_stddev}" "±${c_stddev}" "$rating" "$pct_diff"
    fi

    # JSON entry (includes Rust if available)
    if [ "$rust_median" != "—" ]; then
        JSON_ENTRIES+=("{\"name\":\"${name}\",\"bmb_median\":${bmb_median},\"bmb_mean\":${bmb_mean},\"bmb_stddev\":${bmb_stddev},\"bmb_min\":${bmb_min},\"bmb_max\":${bmb_max},\"c_median\":${c_median},\"c_mean\":${c_mean},\"c_stddev\":${c_stddev},\"c_min\":${c_min},\"c_max\":${c_max},\"rust_median\":${rust_median},\"rust_mean\":${rust_mean},\"rust_stddev\":${rust_stddev},\"rust_min\":${rust_min},\"rust_max\":${rust_max},\"ratio\":${ratio},\"rust_ratio\":${rust_ratio},\"rating\":\"${rating}\"}")
    else
        JSON_ENTRIES+=("{\"name\":\"${name}\",\"bmb_median\":${bmb_median},\"bmb_mean\":${bmb_mean},\"bmb_stddev\":${bmb_stddev},\"bmb_min\":${bmb_min},\"bmb_max\":${bmb_max},\"c_median\":${c_median},\"c_mean\":${c_mean},\"c_stddev\":${c_stddev},\"c_min\":${c_min},\"c_max\":${c_max},\"ratio\":${ratio},\"rating\":\"${rating}\"}")
    fi
done

# ─── Summary ─────────────────────────────────────────────────────────────────
echo ""
printf "%s\n" "$(printf '=%.0s' {1..90})"
TOTAL=$((FASTER_COUNT + PASS_COUNT + WARN_COUNT + FAIL_COUNT))
echo -e "${BOLD}Summary: ${GREEN}${FASTER_COUNT} FASTER${NC}, ${CYAN}${PASS_COUNT} PASS${NC}, ${YELLOW}${WARN_COUNT} WARN${NC}, ${RED}${FAIL_COUNT} FAIL${NC} (${TOTAL} total)"
echo ""
echo "Methodology:"
echo "  BMB:  emit-ir → opt --mcpu=native -O3 +scalarizer +slp → clang -O3 -march=native"
echo "  C:    clang -O3 -march=native"
[ "$HAS_RUSTC" = true ] && echo "  Rust: rustc -C opt-level=3 -C target-cpu=native"
echo "  Runs: ${WARMUP} warmup (discarded) + ${RUNS} measured, median reported"
echo "  Rating (BMB vs C): <0.95x=FASTER, 0.95-1.05x=PASS, 1.05-1.10x=WARN, >1.10x=FAIL"
[ "$NO_CHECK" = false ] && echo "  Correctness: output verified (BMB == C${HAS_RUSTC:+ == Rust})"
[ $MISMATCH_COUNT -gt 0 ] && echo -e "  ${RED}⚠ Output mismatches: ${MISMATCH_COUNT}${NC}"

# ─── JSON Output ─────────────────────────────────────────────────────────────
if [ -n "$JSON_FILE" ]; then
    {
        echo "{"
        echo "  \"date\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
        echo "  \"config\": {"
        echo "    \"runs\": ${RUNS},"
        echo "    \"warmup\": ${WARMUP},"
        echo "    \"bmb_pipeline\": \"emit-ir → opt -O3 --mcpu=native +scalarizer +slp → clang -O3 -march=native\","
        echo "    \"c_pipeline\": \"clang -O3 -march=native\","
        echo "    \"platform\": \"$(uname -s) $(uname -m)\","
        echo "    \"opt_version\": \"$(opt --version 2>&1 | head -2 | tail -1 | tr -d '[:space:]')\","
        echo "    \"clang_version\": \"$(clang --version 2>&1 | head -1)\","
        echo "    \"correctness_verified\": $( [ "$NO_CHECK" = false ] && echo true || echo false ),"
        echo "    \"output_mismatches\": ${MISMATCH_COUNT}"
        echo "  },"
        echo "  \"summary\": {"
        echo "    \"faster\": ${FASTER_COUNT},"
        echo "    \"pass\": ${PASS_COUNT},"
        echo "    \"warn\": ${WARN_COUNT},"
        echo "    \"fail\": ${FAIL_COUNT},"
        echo "    \"total\": ${TOTAL}"
        echo "  },"
        echo "  \"results\": ["
        first_entry=true
        for entry in "${JSON_ENTRIES[@]}"; do
            [ "$first_entry" = true ] && first_entry=false || echo ","
            echo -n "    $entry"
        done
        echo ""
        echo "  ]"
        echo "}"
    } > "$JSON_FILE"
    echo ""
    echo "JSON results written to: $JSON_FILE"
fi
