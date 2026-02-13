# Cycle 463: Runtime Performance Optimization — memcpy + Arena Fast Path

## Date
2026-02-14

## Scope
Identify and reduce the Stage 1 compilation bottleneck (8.4x gap vs Rust). Focus on runtime-level optimizations that benefit all bootstrap compilations without requiring compiler.bmb changes.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- **Bottleneck analysis**: 643 string concatenation patterns in compiler.bmb, 122 `+ int_to_string()` patterns, 72 linear lookup functions
- **Root cause**: Runtime string functions used byte-by-byte copy loops instead of hardware-optimized `memcpy`
- **Impact distribution**: String concatenation ~60-70%, linear lookups ~20-25%, parsing overhead ~10-15%
- **Key insight**: Arena allocator was calling `bmb_arena_init_limit()` on every allocation (millions of calls)

## Implementation
### Files Modified
1. **`bmb/runtime/bmb_runtime.c`** — Comprehensive string/memory optimization:
   - Added `#include <string.h>` for `memcpy`/`memset`/`strlen`/`memcmp`
   - **`bmb_string_concat`**: Replaced 2 byte-by-byte loops with 2 `memcpy` calls
   - **`bmb_string_slice`**: Replaced byte loop with `memcpy`, direct struct init (avoids `bmb_string_wrap` strlen scan)
   - **`bmb_string_eq`**: Replaced byte-by-byte comparison with `memcmp`
   - **`bmb_string_new`**: Replaced byte loop with `memcpy`, direct struct init
   - **`bmb_string_from_cstr`**: Replaced manual strlen + byte loop with `strlen` + `memcpy`, direct struct init
   - **`bmb_string_wrap`**: Replaced manual strlen loop with `strlen`
   - **`bmb_sb_push`**: Replaced byte loop with `memcpy`, added early return for empty strings, fixed capacity growth to single pass
   - **`bmb_sb_build`**: Replaced byte loop with `memcpy`, direct struct init (avoids `bmb_string_wrap` strlen scan)
   - **`bmb_sb_new`**: Increased default capacity from 64 → 1024 (reduces reallocation frequency)
   - **`bmb_arena_alloc`**: Added inline fast path — checks if current block has space before limit check and block allocation

### Key Design Decisions
- **Zero API changes**: All optimizations are internal to the runtime. No changes to compiler.bmb needed.
- **Direct struct init pattern**: Functions like `bmb_string_slice` now create `BmbString` directly instead of going through `bmb_string_wrap`, avoiding a redundant `strlen` call on data whose length is already known.
- **Arena fast path**: Most allocations hit the fast path (current block has space) — now a single comparison + pointer bump without function call overhead.

### Performance Results
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Stage 1 emit-ir | 4.24s | 2.33s | **45% faster** (1.82x) |
| Ratio vs Rust | 8.4x | 4.7x | **44% reduction** |
| Rust emit-ir | 0.50s | 0.50s | (unchanged) |
| IR output | 68,624 lines | 68,624 lines | (identical) |

### Verification
1. **Rust tests**: 5,229 passed
2. **Fixed point**: Stage 1→Stage 2→Stage 3 verified (68,624 lines identical)
3. **Golden tests**: 13/13 PASS
4. **Bootstrap tests**: 5/5 PASS (821 sub-tests)

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Fixed point | Verified (S1=S2=S3) |
| Golden tests | 13/13 PASS |
| Bootstrap tests | 5/5 (821 sub-tests) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified, zero API changes |
| Architecture | 10/10 | Clean runtime optimization — no hacks, no workarounds |
| Philosophy Alignment | 10/10 | "Performance > Everything" — 45% improvement in compilation speed |
| Test Quality | 9/10 | Verified via fixed point + all test suites, no new dedicated perf tests |
| Documentation | 8/10 | Cycle log captures all changes, no inline code comments needed |
| Code Quality | 10/10 | Standard C library functions replacing hand-rolled equivalents |
| **Average** | **9.5/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | 4.7x gap still remains — linear registry lookups are next bottleneck | Replace with `reg_cached_lookup` hash map |
| I-02 | M | 122 `+ int_to_string()` patterns create intermediate strings | Convert hot paths to `sb_push_int` in compiler.bmb |
| I-03 | L | `bmb_sb_push_escaped` still uses byte-by-byte (inherent for escaping) | Low priority |
| I-04 | L | StringBuilder buffers use `realloc` (not arena) | Consider arena-backed SB |

## Next Cycle Recommendation
- Next bottleneck: Replace linear fn registry lookups with `reg_cached_lookup` hash map in compiler.bmb
- OR: Convert hot concatenation chains in LLVM IR gen to use StringBuilder patterns
- The remaining 4.7x gap is split between linear lookups (~50%) and concatenation overhead (~50%)
