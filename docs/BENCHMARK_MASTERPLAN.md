# BMB Benchmark Master Plan

> **Philosophy**: Performance + Stability > Human Convenience
> **Benchmark Goal**: Verify zero-overhead safety and competitive performance vs C/Rust

---

## Core Principles

### 1. Performance First
- Eliminate ALL defense code at machine-code level
- Runtime overhead = 0 (all verification at compile-time)
- If optimization is possible, it MUST be done

### 2. Stability Through Language Complexity
- Replace runtime checks with type system/contracts
- Unverifiable → compile error (no runtime cost)
- AI provides complete specifications → compiler optimizes aggressively

### 3. Verifiable Claims
- All design claims must be verified through benchmarks
- Zero-overhead safety must be proven with assembly comparison
- Reproducible measurement environment required

---

## Current Status (2026-01-17)

### Existing Benchmarks (36 total)

| Category | Count | Status | Notes |
|----------|-------|--------|-------|
| Compute | 10 | ✅ | Benchmarks Game standard |
| Contract | 6 | ⚠️ | Optimization effect below expectations |
| Real-World | 7 | ⚠️ | JSON 2.5x slower |
| Bootstrap | 3 | ✅ | Self-compilation measurement |
| **Zero-Overhead** | **5** | ✅ | Phase 1 complete |
| **Memory** | **5** | ✅ **NEW** | Phase 2 complete (MEM-2 skipped) |

### Gate Status

| Gate | Criteria | Current | Target |
|------|----------|---------|--------|
| #3.1 | Clang ≤1.10x | ✅ 1.00-1.08x | Maintain |
| #3.2 | All ≤1.05x | ❌ Not met | Must achieve |
| #3.3 | 3 faster than C | ❌ Not met | Must achieve |

### Discovered Limitations

| Issue | Impact | Status |
|-------|--------|--------|
| **Array Reference Indexing** | Arrays passed by value, not reference | Documented (ISSUE-20260117) |

---

## Gap Analysis & Enhancement Plan

### Phase 1: Zero-Overhead Proof (P0 - Immediate) ✅ COMPLETE

**Goal**: Prove BMB's safety verification has ZERO performance cost

| ID | Benchmark | Measurement Target | Status |
|----|-----------|-------------------|--------|
| ZO-1 | **bounds_check_proof** | Array index verification | ✅ Implemented |
| ZO-2 | **null_check_proof** | Sentinel-based null check | ✅ Implemented |
| ZO-3 | **overflow_proof** | Integer overflow verification | ✅ Implemented |
| ZO-4 | **aliasing_proof** | Pointer aliasing optimization | ✅ Implemented |
| ZO-5 | **purity_proof** | Pure function optimization | ✅ Implemented |

**Note**: ZO-1 uses smaller arrays (10 elements vs C's 1000) due to array-by-value limitation.
Assembly comparison pending WSL/LLVM setup.

**Verification Method**:
```bash
# Assembly comparison (requires WSL + LLVM)
bmb build bench.bmb --emit-asm -o bmb.s
clang -O3 bench.c -S -o c.s
diff bmb.s c.s  # Must be identical (or BMB shorter)
```

### Phase 2: Memory Benchmarks (P0 - 1 week) ✅ COMPLETE

**Goal**: Verify systems language essential memory performance

| ID | Benchmark | Measurement Target | Status |
|----|-----------|-------------------|--------|
| MEM-1 | **cache_stride** | Cache line access patterns | ✅ Implemented |
| MEM-2 | **allocation_stress** | malloc/free cycles | ⏭️ SKIPPED (no heap in BMB) |
| MEM-3 | **memory_copy** | memcpy replacement | ✅ Implemented |
| MEM-4 | **stack_allocation** | Stack variable access | ✅ Implemented |
| MEM-5 | **pointer_chase** | Linked list traversal | ✅ Implemented (index-based) |
| MEM-6 | **simd_sum** | SIMD vector operations | ✅ Implemented |

**Note**: MEM-2 skipped - BMB doesn't have heap allocation yet.
MEM-5 uses array indices to simulate pointer chasing (no pointers in BMB).

### Phase 3: System Call Benchmarks (P0 - 1 week)

**Goal**: Verify OS interface performance

| ID | Benchmark | Measurement Target | Target vs C |
|----|-----------|-------------------|-------------|
| SYS-1 | **syscall_overhead** | Basic system call | ≤1.00x |
| SYS-2 | **file_io_seq** | Sequential file read/write | ≤1.02x |
| SYS-3 | **file_io_random** | Random file access | ≤1.02x |
| SYS-4 | **process_spawn** | Process creation | ≤1.05x |
| SYS-5 | **signal_handling** | Signal processing latency | ≤1.00x |

### Phase 4: Real-World Workload Improvement (P1 - 2 weeks)

**Problem**: JSON parsing 2.5x slower → String processing bottleneck

| ID | Benchmark | Current | Target | Solution |
|----|-----------|---------|--------|----------|
| RW-1 | **json_parse** | 2.55x | ≤1.10x | String interning, SSO |
| RW-2 | **json_serialize** | ? | ≤1.10x | StringBuilder optimization |
| RW-3 | **regex_match** | N/A | ≤1.20x | New addition |
| RW-4 | **utf8_validate** | N/A | ≤1.00x | New addition |
| RW-5 | **compression_lz4** | N/A | ≤1.10x | New addition |

### Phase 5: Contract Optimization Proof (P1 - 2 weeks)

**Problem**: Contract optimization not achieving expected performance

| ID | Benchmark | Expected | Current | Root Cause Analysis |
|----|-----------|----------|---------|---------------------|
| CO-1 | **bounds_elim** | 10-30% faster | ~0% | LLVM already optimizes? |
| CO-2 | **null_elim** | 15-25% faster | ~0% | Branch prediction efficient? |
| CO-3 | **branch_elim** | 20-50% faster | ~0% | Dead code removal not working? |
| CO-4 | **loop_invariant** | 10-20% faster | ~0% | Hoisting not working? |

**Debugging Method**:
```bash
# LLVM IR comparison
bmb build bench.bmb --emit-llvm -o with_contract.ll
bmb build bench_no_contract.bmb --emit-llvm -o without.ll
diff with_contract.ll without.ll
```

### Phase 6: C/Rust Surpass Benchmarks (P2 - 3 weeks)

**Goal**: Achieve 3+ cases where BMB > C (Gate #3.3)

| ID | Benchmark | Surpass Strategy |
|----|-----------|------------------|
| WIN-1 | **matrix_multiply** | Contract-based aliasing analysis → SIMD maximization |
| WIN-2 | **sort_presorted** | Precondition removes branches |
| WIN-3 | **tree_balance** | Invariant-based rebalancing skip |
| WIN-4 | **string_search** | Compile-time pattern optimization |
| WIN-5 | **graph_traversal** | Reachability proof removes visit checks |

---

## Measurement Infrastructure Enhancement

### Current Issues

1. **Median only** → Need p50/p95/p99
2. **Single execution environment** → Reproducibility issues
3. **Manual execution** → Need CI automation

### Improvement Plan

```yaml
# .github/workflows/benchmark.yml
name: Benchmark CI
on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - name: Run benchmarks
        run: |
          benchmark-bmb run all -i 10 -w 3 --json > results.json

      - name: Check regression
        run: |
          benchmark-bmb compare results.json baseline.json --threshold 2%

      - name: Gate check
        run: |
          benchmark-bmb gate 3.1 3.2 3.3 --strict
```

### Output Enhancement

```json
{
  "benchmark": "fibonacci",
  "language": "bmb",
  "metrics": {
    "p50": 0.016,
    "p95": 0.017,
    "p99": 0.018,
    "min": 0.015,
    "max": 0.019,
    "stddev": 0.001
  },
  "comparison": {
    "vs_c": 1.00,
    "vs_rust": 0.93
  },
  "assembly_size": 1234,
  "llvm_ir_lines": 456
}
```

---

## Priority Summary

### Immediate (This Week)

| Priority | Task | Reason |
|----------|------|--------|
| **P0** | Zero-Overhead Proof (ZO-1~5) | Prove BMB core value |
| **P0** | Contract Optimization Debug (CO-1~4) | Identify root cause |
| **P0** | Add Memory Benchmarks (MEM-1~6) | Systems language essential |

### Short-term (2 weeks)

| Priority | Task | Reason |
|----------|------|--------|
| **P1** | JSON Performance Fix (RW-1) | Real workload credibility |
| **P1** | System Call Benchmarks (SYS-1~5) | Systems language verification |
| **P1** | CI Automation | Regression prevention |

### Medium-term (1 month)

| Priority | Task | Reason |
|----------|------|--------|
| **P2** | C Surpass Cases (WIN-1~5) | Achieve Gate #3.3 |
| **P2** | Real-time Dashboard | Community transparency |
| **P2** | Cross-platform Benchmarks | Portability verification |

---

## Success Criteria

### Gate #3.2: All Benchmarks ≤1.05x vs C

```
[ ] fibonacci      ≤1.05x  (current: 1.00x ✅)
[ ] mandelbrot     ≤1.05x  (current: ? )
[ ] spectral_norm  ≤1.05x  (current: ? )
[ ] binary_trees   ≤1.05x  (current: 1.39x ❌)
[ ] n_body         ≤1.05x  (current: ? )
[ ] json_parse     ≤1.05x  (current: 2.55x ❌)
[ ] ... (20+ more)
```

### Gate #3.3: 3+ Cases Faster Than C

```
[ ] Case 1: _______ (BMB < C by __%)
[ ] Case 2: _______ (BMB < C by __%)
[ ] Case 3: _______ (BMB < C by __%)
```

### Zero-Overhead Proof

```
[ ] bounds_check: BMB safe == C unsafe (identical assembly)
[ ] null_check: BMB Option == C raw pointer (identical assembly)
[ ] overflow_check: BMB checked == C unchecked (identical assembly)
```

---

## Next Actions

1. **Immediate**: Start implementing 5 Zero-Overhead benchmarks
2. **Immediate**: Analyze LLVM IR for contract optimization failures
3. **This week**: Add Memory benchmark category
4. **This week**: Set up CI benchmark automation

---

*Last updated: 2026-01-17*
*Version: v0.50.24*
