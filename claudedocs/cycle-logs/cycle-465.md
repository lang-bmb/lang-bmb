# Cycle 465: Arena Allocation Enabled in Bootstrap Compiler

## Date
2026-02-14

## Scope
Enable arena allocation mode in bootstrap compiler's main() to use bump allocation instead of malloc for all string operations. This is an architectural improvement enabling proper arena_save/arena_restore memory reclamation.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Bootstrap compiler was using `malloc` for all allocations despite having arena_save/arena_restore calls
- Arena mode converts all `bmb_alloc` calls to arena bump allocation (pointer bump + bounds check)
- The Rust type checker knows `arena_mode` as a built-in function (not `bmb_arena_mode`)
- Arena mode adds 2 LLVM IR lines (68,626 vs 68,624) for the arena initialization call

## Implementation
### Files Modified
1. **`bootstrap/compiler.bmb`** — Added `arena_mode(1)` at the start of `main()`:
   ```bmb
   fn main() -> i64 =
       let _arena = arena_mode(1);  // Enable arena allocation
       ...
   ```

### Performance Results
| Metric | Without Arena | With Arena | Change |
|--------|--------------|------------|--------|
| Stage 1 emit-ir | ~2.40s | ~2.34s | ~2.5% (within noise) |
| IR output | 68,624 lines | 68,626 lines | +2 (arena_mode call) |

**Finding**: Arena vs malloc performance difference is negligible on Windows. The main bottleneck is not allocation speed but rather:
1. Number of intermediate string allocations (concatenation chains)
2. Linear registry lookups (O(n) per function call)
3. Function call overhead from string operations

### Why Arena Mode Matters (Architecture)
- Enables existing `arena_save`/`arena_restore` to actually reclaim memory per-function
- Reduces memory fragmentation for long compilations
- Foundation for future arena-backed StringBuilder optimization
- Proper memory management pattern for the bootstrap compiler

### Verification
1. **Rust tests**: 5,229 passed
2. **Fixed point**: Verified (S1→S2→S3, 68,626 lines identical)
3. **Golden tests**: 13/13 PASS

## Test Results
| Test | Status |
|------|--------|
| Rust tests | 5,229 passed |
| Fixed point | Verified (68,626 lines) |
| Golden tests | 13/13 PASS |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 9/10 | Correct use of arena mode, enables proper memory reclamation |
| Philosophy Alignment | 8/10 | Architectural improvement, minimal perf gain |
| Test Quality | 9/10 | Fixed point + golden tests + Rust tests |
| Documentation | 8/10 | Cycle log captures findings |
| Code Quality | 10/10 | Single-line change, clean and correct |
| **Average** | **9.0/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | 4.7x gap remains — allocation speed is not the bottleneck | Focus on reducing allocation COUNT not speed |
| I-02 | M | Linear registry lookups still O(n) per function call | Needs proper investigation of reg_cached_lookup segfault |
| I-03 | L | StringBuilder buffers still use malloc (not arena) | Future: arena-backed SB |
| I-04 | L | 68,626 vs 68,624 line IR difference needs golden binary update | Update when other changes accumulate |

## Next Cycle Recommendation
- Performance optimization: convert hot concatenation chains in LLVM IR gen to StringBuilder
- OR: Investigate remaining bottlenecks with profiling
- OR: Focus on bootstrap feature expansion (closures, generics)
- The 4.7x gap (2.34s vs 0.50s) is now dominated by algorithmic overhead, not allocation
