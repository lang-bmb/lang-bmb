#!/bin/bash
# BMB Bootstrap Compiler Benchmark Runner
# Compares performance between Rust BMB, Bootstrap BMB (Stage 1/2), and C
#
# Usage:
#   ./scripts/benchmark-bootstrap.sh [options]
#
# Options:
#   --stage STAGE       Use stage1 or stage2 bootstrap compiler (default: stage2)
#   --tier TIER         Run specific tier (1 = compute, 2 = contract, all)
#   --runs N            Number of runs per benchmark (default: 5)
#   --build-only        Only build, don't run benchmarks
#   --compare           Compare Rust BMB vs Bootstrap BMB compile times
#   --verbose           Show detailed output
#
# This script measures:
#   1. Compile time: How long the bootstrap compiler takes to compile benchmarks
#   2. Runtime: How fast the compiled executables run
#   3. Comparison with Rust BMB and C baselines

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BENCHMARK_DIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb"
RESULTS_DIR="${PROJECT_ROOT}/target/benchmarks-bootstrap"
RUNTIME_PATH="${PROJECT_ROOT}/bmb/runtime"

# Rust BMB compiler
RUST_BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"

# Bootstrap compilers
STAGE1_BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1.exe"
STAGE2_BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage2.exe"

# Parse arguments
STAGE="stage2"
TIER="1"
RUNS=5
BUILD_ONLY=false
COMPARE_MODE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --stage)
            STAGE="$2"
            shift 2
            ;;
        --tier)
            TIER="$2"
            shift 2
            ;;
        --runs)
            RUNS="$2"
            shift 2
            ;;
        --build-only)
            BUILD_ONLY=true
            shift
            ;;
        --compare)
            COMPARE_MODE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Select bootstrap compiler
if [ "$STAGE" = "stage1" ]; then
    BOOTSTRAP_BMB="$STAGE1_BMB"
    STAGE_NAME="Stage 1"
else
    BOOTSTRAP_BMB="$STAGE2_BMB"
    STAGE_NAME="Stage 2"
fi

log() {
    echo -e "$1"
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
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

# Build BMB source using bootstrap compiler
# Returns: compile_time_ms or "FAIL"
build_with_bootstrap() {
    local src=$1
    local output_base=$2
    local ll_file="${output_base}.ll"
    local bc_file="${output_base}.bc"
    local obj_file="${output_base}.o"
    local exe_file="${output_base}.exe"

    local start=$(get_time_ms)

    # Step 1: Generate LLVM IR with bootstrap compiler
    if ! "$BOOTSTRAP_BMB" "$src" "$ll_file" > /dev/null 2>&1; then
        echo "FAIL"
        return 1
    fi

    # Step 2: Optimize with opt (use default<O3> + scalarizer to match Rust compiler)
    # The scalarizer undoes inefficient auto-vectorization (e.g., <2 x i64> udiv)
    # v0.90.19: Early instcombine cleans identity copies before SimplifyCFG (branch→select)
    if ! opt "--passes=function(instcombine),default<O3>,scalarizer" -o "$bc_file" "$ll_file" > /dev/null 2>&1; then
        # Fallback: try plain -O3
        if ! opt -O3 -o "$bc_file" "$ll_file" > /dev/null 2>&1; then
            echo "FAIL"
            return 1
        fi
    fi

    # Step 3: Compile to object file (--mcpu=native for target-specific optimizations)
    if ! llc -O3 --mcpu=native -filetype=obj -o "$obj_file" "$bc_file" > /dev/null 2>&1; then
        echo "FAIL"
        return 1
    fi

    # Step 4: Link with runtime (needs both bmb_runtime.o and bmb_event_loop.o)
    local runtime_obj="${RUNTIME_PATH}/bmb_runtime.o"
    local event_loop_obj="${RUNTIME_PATH}/bmb_event_loop.o"
    if [ ! -f "$runtime_obj" ]; then
        gcc -c -O2 -o "$runtime_obj" "${RUNTIME_PATH}/bmb_runtime.c" > /dev/null 2>&1 || true
    fi
    if [ ! -f "$event_loop_obj" ]; then
        gcc -c -O2 -o "$event_loop_obj" "${RUNTIME_PATH}/bmb_event_loop.c" > /dev/null 2>&1 || true
    fi

    # Windows requires -lws2_32 for socket functions in event loop
    local link_flags="-lm"
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "mingw"* ]] || [[ "$(uname -s)" == *MINGW* ]] || [[ "$(uname -s)" == *MSYS* ]]; then
        link_flags="-lm -lws2_32"
    fi

    if ! gcc -o "$exe_file" "$obj_file" "$runtime_obj" "$event_loop_obj" $link_flags > /dev/null 2>&1; then
        echo "FAIL"
        return 1
    fi

    local end=$(get_time_ms)
    echo $((end - start))
}

# Build BMB source using Rust BMB compiler
build_with_rust_bmb() {
    local src=$1
    local output=$2

    local start=$(get_time_ms)

    if ! "$RUST_BMB" build "$src" -o "$output" > /dev/null 2>&1; then
        echo "FAIL"
        return 1
    fi

    local end=$(get_time_ms)
    echo $((end - start))
}

# Build C source
build_c() {
    local src=$1
    local output=$2

    local start=$(get_time_ms)

    if ! clang -O3 -march=native -o "$output" "$src" -lm 2>/dev/null; then
        if ! gcc -O3 -march=native -o "$output" "$src" -lm 2>/dev/null; then
            echo "FAIL"
            return 1
        fi
    fi

    local end=$(get_time_ms)
    echo $((end - start))
}

# Check prerequisites
check_prerequisites() {
    local missing=()

    if [ ! -f "$BOOTSTRAP_BMB" ]; then
        missing+=("Bootstrap compiler ($STAGE_NAME)")
    fi

    if [ ! -f "$RUST_BMB" ]; then
        missing+=("Rust BMB compiler")
    fi

    if ! command -v opt &> /dev/null; then
        missing+=("LLVM opt")
    fi

    if ! command -v llc &> /dev/null; then
        missing+=("LLVM llc")
    fi

    if ! command -v gcc &> /dev/null; then
        missing+=("GCC")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        log "${RED}Missing prerequisites:${NC}"
        for item in "${missing[@]}"; do
            log "  - $item"
        done
        log ""
        log "Run ${CYAN}bash scripts/bootstrap.sh${NC} to build the bootstrap compiler."
        exit 1
    fi
}

# Create results directory
mkdir -p "$RESULTS_DIR"

log "======================================"
log "BMB Bootstrap Compiler Benchmark"
log "======================================"
log "Bootstrap Compiler: ${CYAN}$STAGE_NAME${NC}"
log "Runs per benchmark: $RUNS"
log "Tier: $TIER"
log ""

check_prerequisites

# =============================================================================
# COMPARE MODE: Rust BMB vs Bootstrap BMB compile times
# =============================================================================
if [ "$COMPARE_MODE" = true ]; then
    log "${BLUE}=== Compile Time Comparison: Rust BMB vs Bootstrap ($STAGE_NAME) ===${NC}"
    log ""
    log "$(printf '%-25s %-15s %-15s %-10s' 'Benchmark' 'Rust BMB' 'Bootstrap' 'Ratio')"
    log "$(printf '%s' '----------------------------------------------------------------------')"

    for bench_dir in "${BENCHMARK_DIR}/benches/compute/"*/; do
        [ -d "$bench_dir" ] || continue
        name=$(basename "$bench_dir")
        bmb_src="${bench_dir}bmb/main.bmb"

        [ -f "$bmb_src" ] || continue

        rust_out="${RESULTS_DIR}/${name}_rust"
        boot_out="${RESULTS_DIR}/${name}_boot"

        # Measure Rust BMB compile time
        rust_compile=$(build_with_rust_bmb "$bmb_src" "$rust_out")

        # Measure Bootstrap compile time
        boot_compile=$(build_with_bootstrap "$bmb_src" "$boot_out")

        # Calculate ratio
        ratio="-"
        if [ "$rust_compile" != "FAIL" ] && [ "$boot_compile" != "FAIL" ] && [ "$rust_compile" -gt 0 ]; then
            ratio=$(python3 -c "print(f'{$boot_compile / $rust_compile:.2f}x')")
        fi

        rust_str="$rust_compile"
        boot_str="$boot_compile"
        [ "$rust_compile" = "FAIL" ] && rust_str="${RED}FAIL${NC}"
        [ "$boot_compile" = "FAIL" ] && boot_str="${RED}FAIL${NC}"

        printf "%-25s %-15s %-15s %-10s\n" "$name" "${rust_str}ms" "${boot_str}ms" "$ratio"
    done

    log ""
    exit 0
fi

# =============================================================================
# TIER 1: Core Compute Benchmarks
# =============================================================================
run_tier_1() {
    log "${BLUE}=== TIER 1: Core Compute Benchmarks ===${NC}"
    log "Compiler: ${CYAN}Bootstrap $STAGE_NAME${NC}"
    log ""
    log "$(printf '%-20s %-12s %-12s %-12s %-10s %-10s' 'Benchmark' 'Boot(ms)' 'Rust(ms)' 'C(ms)' 'vs Rust' 'vs C')"
    log "$(printf '%s' '------------------------------------------------------------------------------------')"

    for bench_dir in "${BENCHMARK_DIR}/benches/compute/"*/; do
        [ -d "$bench_dir" ] || continue
        local name=$(basename "$bench_dir")

        local bmb_src="${bench_dir}bmb/main.bmb"
        local c_src="${bench_dir}c/main.c"
        local boot_exe="${RESULTS_DIR}/${name}_boot"
        local rust_exe="${RESULTS_DIR}/${name}_rust"
        local c_exe="${RESULTS_DIR}/${name}_c"

        local boot_time="null"
        local rust_time="null"
        local c_time="null"
        local ratio_rust="null"
        local ratio_c="null"

        # Build with Bootstrap compiler
        if [ -f "$bmb_src" ]; then
            log_verbose "Building $name with Bootstrap..."
            local compile_result=$(build_with_bootstrap "$bmb_src" "$boot_exe")
            if [ "$compile_result" != "FAIL" ]; then
                local exe_actual="${boot_exe}.exe"
                if [ -f "$exe_actual" ]; then
                    boot_time=$(run_benchmark "$exe_actual")
                    log_verbose "  Bootstrap compile: ${compile_result}ms, runtime: ${boot_time}ms"
                fi
            else
                log_verbose "  ${RED}Bootstrap build failed${NC}"
            fi
        fi

        # Build with Rust BMB
        if [ -f "$bmb_src" ]; then
            log_verbose "Building $name with Rust BMB..."
            local compile_result=$(build_with_rust_bmb "$bmb_src" "$rust_exe")
            if [ "$compile_result" != "FAIL" ]; then
                local exe_actual="${rust_exe}.exe"
                [ ! -f "$exe_actual" ] && exe_actual="$rust_exe"
                if [ -f "$exe_actual" ]; then
                    rust_time=$(run_benchmark "$exe_actual")
                    log_verbose "  Rust compile: ${compile_result}ms, runtime: ${rust_time}ms"
                fi
            fi
        fi

        # Build C baseline
        if [ -f "$c_src" ]; then
            log_verbose "Building $name C baseline..."
            local compile_result=$(build_c "$c_src" "$c_exe")
            if [ "$compile_result" != "FAIL" ]; then
                local exe_actual="${c_exe}.exe"
                [ ! -f "$exe_actual" ] && exe_actual="$c_exe"
                if [ -f "$exe_actual" ]; then
                    c_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        # Calculate ratios
        if [ "$boot_time" != "null" ] && [ "$rust_time" != "null" ] && [ "$rust_time" -gt 0 ]; then
            ratio_rust=$(python3 -c "print(f'{$boot_time / $rust_time:.2f}')")
        fi

        if [ "$boot_time" != "null" ] && [ "$c_time" != "null" ] && [ "$c_time" -gt 0 ]; then
            ratio_c=$(python3 -c "print(f'{$boot_time / $c_time:.2f}')")
        fi

        # Format output
        local boot_str="$boot_time"
        local rust_str="$rust_time"
        local c_str="$c_time"
        local ratio_rust_str="-"
        local ratio_c_str="-"

        [ "$boot_time" = "null" ] && boot_str="${RED}FAIL${NC}"
        [ "$rust_time" = "null" ] && rust_str="N/A"
        [ "$c_time" = "null" ] && c_str="N/A"

        if [ "$ratio_rust" != "null" ]; then
            if [ "$(python3 -c "print('1' if $ratio_rust <= 1.05 else '0')")" = "1" ]; then
                ratio_rust_str="${GREEN}${ratio_rust}x${NC}"
            elif [ "$(python3 -c "print('1' if $ratio_rust <= 1.20 else '0')")" = "1" ]; then
                ratio_rust_str="${YELLOW}${ratio_rust}x${NC}"
            else
                ratio_rust_str="${RED}${ratio_rust}x${NC}"
            fi
        fi

        if [ "$ratio_c" != "null" ]; then
            if [ "$(python3 -c "print('1' if $ratio_c <= 1.10 else '0')")" = "1" ]; then
                ratio_c_str="${GREEN}${ratio_c}x${NC}"
            elif [ "$(python3 -c "print('1' if $ratio_c <= 1.50 else '0')")" = "1" ]; then
                ratio_c_str="${YELLOW}${ratio_c}x${NC}"
            else
                ratio_c_str="${RED}${ratio_c}x${NC}"
            fi
        fi

        printf "%-20s %-12s %-12s %-12s %b     %b\n" "$name" "$boot_str" "$rust_str" "$c_str" "$ratio_rust_str" "$ratio_c_str"
    done

    log ""
}

# =============================================================================
# TIER 2: Contract Feature Benchmarks
# =============================================================================
run_tier_2() {
    log "${BLUE}=== TIER 2: Contract Feature Benchmarks ===${NC}"
    log "Compiler: ${CYAN}Bootstrap $STAGE_NAME${NC}"
    log ""

    for bench_dir in "${BENCHMARK_DIR}/benches/contract/"*/; do
        [ -d "$bench_dir" ] || continue
        local name=$(basename "$bench_dir")

        local bmb_src="${bench_dir}bmb/main.bmb"
        local boot_exe="${RESULTS_DIR}/${name}_contract_boot"

        local boot_time="null"

        if [ -f "$bmb_src" ]; then
            local compile_result=$(build_with_bootstrap "$bmb_src" "$boot_exe")
            if [ "$compile_result" != "FAIL" ]; then
                local exe_actual="${boot_exe}.exe"
                if [ -f "$exe_actual" ]; then
                    boot_time=$(run_benchmark "$exe_actual")
                fi
            fi
        fi

        if [ "$boot_time" != "null" ]; then
            printf "%-20s Bootstrap: %6dms\n" "$name" "$boot_time"
        else
            printf "%-20s Bootstrap: ${RED}FAIL${NC}\n" "$name"
        fi
    done

    log ""
}

# =============================================================================
# Main execution
# =============================================================================
TOTAL_START=$(get_time_ms)

case $TIER in
    1)
        run_tier_1
        ;;
    2)
        run_tier_2
        ;;
    all)
        run_tier_1
        run_tier_2
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
log ""
log "${GREEN}Key:${NC}"
log "  Boot(ms)  = Runtime of executable compiled by Bootstrap $STAGE_NAME"
log "  Rust(ms)  = Runtime of executable compiled by Rust BMB"
log "  C(ms)     = Runtime of C baseline (clang -O3)"
log "  vs Rust   = Boot / Rust ratio (${GREEN}green${NC} = same, ${YELLOW}yellow${NC} = 5-20% slower, ${RED}red${NC} = >20% slower)"
log "  vs C      = Boot / C ratio (${GREEN}green${NC} = <=1.1x, ${YELLOW}yellow${NC} = <=1.5x, ${RED}red${NC} = >1.5x)"
