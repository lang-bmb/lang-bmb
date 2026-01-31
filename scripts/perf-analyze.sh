#!/bin/bash
# BMB Performance Analysis Script
# Part of the Bootstrap + Benchmark Cycle System
#
# Detailed performance analysis for specific benchmarks.
# Supports CPU profiling with perf/Instruments and memory profiling.
#
# Usage:
#   ./scripts/perf-analyze.sh <benchmark-name> [options]
#
# Options:
#   --profile       Enable CPU profiling (requires perf on Linux)
#   --memory        Enable memory profiling (requires valgrind)
#   --compare-ir    Compare LLVM IR between BMB and C
#   --asm           Generate and analyze assembly output
#   --runs N        Number of runs for timing (default: 10)
#   --output DIR    Output directory for results

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BMB="${PROJECT_ROOT}/target/release/bmb"
BENCHMARK_DIR="${PROJECT_ROOT}/ecosystem/benchmark-bmb"
OUTPUT_DIR="${PROJECT_ROOT}/target/perf-analysis"

# Parse arguments
BENCHMARK=""
PROFILE=false
MEMORY=false
COMPARE_IR=false
ASM=false
RUNS=10

while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE=true
            shift
            ;;
        --memory)
            MEMORY=true
            shift
            ;;
        --compare-ir)
            COMPARE_IR=true
            shift
            ;;
        --asm)
            ASM=true
            shift
            ;;
        --runs)
            RUNS="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -*)
            echo "Unknown option: $1"
            exit 1
            ;;
        *)
            BENCHMARK="$1"
            shift
            ;;
    esac
done

if [ -z "$BENCHMARK" ]; then
    echo "Usage: $0 <benchmark-name> [options]"
    echo ""
    echo "Available benchmarks:"
    for d in "${BENCHMARK_DIR}/benches/compute/"*/; do
        echo "  - $(basename "$d")"
    done
    exit 1
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "$1"
}

# Find benchmark
BENCH_DIR=""
for category in compute contract real_world; do
    if [ -d "${BENCHMARK_DIR}/benches/${category}/${BENCHMARK}" ]; then
        BENCH_DIR="${BENCHMARK_DIR}/benches/${category}/${BENCHMARK}"
        break
    fi
done

if [ -z "$BENCH_DIR" ]; then
    log "${RED}Error: Benchmark '$BENCHMARK' not found${NC}"
    exit 1
fi

log "======================================"
log "${BLUE}BMB Performance Analysis${NC}"
log "======================================"
log "Benchmark: $BENCHMARK"
log "Directory: $BENCH_DIR"
log ""

mkdir -p "$OUTPUT_DIR/$BENCHMARK"

# Handle Windows .exe extension
BMB_ACTUAL="$BMB"
[ -f "${BMB}.exe" ] && BMB_ACTUAL="${BMB}.exe"

# Timing function
get_time_ms() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        python3 -c 'import time; print(int(time.time() * 1000))'
    else
        date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
    fi
}

# =============================================================================
# Build Executables
# =============================================================================
log "${YELLOW}[1/5] Building executables...${NC}"

BMB_SRC="${BENCH_DIR}/bmb/main.bmb"
C_SRC="${BENCH_DIR}/c/main.c"
BMB_EXE="${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_bmb"
C_EXE="${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_c"
BMB_LL="${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_bmb.ll"
C_LL="${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_c.ll"

# Build BMB
if [ -f "$BMB_SRC" ]; then
    log "Building BMB version..."
    "$BMB_ACTUAL" build "$BMB_SRC" --emit-ir -o "$BMB_LL" 2>/dev/null || true
    "$BMB_ACTUAL" build "$BMB_SRC" -o "$BMB_EXE" 2>/dev/null || true
    [ -f "${BMB_EXE}.exe" ] && BMB_EXE="${BMB_EXE}.exe"
fi

# Build C
if [ -f "$C_SRC" ]; then
    log "Building C version..."
    gcc -O3 -march=native -S -o "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_c.s" "$C_SRC" -lm 2>/dev/null || true
    gcc -O3 -march=native -o "$C_EXE" "$C_SRC" -lm 2>/dev/null || true
    [ -f "${C_EXE}.exe" ] && C_EXE="${C_EXE}.exe"

    # Generate LLVM IR for comparison
    if command -v clang &> /dev/null; then
        clang -O3 -S -emit-llvm -o "$C_LL" "$C_SRC" 2>/dev/null || true
    fi
fi
log ""

# =============================================================================
# Timing Analysis
# =============================================================================
log "${YELLOW}[2/5] Running timing analysis ($RUNS runs)...${NC}"

run_timing() {
    local exe=$1
    local name=$2
    local times=()

    for ((i=1; i<=RUNS; i++)); do
        local start=$(get_time_ms)
        "$exe" > /dev/null 2>&1 || true
        local end=$(get_time_ms)
        times+=($((end - start)))
    done

    # Calculate statistics
    local sum=0
    local min=${times[0]}
    local max=${times[0]}

    for t in "${times[@]}"; do
        sum=$((sum + t))
        [ $t -lt $min ] && min=$t
        [ $t -gt $max ] && max=$t
    done

    local mean=$((sum / RUNS))

    # Median
    IFS=$'\n' sorted=($(sort -n <<<"${times[*]}")); unset IFS
    local median=${sorted[$((RUNS/2))]}

    # Standard deviation
    local sq_sum=0
    for t in "${times[@]}"; do
        local diff=$((t - mean))
        sq_sum=$((sq_sum + diff * diff))
    done
    local variance=$((sq_sum / RUNS))
    local stddev=$(echo "scale=2; sqrt($variance)" | bc)

    log "$name:"
    log "  Min: ${min}ms  Max: ${max}ms"
    log "  Mean: ${mean}ms  Median: ${median}ms"
    log "  StdDev: ${stddev}ms"
    log ""
}

if [ -f "$BMB_EXE" ]; then
    run_timing "$BMB_EXE" "BMB"
fi

if [ -f "$C_EXE" ]; then
    run_timing "$C_EXE" "C"
fi

# =============================================================================
# CPU Profiling
# =============================================================================
if [ "$PROFILE" = true ]; then
    log "${YELLOW}[3/5] CPU profiling...${NC}"

    if command -v perf &> /dev/null; then
        # Linux perf
        if [ -f "$BMB_EXE" ]; then
            log "Profiling BMB..."
            perf record -o "${OUTPUT_DIR}/${BENCHMARK}/bmb_perf.data" "$BMB_EXE" 2>/dev/null || true
            perf report -i "${OUTPUT_DIR}/${BENCHMARK}/bmb_perf.data" --stdio > "${OUTPUT_DIR}/${BENCHMARK}/bmb_perf_report.txt" 2>/dev/null || true
        fi

        if [ -f "$C_EXE" ]; then
            log "Profiling C..."
            perf record -o "${OUTPUT_DIR}/${BENCHMARK}/c_perf.data" "$C_EXE" 2>/dev/null || true
            perf report -i "${OUTPUT_DIR}/${BENCHMARK}/c_perf.data" --stdio > "${OUTPUT_DIR}/${BENCHMARK}/c_perf_report.txt" 2>/dev/null || true
        fi

        log "Profile reports saved to ${OUTPUT_DIR}/${BENCHMARK}/"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        log "Use Instruments on macOS for profiling"
        log "  instruments -t 'Time Profiler' $BMB_EXE"
    else
        log "${YELLOW}perf not available, skipping CPU profiling${NC}"
    fi
    log ""
else
    log "${YELLOW}[3/5] Skipping CPU profiling (use --profile to enable)${NC}"
    log ""
fi

# =============================================================================
# Memory Profiling
# =============================================================================
if [ "$MEMORY" = true ]; then
    log "${YELLOW}[4/5] Memory profiling...${NC}"

    if command -v valgrind &> /dev/null; then
        if [ -f "$BMB_EXE" ]; then
            log "Profiling BMB memory usage..."
            valgrind --tool=massif --massif-out-file="${OUTPUT_DIR}/${BENCHMARK}/bmb_massif.out" "$BMB_EXE" 2>/dev/null || true

            if command -v ms_print &> /dev/null; then
                ms_print "${OUTPUT_DIR}/${BENCHMARK}/bmb_massif.out" > "${OUTPUT_DIR}/${BENCHMARK}/bmb_memory_report.txt"
            fi
        fi

        if [ -f "$C_EXE" ]; then
            log "Profiling C memory usage..."
            valgrind --tool=massif --massif-out-file="${OUTPUT_DIR}/${BENCHMARK}/c_massif.out" "$C_EXE" 2>/dev/null || true
        fi

        log "Memory profiles saved to ${OUTPUT_DIR}/${BENCHMARK}/"
    else
        log "${YELLOW}valgrind not available, skipping memory profiling${NC}"
    fi
    log ""
else
    log "${YELLOW}[4/5] Skipping memory profiling (use --memory to enable)${NC}"
    log ""
fi

# =============================================================================
# IR/ASM Comparison
# =============================================================================
log "${YELLOW}[5/5] Code analysis...${NC}"

if [ "$COMPARE_IR" = true ] && [ -f "$BMB_LL" ] && [ -f "$C_LL" ]; then
    log "Comparing LLVM IR..."

    BMB_IR_LINES=$(wc -l < "$BMB_LL")
    C_IR_LINES=$(wc -l < "$C_LL")

    log "  BMB LLVM IR: $BMB_IR_LINES lines"
    log "  C LLVM IR:   $C_IR_LINES lines"

    # Count key IR patterns
    log ""
    log "  Key patterns in BMB IR:"
    log "    - Function calls: $(grep -c 'call ' "$BMB_LL" 2>/dev/null || echo 0)"
    log "    - Branches: $(grep -c 'br ' "$BMB_LL" 2>/dev/null || echo 0)"
    log "    - Loads: $(grep -c 'load ' "$BMB_LL" 2>/dev/null || echo 0)"
    log "    - Stores: $(grep -c 'store ' "$BMB_LL" 2>/dev/null || echo 0)"

    log ""
    log "  Key patterns in C IR:"
    log "    - Function calls: $(grep -c 'call ' "$C_LL" 2>/dev/null || echo 0)"
    log "    - Branches: $(grep -c 'br ' "$C_LL" 2>/dev/null || echo 0)"
    log "    - Loads: $(grep -c 'load ' "$C_LL" 2>/dev/null || echo 0)"
    log "    - Stores: $(grep -c 'store ' "$C_LL" 2>/dev/null || echo 0)"
fi

if [ "$ASM" = true ]; then
    log ""
    log "Assembly analysis:"

    # Generate BMB assembly
    if [ -f "$BMB_LL" ] && command -v llc &> /dev/null; then
        llc -O3 -o "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_bmb.s" "$BMB_LL" 2>/dev/null || true
    fi

    if [ -f "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_bmb.s" ]; then
        BMB_ASM_LINES=$(wc -l < "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_bmb.s")
        log "  BMB assembly: $BMB_ASM_LINES lines"
    fi

    if [ -f "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_c.s" ]; then
        C_ASM_LINES=$(wc -l < "${OUTPUT_DIR}/${BENCHMARK}/${BENCHMARK}_c.s")
        log "  C assembly:   $C_ASM_LINES lines"
    fi
fi

log ""
log "======================================"
log "Analysis complete"
log "======================================"
log "Results saved to: ${OUTPUT_DIR}/${BENCHMARK}/"
log ""
log "Generated files:"
ls -la "${OUTPUT_DIR}/${BENCHMARK}/" 2>/dev/null | tail -20
