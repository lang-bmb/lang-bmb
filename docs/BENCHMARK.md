# BMB Benchmark Suite

## Overview

BMB includes a benchmark suite comparing performance against C (Clang -O3, GCC -O3) and Rust. The suite contains 67 benchmarks across 11 categories.

**Key Principle**: All performance claims are measured and verifiable. Where BMB is slower, we investigate and fix. Where the gap is LLVM-inherent, we document it.

---

## Methodology

### How Benchmarks Are Run

```bash
# BMB compilation
bmb build bench.bmb -o bench_bmb    # LLVM IR → opt -O2 → llc -O3 → native

# C compilation (Clang)
clang -O3 -march=native bench.c -o bench_c -lm

# C compilation (GCC)
gcc -O3 -march=native bench.c -o bench_c -lm

# Timing
hyperfine --warmup 2 --runs 5 './bench_bmb' './bench_c'
```

### Fairness Rules

1. **Same algorithm**: BMB and C implementations must use the same algorithm
2. **Same output**: Both must produce identical output (verified automatically)
3. **Same optimization level**: Both use -O3 (BMB via `opt -O2` + `llc -O3`)
4. **Same hardware**: All benchmarks run on the same machine in the same session
5. **Same LLVM backend**: BMB and Clang both use LLVM — fair for IR comparison

### What This Benchmark Measures

- **IR generation quality**: Does BMB generate optimal LLVM IR?
- **Compiler optimization effectiveness**: Do BMB's MIR passes help LLVM?
- **Contract overhead**: Do contracts add any runtime cost? (Answer: no)

### What This Benchmark Does NOT Measure

- **I/O performance**: No file, network, or database benchmarks
- **Memory allocation patterns**: No large-scale allocation stress tests
- **Concurrency**: No multi-threaded benchmarks
- **Real-world applications**: No end-to-end application benchmarks
- **Compile time**: Compilation speed is not measured here

These gaps are acknowledged and planned for future work (see Roadmap v0.95.4).

### Statistical Note

Current results use median of 5 runs with 2 warmup iterations. Standard deviation and confidence intervals are planned but not yet implemented. For benchmarks under 50ms, measurement noise is significant — these results should be interpreted with caution.

---

## Tier 1: Fair Comparison (>50ms runtime)

These benchmarks run long enough for reliable measurement and use **no BMB-specific optimizations** that the C compiler cannot also perform.

**Results (v0.90.22, vs Clang -O3):**

| Benchmark | BMB/Clang | Verdict | Notes |
|-----------|-----------|---------|-------|
| fasta | 0.94x | **FASTER** | BMB generates better IR for string operations |
| gcd | 0.97x | **FASTER** | |
| binary_trees | 0.99x | **FASTER** | |
| spectral_norm | 1.00x | PASS | Identical IR verified |
| mandelbrot | 1.01x | PASS | Identical IR verified |
| nqueen | 1.01x | PASS | |
| collatz | 1.05x | OK | Loop bound narrowing residual |
| primes_count | 1.05x | OK | |
| ackermann | 1.06x | OK | Recursive call narrowing residual |
| sieve | 1.07x | WARN | Mutable var pattern (not phi-optimizable) |
| digital_root | 1.07x | WARN | Sub-50ms, high variance — unreliable |

**Summary**: 3 FASTER, 3 PASS (≤1.02x), 3 OK (≤1.06x), 2 WARN (≤1.10x)

**Verdict criteria**:
- FASTER: BMB < 1.00x (BMB is faster)
- PASS: ≤ 1.02x (within measurement noise)
- OK: ≤ 1.06x (small gap, acceptable)
- WARN: ≤ 1.10x (gap exists, under investigation)
- FAIL: > 1.10x (unacceptable, must fix)

---

## Tier 2: Compiler Optimization Showcase

> **⚠️ WARNING**: These benchmarks demonstrate BMB-specific compiler optimizations (TCO, linear recurrence detection). They are NOT fair language comparisons. GCC/Clang can achieve similar results with appropriate flags or source transformation.

These results show that BMB's MIR optimization passes can detect and transform patterns that the C source doesn't express:

| Benchmark | BMB | GCC -O3 | Speedup | Optimization | Fair? |
|-----------|-----|---------|---------|-------------|-------|
| ackermann | 55ms | 10,968ms | 200x | Tail Call Optimization | ⚠️ GCC -foptimize-sibling-calls achieves similar |
| fibonacci | 29ms | 10,499ms | 362x | Linear Recurrence → closed form | ⚠️ C can use iterative version |
| tak | 20ms | 30ms | 1.5x | TCO | ⚠️ GCC TCO flag achieves parity |

**Why these exist**: BMB's MIR passes can optimize recursive patterns that would require manual transformation in C. This is a real capability, but it should not be confused with BMB being "362x faster than C" in general.

**Fair interpretation**: BMB's compiler is smarter about recursive patterns. For iterative code (which most production C uses), BMB and C generate equivalent IR.

---

## Tier 0: Sub-50ms Benchmarks

These benchmarks run too fast for reliable timing. Results are included for completeness but should be interpreted with caution.

| Benchmark | BMB | C -O3 | Notes |
|-----------|-----|-------|-------|
| sum_of_squares | ~20ms | ~20ms | Measurement noise dominant |
| matrix_multiply | ~30ms | ~30ms | Near parity |
| digital_root | ~20ms | ~30ms | High variance |

---

## Full Benchmark Suite (67 benchmarks)

All 67 benchmarks compile and produce correct output. The suite covers:

| Category | Count | Description |
|----------|-------|-------------|
| Recursion | 8 | ackermann, tak, fibonacci, nqueen, ... |
| Iteration | 7 | sieve, collatz, primes_count, ... |
| Memory | 6 | matrix_multiply, hash_table, sorting, ... |
| Floating-point | 5 | spectral_norm, n_body, mandelbrot, ... |
| String/Parsing | 7 | fasta, json_parse, lexer, csv_parse, ... |
| Numeric | 6 | gcd, digital_root, perfect_numbers, ... |
| Other | 28 | Various compute patterns |

---

## Known Gaps and Investigation Status

| Benchmark | Gap | Root Cause | Status |
|-----------|-----|-----------|--------|
| sieve | 1.07x | While-loop bound not phi-optimizable | Investigating |
| digital_root | 1.07x | Sub-50ms, measurement noise | Low priority (unreliable data) |

### Resolved Gaps (History)

| Cycle | Fix | Effect |
|-------|-----|--------|
| 172 | div/mod param narrowing skip | digital_root 1.26x→1.07x |
| 174 | Loop-invariant bound skip | collatz 1.07x→1.05x, sieve 1.07x→1.04x |
| 175 | Self-recursive call narrowing skip | ackermann 1.08x→1.06x |
| 528 | Identity copy elimination (16K copies) | 22.2% IR reduction |
| 529-532 | Zext/trunc/inttoptr/ptrtoint elimination | 37.5% total IR reduction |

---

## IR Verification

Near-parity benchmarks are verified by comparing BMB and Clang LLVM IR:

| Benchmark | IR Equivalent? | Notes |
|-----------|---------------|-------|
| mandelbrot | ✅ | TCO → loop conversion, identical iterate() |
| spectral_norm | ✅ | Identical hot loop |
| sieve | ⚠️ | Vectorization threshold difference (LLVM decision) |
| n_body | ⚠️ | sqrt inlining choice (LLVM decision) |

When BMB and Clang generate equivalent IR, performance is identical. Remaining gaps are LLVM backend decisions (vectorization thresholds, unrolling factors), not BMB IR quality.

---

## Running Benchmarks

```bash
# Using benchmark script
./scripts/benchmark.sh --tier 1       # Tier 1 only (recommended)
./scripts/benchmark.sh --tier all     # All tiers
./scripts/benchmark.sh --list         # List available benchmarks

# Manual (single benchmark)
bmb build bench.bmb -o bench_bmb
clang -O3 -march=native bench.c -o bench_c -lm
hyperfine --warmup 2 --runs 5 './bench_bmb' './bench_c'
```

## Adding New Benchmarks

1. Create directory: `ecosystem/benchmark-bmb/benches/compute/<name>/`
2. Add BMB version: `bmb/main.bmb`
3. Add C version: `c/main.c`
4. Both must produce identical output for verification
5. Classify into Tier 0/1/2 based on runtime and optimization usage

---

## Future Work

- [ ] Standard deviation and 95% confidence intervals for all results
- [ ] GCC with TCO flags (-foptimize-sibling-calls) comparison in Tier 2
- [ ] Real-world application benchmarks (JSON parser, HTTP server)
- [ ] Memory allocation stress tests
- [ ] Concurrency benchmarks
- [ ] Compilation speed comparison
