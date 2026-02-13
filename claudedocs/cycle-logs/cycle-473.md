# Cycle 473: Trampoline Optimization — Escape-Free Work Items + Leaf Fast Path

## Date
2025-02-12

## Scope
Optimize the trampoline_v2 lowering engine to reduce per-step overhead.
Primary target: the 1,774ms lowering phase that was 79% of compilation time.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
Cycle 472 profiling identified that the trampoline processes ~25K steps for compiler.bmb.
Each step involved escape_field/unescape_field calls creating StringBuilders per field.
Analysis showed ~115K+ SB creations from escape/unescape alone.

## Implementation

### Optimization 1: Escape-Free Field Separator (PRIMARY WIN)
**Problem**: Work items use `~` as field separator. Fields containing `~` must be escaped
with `escape_field()` (creates SB, char-by-char scan) and unescaped with `unescape_field()`
(another SB, char-by-char scan). Each make_work3 call created 3 SBs, each make_work4
created 4 SBs — totaling ~115K+ SB creations per compilation.

**Solution**: Changed field separator from `~` (byte 126) to `chr(1)` (byte 1, SOH).
ASCII SOH never appears in code, AST, MIR, or LLVM IR data. This eliminates ALL
escape/unescape overhead.

**Files modified**: `bootstrap/compiler.bmb`
- `field_sep()`: `"~"` → `chr(1)`
- `find_field_sep()`: byte 126 → byte 1
- `get_field()`: removed `unescape_field` call
- `make_work/3/4/6/7/10()`: removed `escape_field` calls, `"~"` → `chr(1)`

### Optimization 2: Trampoline v3 with Reusable SBs + Leaf Fast Path
**Problem**: trampoline_v2 created 2 StringBuilders per step (exit_sb, work_sb)
and parsed the step result 4 separate times.

**Solution**: trampoline_v3 with:
- Pre-allocated exit_sb and work_sb, reused via sb_clear (not alloc/free per step)
- Leaf step fast path: steps returning `make_step_leaf(temp)` (no pipes) skip
  exit_label parsing, block parsing, and exit_sb population
- Single-pass pipe parsing: all pipes found in one pass instead of separate functions
- 26 step handlers converted to use `make_step_leaf()`

### Performance Results
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Stage 1 emit-ir | ~2.34s | **~1.81s** | **-23% (530ms saved)** |
| Performance gap | 4.7x vs Rust | **3.6x vs Rust** | **-23%** |
| Fixed point lines | 68,993 | 68,953 | -40 (dead code removed) |
| Golden tests | 17/17 | 17/17 | No change |

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,229 passed |
| Golden tests (Stage 1) | 17/17 PASS |
| Golden tests (Stage 2) | 17/17 PASS |
| Fixed point (S2==S3) | VERIFIED (68,953 lines, zero diff) |
| Stage 2 emit-ir | ~1.90s |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, fixed point verified |
| Architecture | 9/10 | Clean optimization preserving architecture |
| Philosophy Alignment | 10/10 | Direct perf improvement per BMB principles |
| Test Quality | 9/10 | Full verification: Rust + golden + fixed-point |
| Documentation | 9/10 | Code comments + cycle log |
| Code Quality | 9/10 | Dead code (escape/unescape) left for reference |
| **Average** | **9.3/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | 3.6x gap remains (1.81s vs 0.50s) | Continue optimization |
| I-02 | L | Dead code: escape_field/unescape_field, step_temp/block/exit/work | Clean up in future |
| I-03 | M | make_step still uses 7 string concatenations | Consider integer packing |
| I-04 | M | gen_program_sb (396ms / 18%) not yet optimized | Profile in next cycle |
| I-05 | L | chr(1) calls in make_work create temp strings | Could cache separator |

## Next Cycle Recommendation
- Cycle 474: Profile remaining bottlenecks with updated timing
- Re-profile to see if lowering is still dominant or gen_program_sb is now larger share
- Consider further make_step optimization (integer packing for temp/block)
