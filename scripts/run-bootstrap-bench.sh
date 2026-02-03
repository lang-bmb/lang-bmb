#!/bin/bash
# Safe bootstrap benchmark runner - sequential execution

PROJECT_ROOT="D:/data/lang-bmb"
RESULTS_DIR="$PROJECT_ROOT/target/benchmarks-bootstrap"
RUNTIME="$PROJECT_ROOT/bmb/runtime/bmb_runtime.o"
STAGE2="$PROJECT_ROOT/target/bootstrap/bmb-stage2.exe"
RUST_BMB="$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/bmb.exe"

mkdir -p "$RESULTS_DIR"

# Benchmarks that bootstrap compiler can compile (no while/mut)
BENCHMARKS="gcd collatz ackermann sum_of_squares primes_count"

echo "======================================"
echo "Bootstrap Compiler Benchmark"
echo "======================================"
echo ""

time_run() {
    local start=$(date +%s%3N)
    "$@" > /dev/null 2>&1
    local end=$(date +%s%3N)
    echo $((end - start))
}

for bench in $BENCHMARKS; do
    src="$PROJECT_ROOT/ecosystem/benchmark-bmb/benches/compute/$bench/bmb/main.bmb"
    c_src="$PROJECT_ROOT/ecosystem/benchmark-bmb/benches/compute/$bench/c/main.c"
    
    [ -f "$src" ] || continue
    
    echo "--- $bench ---"
    
    # Build with Bootstrap
    boot_out="$RESULTS_DIR/${bench}_boot"
    boot_time="-"
    if $STAGE2 "$src" "${boot_out}.ll" 2>/dev/null; then
        if opt -O3 -o "${boot_out}.bc" "${boot_out}.ll" 2>/dev/null; then
            if llc -O3 -filetype=obj -o "${boot_out}.o" "${boot_out}.bc" 2>/dev/null; then
                if gcc -o "${boot_out}.exe" "${boot_out}.o" "$RUNTIME" -lm 2>/dev/null; then
                    boot_time=$(time_run "${boot_out}.exe")
                fi
            fi
        fi
    fi
    echo "  Bootstrap: ${boot_time}ms"
    
    # Build with Rust BMB
    rust_out="$RESULTS_DIR/${bench}_rust"
    rust_time="-"
    if $RUST_BMB build "$src" -o "$rust_out" 2>/dev/null; then
        exe="${rust_out}.exe"
        [ -f "$exe" ] && rust_time=$(time_run "$exe")
    fi
    echo "  Rust BMB:  ${rust_time}ms"
    
    # Build C baseline
    c_out="$RESULTS_DIR/${bench}_c"
    c_time="-"
    if [ -f "$c_src" ]; then
        if clang -O3 -march=native -o "${c_out}.exe" "$c_src" -lm 2>/dev/null; then
            c_time=$(time_run "${c_out}.exe")
        fi
    fi
    echo "  C (clang): ${c_time}ms"
    echo ""
done

echo "======================================"
echo "Done"
echo "======================================"
