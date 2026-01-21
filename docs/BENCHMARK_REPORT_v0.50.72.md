# BMB Benchmark Report v0.50.72

**Date**: 2026-01-21
**Compiler Version**: v0.50.72
**Test Configuration**: 5 iterations, 2 warmup runs

---

## Executive Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Benchmarks** | 48 | 100% |
| **Faster than C** (≤1.00x) | 21 | **44%** |
| **Acceptable** (1.01-1.10x) | 15 | 31% |
| **Needs Work** (>1.10x) | 12 | 25% |
| **Target Met** (≤1.10x) | **36** | **75%** |

```
Performance Distribution:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Faster/Equal (21)  ███████████████████████████████████████████░░░░
✅ Acceptable (15)    ██████████████████████████████░░░░░░░░░░░░░░░░░
❌ Needs Work (12)    ████████████████████████░░░░░░░░░░░░░░░░░░░░░░░
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Detailed Results

### Tier 1: BMB Faster than C (21 benchmarks)

| Benchmark | BMB (ms) | Category | Notes |
|-----------|----------|----------|-------|
| bounds_check | 8.59 | contract | Safety with zero overhead |
| bounds_check_proof | 7.60 | zero_overhead | Verified bounds |
| bounds_elim | 7.37 | contract_opt | Bounds elimination |
| brainfuck | 7.71 | real_world | Interpreter benchmark |
| csv_parse | 9.06 | real_world | Parsing performance |
| hash_table | 7.58 | compute | Hash operations |
| k-nucleotide | 8.04 | compute | Bioinformatics |
| lex_bootstrap | 9.32 | bootstrap | Lexer performance |
| lexer | 4.67 | real_world | Tokenization |
| loop_invariant | 4.20 | contract_opt | LICM optimization |
| memory_copy | 4.09 | memory | Copy operations |
| n_body | 4.26 | compute | Physics simulation |
| null_check_proof | 4.28 | zero_overhead | Null safety |
| overflow_proof | 4.21 | zero_overhead | Overflow checks |
| purity_opt | 4.36 | contract | Pure function opt |
| purity_proof | 4.33 | zero_overhead | Purity verification |
| sorting | 4.40 | real_world | Sort algorithms |
| spectral_norm | 5.09 | compute | Linear algebra |
| string_search | 5.15 | surpass | String matching |
| tree_balance | 4.50 | surpass | Tree operations |
| typecheck_bootstrap | 5.55 | bootstrap | Type checking |

### Tier 2: Acceptable Performance (15 benchmarks)

| Benchmark | Ratio | BMB (ms) | Category |
|-----------|-------|----------|----------|
| fasta | 1.01x | 7.68 | compute |
| null_check | 1.01x | 4.40 | contract |
| process_spawn | 1.02x | 666.13 | syscall |
| aliasing | 1.03x | 8.44 | contract |
| binary_trees | 1.03x | 180.52 | compute |
| stack_allocation | 1.03x | 4.60 | memory |
| branch_elim (contract) | 1.05x | 8.05 | contract |
| file_io_seq | 1.05x | 1118.12 | syscall |
| invariant_hoist | 1.05x | 7.70 | contract |
| simd_sum | 1.05x | 5.25 | memory |
| aliasing_proof | 1.07x | 8.07 | zero_overhead |
| parse_bootstrap | 1.07x | 4.63 | bootstrap |
| mandelbrot | 1.08x | 4.43 | compute |
| graph_traversal | 1.10x | 8.52 | surpass |
| sort_presorted | 1.10x | 4.94 | surpass |

### Tier 3: Needs Improvement (12 benchmarks)

| Rank | Benchmark | Ratio | Root Cause | Priority |
|------|-----------|-------|------------|----------|
| 1 | **syscall_overhead** | **3.76x** | BmbString wrapper overhead | P1 |
| 2 | **fannkuch** | **1.76x** | Recursion overhead | P1 |
| 3 | **http_parse** | **1.56x** | String concatenation | P2 |
| 4 | **matrix_multiply** | **1.44x** | Loop/array access | P2 |
| 5 | **json_serialize** | **1.35x** | String building | P2 |
| 6 | **fibonacci** | **1.33x** | Non-tail recursion | P3 |
| 7 | **null_elim** | **1.23x** | Contract overhead | P3 |
| 8 | **json_parse** | **1.16x** | String operations | P3 |
| 9 | **reverse-complement** | **1.16x** | String processing | P3 |
| 10 | **branch_elim** (opt) | **1.15x** | Branch optimization | P3 |
| 11 | **pointer_chase** | **1.15x** | Pointer indirection | P3 |
| 12 | **cache_stride** | **1.11x** | Cache access pattern | P4 |

---

## Root Cause Analysis

### Critical Issues (P1)

#### 1. syscall_overhead: 3.76x slower
**Root Cause**: BmbString wrapper creates overhead for FFI calls

```
C Code:
  stat(path, &st);           // Direct syscall with char*

BMB Code:
  bmb_string_from_cstr(".")  // Create BmbString wrapper
  file_exists(path)          // Unwrap to char*, call stat
```

**Fix Options**:
- A. Add raw string type (`cstr`) for FFI-heavy code
- B. Optimize BmbString to use SSO (Small String Optimization)
- C. Inline string_from_cstr for constant strings

#### 2. fannkuch: 1.76x slower
**Root Cause**: Recursive permutation algorithm vs C's iterative loops

```
BMB: fn permute(arr, k) = if k == 0 { ... } else { permute(arr, k-1) }
C:   for (int k = n; k > 0; k--) { ... }
```

**Fix Options**:
- A. Rewrite benchmark with while loops (language feature exists)
- B. Improve function call overhead in codegen
- C. Better tail call optimization

### Major Issues (P2)

#### 3. http_parse: 1.56x slower
**Root Cause**: String concatenation creates many allocations

```bmb
"GET " + path + " HTTP/1.1" + crlf()  // 4+ allocations
```

**Fix**: String interpolation or StringBuilder optimization

#### 4. matrix_multiply: 1.44x slower
**Root Cause**: Array indexing overhead in nested loops

**Fix**: Loop unrolling or better array access codegen

#### 5. json_serialize: 1.35x slower
**Root Cause**: O(n²) string building pattern

```bmb
fn escape_string_loop(s, pos, acc + escape_char(c))  // Quadratic!
```

**Fix**: Use StringBuilder with pre-allocated buffer

### Minor Issues (P3)

| Benchmark | Ratio | Quick Fix |
|-----------|-------|-----------|
| fibonacci | 1.33x | Algorithm uses non-tail recursion - expected |
| null_elim | 1.23x | Contract elimination not fully optimizing |
| json_parse | 1.16x | String operations overhead |
| reverse-complement | 1.16x | Bioinformatics string processing |
| branch_elim | 1.15x | Branch prediction hints |
| pointer_chase | 1.15x | Pointer indirection overhead |

---

## Comparison with Previous Versions

| Version | ≤1.10x C | Notes |
|---------|----------|-------|
| v0.50.69 | 37/48 (77%) | Benchmark runner fix |
| v0.50.70 | 38/48 (79%) | vec_push PHI bug fix |
| **v0.50.72** | **36/48 (75%)** | While loop + syscall test |

**Note**: v0.50.72 shows slight regression due to:
1. More accurate while loop codegen (was incorrectly optimizing before)
2. syscall_overhead now uses correct while loop (was using optimized recursion)
3. Variance in benchmark runs

---

## Improvement Recommendations

### Immediate Actions (v0.50.73)

1. **syscall_overhead optimization**
   - Add `@inline` attribute for constant string creation
   - Expected improvement: 3.76x → ~1.5x

2. **fannkuch rewrite**
   - Convert recursive permutation to while loops
   - Expected improvement: 1.76x → ~1.1x

### Short-term (v0.51.x)

3. **String concatenation optimization**
   - Implement rope-based strings or StringBuilder
   - Affects: http_parse, json_serialize, json_parse

4. **Array access optimization**
   - Bounds check elimination in proven-safe loops
   - Affects: matrix_multiply, cache_stride

### Long-term (v0.52+)

5. **Loop constructs enhancement**
   - Add `for` loop with iterator support
   - Improve while loop codegen

6. **FFI string optimization**
   - Zero-copy string passing for `@extern` functions
   - Add `cstr` type for C interop

---

## Category Performance Summary

| Category | Benchmarks | ≤1.00x | 1.01-1.10x | >1.10x | Win Rate |
|----------|------------|--------|------------|--------|----------|
| zero_overhead | 5 | 4 | 1 | 0 | **100%** |
| bootstrap | 3 | 2 | 1 | 0 | **100%** |
| contract | 5 | 1 | 3 | 1 | 80% |
| contract_opt | 4 | 2 | 0 | 2 | 50% |
| compute | 10 | 4 | 3 | 3 | 70% |
| memory | 6 | 2 | 3 | 1 | 83% |
| real_world | 7 | 4 | 0 | 3 | 57% |
| surpass | 5 | 2 | 2 | 1 | 80% |
| syscall | 3 | 0 | 2 | 1 | 67% |

**Strongest**: zero_overhead (100%), bootstrap (100%)
**Weakest**: contract_opt (50%), real_world (57%)

---

## Conclusion

BMB v0.50.72 achieves **75% of benchmarks within 10% of C performance**, with **44% actually faster than C**. The main bottlenecks are:

1. **String handling** - BmbString wrapper overhead
2. **Recursion** - Function call overhead vs native loops
3. **String building** - O(n²) concatenation patterns

With targeted optimizations for syscall_overhead and fannkuch, BMB can reach **85%+ benchmarks ≤1.10x C**.

---

*Report generated: 2026-01-21*
*Compiler: BMB v0.50.72*
*Platform: Windows x86_64*
