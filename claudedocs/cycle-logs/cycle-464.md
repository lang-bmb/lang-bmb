# Cycle 464: Hash Map Registry Attempt + Arena Fast Path Restoration

## Date
2026-02-14

## Scope
Attempt to replace linear O(n) function registry lookups with O(1) hash map lookups using `reg_cached_lookup`. Also restore arena allocator fast path from Cycle 463.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 4/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Identified `lookup_fn_ret` and `lookup_fn_params` as the main linear-scan bottleneck (called for every function call during codegen)
- compiler.bmb has 850 functions — each lookup scans ~425 entries on average
- `reg_cached_lookup` runtime function exists (from Cycle 452) with hash map caching, slot-based invalidation
- Changed registry format from comma-separated (`@name:ret:params,`) to equals-semicolon (`name=ret:params;`)
- **Segfault**: Stage 1 crashes on files with 100+ functions when using `reg_cached_lookup`
  - lexer.bmb (66 functions): OK
  - parser.bmb (145 functions): SEGFAULT
  - types.bmb (500+ functions): SEGFAULT
  - compiler.bmb (850 functions): SEGFAULT
  - All golden tests (small files): PASS
- Root cause: Unknown — needs deeper investigation with GDB/ASAN. Likely related to hash map and string lifetime interaction in non-arena allocation mode.

## Implementation
### Files Modified
1. **`bmb/runtime/bmb_runtime.c`** — Restored arena fast path:
   - `bmb_arena_alloc`: inline fast path checks current block before limit check
   - Net effect: reduces function call overhead on every arena allocation

### Attempted (Reverted)
- **`bootstrap/compiler.bmb`** — Hash map registry integration:
  - Changed `register_fn_type` to use `=`/`;` separators
  - Replaced `lookup_fn_ret_at` (linear) with `reg_cached_lookup` (hash map)
  - Replaced `lookup_fn_params_at` (linear) with `reg_cached_lookup` (hash map)
  - **REVERTED** due to segfault on files with 100+ functions

### Verification
1. **Rust tests**: 5,229 passed
2. **Fixed point**: S1→S2→S3 verified (68,624 lines identical)
3. **Golden tests**: 13/13 PASS
4. **Performance**: ~2.4s (consistent with Cycle 463 — 45% improvement maintained)

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Fixed point | Verified |
| Golden tests | 13/13 PASS |
| Stage 1 emit-ir | ~2.4s (maintained) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 8/10 | Hash map attempt failed, but clean revert — no regressions |
| Architecture | 7/10 | Identified correct approach (hash map) but execution had unknown bug |
| Philosophy Alignment | 8/10 | Performance optimization attempt aligns with core philosophy |
| Test Quality | 9/10 | Thorough testing caught the segfault early (before commit) |
| Documentation | 8/10 | Cycle log captures the attempt, failure analysis, and revert |
| Code Quality | 8/10 | Arena fast path restoration is clean, reverted code left no artifacts |
| **Average** | **8.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | H | `reg_cached_lookup` segfaults on 100+ function files in bootstrap | Debug with GDB/ASAN, check hash map + non-arena string interaction |
| I-02 | M | Linear registry lookups remain (O(n) per lookup, 850 entries) | Future cycle: proper investigation of hash map crash |
| I-03 | L | Arena fast path added but arena not enabled in bootstrap compiler | Enable arena mode in compiler.bmb main() |
| I-04 | L | `str_key_eq` in hash map still uses byte-by-byte comparison | Change to `memcmp` |

## Next Cycle Recommendation
- The hash map segfault needs GDB debugging — likely a string lifetime or pointer aliasing issue
- Alternative approach: enable arena mode in compiler.bmb to reduce malloc overhead
- OR: Focus on other bootstrap improvements (new features, error handling)
- The 4.7x gap (2.4s vs 0.5s) breakdown: ~50% string concat, ~50% linear lookups
