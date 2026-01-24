# RFC-0005: Benchmark-Driven Package Roadmap

> Status: Draft
> Created: 2026-01-08
> Phase: v0.33.6

## Summary

Define a benchmark-driven development roadmap for BMB packages that:
1. Maximizes performance gains with current compiler capabilities
2. Identifies critical feature gaps blocking benchmarks
3. Plans incremental improvements aligned with BMB philosophy

## Motivation

### Current State Analysis

**Benchmark Results (Phase 33.5)**:
| Benchmark | Rust (ms) | BMB (ms) | Ratio | Issue |
|-----------|-----------|----------|-------|-------|
| fibonacci(35) | 60 | 58 | 0.97x | ✅ None |
| mandelbrot | 15 | 14 | 0.93x | ✅ Fixed-point works |
| spectral_norm | 9 | 8 | 0.89x | ✅ Integer approximation |
| purity_opt | 9 | 8 | 0.89x | ✅ Contract advantage |
| json_parse | 8 | 10 | 1.25x | ⚠️ **25% slower** |

**Critical Observations**:
1. ✅ **Compute benchmarks pass** - BMB >= Rust in pure computation
2. ⚠️ **String processing lags** - json_parse 25% slower
3. ❌ **Feature gaps block benchmarks** - n_body (f64), binary_trees (heap)

### Philosophy Alignment Check

Per `docs/ROADMAP.md` Non-Negotiable Priorities:

| Priority | Principle | Current Status |
|----------|-----------|----------------|
| **P0 Performance** | Maximum Performance Syntax | ⚠️ json_parse 25% slower |
| **P0 Correctness** | Compile-Time Verification | ✅ All contracts verified |
| **P0 Self-Hosting** | BMB-only composition | ✅ Bootstrap complete |

**Critical Gap**: json_parse performance violates P0 Performance principle.

## Design

### Phase 33.6: Benchmark Enhancement & Package Preparation

#### 33.6.1: String Processing Optimization (P0)

**Goal**: Fix json_parse 25% performance gap

**Root Cause Analysis**:
```
Rust json_parse: 8ms
BMB json_parse: 10ms (+25%)

Suspected causes:
1. String.slice() overhead - Creates new allocation per call
2. Recursive descent without memoization
3. Character-by-character processing vs chunk processing
```

**Proposed Optimizations**:

1. **Avoid excessive slice()** - Use index-based parsing
```bmb
-- Current (slow)
fn parse_value(s: String) -> JsonResult =
    let trimmed = s.trim();  -- Creates new String
    parse_inner(trimmed);

-- Proposed (fast)
fn parse_value(s: String, start: i64, end: i64) -> JsonResult =
    let pos = skip_ws(s, start);  -- Just advances index
    parse_inner(s, pos, end);
```

2. **Batch character classification**
```bmb
-- Current: Per-character checks
fn is_digit(c: i64) -> bool = c >= 48 and c <= 57;

-- Proposed: Lookup table (compile-time)
@const
fn DIGIT_TABLE() -> [bool; 256] = -- Precomputed lookup
```

3. **Tail-call optimization verification**
```bmb
-- Verify TCO applies to recursive parsers
@pure @decreases(end - pos)
fn parse_array(s: String, pos: i64, end: i64) -> JsonResult = ...
```

**Success Criteria**: json_parse BMB <= Rust (eliminate 25% gap)

#### 33.6.2: New Benchmarks with Current Capabilities

**Available benchmarks (no compiler changes needed)**:

| Benchmark | Category | Required Features | Status |
|-----------|----------|-------------------|--------|
| string_search | String | String, char_at | Can add |
| lexer_bench | String | Bootstrap lexer | Can add |
| int_parse | Parse | parse_i64 | Can add |
| sorting_variants | Memory | Fixed arrays | Can add |
| contract_bounds | Contract | pre/post | Can add |

**Implementation Plan**:

```
benches/
├── string/
│   ├── string_search/  -- Boyer-Moore vs naive
│   └── lexer_bench/    -- BMB tokenization
├── parse/
│   └── int_parse/      -- Integer parsing speed
└── contract/
    ├── bounds_elim/    -- Bounds check elimination proof
    └── null_elim/      -- Option check elimination
```

#### 33.6.3: Feature Gap Specification (RFC Preparation)

**For v0.34 implementation**:

##### RFC-0006: f64 Type (Draft)

```
Title: Floating-Point Type Support
Status: Draft
Priority: P0 (blocks n_body, mandelbrot_fp)

Specification:
- Type: f64 (IEEE 754 double precision)
- Literals: 3.14, 1.0e-5, 0x1.0p-1 (hex float)
- Operations: +, -, *, /, %, <, <=, >, >=, ==, !=
- Functions: sqrt, sin, cos, tan, exp, log, pow
- Contract support: pre x >= 0.0, post ret >= 0.0

Implementation:
- Lexer: Add f64 literal token
- Parser: Parse f64 expressions
- Type checker: f64 type rules
- LLVM: Use LLVM double type
- SMT: Z3 Real theory (approximation)

Benchmarks Enabled:
- n_body (physics simulation)
- mandelbrot_fp (floating-point complex)
- spectral_norm_fp (accurate matrix ops)
```

##### RFC-0007: Dynamic Collections (Draft)

```
Title: Dynamic Collection Types
Status: Draft
Priority: P1 (blocks binary_trees, hash_table)

Specification:
- Vec<T>: Growable array
- HashMap<K, V>: Hash-based dictionary
- LinkedList<T>: Doubly-linked list

Implementation Approach:
Option A: Arena allocation (no GC)
Option B: Reference counting
Option C: Ownership-based (Rust-like)

Recommended: Option C (aligns with BMB ownership model)

Benchmarks Enabled:
- binary_trees (heap allocation)
- hash_table (HashMap operations)
- lru_cache (HashMap + LinkedList)
```

### Critical Evaluation & Improvements

#### Philosophy Violations to Fix

1. **json_parse P0 Performance Violation**
   - Current: 25% slower than Rust
   - Required: BMB <= Rust
   - Action: Immediate optimization in Phase 33.6.1

2. **Incomplete Benchmark Coverage**
   - Current: 5/12 benchmarks fully comparable
   - Blocked by: f64 (3), heap allocation (2), other (2)
   - Action: Document gaps, plan for v0.34

3. **Missing Contract Advantage Proof**
   - Claim: "Contracts enable 10-30% optimization"
   - Current: purity_opt shows 11% advantage
   - Required: bounds_check, null_check benchmarks with measurable advantage
   - Action: Implement contract-specific benchmarks in 33.6.2

#### Proposed Package Improvements (Without Compiler Changes)

| Package | Current Issue | Proposed Fix | Impact |
|---------|--------------|--------------|--------|
| bmb-json | 25% slower | Index-based parsing | P0 Fix |
| bmb-regex | Literal only | Extend patterns | Benchmark accuracy |
| bmb-math | Integer sqrt | Newton's method optimization | Minor |
| bmb-time | Complete | None needed | - |

## Implementation Plan

### Phase 33.6.1 (Week 1)

**Tasks**:
1. [ ] Profile bmb-json to identify bottlenecks
2. [ ] Implement index-based parsing (avoid slice())
3. [ ] Add character classification optimization
4. [ ] Re-run json_parse benchmark
5. [ ] Target: json_parse BMB <= Rust

### Phase 33.6.2 (Week 2)

**Tasks**:
1. [ ] Add string_search benchmark (Boyer-Moore)
2. [ ] Add lexer_bench using bootstrap lexer
3. [ ] Add bounds_elim contract benchmark
4. [ ] Add null_elim contract benchmark
5. [ ] Document contract optimization benefits

### Phase 33.6.3 (Week 3)

**Tasks**:
1. [ ] Draft RFC-0006 (f64)
2. [ ] Draft RFC-0007 (Dynamic Collections)
3. [ ] Update MODULE_ROADMAP.md with RFC links
4. [ ] Create v0.34 planning document
5. [ ] Update ROADMAP.md Phase 33.6 completion

## Success Criteria

### Phase 33.6 Complete When:

| Criterion | Target | Verification |
|-----------|--------|--------------|
| json_parse | BMB <= Rust | Benchmark ratio <= 1.0 |
| New benchmarks | 4+ added | Benchmark suite count |
| Contract proofs | 2+ demonstrations | Documented % improvement |
| RFC drafts | 2 complete | RFC-0006, RFC-0007 |
| Documentation | All updated | ROADMAP, MODULE_ROADMAP |

### Gate #1 Status After 33.6:

| Benchmark | Target | Expected |
|-----------|--------|----------|
| fibonacci | BMB <= Rust | ✅ Already achieved |
| mandelbrot | BMB <= Rust | ✅ Already achieved |
| spectral_norm | BMB <= Rust | ✅ Already achieved |
| json_parse | BMB <= Rust | 🎯 Phase 33.6.1 target |
| Contract advantage | > 10% | 🎯 Phase 33.6.2 target |

## References

- [MODULE_ROADMAP.md](../../ecosystem/gotgan-packages/MODULE_ROADMAP.md)
- [BENCHMARK_STRATEGY.md](../BENCHMARK_STRATEGY.md)
- [Benchmark Results](../../ecosystem/benchmark-bmb/results/)
- [LLVM Language Reference](https://llvm.org/docs/LangRef.html) - f64 implementation
- [LLVM fcmp codegen](https://theunixzoo.co.uk/blog/2025-04-15-llvm-fcmp-codegen.html) - Floating-point comparison

## Appendix: Fixed-Size HashMap Design

For Phase 33.6.2 contract benchmarks, a fixed-size HashMap can be implemented:

```bmb
-- Fixed-size hash map (no dynamic allocation)
-- Uses open addressing with linear probing

type FixedHashMap = struct {
    keys: [i64; 64],
    values: [i64; 64],
    occupied: [bool; 64],
    size: i64
};

@pure
fn hash_index(key: i64) -> i64
  post ret >= 0 and ret < 64
= let h = key * 2654435761;  -- Knuth's multiplicative hash
  let idx = h - (h / 64) * 64;
  if idx < 0 then idx + 64 else idx;

fn map_get(m: &FixedHashMap, key: i64) -> Option =
  let idx = hash_index(key);
  find_key(m, key, idx, 0);

@pure @decreases(64 - attempts)
fn find_key(m: &FixedHashMap, key: i64, idx: i64, attempts: i64) -> Option
  pre attempts >= 0 and attempts <= 64
  pre idx >= 0 and idx < 64
=
  if attempts >= 64 then Option::None
  else if not m.occupied[idx] then Option::None
  else if m.keys[idx] == key then Option::Some(m.values[idx])
  else find_key(m, key, (idx + 1) - ((idx + 1) / 64) * 64, attempts + 1);
```

This enables hash_table benchmark without dynamic allocation.
