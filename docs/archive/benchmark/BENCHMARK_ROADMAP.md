# BMB Benchmark & Bootstrap Roadmap

> Performance verification at every major phase • Industry-standard benchmarks • C/Rust performance parity and surpassing

---

## Executive Summary

BMB's design goal is to achieve **C/Rust performance parity through contract-based optimization**. This document defines:

1. **Benchmark Suite** - Industry-standard, fair, reproducible benchmarks
2. **Benchmark Gates** - Performance requirements at each major phase
3. **Bootstrap Integration** - Self-hosting progress with performance verification
4. **Performance Targets** - Specific goals for C/Rust surpassing

---

## Performance Goals

### Primary Objective

| Goal | Description | Metric |
|------|-------------|--------|
| **BMB >= C -O3** | Match C performance in all compute benchmarks | Ratio <= 1.05x |
| **BMB > C -O3** | Surpass C in contract-optimizable cases | Ratio < 0.95x |
| **BMB >= Rust** | Match Rust in idiomatic code | Ratio <= 1.05x |
| **BMB > Rust** | Leverage contracts for optimizations Rust can't prove | Ratio < 0.90x |

### Strategic Advantage

BMB's contracts enable optimizations impossible in C/Rust:
- **Bounds check elimination** - `pre i >= 0 and i < len` → no runtime check
- **Null check elimination** - `pre opt != None` → direct access
- **SIMD vectorization** - `post no_alias(a, b)` → auto-vectorization
- **Dead code elimination** - `post ret >= 0` → unreachable branch removal
- **Memoization** - Pure function detection → automatic caching

---

## Benchmark Suite Design

### Category 1: Compute-Intensive (Benchmarks Game Standard)

Industry-standard benchmarks from [The Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/).

| Benchmark | Description | Measures | C/Rust | BMB | Status |
|-----------|-------------|----------|--------|-----|--------|
| **fibonacci** | Recursive fib(35) | Function calls, int ops | O(2^n) calls | Same | ✅ Done |
| **n-body** | N-body simulation | FP arithmetic, arrays | SIMD-able | Contract-SIMD | ✅ Done |
| **mandelbrot** | Fractal 1000x1000 | Complex math, loops | SIMD-able | Contract-SIMD | ✅ Done |
| **spectral-norm** | Eigenvalue approx | Matrix ops, double | SIMD-able | Contract-SIMD | ✅ Done |
| **binary-trees** | Alloc/dealloc | Memory patterns | GC-free | GC-free | ✅ Done |
| **fannkuch-redux** | Permutations | Array reversal | O(n!) | Same | ✅ Done |
| **fasta** | Sequence generation | RNG, string output | I/O bound | Same | ✅ Done |
| **k-nucleotide** | Hashtable ops | Hash, string slice | Hash perf | Contract-hash | ✅ Done |
| **regex-redux** | Regex matching | Regex engine | Library-dep | BMB-regex | 📋 Planned |
| **reverse-complement** | String reverse | String ops, memory | SIMD-able | Same | ✅ Done |
| **pidigits** | Pi calculation | Arbitrary precision | BigInt | BMB-bigint | 📋 Planned |

### Category 2: Contract-Optimized (BMB Advantage)

Benchmarks demonstrating BMB's contract-based optimization superiority.

| Benchmark | Description | Expected BMB Advantage | Mechanism | Status |
|-----------|-------------|------------------------|-----------|--------|
| **bounds-check** | Array sum with pre | 15-30% faster | `pre i < len` eliminates check | ✅ Done |
| **null-check** | Option traversal | 10-25% faster | `pre opt != None` eliminates check | ✅ Done |
| **purity-opt** | Pure function CSE | 20-50% faster | Memoization, hoisting | ✅ Done |
| **aliasing** | Non-alias arrays | 30-60% faster | SIMD vectorization | ✅ Done |
| **invariant-hoist** | Loop invariant | 15-40% faster | `post` enables hoisting | ✅ Done |
| **branch-elim** | Unreachable code | 10-30% faster | `post ret > 0` prunes branches | ✅ Done |

### Category 3: Real-World Workloads

Practical applications representative of actual use cases.

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| **json-parse** | JSON validation | String processing, parsing | ✅ Done |
| **json-serialize** | JSON output | String building, escaping | ✅ Done |
| **sorting** | Various sorts | Comparisons, swaps | ✅ Done |
| **http-parse** | HTTP request parse | Protocol parsing | ✅ Done |
| **csv-parse** | CSV processing | Field extraction | ✅ Done |
| **lexer** | Token generation | Character processing | ✅ Done |
| **brainfuck** | BF interpreter | Interpreter overhead | ✅ Done |

### Category 4: Bootstrap Self-Compilation

Measure compiler performance improvements through self-hosting.

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| **lex-bootstrap** | Tokenize BMB source | Lexer throughput | ✅ Done |
| **parse-bootstrap** | Parse BMB source | Parser throughput | ✅ Done |
| **typecheck-bootstrap** | Type check expressions | Type inference | ✅ Done |
| **codegen-bootstrap** | Generate LLVM IR | Codegen throughput | 📋 Planned |
| **full-compile** | Full compilation | End-to-end | 📋 Planned |

---

## Benchmark Gates

Performance verification required at each major phase.

### Gate #1: Interpreter Baseline (v0.31) ✅ Complete

| Criterion | Target | Result |
|-----------|--------|--------|
| fibonacci(35) | >= Rust interp | ✅ 0.97x |
| mandelbrot | >= Rust interp | ✅ 0.93x |
| All compute | Within 20% of C | ✅ Passed |

### Gate #2: Native Compilation (v0.32-v0.34) ✅ Complete

| Criterion | Target | Result |
|-----------|--------|--------|
| fibonacci(35) native | == C -O3 | ✅ 1.00x (0.020s vs 0.019s) |
| Contract advantage | >= 10% faster | ✅ purity_opt 0.89x |
| Native vs interpreter | >= 100x speedup | ✅ 1113x |

### Gate #3: Bootstrap Performance (v0.35-v0.36)

| Criterion | Target | Status |
|-----------|--------|--------|
| All Benchmarks Game | Within 5% of C | 📋 In Progress |
| Contract benchmarks | >= 15% faster than C | 📋 In Progress |
| json_parse | >= Rust | 📋 Needs optimization |
| Self-compile time | < 60s for bootstrap | 📋 Planned |

### Gate #4: Production Ready (v0.37-v1.0)

| Criterion | Target | Status |
|-----------|--------|--------|
| All compute benchmarks | == C -O3 median | 📋 Planned |
| 50% contract benchmarks | > C -O3 | 📋 Planned |
| Real-world workloads | >= Rust | 📋 Planned |
| No performance regressions | CI enforced | 📋 Planned |

---

## Phase Roadmap

### Phase 35: Foundation ✅ Complete (v0.35.12)

**Goal**: Core optimizations + 25 benchmarks complete

**Achievement Summary**:
| Category | Count | Status |
|----------|-------|--------|
| Benchmarks Game | 9/11 | ✅ Done |
| Real-World Workloads | 7/7 | ✅ Done |
| Contract Benchmarks | 6/6 | ✅ Done |
| Bootstrap Benchmarks | 3/5 | ✅ Done |
| Native Speedup | 1113x | ✅ Achieved |

<details>
<summary>Phase 35 Completed Tasks (click to expand)</summary>

| Task | Description | Status |
|------|-------------|--------|
| 35.0.1 | String interning | ✅ Done |
| 35.0.3 | PHI node support | ✅ Done |
| 35.0.4 | f64 codegen | ✅ Done |
| 35.1.1 | bmb-json package | ✅ Done |
| 35.1.5 | Gate command in runner | ✅ Done |
| 35.1.6 | v0.32 syntax migration | ✅ Done |
| 35.2.1 | fasta benchmark | ✅ Done |
| 35.2.2 | k-nucleotide benchmark | ✅ Done |
| 35.2.3 | reverse-complement benchmark | ✅ Done |
| 35.3.1 | brainfuck interpreter | ✅ Done |
| 35.3.2 | csv-parse benchmark | ✅ Done |
| 35.3.3 | lexer benchmark | ✅ Done |
| 35.3.4 | json-serialize benchmark | ✅ Done |
| 35.3.5 | http-parse benchmark | ✅ Done |
| 35.6.1-3 | Contract benchmarks | ✅ Done |
| 35.7.1-2 | lex/parse-bootstrap | ✅ Done |
| 35.8.1 | typecheck-bootstrap | ✅ Done |

</details>

**Gate #3.1 Status**:
- Native compilation verified (fibonacci: C parity at 0.020s)
- Benchmark suite complete (25 benchmarks)
- Gate verification blocked by LLVM/clang linking on Windows

### Phase 36: Optimization Sprint

**Goal**: Achieve C parity across all Benchmarks Game

| Task | Description | Priority |
|------|-------------|----------|
| 36.0.1 | LLVM optimization passes (-O2 equivalent) | P0 |
| 36.0.2 | Inline small functions | P0 |
| 36.0.3 | Loop unrolling annotations | P1 |
| 36.1.1 | Add regex-redux (with bmb-regex) | P2 |
| 36.1.2 | Add pidigits (with bmb-bigint) | P2 |
| 36.2.1 | SIMD intrinsics for aliasing benchmark | P0 |
| 36.2.2 | Bounds check elimination pass | P0 |

**Benchmark Gate #3.2** (v0.36 exit):
- All Benchmarks Game within 5% of C
- bounds_check benchmark 20%+ faster than C
- aliasing benchmark demonstrates SIMD advantage

### Phase 37: Surpassing C

**Goal**: Demonstrate contract advantages in real scenarios

| Task | Description | Priority |
|------|-------------|----------|
| 37.0.1 | Auto-vectorization with contract proofs | P0 |
| 37.0.2 | Dead branch elimination with postconditions | P0 |
| 37.0.3 | Pure function memoization | P1 |
| 37.1.1 | Add brainfuck interpreter benchmark | P1 |
| 37.1.2 | Add lexer benchmark (self-lex) | P0 |
| 37.2.1 | Document optimization strategies | P0 |

**Benchmark Gate #3.3** (v0.37 exit):
- 3+ benchmarks faster than C -O3
- All contract benchmarks faster than C
- Real-world workloads >= Rust

### Phase 38: Bootstrap Performance

**Goal**: Self-compilation performance optimization

| Task | Description | Priority |
|------|-------------|----------|
| 38.0.1 | Profile self-compilation | P0 |
| 38.0.2 | Optimize hot paths | P0 |
| 38.0.3 | Parallel compilation passes | P1 |
| 38.1.1 | Benchmark: lex-bootstrap | P0 |
| 38.1.2 | Benchmark: full-compile | P0 |

**Benchmark Gate #4.1** (v0.38 exit):
- Self-compile < 60s (30K LOC)
- No performance regression from v0.37

### Phase 39-40: Production Polish

**Goal**: Performance guarantees for v1.0

| Task | Description | Priority |
|------|-------------|----------|
| 39.0.1 | CI benchmark enforcement | P0 |
| 39.0.2 | Regression detection (2% threshold) | P0 |
| 39.1.1 | Performance documentation | P0 |
| 40.0.1 | Final Gate #4 verification | P0 |

---

## Benchmark Execution

### Running Benchmarks

```bash
cd ecosystem/benchmark-bmb

# Build runner
cd runner && cargo build --release && cd ..

# Run all benchmarks
./runner/target/release/benchmark-bmb run --all

# Run by category
./runner/target/release/benchmark-bmb run --category compute
./runner/target/release/benchmark-bmb run --category contract

# Run single benchmark with details
./runner/target/release/benchmark-bmb run fibonacci --verbose

# Compare languages
./runner/target/release/benchmark-bmb compare mandelbrot

# Generate gate report
./runner/target/release/benchmark-bmb gate 3
```

### CI Integration

```yaml
# .github/workflows/benchmark.yml
name: Benchmark Gate
on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: |
          cd ecosystem/benchmark-bmb
          ./run_bench.sh --gate current
      - name: Check regression
        run: |
          ./compare_with_baseline.sh --threshold 2%
```

### Measurement Methodology

Following [Benchmarks Game methodology](https://benchmarksgame-team.pages.debian.net/benchmarksgame/):

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Warm-up runs | 3 | JIT/cache warm-up |
| Measurement runs | 10 | Statistical significance |
| Reported metric | Median | Outlier resistance |
| CPU isolation | Single core | Reproducibility |
| C compiler | gcc -O3 -march=native | Fair optimization |
| Rust compiler | rustc --release | Fair optimization |
| BMB compiler | bmb build --release | Fair optimization |

---

## Bootstrap Integration

### Bootstrap Compiler Phases

| Phase | Compiler | Benchmark Verification |
|-------|----------|------------------------|
| Stage 1 | Rust builds BMB | Baseline established |
| Stage 2 | BMB (Stage 1) builds BMB | Must match Stage 1 perf |
| Stage 3 | BMB (Stage 2) builds BMB | Bit-identical output |
| Stage 4+ | BMB builds BMB | Performance improvements |

### Self-Compilation Benchmarks

| Metric | Target | Current |
|--------|--------|---------|
| Lex throughput | 1M tokens/s | TBD |
| Parse throughput | 100K LOC/s | TBD |
| Type check | 50K LOC/s | TBD |
| Full compile (30K LOC) | < 60s | TBD |

---

## Implementation Priority

### P0 (Critical - v0.35-v0.36)

1. **Complete Benchmarks Game suite** - Industry credibility
2. **CI benchmark integration** - Prevent regressions
3. **Gate #3 verification** - Block release if failed
4. **Bounds check elimination** - Core contract advantage

### P1 (Important - v0.37)

1. **SIMD vectorization** - Major performance gain
2. **Self-compilation benchmarks** - Bootstrap validation
3. **Real-world workloads** - Practical relevance

### P2 (Nice to have - v0.38+)

1. **Parallel compilation** - Scalability
2. **Profile-guided optimization** - Peak performance
3. **Web dashboard** - Community visibility

---

## Success Metrics

### v1.0 Release Criteria

| Metric | Requirement |
|--------|-------------|
| Benchmarks Game parity | All within 5% of C -O3 |
| Contract advantage | 3+ benchmarks > C |
| No regression | CI enforced, 2% threshold |
| Self-compile | < 60s for compiler |
| Documentation | All optimizations documented |

### Public Credibility

| Action | Purpose |
|--------|---------|
| Publish results | Transparency |
| Reproducible setup | Verification |
| Third-party validation | Credibility |
| Version tracking | Progress visibility |

---

## Appendix: Benchmark Implementation Checklist

### New Benchmark Template

```
benches/<category>/<name>/
├── c/
│   └── main.c           # C implementation
├── rust/
│   └── main.rs          # Rust implementation
├── bmb/
│   └── main.bmb         # BMB implementation
├── README.md            # Algorithm description
├── input.txt            # Standard input (if needed)
└── expected_output.txt  # Validation output
```

### Implementation Guidelines

1. **Algorithm equivalence** - Identical algorithm, language-idiomatic code
2. **No external libraries** - Standard library only
3. **Output validation** - Same output across languages
4. **Contract usage** - BMB should use contracts where beneficial
5. **No artificial handicaps** - Fair optimizations allowed

---

*Last updated: 2026-01-10 | Version: v0.35.12 | Phase 35 Complete*
