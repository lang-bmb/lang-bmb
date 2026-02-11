# Cycle 220: ContractBasedOptimization, LICM, CPN Unit Tests

## Date
2026-02-11

## Scope
Add unit tests for ContractBasedOptimization, LoopInvariantCodeMotion, and ConstantPropagationNarrowing passes in mir/optimize.rs.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- ContractBasedOptimization: Eliminates redundant comparisons using precondition proven facts, simplifies branches
- LoopInvariantCodeMotion: Hoists pure function calls (len, char_at, ord, byte_at) from loop headers to entry blocks
- ConstantPropagationNarrowing: Narrows i64 params to i32 when called with small constants and has decreasing recursion
- Identified key rejection conditions: multiplication, div/mod flow, remaining self-recursive calls, no call sites

## Implementation

### MIR Optimize Tests (`bmb/src/mir/optimize.rs`)
Added 10 new tests across 3 optimization passes:

**ContractBasedOptimization (+4 tests)**
- `test_contract_no_preconditions_no_change`: No preconditions → no optimization
- `test_contract_lt_comparison_elimination`: x < 10 implies x < 20 → true
- `test_contract_branch_simplification_to_goto`: Branch on proven-true condition → simplified
- `test_contract_le_not_provable`: x >= 5 implies x <= 3 is FALSE

**LoopInvariantCodeMotion (+3 tests)**
- `test_licm_no_loop_no_change`: No loops → nothing to hoist
- `test_licm_hoists_byte_at_pure_call`: byte_at(s, 0) in loop → hoisted to entry
- `test_licm_phi_dependent_call_not_hoisted`: char_at(s, i) depends on phi → NOT hoisted

**ConstantPropagationNarrowing (+3 tests)**
- `test_cpn_no_narrow_with_multiplication`: Mul operation prevents narrowing
- `test_cpn_no_narrow_non_decreasing_recursion`: Non-decreasing recursion → no narrow
- `test_cpn_no_call_sites_no_narrow`: No constant call sites → no narrow

## Test Results
- Standard tests: 2581 / 2581 passed (+10 from 2571)
- Clippy: CLEAN (0 errors)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 9/10 | Follows established test patterns |
| Philosophy Alignment | 10/10 | Contract-based optimization is core BMB differentiator |
| Test Quality | 9/10 | Good mix of positive/negative cases |
| Code Quality | 9/10 | Clean, descriptive test names |
| **Average** | **9.4/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | LoopBoundedNarrowing (1303 LOC, 9 tests) still has low coverage ratio | Future cycle |
| I-02 | L | LICM only hoists pure function calls, not invariant BinOps | Design decision |

## Next Cycle Recommendation
- Add integration tests for compiler pipeline (end-to-end .bmb → optimization)
- Or add more tests for LoopBoundedNarrowing
