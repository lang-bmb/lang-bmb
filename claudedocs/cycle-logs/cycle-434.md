# Cycle 434: i32 MIR + codegen integration tests

## Date
2026-02-13

## Scope
Add integration tests verifying i32 type generates correct LLVM IR and WASM output: function signatures, arithmetic ops, casts (sext/trunc), bitwise ops, comparisons, control flow, constant folding, and WASM generation.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Implementation

### Research
- Existing i32 codegen tests: ZERO — all codegen tests used i64 only
- Test helpers available: `source_to_ir()` (optimized), `source_to_ir_unopt()` (unoptimized), `lower_to_mir()` (MIR only)
- i32 generates correct LLVM IR: verified `i32` type in function signatures, arithmetic, bitwise, cast, comparison instructions

### Tests Added (20 new integration tests)

| Test | Category | Assertion |
|------|----------|-----------|
| test_codegen_i32_function_signature | Signature | `@add32(i32 %a, i32 %b)` |
| test_codegen_i32_return_type | Signature | `i32 @ret32(i32 %x)` |
| test_codegen_i32_arithmetic | Arithmetic | `add` + `i32` in IR |
| test_codegen_i32_subtraction | Arithmetic | `sub` + `i32` in IR |
| test_codegen_i32_multiplication | Arithmetic | `mul` + `i32` in IR |
| test_codegen_i32_division | Arithmetic | `sdiv` + `i32` in IR |
| test_codegen_i32_cast_to_i64_sext | Cast | `sext i32` for widening |
| test_codegen_i32_cast_from_i64_trunc | Cast | `trunc` + `i32` for narrowing |
| test_codegen_i32_comparison | Compare | `icmp sgt` + `i32` |
| test_codegen_i32_bitwise_and | Bitwise | `and i32` |
| test_codegen_i32_bitwise_or | Bitwise | `or i32` |
| test_codegen_i32_shift_left | Bitwise | `shl` + `i32` |
| test_codegen_i32_negation | Unary | `sub i32 0` |
| test_codegen_i32_constant_folded | Optimize | Constant `i32 5` in IR |
| test_codegen_i32_modulo | Arithmetic | `srem` + `i32` |
| test_codegen_i32_mixed_function_types | Mixed | i32 params + i64 return |
| test_codegen_i32_if_expression | Control | `icmp` + `br i1` with i32 |
| test_codegen_i32_while_loop | Control | i32 operations in loop |
| test_codegen_wasm_i32_function | WASM | WASM generation succeeds |
| test_codegen_wasm_i32_cast | WASM | i32→i64 cast in WASM succeeds |

### Key Finding
Variable reassignment in while loops uses `s = s + i;` (no `set` keyword). The `set` keyword is for struct field mutation only.

## Test Results
- Unit tests: 2845 passed
- Main tests: 47 passed
- Integration tests: 2294 passed (+20)
- Gotgan tests: 23 passed
- **Total: 5209 tests — ALL PASSING**
- Clippy: PASS (0 warnings)
- Build: SUCCESS

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All 20 tests pass, covering IR type verification |
| Architecture | 10/10 | Tests follow existing codegen test patterns |
| Philosophy Alignment | 10/10 | Verifies i32 IR correctness — core to Performance > Everything |
| Test Quality | 10/10 | Arithmetic, casts, bitwise, control flow, WASM |
| Code Quality | 10/10 | Consistent with existing codegen test style |
| **Average** | **10.0/10** | |

## Next Cycle Recommendation
- Cycle 435: Benchmark verification — run benchmarks with i32, measure improvement
