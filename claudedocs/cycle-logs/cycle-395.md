# Cycle 395: LLVM codegen tests — instruction + terminator coverage

## Date
2026-02-13

## Scope
Add LLVM codegen unit tests covering MIR instruction codegen (copy, const, unary, cast, phi) and terminator codegen (goto, switch).

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Tests (10 new)
| Test | Description |
|------|-------------|
| test_generate_ir_copy_instruction | Copy place-to-place instruction |
| test_generate_ir_const_bool | Bool constant (true) |
| test_generate_ir_const_float | Float constant (3.14, double) |
| test_generate_ir_unary_neg | Integer negation (sub/neg) |
| test_generate_ir_unary_not | Boolean NOT (xor) |
| test_generate_ir_goto_terminator | Unconditional branch (br label) |
| test_generate_ir_switch_terminator | Switch terminator with cases + default |
| test_generate_ir_multiplication | Integer multiplication (mul) |
| test_generate_ir_cast_i64_to_f64 | Integer-to-float cast (sitofp) |
| test_generate_ir_phi_instruction | Phi node from branch merge |

## Test Results
- Unit tests: 2181 passed (LLVM codegen: 63 with --features llvm)
- Main tests: 15 passed
- Integration tests: 2179 passed
- Gotgan tests: 23 passed
- **Total: 4398 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass |
| Architecture | 10/10 | Uses existing test helpers |
| Philosophy Alignment | 10/10 | Verifies codegen correctness |
| Test Quality | 10/10 | Covers instructions + terminators |
| Code Quality | 10/10 | Clean, clippy-clean |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 396: New lint rule — double negation detection
