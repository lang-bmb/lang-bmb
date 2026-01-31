#!/bin/bash
# BMB Benchmark Runner with JSON Output
# Part of the Bootstrap + Benchmark Cycle System
#
# Usage:
#   ./scripts/benchmark.sh [options]
#
# Options:
#   --json          Output results in JSON format only
#   --tier TIER     Run specific tier (0, 1, 2, 3, or all)
#   --runs N        Number of runs per benchmark (default: 5)
#   --output FILE   Write JSON results to file
#   --verbose       Show detailed output
#   --list          List available benchmarks and exit
#
# Tiers:
#   0 - Bootstrap compile times (lexer, parser, types)
#   1 - Core compute benchmarks (fair comparison: BMB vs C vs Rust)
#   2 - Contract feature benchmarks
#   3 - Real-world benchmarks

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
# Try MinGW build first (required on Windows for native compilation), then fall back to regular
if [ -f "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe" ]; then
    BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"
elif [ -f "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb" ]; then
    BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb"
elif [ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]; then
    BMB="${PROJECT_ROOT}/target/release/bmb.exe"
else
    BMB="${PROJECT_ROOT}/target/release/bmb"
fi
BENCHMARK_DIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb"
RESULTS_DIR="${PROJECT_ROOT}/target/benchmarks"

# Parse arguments
JSON_OUTPUT=false
TIER="all"
RUNS=5
OUTPUT_FILE=""
VERBOSE=false
LIST_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --tier)
            TIER="$2"
            shift 2
            ;;
        --runs)
            RUNS="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --list)
            LIST_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors (disabled for JSON mode)
if [ "$JSON_OUTPUT" = true ]; then
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
fi

# Logging
log() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

log_verbose() {
    if [ "$VERBOSE" = true ] && [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

# Timing function (returns milliseconds)
get_time_ms() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        python3 -c 'import time; print(int(time.time() * 1000))'
    else
        date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
    fi
}

# Run command and measure time in milliseconds
time_cmd() {
    local start=$(get_time_ms)
    "$@" > /dev/null 2>&1 || true
    local end=$(get_time_ms)
    echo $((end - start))
}

# Run benchmark multiple times and get median
run_benchmark() {
    local exe=$1
    local times=()

    for ((i=1; i<=RUNS; i++)); do
        local t=$(time_cmd "$exe")
        times+=($t)
    done

    # Sort and get median
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
    local median=${sorted[$((RUNS/2))]}
    echo $median
}

# JSON result accumulator
declare -a JSON_RESULTS
JSON_RESULTS=()

add_result() {
    local tier=$1
    local name=$2
    local bmb_ms=$3
    local c_ms=$4
    local rust_ms=$5
    local ratio_c=$6
    local ratio_rust=$7

    JSON_RESULTS+=("{\"tier\":$tier,\"name\":\"$name\",\"bmb_ms\":$bmb_ms,\"c_ms\":$c_ms,\"rust_ms\":$rust_ms,\"ratio_c\":$ratio_c,\"ratio_rust\":$ratio_rust}")
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check prerequisites
if [ ! -f "$BMB" ]; then
    log "${RED}Error: BMB compiler not found at $BMB${NC}"
    log "Build it first with: cargo build --release"
    exit 1
fi

# Handle Windows .exe extension
BMB_ACTUAL="$BMB"
[ -f "${BMB}.exe" ] && BMB_ACTUAL="${BMB}.exe"

# List benchmarks mode
if [ "$LIST_ONLY" = true ]; then
    echo "Available Benchmarks:"
    echo ""
    echo "Tier 0 - Bootstrap:"
    for f in "${PROJECT_ROOT}/bootstrap/"*.bmb; do
        echo "  - $(basename "$f" .bmb)"
    done
    echo ""
    echo "Tier 1 - Core Compute:"
    for d in "${BENCHMARK_DIR}/benches/compute/"*/; do
        echo "  - $(basename "$d")"
    done
    echo ""
    echo "Tier 2 - Contract Features:"
    for d in "${BENCHMARK_DIR}/benches/contract/"*/; do
        echo "  - $(basename "$d")"
    done
    echo ""
    echo "Tier 3 - Real World:"
    for d in "${BENCHMARK_DIR}/benches/real_world/"*/; do
        echo "  - $(basename "$d")"
    done
    exit 0
fi

TOTAL_START=$(get_time_ms)

log "======================================"
log "BMB Benchmark Suite"
log "======================================"
log "Runs per benchmark: $RUNS"
log "Tier: $TIER"
log ""

# =============================================================================
# TIER 0: Bootstrap Compile Times
# =============================================================================
run_tier_0() {
    log "${BLUE}=== TIER 0: Bootstrap Compile Times ===${NC}"
    log ""

    BOOTSTRAP_FILES=("lexer" "parser" "types" "mir" "llvm_ir" "compiler")

    for name in "${BOOTSTRAP_FILES[@]}"; do
        local src="${PROJECT_ROOT}/bootstrap/${name}.bmb"
        if [ ! -f "$src" ]; then
            log_verbose "Skipping $name (not found)"
            continue
        fi

        # Measure type check time
        local times=()
        for ((i=1; i<=RUNS; i++)); do
            local start=$(get_time_ms)
            "$BMB_ACTUAL" check "$src" > /dev/null 2>&1 || true
            local end=$(get_time_ms)
            times+=($((end - start)))
        done

        IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
        local median=${sorted[$((RUNS/2))]}

        if [ "$JSON_OUTPUT" = false ]; then
            printf "%-20s check: %6dms\n" "$name" "$median"
        fi
        add_result 0 "$name" "$median" "null" "null" "null" "null"
    done
    log ""
}

# =============================================================================
# TIER 1: Core Compute Benchmarks
# =============================================================================
run_tier_1() {
    log "${BLUE}=== TIER 1: Core Compute Benchmarks ===${NC}"
    log ""
    log "$(printf '%-20s %-12s %-12s %-12s %-10s' 'Benchmark' 'BMB (ms)' 'C (ms)' 'Rust (ms)' 'vs C')"
    log "$(printf '%s' '------------------------------------------------------------------------')"

    for bench_dir in "${BENCHMARK_DIR}/benches/compute/"*/; do
        [ -d "$bench_dir" ] || continue
        local name=$(basename "$bench_dir")

        local bmb_src="${bench_dir}bmb/main.bmb"
        local c_src="${bench_dir}c/main.c"
        local rust_src="${bench_dir}rust/main.rs"
        local bmb_exe="${RESULTS_DIR}/${name}_bmb"
        local c_exe="${RESULTS_DIR}/${name}_c"
        local rust_exe="${RESULTS_DIR}/${name}_rust"

        local bmb_time="null"
        local c_time="null"
        local rust_time="null"
        local ratio_c="null"
        local ratio_rust="null"

        # Build and run BMB
        if [ -f "$bmb_src" ]; then
            if "$BMB_ACTUAL" build "$bmb_src" -o "$bmb_exe" > /dev/null 2>&1; then
                local exe_actual="$bmb_exe"
                [ -f "${bmb_exe}.exe" ] && exe_actual="${bmb_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    bmb_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Build and run C (use clang for fair LLVM-to-LLVM comparison)
        if [ -f "$c_src" ]; then
            if clang -O3 -march=native -o "$c_exe" "$c_src" -lm 2>/dev/null || \
               gcc -O3 -march=native -o "$c_exe" "$c_src" -lm 2>/dev/null; then
                local exe_actual="$c_exe"
                [ -f "${c_exe}.exe" ] && exe_actual="${c_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    c_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Build and run Rust
        if [ -f "$rust_src" ]; then
            if rustc -O -o "$rust_exe" "$rust_src" 2>/dev/null; then
                local exe_actual="$rust_exe"
                [ -f "${rust_exe}.exe" ] && exe_actual="${rust_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    rust_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Calculate ratios (use Python if bc not available)
        calc_ratio() {
            local num=$1
            local denom=$2
            if command -v bc &> /dev/null; then
                echo "scale=2; $num / $denom" | bc
            else
                python3 -c "print(f'{$num / $denom:.2f}')"
            fi
        }

        if [ "$bmb_time" != "null" ] && [ "$c_time" != "null" ] && [ "$c_time" -gt 0 ]; then
            ratio_c=$(calc_ratio "$bmb_time" "$c_time")
        fi

        if [ "$bmb_time" != "null" ] && [ "$rust_time" != "null" ] && [ "$rust_time" -gt 0 ]; then
            ratio_rust=$(calc_ratio "$bmb_time" "$rust_time")
        fi

        # Print results
        local bmb_str="${bmb_time}"
        local c_str="${c_time}"
        local rust_str="${rust_time}"
        local ratio_str="-"

        [ "$bmb_time" = "null" ] && bmb_str="FAIL"
        [ "$c_time" = "null" ] && c_str="N/A"
        [ "$rust_time" = "null" ] && rust_str="N/A"

        if [ "$ratio_c" != "null" ]; then
            # Compare ratio using Python (bc not available on all systems)
            compare_le() {
                python3 -c "print('1' if $1 <= $2 else '0')"
            }
            if [ "$(compare_le "$ratio_c" "1.10")" = "1" ]; then
                ratio_str="${GREEN}${ratio_c}x${NC}"
            elif [ "$(compare_le "$ratio_c" "1.50")" = "1" ]; then
                ratio_str="${YELLOW}${ratio_c}x${NC}"
            else
                ratio_str="${RED}${ratio_c}x${NC}"
            fi
        fi

        if [ "$JSON_OUTPUT" = false ]; then
            printf "%-20s %-12s %-12s %-12s %b\n" "$name" "$bmb_str" "$c_str" "$rust_str" "$ratio_str"
        fi

        add_result 1 "$name" "$bmb_time" "$c_time" "$rust_time" "$ratio_c" "$ratio_rust"
    done
    log ""
}

# =============================================================================
# TIER 2: Contract Feature Benchmarks
# =============================================================================
run_tier_2() {
    log "${BLUE}=== TIER 2: Contract Feature Benchmarks ===${NC}"
    log ""

    for bench_dir in "${BENCHMARK_DIR}/benches/contract/"*/; do
        [ -d "$bench_dir" ] || continue
        local name=$(basename "$bench_dir")

        local bmb_src="${bench_dir}bmb/main.bmb"
        local bmb_exe="${RESULTS_DIR}/${name}_contract"

        local bmb_time="null"

        if [ -f "$bmb_src" ]; then
            if "$BMB_ACTUAL" build "$bmb_src" -o "$bmb_exe" > /dev/null 2>&1; then
                local exe_actual="$bmb_exe"
                [ -f "${bmb_exe}.exe" ] && exe_actual="${bmb_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    bmb_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        if [ "$JSON_OUTPUT" = false ]; then
            if [ "$bmb_time" != "null" ]; then
                printf "%-20s BMB: %6dms\n" "$name" "$bmb_time"
            else
                printf "%-20s BMB: ${RED}FAIL${NC}\n" "$name"
            fi
        fi

        add_result 2 "$name" "$bmb_time" "null" "null" "null" "null"
    done
    log ""
}

# =============================================================================
# TIER 3: Real World Benchmarks
# =============================================================================
run_tier_3() {
    log "${BLUE}=== TIER 3: Real World Benchmarks ===${NC}"
    log ""
    log "$(printf '%-20s %-12s %-12s %-10s' 'Benchmark' 'BMB (ms)' 'C (ms)' 'vs C')"
    log "$(printf '%s' '----------------------------------------------------')"

    for bench_dir in "${BENCHMARK_DIR}/benches/real_world/"*/; do
        [ -d "$bench_dir" ] || continue
        local name=$(basename "$bench_dir")

        local bmb_src="${bench_dir}bmb/main.bmb"
        local c_src="${bench_dir}c/main.c"
        local bmb_exe="${RESULTS_DIR}/${name}_rw_bmb"
        local c_exe="${RESULTS_DIR}/${name}_rw_c"

        local bmb_time="null"
        local c_time="null"
        local ratio_c="null"

        # Build and run BMB
        if [ -f "$bmb_src" ]; then
            if "$BMB_ACTUAL" build "$bmb_src" -o "$bmb_exe" > /dev/null 2>&1; then
                local exe_actual="$bmb_exe"
                [ -f "${bmb_exe}.exe" ] && exe_actual="${bmb_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    bmb_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Build and run C (use clang for fair LLVM-to-LLVM comparison)
        if [ -f "$c_src" ]; then
            if clang -O3 -march=native -o "$c_exe" "$c_src" -lm 2>/dev/null || \
               gcc -O3 -march=native -o "$c_exe" "$c_src" -lm 2>/dev/null; then
                local exe_actual="$c_exe"
                [ -f "${c_exe}.exe" ] && exe_actual="${c_exe}.exe"
                if [ -x "$exe_actual" ] || [ -f "$exe_actual" ]; then
                    c_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Calculate ratio (use Python if bc not available)
        if [ "$bmb_time" != "null" ] && [ "$c_time" != "null" ] && [ "$c_time" -gt 0 ]; then
            if command -v bc &> /dev/null; then
                ratio_c=$(echo "scale=2; $bmb_time / $c_time" | bc)
            else
                ratio_c=$(python3 -c "print(f'{$bmb_time / $c_time:.2f}')")
            fi
        fi

        # Print results
        local bmb_str="${bmb_time}"
        local c_str="${c_time}"
        local ratio_str="-"

        [ "$bmb_time" = "null" ] && bmb_str="FAIL"
        [ "$c_time" = "null" ] && c_str="N/A"

        if [ "$ratio_c" != "null" ]; then
            ratio_str="${ratio_c}x"
        fi

        if [ "$JSON_OUTPUT" = false ]; then
            printf "%-20s %-12s %-12s %-10s\n" "$name" "$bmb_str" "$c_str" "$ratio_str"
        fi

        add_result 3 "$name" "$bmb_time" "$c_time" "null" "$ratio_c" "null"
    done
    log ""
}

# Run selected tiers
case $TIER in
    0)
        run_tier_0
        ;;
    1)
        run_tier_1
        ;;
    2)
        run_tier_2
        ;;
    3)
        run_tier_3
        ;;
    all)
        run_tier_0
        run_tier_1
        run_tier_2
        run_tier_3
        ;;
    *)
        log "${RED}Unknown tier: $TIER${NC}"
        exit 1
        ;;
esac

TOTAL_END=$(get_time_ms)
TOTAL_TIME=$((TOTAL_END - TOTAL_START))

log "======================================"
log "Total benchmark time: ${TOTAL_TIME}ms"
log "======================================"

# Generate JSON output
generate_json() {
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    echo "{"
    echo "  \"timestamp\": \"$timestamp\","
    echo "  \"total_time_ms\": $TOTAL_TIME,"
    echo "  \"runs_per_benchmark\": $RUNS,"
    echo "  \"results\": ["

    local first=true
    for result in "${JSON_RESULTS[@]}"; do
        if [ "$first" = true ]; then
            first=false
        else
            echo ","
        fi
        echo -n "    $result"
    done

    echo ""
    echo "  ]"
    echo "}"
}

if [ "$JSON_OUTPUT" = true ]; then
    if [ -n "$OUTPUT_FILE" ]; then
        generate_json > "$OUTPUT_FILE"
    else
        generate_json
    fi
elif [ -n "$OUTPUT_FILE" ]; then
    generate_json > "$OUTPUT_FILE"
    log "Results written to: $OUTPUT_FILE"
fi
