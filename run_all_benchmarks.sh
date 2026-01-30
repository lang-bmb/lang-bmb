#!/bin/bash
# Comprehensive Benchmark Suite Runner

cd D:/data/lang-bmb
BMB="./target/x86_64-pc-windows-gnu/release/bmb"

echo "=============================================="
echo "BMB Comprehensive Benchmark Suite"
echo "=============================================="
echo ""

# Arrays to store results
declare -a bench_names
declare -a bmb_times
declare -a c_times
declare -a ratios
declare -a statuses

run_benchmark() {
    local category=$1
    local name=$2
    local bmb_src="ecosystem/benchmark-bmb/benches/$category/$name/bmb/main.bmb"
    local c_src="ecosystem/benchmark-bmb/benches/$category/$name/c/main.c"

    echo "=== $category/$name ==="

    # Check if BMB source exists
    if [ ! -f "$bmb_src" ]; then
        echo "  BMB: MISSING SOURCE"
        return 1
    fi

    # Build BMB
    $BMB build "$bmb_src" -o "temp_bench.exe" 2>&1 | grep -v "^Note:" | grep -v "build_success"
    if [ ! -f "temp_bench.exe" ]; then
        echo "  BMB: BUILD FAILED"
        return 1
    fi

    # Run BMB (3 times, take median)
    local bmb_out=$(./temp_bench.exe 2>&1 | head -5)
    local t1=$( { time ./temp_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
    local t2=$( { time ./temp_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
    local t3=$( { time ./temp_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
    local bmb_time=$(echo "$t1 $t2 $t3" | tr ' ' '\n' | sort -n | sed -n '2p')

    echo "  BMB: ${bmb_time}s  (output: ${bmb_out:0:30}...)"

    # Build and run C if exists
    if [ -f "$c_src" ]; then
        gcc -O3 -march=native -o "temp_c_bench.exe" "$c_src" -lm 2>/dev/null
        if [ -f "temp_c_bench.exe" ]; then
            local c_out=$(./temp_c_bench.exe 2>&1 | head -5)
            local ct1=$( { time ./temp_c_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
            local ct2=$( { time ./temp_c_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
            local ct3=$( { time ./temp_c_bench.exe > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}' | sed 's/0m//' | sed 's/s//')
            local c_time=$(echo "$ct1 $ct2 $ct3" | tr ' ' '\n' | sort -n | sed -n '2p')

            echo "  C:   ${c_time}s  (output: ${c_out:0:30}...)"

            # Check output match
            if [ "$bmb_out" = "$c_out" ]; then
                echo "  Status: OUTPUTS MATCH"
            else
                echo "  Status: OUTPUT MISMATCH!"
                echo "    BMB: $bmb_out"
                echo "    C:   $c_out"
            fi

            rm -f temp_c_bench.exe
        else
            echo "  C: BUILD FAILED"
        fi
    else
        echo "  C: NO SOURCE"
    fi

    rm -f temp_bench.exe temp_bench.o
    echo ""
}

# Run compute benchmarks
echo "=== COMPUTE BENCHMARKS ==="
echo ""
for bench in ackermann binary_trees collatz digital_root fannkuch fasta fibonacci gcd hash_table k-nucleotide mandelbrot matrix_multiply n_body nqueen perfect_numbers pidigits primes_count regex_redux reverse-complement sieve spectral_norm sum_of_squares tak; do
    run_benchmark "compute" "$bench"
done

echo ""
echo "=== REAL-WORLD BENCHMARKS ==="
echo ""
for bench in brainfuck csv_parse http_parse json_parse json_serialize lexer sorting; do
    run_benchmark "real_world" "$bench"
done

echo ""
echo "=============================================="
echo "Benchmark Suite Complete"
echo "=============================================="
