# Cycle 1826: Codebase Cleanup + Optimization Review
Date: 2026-03-10

## Inherited → Addressed
From 1825: Continue optimization work. Evaluate remaining MIR/codegen improvement opportunities.

## Scope & Implementation

### phi_operands_equal Simplification
Simplified `phi_operands_equal` function using the newly-derived `PartialEq` on `Operand` — reduced 15 lines of manual comparison to `a == b`.

### Optimization Opportunity Assessment
Conducted thorough analysis of remaining optimization opportunities:

1. **MIR CSE**: Per-block only (cross-block requires dominator analysis). LLVM GVN handles cross-block.
2. **LICM**: Only hoists pure function calls. LLVM LICM handles general cases.
3. **IfElseToSelect**: Mature, handles 1-3 instruction branches.
4. **ConditionalIncrementToSelect**: Mature, eliminates branch for counter patterns.
5. **nsw flags**: Already on add/sub/mul. Can't add to sdiv/srem (doesn't help).
6. **fast flags**: Already on all float ops (fadd/fsub/fmul/fdiv fast).
7. **Dead address arithmetic alongside GEP**: Confirmed LLVM DCE removes it — no runtime impact.

### Conclusion
The MIR optimizer has 23 passes, all well-implemented. The LLVM codegen emits near-optimal IR with comprehensive attributes, metadata, and flags. The 2 remaining benchmark FAILs are toolchain limitations, not codegen deficiencies. Further MIR/codegen optimization work produces diminishing returns.

### Files Changed
- `bmb/src/mir/optimize.rs` — Simplified `phi_operands_equal` using PartialEq

## Review & Resolution
- All 6,186 tests pass
- No regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Version bump and commit
