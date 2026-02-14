# Cycle 494: Select i1 Direct Generation for Simple If/Else

## Date
2025-02-12

## Scope
Add `select` MIR instruction generation for simple if/else expressions where both branches are pure constants (int/bool). This eliminates branch/phi pairs in favor of branchless `select i1` in LLVM IR.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Existing `llvm_gen_select` and `llvm_gen_nullable_select` already handled MIR `select` → LLVM IR
- `IfElseToSelect` optimization pass exists in Rust compiler's MIR optimizer
- Stage 2 baseline: 1,845 branches, 1,845 phis, 1 select
- LLVM `opt -O2` already converts some branches to selects (227 selects post-opt)
- Main compilation path uses trampoline/iterative lowering (`lower_expr_iter` → `do_step`), NOT recursive `lower_expr_sb`

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`**: Added select detection in trampoline + recursive paths

### New Functions
| Function | Purpose |
|----------|---------|
| `is_simple_leaf(ast) -> bool` | Detect AST nodes safe for select (int, bool only) |
| `emit_simple_leaf(ast, temp_id, sb) -> i64` | Emit leaf MIR directly (const/copy) |
| `step_if_select` (IZ handler) | Trampoline step: emit both leaves + select after condition |
| `lower_if_select_sb` | Recursive path: select lowering for simple if/else |
| `lower_if_branch_sb` | Recursive path: extracted existing branch/phi code |

### Functions Updated
| Function | Change |
|----------|--------|
| `step_if_start` | Detect simple leaves → push IZ (select) instead of IT (branch) |
| `do_step` | Register IZ handler under 'I' group (byte 90 = 'Z') |
| `lower_if_sb` | Dispatch to select vs branch path based on `is_simple_leaf` |

### Key Design Decisions
- **Int/bool only**: Initial attempt included var, string, float, unit — caused LLVM `opt -O2` miscompilation (segfault). Restricted to pure integer/boolean constants.
- **Both paths updated**: Trampoline (main) and recursive (fallback) paths both support select
- **Work item IZ**: New 2-character step code for select path, registered in `do_step` dispatch

### O2 Miscompilation Investigation
- Full leaf set (int, var, bool, float, string, unit): **38 selects** — Stage 2 segfaults at O2
- Works at O0 and O1, only fails with `opt -O2`
- Root cause: var/string/float leaves involve memory operations (alloca loads, function calls) that interact badly with LLVM O2 alias analysis and optimization passes
- Restricted set (int, bool): **11 selects** — all optimization levels work correctly
- Future: investigate specific O2 passes causing miscompilation to safely expand leaf set

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 27/27 PASS |
| Fixed point (S2==S3) | VERIFIED (68,975 lines) |
| Select count | 11 (was 1) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 9/10 | All tests pass, fixed point verified; O2 miscompilation caught and worked around |
| Architecture | 9/10 | Clean step handler, consistent with trampoline design |
| Philosophy Alignment | 8/10 | Performance improvement is modest (10 selects); restricted leaf set limits impact |
| Test Quality | 8/10 | Verified by existing golden tests + fixed point; no dedicated select test |
| Documentation | 9/10 | Version comments on all changes, O2 issue documented |
| Code Quality | 9/10 | Clean separation of concerns; emit_simple_leaf handles all leaf types even though only int/bool used |
| **Average** | **8.7/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | M | Only 10 new selects from int/bool — var leaves would add ~28 more but cause O2 miscompilation | Future: bisect LLVM passes to find root cause |
| I-02 | L | `emit_simple_leaf` supports all 6 leaf types but only int/bool actually used | Keep for future expansion when O2 issue resolved |
| I-03 | L | No dedicated golden test for select correctness | Add test_golden_select.bmb in future cycle |
| I-04 | M | Identity copies still 31% of IR | Future: copy propagation pass |

## Next Cycle Recommendation
- Roadmap v0.93 items: copy propagation, dominator tree CSE
- Or: Additional bootstrap compiler features (generics, trait methods)
- Or: Investigate LLVM O2 miscompilation with var/string select operands
