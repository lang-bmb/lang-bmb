# BMB Benchmark Suite

## Overview

BMB includes a comprehensive benchmark suite comparing performance against C (-O3). The suite contains 30 benchmarks covering compute-intensive algorithms and real-world workloads.

## Current Results (v0.60.40)

### Performance Summary

| Category | Benchmarks | BMB Faster | C Faster | Parity |
|----------|-----------|------------|----------|--------|
| Compute | 23 | 15 | 5 | 3 |
| Real-world | 7 | 5 | 1 | 1 |
| **Total** | **30** | **20** | **6** | **4** |

### Highlights

**BMB Significantly Faster (TCO Benefits):**
| Benchmark | Speedup | Reason |
|-----------|---------|--------|
| ackermann | 262x | Deep recursion, TCO |
| nqueen | 7.3x | Backtracking, TCO |
| sorting | 2.7x | Tail-recursive algorithms |

**At or Near Parity:**
- fibonacci, mandelbrot, n_body, hash_table, fannkuch, binary_trees

### Compute Benchmarks

| Benchmark | BMB | C -O3 | Ratio | Notes |
|-----------|-----|-------|-------|-------|
| ackermann | 0.042s | 11.01s | 0.4% | BMB 262x faster |
| nqueen | 0.92s | 6.71s | 14% | BMB 7.3x faster |
| sorting | 0.27s | 0.73s | 37% | BMB 2.7x faster |
| perfect_numbers | 0.60s | 0.99s | 61% | BMB faster |
| tak | 0.02s | 0.03s | 65% | BMB faster |
| fibonacci | 0.09s | 0.13s | 69% | BMB faster |
| fasta | 0.13s | 0.17s | 78% | BMB faster |
| primes_count | 0.03s | 0.04s | 78% | BMB faster |
| spectral_norm | 0.12s | 0.15s | 79% | BMB faster |
| hash_table | 0.09s | 0.10s | 89% | BMB faster |
| sum_of_squares | 0.02s | 0.02s | 89% | BMB faster |
| matrix_multiply | 0.03s | 0.03s | 90% | Near parity |
| digital_root | 0.02s | 0.03s | 92% | Near parity |
| gcd | 0.04s | 0.03s | 103% | Near parity |
| mandelbrot | 0.22s | 0.21s | 104% | Near parity |
| n_body | 0.16s | 0.14s | 108% | Near parity |
| collatz | 0.03s | 0.03s | 112% | Near parity |
| sieve | 0.05s | 0.03s | 185% | C faster |

### Real-world Benchmarks

| Benchmark | Description |
|-----------|-------------|
| sorting | Bubble, insertion, merge, quick sort |
| brainfuck | Brainfuck interpreter |
| csv_parse | CSV parsing |
| http_parse | HTTP request parsing |
| json_parse | JSON parsing |
| json_serialize | JSON serialization |
| lexer | Tokenization benchmark |

## Running Benchmarks

```bash
# Using verification scripts (recommended)
./scripts/benchmark.sh --tier 1       # Run Tier 1 benchmarks
./scripts/benchmark.sh --tier all     # Run all tiers
./scripts/benchmark.sh --list         # List available benchmarks

# Manual build and run
cargo build --release --features llvm --target x86_64-pc-windows-gnu

./target/x86_64-pc-windows-gnu/release/bmb build \
    ecosystem/benchmark-bmb/benches/compute/fibonacci/bmb/main.bmb \
    -o fib.exe
./fib.exe

# Compare with C
gcc -O3 -march=native -o fib_c.exe \
    ecosystem/benchmark-bmb/benches/compute/fibonacci/c/main.c
./fib_c.exe
```

See `docs/BOOTSTRAP_BENCHMARK.md` for CI/CD integration and regression detection.

## Benchmark Categories

### Algorithm Types
- **Recursion**: ackermann, tak, fibonacci, nqueen
- **Iteration**: sieve, collatz, sum_of_squares
- **Memory**: matrix_multiply, hash_table, sorting
- **Floating-point**: spectral_norm, n_body, mandelbrot
- **String/Parsing**: fasta, json_parse, lexer

### Optimization Demonstrations
- **Tail Call Optimization**: ackermann, nqueen, tak show massive speedups
- **Loop Optimization**: fibonacci, collatz benefit from TCO-to-loop
- **Memory Access**: matrix_multiply, sorting test cache efficiency

## Adding New Benchmarks

1. Create directory: `ecosystem/benchmark-bmb/benches/compute/<name>/`
2. Add BMB version: `bmb/main.bmb`
3. Add C version: `c/main.c`
4. Both must produce identical output for verification

## Architecture

```
ecosystem/benchmark-bmb/
├── benches/
│   ├── compute/          # 23 algorithmic benchmarks
│   │   ├── fibonacci/
│   │   │   ├── bmb/main.bmb
│   │   │   └── c/main.c
│   │   └── ...
│   └── real_world/       # 7 practical workload benchmarks
│       ├── sorting/
│       └── ...
└── README.md
```

## Version History

- **v0.60.40**: 30 benchmarks, TCO phi fix, GCD correct
- **v0.60.38**: Default Release optimization
- **v0.60.37**: LoopBoundedNarrowing fix
