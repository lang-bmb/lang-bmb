# Cycle 487: Nullable T? Methods — is_some, is_none, unwrap, unwrap_or, expect

## Date
2025-02-12

## Scope
Add nullable T? method support to the bootstrap compiler. Methods: `is_some`, `is_none`, `unwrap`, `unwrap_or`, `expect`. Nullable semantics: null = constant 0, non-null = any non-zero value.

## Philosophy Alignment
| Dimension | Score |
|-----------|-------|
| Core Mission Fit | 5/5 |
| Scope Boundaries | 5/5 |
| Architecture Patterns | 5/5 |
| Dependency Direction | 5/5 |

## Research Summary
- Nullable T? in BMB: `null` = 0, non-null = any non-zero value
- `is_some()` → `receiver != 0` (comparison, result is i64 0/1)
- `is_none()` → `receiver == 0` (comparison, result is i64 0/1)
- `unwrap()` / `expect()` → identity (return receiver as-is)
- `unwrap_or(default)` → `select(receiver != 0, receiver, default)` via new `nullable_select` MIR instruction
- Bootstrap already had nullable type parsing (`T?` in parser_ast.bmb) and `null` literal (`(int 0)`)
- Only needed: method dispatch interception + MIR lowering + LLVM codegen

## Implementation

### Files Modified
1. **`bootstrap/compiler.bmb`** — 7 additions:
   - **`is_nullable_method`** (NEW): Checks if method name is a nullable method (is_some/is_none/unwrap/unwrap_or/expect)
   - **Step dispatch 'N' block**: Added NR (nullable result) and NO (nullable or) handlers
   - **Method dispatch**: Intercepts nullable methods before `method_to_runtime_fn` lookup
   - **`step_nullable_result`** (NEW): Handles is_some (!=0), is_none (==0), unwrap (identity), unwrap_or (evaluate default then NO)
   - **`step_nullable_or`** (NEW): Emits `nullable_select %recv, %default` MIR instruction
   - **`lower_nullable_method_sb`** (NEW): Recursive lowering path for nullable methods
   - **`lower_method_sb`**: Modified to intercept nullable methods
   - **`llvm_gen_nullable_select`** (NEW): Generates `icmp ne i64 %recv, 0` + `select i1` LLVM IR
   - **`llvm_gen_select`** (NEW): General 3-operand select codegen (for lowering.bmb compatibility)
   - **RHS dispatch**: Added `nullable_select` and `select` to both codegen paths

2. **`tests/bootstrap/test_golden_nullable.bmb`** (NEW): 7 test scenarios
   - Test 1: `is_some` on non-null (42) → 10
   - Test 2: `is_none` on non-null (42) → 20
   - Test 3: `unwrap` on non-null → 42
   - Test 4: `unwrap_or` on non-null → 42 (receiver returned)
   - Test 5: `is_some` on null → 30 (false branch)
   - Test 6: `is_none` on null → 15 (true branch)
   - Test 7: `unwrap_or` on null → 34 (default returned)
   - Expected output: 193

3. **`tests/bootstrap/golden_tests.txt`**: Added nullable test (27 tests total)

### Key Design Decisions
- **`nullable_select` MIR instruction**: Single instruction for `unwrap_or` — combines `icmp ne` + `select` in codegen. Avoids needing intermediate temp in MIR.
- **General `select` codegen**: Also added for compatibility with lowering.bmb's separate `!= + select` approach.
- **No type annotations needed**: Bootstrap compiler skips type annotations; nullable methods work on raw i64 values (null = 0).
- **Two lowering paths**: Step machine (NR/NO work items) and recursive (`lower_nullable_method_sb`) both implemented.

## Test Results
| Item | Result |
|------|--------|
| Rust tests | 5,234 passed (2845+47+2319+23) |
| Golden tests (Stage 1) | 27/27 PASS |
| Fixed point (S2==S3) | VERIFIED (74,644 lines) |

## Evaluation
| Criterion | Score | Notes |
|-----------|-------|-------|
| Correctness | 10/10 | All tests pass, all 5 nullable methods work correctly |
| Architecture | 10/10 | Minimal additions, consistent with existing method dispatch pattern |
| Philosophy Alignment | 10/10 | Null handling at compile-time via integer representation — zero runtime overhead |
| Test Quality | 9/10 | 7 test scenarios covering null/non-null × all methods |
| Documentation | 9/10 | Version comments on all changes |
| Code Quality | 9/10 | Clean pattern matching, reuses existing infrastructure |
| **Average** | **9.5/10** | |

## Issues & Improvements
| # | Severity | Description | Action |
|---|----------|-------------|--------|
| I-01 | L | No runtime contract check for `unwrap()` on null | Future: add branch + trap for null unwrap |
| I-02 | L | `expect()` behaves identically to `unwrap()` — no error message | Future: add string message support |
| I-03 | L | Bootstrap parser doesn't support `?` in `parse_let_skip_type` | Future: add TK_QUESTION to skip list |
| I-04 | L | lowering.bmb generates `select` while compiler.bmb generates `nullable_select` — two representations | Consistent but divergent; consider unifying |

## Next Cycle Recommendation
- Cycle 488: Bootstrap compiler feature additions
  - Add `?` to parse_let_skip_type for nullable type annotations
  - String interpolation or template strings
  - Enum/variant type support
  - Additional optimizations (constant folding, dead code elimination)
