# BMB Benchmark Analysis (2026-01-24, Post v0.51.18 Fixes)

## Summary

| Category | Count | Description |
|----------|-------|-------------|
| **BMB Faster** | 12 | BMB outperforms C |
| **Competitive** | 8 | Within ±10% of C |
| **Needs Optimization** | 2 | 10-40% slower than C |
| **Codegen Errors** | 19 | Unknown variable/function (array params) |
| **Linker Errors** | 3 | Missing runtime functions |

---

## 1. BMB Faster than C (Wins)

| Benchmark | BMB (ms) | C (ms) | Ratio | Speedup |
|-----------|----------|--------|-------|---------|
| typecheck_bootstrap | 3.36 | 16.11 | **0.21x** | 4.8x faster |
| sorting | 6.97 | 15.36 | **0.45x** | 2.2x faster |
| csv_parse | 3.39 | 5.13 | **0.66x** | 1.5x faster |
| purity_opt | 3.72 | 4.39 | **0.85x** | 1.2x faster |
| branch_elim | 3.58 | 3.65 | **0.98x** | 1.02x faster |
| stack_allocation | 3.47 | 3.55 | **0.98x** | 1.02x faster |
| syscall_overhead | 30.71 | 30.86 | **0.99x** | ~equal |

---

## 2. Competitive with C (±10%)

| Benchmark | BMB (ms) | C (ms) | Ratio | Status |
|-----------|----------|--------|-------|--------|
| fannkuch | 63.97 | 63.60 | 1.01x | ✅ Excellent |
| invariant_hoist | 3.34 | 3.30 | 1.01x | ✅ Excellent |
| spectral_norm | 3.65 | 3.55 | 1.03x | ✅ Good |
| mandelbrot | 3.61 | 3.44 | 1.05x | ✅ Good |
| json_parse | 4.13 | 3.94 | 1.05x | ✅ Good |
| binary_trees | 84.85 | 78.51 | 1.08x | ✅ Acceptable |
| n_body | 20.85 | 19.10 | 1.09x | ✅ Acceptable |

---

## 3. Needs Optimization Work

| Benchmark | BMB (ms) | C (ms) | Ratio | Root Cause |
|-----------|----------|--------|-------|------------|
| null_check | 3.92 | 3.25 | **1.21x** | Option<T> overhead |
| fibonacci | 21.49 | 15.52 | **1.38x** | 재귀 최적화 퇴행 |

### fibonacci: NOT a Regression (GCC vs LLVM Difference)

| Compiler | Time (ms) | Notes |
|----------|-----------|-------|
| GCC -O3 | ~15ms | Benchmark baseline |
| Clang -O3 | ~23ms | LLVM backend |
| BMB (LLVM) | ~24ms | Matches Clang |

**Root cause**: GCC has superior recursive function optimization compared to LLVM.
BMB correctly matches Clang/LLVM performance. The benchmark comparison is unfair
because it uses GCC for C but LLVM for BMB.

**Recommendation**: Either:
1. Use `clang -O3` for C benchmarks (fair LLVM-to-LLVM comparison)
2. Document this as a known LLVM vs GCC difference, not a BMB issue

---

## 4. Codegen Errors (19 benchmarks)

### Error Pattern: "Unknown variable"
These benchmarks use features not yet supported in native codegen:

| Benchmark | Missing Variable | Likely Cause |
|-----------|------------------|--------------|
| aliasing | `a` | Array/slice parameter |
| bounds_check | `arr` | Array parameter |
| cache_stride | `arr` | Array parameter |
| graph_traversal | `edges` | Complex data structure |
| matrix_multiply | `mat` | 2D array |
| pointer_chase | `next_indices` | Array parameter |
| simd_sum | `arr` | Array parameter |
| tree_balance | `tree` | Recursive data structure |

### Error Pattern: "Unknown function"
| Benchmark | Missing Function | Likely Cause |
|-----------|------------------|--------------|
| process_spawn | `system` | FFI not declared |

### Root Cause
The native codegen (llvm.rs) doesn't handle:
1. Array/slice parameters in function signatures
2. Complex data structure lowering
3. Some extern function declarations

---

## 5. Linker Errors (10 benchmarks)

| Benchmark | Error Code | Likely Cause |
|-----------|------------|--------------|
| brainfuck | ld returned 1 | Missing symbol |
| fasta | ld returned 1 | Missing symbol |
| hash_table | ld returned 5 | Multiple missing symbols |
| http_parse | ld returned 1 | Missing symbol |
| json_serialize | ld returned 5 | Multiple missing symbols |
| k-nucleotide | ld returned 5 | Multiple missing symbols |
| lex_bootstrap | ld returned 5 | Multiple missing symbols |
| lexer | ld returned 1 | Missing symbol |
| parse_bootstrap | ld returned 1 | Missing symbol |
| reverse-complement | ld returned 1 | Missing symbol |

### Root Cause
- Missing runtime functions in libbmb_runtime.a
- Undefined references to StringBuilder, HashMap, or other stdlib functions

---

## 6. Anomalies

### file_io_seq
| Language | Time (ms) |
|----------|-----------|
| BMB | 3.53 |
| C | 625.24 |

**Issue**: Results differ by 177x - likely measuring different things or BMB version is broken.

---

## Priority Action Items

### P0: Critical (Blocks benchmark validity)
1. **Investigate fibonacci regression** (1.04x → 1.38x)
   - Check if O3 is being applied consistently
   - May be MIR optimization issue

2. **Fix linker errors** for 10 benchmarks
   - Identify missing symbols with `nm` or verbose linker output
   - Add missing functions to runtime

### P1: High (Expand benchmark coverage)
3. **Fix codegen for array parameters** (19 benchmarks blocked)
   - `Unknown variable: arr/mat/etc` pattern
   - Need to handle array/slice lowering in llvm.rs

4. **Fix file_io_seq anomaly**
   - Verify both versions do the same work
   - Likely BMB version is no-op or cached

### P2: Medium (Performance optimization)
5. **Optimize null_check** (1.21x → 1.0x)
   - Option<T> unwrap overhead
   - Consider inline optimization

6. **Verify syscall_overhead** fix persists
   - Currently 0.99x (excellent!)
   - Document _cstr optimization limitation

### P3: Low (Future enhancements)
7. **Add string constant propagation**
   - Allow `let path = "."; file_exists(path)` to trigger _cstr optimization
   - Currently requires direct literals

---

## Benchmark Health Score

```
Working benchmarks:     21/50 (42%)
BMB competitive:        15/21 (71%)
BMB wins:                8/21 (38%)
Needs work:              3/21 (14%)
```

**Overall: Good progress, but 58% of benchmarks are blocked by codegen/linker issues.**
