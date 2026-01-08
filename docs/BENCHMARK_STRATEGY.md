# BMB Comprehensive Benchmark Strategy

> Phase 33.5: C/Rust/BMB 비교 벤치마크 전략 및 기능 갭 분석

## Executive Summary

리서치 기반 표준 벤치마크 수트 구성:
- [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) 표준 알고리즘
- [LLVM Test Suite](https://llvm.org/docs/TestSuiteGuide.html) 컴파일러 벤치마크
- [kostya/benchmarks](https://github.com/kostya/benchmarks) 다중 언어 비교
- Contract-based optimization 벤치마크 (BMB 고유)

---

## 1. Benchmark Categories

### Category A: Compute-Intensive (Benchmarks Game Standard)

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| fibonacci | Recursive function | Integer ops, function calls | ✅ Implemented |
| mandelbrot | Fractal generation | Fixed-point math, iteration | ✅ Implemented |
| n_body | Physics simulation | **F64 required** | ⚠️ Placeholder |
| spectral_norm | Eigenvalue approximation | Matrix-vector multiply | ✅ Implemented |
| binary_trees | Tree allocate/deallocate | Memory patterns, recursion | ✅ Implemented |
| fannkuch | Pancake flipping | Permutation, array reversal | ✅ Implemented |
| pidigits | Pi digit extraction | Arbitrary precision | ⚠️ Requires BigInt |

### Category B: String & Parsing

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| json_parse | JSON validation | String processing | ✅ Implemented |
| regex_match | Pattern matching | State machine | ❌ Need regex lib |
| csv_parse | CSV field extraction | Delimiter parsing | 📋 Planned |
| lexer_bench | Token generation | Character classification | 📋 Can use lexer.bmb |

### Category C: Memory & Data Structures

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| sorting | Various sort algorithms | Comparisons, swaps | ✅ Implemented |
| hash_table | Insert/lookup ops | Hashing, collision | ❌ Need HashMap |
| binary_search | Sorted array search | Comparisons | 📋 Easy to add |
| linked_list | Node traversal | Pointer chasing | ⚠️ Need heap alloc |

### Category D: Contract-Optimized (BMB Advantage)

| Benchmark | Description | Expected Gain | Status |
|-----------|-------------|---------------|--------|
| bounds_check | Array access with pre | 10-30% | ✅ Implemented |
| null_check | Option handling | 15-25% | ✅ Implemented |
| purity_opt | Pure function CSE | 20-50% | ✅ Implemented |
| aliasing | Non-aliasing arrays | 30-50% (SIMD) | ✅ Implemented |
| invariant_loop | Loop invariant hoisting | 15-40% | 📋 Planned |
| divide_elim | Division by constant | 10-20% | 📋 Planned |

### Category E: Real-World Workloads

| Benchmark | Description | Measures | Status |
|-----------|-------------|----------|--------|
| compiler_pass | Parser/type checker | Mixed workload | ✅ Can use bootstrap |
| file_process | File read/transform | I/O + computation | 📋 Planned |
| http_parse | HTTP header parsing | String + struct | 📋 Planned |

---

## 2. BMB Feature Gap Analysis

### Critical Gaps (Blocking Benchmarks)

| Feature | Impact | Benchmarks Blocked | Priority |
|---------|--------|-------------------|----------|
| **f64 (float)** | n_body, scientific | n_body, spectral_norm_fp | P0 |
| **sqrt/math lib** | Numerical algorithms | n_body, mandelbrot_fp | P0 |
| **HashMap** | Data structure bench | hash_table, lru_cache | P1 |
| **Dynamic alloc** | Real heap management | binary_trees_real, linked_list | P1 |

### Important Gaps (Limiting Accuracy)

| Feature | Impact | Workaround | Priority |
|---------|--------|------------|----------|
| while loops | All iterative code | Tail recursion | P1 - Partly done |
| Mutable arrays | In-place algorithms | Copy semantics | P2 |
| Pointer arithmetic | Low-level ops | Index-based | P2 |
| SIMD intrinsics | Vector ops | LLVM auto-vectorize | P3 |

### Verification Gaps

| Feature | Impact | Status |
|---------|--------|--------|
| Contract verification output | Debug contract benefits | 📋 Planned |
| Bounds check elimination proof | Prove optimization | ⚠️ Needs SMT output |
| Dead code elimination from contracts | Measurable benefit | 📋 Planned |

---

## 3. Benchmark Execution Plan

### Phase 1: Current Capability Assessment (Week 1)

**Goal**: Run all currently-working benchmarks with C/Rust/BMB

| Day | Task |
|-----|------|
| 1 | Set up build environment (GCC, Rust, BMB) |
| 2 | Run fibonacci benchmark on all 3 languages |
| 3 | Run mandelbrot benchmark (fixed-point) |
| 4 | Run binary_trees, fannkuch benchmarks |
| 5 | Run json_parse benchmark |
| 6 | Compile results, identify failures |
| 7 | Document gaps and blockers |

**Deliverable**: `results/phase1_current_capability.md`

### Phase 2: Contract Optimization Verification (Week 2)

**Goal**: Measure BMB contract-based advantages

| Task | Description |
|------|-------------|
| bounds_check_bench | Measure array access with/without contracts |
| Compare generated IR | Verify bounds checks eliminated in BMB |
| null_check_bench | Measure Option handling |
| purity_bench | Measure CSE/hoisting opportunities |

**Deliverable**: `results/phase2_contract_benefits.md`

### Phase 3: Feature Implementation (Week 3-4)

**Goal**: Add missing features to enable more benchmarks

| Feature | Effort | Benchmarks Enabled |
|---------|--------|-------------------|
| f64 type | 3-5 days | n_body, spectral_norm_fp |
| sqrt builtin | 1 day | n_body |
| while statement | 2-3 days | All iterative benchmarks |

### Phase 4: Extended Benchmark Suite (Week 5-6)

**Goal**: Add more comprehensive benchmarks

| Category | New Benchmarks |
|----------|----------------|
| Parsing | csv_parse, ini_parse |
| Algorithms | quicksort, mergesort, heapsort |
| Compiler | lexer_bench (using lexer.bmb) |
| Strings | string_concat, string_search |

---

## 4. Performance Targets

### Tier 1: Must Meet (Release Blocker)

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| fibonacci | BMB <= Rust | ✅ Already achieved |
| mandelbrot | BMB <= Rust | Integer compute |
| binary_trees | BMB <= Rust | Recursion heavy |
| json_parse | BMB <= Rust | String processing |

### Tier 2: Should Meet (Quality Goal)

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| bounds_check | BMB < C by 10%+ | Contract advantage |
| purity_opt | BMB < C by 20%+ | Pure function CSE |
| n_body | BMB <= Rust | Numerical compute |

### Tier 3: Nice to Have (Stretch Goal)

| Benchmark | Target | Rationale |
|-----------|--------|-----------|
| All Benchmarks Game | BMB <= C -O3 | Full parity |
| Contract benchmarks | BMB < C by 20%+ | Contract advantages proven |

---

## 5. Measurement Methodology

### Environment

```yaml
Hardware:
  CPU: Document specific model
  RAM: Document size
  OS: Windows 11 / Linux

Compilers:
  C: GCC -O3 / Clang -O3
  Rust: rustc --release (default LLVM opts)
  BMB: bmb build --release (LLVM -O3)
```

### Measurement Protocol

1. **Warm-up**: 2 iterations before measurement
2. **Iterations**: 10 runs, report median
3. **Isolation**: Single-threaded, no background processes
4. **Validation**: Verify output correctness before timing
5. **Variation**: Report min/max/stddev

### Metrics

| Metric | Description |
|--------|-------------|
| Wall time | Total execution time (primary) |
| CPU time | User + system time |
| Peak memory | Maximum RSS |
| Binary size | Compiled executable size |
| Compile time | Time to build |

---

## 6. Reporting Format

### Individual Benchmark Report

```markdown
## fibonacci(35)

| Language | Time (ms) | Memory (KB) | Binary (KB) |
|----------|-----------|-------------|-------------|
| C -O3    | 55        | 120         | 16          |
| Rust     | 57        | 180         | 320         |
| BMB      | 54        | 150         | 24          |

**BMB/C Ratio**: 0.98x (2% faster)
**BMB/Rust Ratio**: 0.95x (5% faster)

**Analysis**: BMB matches C performance due to equivalent LLVM backend
optimization. Contracts provide zero overhead in this pure compute case.
```

### Summary Dashboard

```
=== BMB Benchmark Suite v0.4 ===
Date: 2026-01-XX
Version: BMB v0.33.X

Category: Compute-Intensive
─────────────────────────────────────────────
Benchmark       C (ms)   Rust    BMB     Status
─────────────────────────────────────────────
fibonacci         55      57      54     ✅ BMB fastest
mandelbrot       123     125     124     ✅ Parity
binary_trees     450     455     448     ✅ BMB fastest
─────────────────────────────────────────────

Category: Contract-Optimized
─────────────────────────────────────────────
Benchmark       C (ms)   Rust    BMB     Advantage
─────────────────────────────────────────────
bounds_check    100      98      75      25% faster
null_check      200     190     160      20% faster
purity_opt      300     310     180      40% faster
─────────────────────────────────────────────
```

---

## 7. Success Criteria

### Gate 1: Compute Parity ✅

- [x] fibonacci: BMB <= Rust
- [ ] mandelbrot: BMB <= Rust
- [ ] binary_trees: BMB <= Rust
- [ ] 3/3 compute benchmarks pass

### Gate 2: Contract Advantage

- [ ] bounds_check: BMB < C by 10%+
- [ ] purity_opt: BMB < C by 15%+
- [ ] 2/2 contract benchmarks show advantage

### Gate 3: Full Benchmarks Game

- [ ] All 6 compute benchmarks implemented
- [ ] All pass BMB <= Rust threshold
- [ ] f64 support added for n_body

### Gate 4: Real-World Parity

- [ ] json_parse: BMB <= Rust
- [ ] compiler_pass: BMB <= Rust
- [ ] 2/2 real-world benchmarks pass

---

## 8. Next Steps

1. **Immediate**: Run Phase 1 benchmarks with current BMB
2. **Week 1**: Document current performance baseline
3. **Week 2**: Measure contract optimization benefits
4. **Week 3-4**: Implement f64, while loops
5. **Week 5-6**: Extended benchmark suite
6. **Ongoing**: Continuous regression testing

---

## References

- [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)
- [LLVM Test Suite Guide](https://llvm.org/docs/TestSuiteGuide.html)
- [kostya/benchmarks](https://github.com/kostya/benchmarks)
- [Programming Language Benchmarks](https://programming-language-benchmarks.vercel.app/)
- [Academic: Rust Runtime Performance](https://dl.acm.org/doi/fullHtml/10.1145/3551349.3559494)
