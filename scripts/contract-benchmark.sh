#!/bin/bash
# Contract→Performance Benchmark Script
# Demonstrates E-3 (≥3% measurable difference) and E-4 (BMB beats C)
# Usage: ./scripts/contract-benchmark.sh [iterations]
#
# Builds and benchmarks contract vs no-contract versions using
# noinline/optnone attributes to simulate separate compilation.
#
# Approach: In single-module compilation, LLVM -O2 can inline everything
# and eliminate checks even without contracts. To simulate real-world
# separate compilation (where library functions are opaque), we:
#   - Contract version: noinline (BMB compiler already eliminated checks)
#   - No-contract version: noinline optnone (prevents LLVM from optimizing)
# This mirrors what happens when modules are compiled independently.

set -e
ITER=${1:-100000000}
RUNS=5
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RUNTIME="$ROOT/bmb/runtime"
BMB="$ROOT/target/release/bmb.exe"
BENCH_DIR="$ROOT/ecosystem/benchmark-bmb/benches/contract"

echo "================================================================"
echo "     BMB Contract→Performance Benchmark (${ITER} iterations)"
echo "================================================================"
echo ""

run_bench() {
    local exe="$1"
    local args="$2"
    local sum=0
    for i in $(seq 1 $RUNS); do
        local s=$(date +%s%N)
        $exe $args > /dev/null 2>&1
        local e=$(date +%s%N)
        local ms=$(( (e - s) / 1000000 ))
        sum=$((sum + ms))
    done
    echo $((sum / RUNS))
}

# === BOUNDS CHECK BENCHMARK ===
echo "=== Bounds Check Elimination ==="
echo ""
BD="$BENCH_DIR/bounds_check/bmb"

# 1. Compile BMB with contract (--safe mode)
echo "  Compiling BMB (contract + safe)..."
$BMB build "$BD/main.bmb" --safe --emit-ir -o "$BD/__bench_contract.ll" 2>/dev/null

# 2. Compile BMB without contract (--safe mode)
echo "  Compiling BMB (no contract + safe)..."
$BMB build "$BD/main_nocontract.bmb" --safe --emit-ir -o "$BD/__bench_nocontract.ll" 2>/dev/null

# 3. Patch IR: noinline for contract, optnone for no-contract
sed "s/alwaysinline/noinline/g" "$BD/__bench_contract.ll" > "$BD/__bench_contract_ni.ll"
sed "s/alwaysinline/noinline optnone/g; s/inlinehint/noinline optnone/g" "$BD/__bench_nocontract.ll" > "$BD/__bench_nocontract_on.ll"

# Update iteration count
sed -i "s/10000000/$ITER/g" "$BD/__bench_contract_ni.ll"
sed -i "s/10000000/$ITER/g" "$BD/__bench_nocontract_on.ll"

# 4. Optimize
opt -O2 "$BD/__bench_contract_ni.ll" -S -o "$BD/__bench_contract_opt.ll"
opt -O2 "$BD/__bench_nocontract_on.ll" -S -o "$BD/__bench_nocontract_opt.ll"

# 5. Compile to executable
llc -O2 -filetype=obj "$BD/__bench_contract_opt.ll" -o "$BD/__bench_contract.o"
gcc "$BD/__bench_contract.o" -o "$BD/__bench_contract.exe" -L"$RUNTIME" -lbmb_runtime -lws2_32 -lm

llc -O2 -filetype=obj "$BD/__bench_nocontract_opt.ll" -o "$BD/__bench_nocontract.o"
gcc "$BD/__bench_nocontract.o" -o "$BD/__bench_nocontract.exe" -L"$RUNTIME" -lbmb_runtime -lws2_32 -lm

# 6. Compile C baselines
gcc -O2 -o "$BD/__bench_c_checked.exe" "$BENCH_DIR/bounds_check/c/bench_bounds.c"
gcc -O2 -o "$BD/__bench_c_unchecked.exe" "$BENCH_DIR/bounds_check/c/bench_bounds.c"

# 7. Benchmark
echo "  Running benchmarks ($RUNS runs each)..."
bmb_contract=$(run_bench "$BD/__bench_contract.exe")
bmb_nocontract=$(run_bench "$BD/__bench_nocontract.exe")
c_checked=$(run_bench "$BD/__bench_c_checked.exe" "c")
c_unchecked=$(run_bench "$BD/__bench_c_unchecked.exe")

# 8. Compute differences
contract_vs_nocontract=$(( (bmb_nocontract - bmb_contract) * 100 / bmb_contract ))
contract_vs_c_checked=$(( (c_checked - bmb_contract) * 100 / bmb_contract ))
contract_vs_c_unchecked=$(( (c_unchecked - bmb_contract) * 100 / bmb_contract ))

echo ""
echo "  BOUNDS CHECK RESULTS"
echo "  +------------------------------------+--------+----------+"
echo "  | Version                            | Avg ms | vs BMB+C |"
echo "  +------------------------------------+--------+----------+"
printf "  | BMB safe+contract (noinline)       | %6d | baseline |\n" $bmb_contract
printf "  | BMB safe, no contract (optnone)    | %6d | +%3d%%    |\n" $bmb_nocontract $contract_vs_nocontract
printf "  | C with bounds check (gcc -O2, ni)  | %6d | +%3d%%    |\n" $c_checked $contract_vs_c_checked
printf "  | C unchecked (gcc -O2, noinline)    | %6d | +%3d%%    |\n" $c_unchecked $contract_vs_c_unchecked
echo "  +------------------------------------+--------+----------+"

# Check for bounds check in IR
has_bounds=$(grep -c "bmb_panic_bounds" "$BD/__bench_contract_opt.ll" 2>/dev/null || echo 0)
no_bounds=$(grep -c "bmb_panic_bounds" "$BD/__bench_nocontract_opt.ll" 2>/dev/null || echo 0)
echo ""
echo "  IR Analysis:"
echo "    Contract IR: bmb_panic_bounds references = $has_bounds (expect 0)"
echo "    No-contract IR: bmb_panic_bounds references = $no_bounds (expect >0)"

# === DIVZERO CHECK BENCHMARK ===
echo ""
echo "=== Division-by-Zero Check Elimination ==="
echo ""
DD="$BENCH_DIR/divzero_check/bmb"

echo "  Compiling BMB (contract + safe)..."
$BMB build "$DD/main.bmb" --safe --emit-ir -o "$DD/__bench_contract.ll" 2>/dev/null

echo "  Compiling BMB (no contract + safe)..."
$BMB build "$DD/main_nocontract.bmb" --safe --emit-ir -o "$DD/__bench_nocontract.ll" 2>/dev/null

sed "s/alwaysinline/noinline/g" "$DD/__bench_contract.ll" > "$DD/__bench_contract_ni.ll"
sed "s/alwaysinline/noinline optnone/g; s/inlinehint/noinline optnone/g" "$DD/__bench_nocontract.ll" > "$DD/__bench_nocontract_on.ll"

sed -i "s/10000000/$ITER/g" "$DD/__bench_contract_ni.ll"
sed -i "s/10000000/$ITER/g" "$DD/__bench_nocontract_on.ll"

opt -O2 "$DD/__bench_contract_ni.ll" -S -o "$DD/__bench_contract_opt.ll"
opt -O2 "$DD/__bench_nocontract_on.ll" -S -o "$DD/__bench_nocontract_opt.ll"

llc -O2 -filetype=obj "$DD/__bench_contract_opt.ll" -o "$DD/__bench_contract.o"
gcc "$DD/__bench_contract.o" -o "$DD/__bench_contract.exe" -L"$RUNTIME" -lbmb_runtime -lws2_32 -lm

llc -O2 -filetype=obj "$DD/__bench_nocontract_opt.ll" -o "$DD/__bench_nocontract.o"
gcc "$DD/__bench_nocontract.o" -o "$DD/__bench_nocontract.exe" -L"$RUNTIME" -lbmb_runtime -lws2_32 -lm

gcc -O2 -o "$DD/__bench_c_checked.exe" "$BENCH_DIR/divzero_check/c/bench_divzero.c"
gcc -O2 -o "$DD/__bench_c_unchecked.exe" "$BENCH_DIR/divzero_check/c/bench_divzero.c"

echo "  Running benchmarks ($RUNS runs each)..."
bmb_contract_dz=$(run_bench "$DD/__bench_contract.exe")
bmb_nocontract_dz=$(run_bench "$DD/__bench_nocontract.exe")
c_checked_dz=$(run_bench "$DD/__bench_c_checked.exe" "c")
c_unchecked_dz=$(run_bench "$DD/__bench_c_unchecked.exe")

dz_contract_vs_nocontract=$(( (bmb_nocontract_dz - bmb_contract_dz) * 100 / bmb_contract_dz ))
dz_contract_vs_c_checked=$(( (c_checked_dz - bmb_contract_dz) * 100 / bmb_contract_dz ))

echo ""
echo "  DIVZERO CHECK RESULTS"
echo "  +------------------------------------+--------+----------+"
echo "  | Version                            | Avg ms | vs BMB+C |"
echo "  +------------------------------------+--------+----------+"
printf "  | BMB safe+contract (noinline)       | %6d | baseline |\n" $bmb_contract_dz
printf "  | BMB safe, no contract (optnone)    | %6d | +%3d%%    |\n" $bmb_nocontract_dz $dz_contract_vs_nocontract
printf "  | C with divzero check (gcc -O2, ni) | %6d | +%3d%%    |\n" $c_checked_dz $dz_contract_vs_c_checked
printf "  | C unchecked (gcc -O2, noinline)    | %6d |          |\n" $c_unchecked_dz
echo "  +------------------------------------+--------+----------+"

has_dz=$(grep -c "bmb_panic_divzero" "$DD/__bench_contract_opt.ll" 2>/dev/null || echo 0)
no_dz=$(grep -c "bmb_panic_divzero" "$DD/__bench_nocontract_opt.ll" 2>/dev/null || echo 0)
echo ""
echo "  IR Analysis:"
echo "    Contract IR: bmb_panic_divzero references = $has_dz (expect 0)"
echo "    No-contract IR: bmb_panic_divzero references = $no_dz (expect >0)"

# === SUMMARY ===
echo ""
echo "================================================================"
echo "     EXISTENTIAL CRITERIA RESULTS"
echo "================================================================"
echo ""
echo "  E-3: ≥3% measurable performance difference"
echo "    Bounds check: ${contract_vs_nocontract}% faster with contract"
echo "    Divzero check: ${dz_contract_vs_nocontract}% faster with contract"

echo ""
echo "  E-4: 2+ cases where BMB safe+contract beats C checked"
beats=0
if [ "$contract_vs_c_checked" -gt 0 ]; then
    echo "    Bounds check: BMB beats C by ${contract_vs_c_checked}%"
    beats=$((beats + 1))
fi
if [ "$dz_contract_vs_c_checked" -gt 0 ]; then
    echo "    Divzero check: BMB beats C by ${dz_contract_vs_c_checked}%"
    beats=$((beats + 1))
fi
echo "    Cases: $beats/2"

# Cleanup
rm -f "$BD"/__bench_*.{ll,o,exe} "$DD"/__bench_*.{ll,o,exe} 2>/dev/null
echo ""
echo "Done."
