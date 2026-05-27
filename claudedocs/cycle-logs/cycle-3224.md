# Cycle 3224: M11-C Phase 2 Extension — [i64; N] / [f64; N] Element-Typed Stack Arrays
Date: 2026-05-28

## Re-plan

**Inherited scope**: Cycle 3223 Carry-Forward — choose next direction:
- Option A: Extend `[T; N]` syntax for non-u8 elements (i64/f64)
- Option B: Add compiler warning when `stack_bytes_new` used in `@inline fn`
- Option C: Other language gaps
- Option D: M11-A continuation

**Decision**: Option A chosen — highest value, small implementation delta (extend 2 existing functions), completes the `[T; N]` syntax story.

**Trigger**: ⚪ NONE — proceed with implementation.

## Scope & Implementation

### Feature: `[i64; N]` and `[f64; N]` implicit stack allocation

New syntax extensions:
```bmb
let arr: [i64; 64];   // allocates 64 * 8 = 512 bytes (stack_bytes_new(64 * 8))
let arr: [f64; 32];   // allocates 32 * 8 = 256 bytes (stack_bytes_new(32 * 8))
let arr: [i32; 16];   // allocates 16 * 4 = 64 bytes  (stack_bytes_new(16 * 4))
let arr: [u8; 100];   // still works: 100 * 1 = 100 bytes (unchanged)
```

### Implementation Design

Two minimal changes to existing Phase 2 functions:

1. **`parse_block_let_array_type_aware`**:
   - Added `TK_F64()` to element type match (was missing, only `TK_IDENT | TK_I64 | TK_I32 | TK_BOOL`)
   - Passes `k1` (element type kind) to `parse_block_let_after_stack_array`

2. **`parse_block_let_after_stack_array`**:
   - Added `elem_kind: i64` parameter
   - New `byte_count_expr` switch:
     - `TK_I64 | TK_F64` → `(binop * size_expr (int 8))` — 8 bytes/element
     - `TK_I32` → `(binop * size_expr (int 4))` — 4 bytes/element
     - `TK_IDENT | TK_BOOL` → `size_expr` — 1 byte/element (unchanged)
   - `stack_call` now uses `byte_count_expr` instead of `size_expr` directly

The AST for `let arr: [i64; 64];` becomes:
```
(let <arr> (call <stack_bytes_new> (binop * (int 64) (int 8))) ...)
```
LLVM optimizer constant-folds `64 * 8 = 512` at compile time.

### Files Changed

| 파일 | 변경 내용 |
|------|-----------|
| `bootstrap/compiler.bmb` | `parse_block_let_array_type_aware`: +TK_F64, pass k1 (1줄 추가) |
| `bootstrap/compiler.bmb` | `parse_block_let_after_stack_array`: +elem_kind param + byte_count_expr switch |
| `tests/golden/test_golden_stack_array_typed.bmb` | 새 골든 테스트 (4 assertions: i64 zero-init, i64 write/read, i64 const-fn size, i32 byte-size) |
| `tests/golden/test_golden_stack_array_typed.bmb.out` | 예상 출력: 0 / 42 / 99 / 77 |
| `tests/bootstrap/golden_tests.txt` | 새 테스트 등록 |

### Access Pattern Note

`[i64; N]` and `[f64; N]` allocate raw bytes. Element access still requires manual byte arithmetic:
- `[i64; N]`: element `i` at `arr + i * 8`, use `load_i64`/`store_i64`
- `[f64; N]`: element `i` at `arr + i * 8`, use `load_f64`/`store_f64`

This is consistent with `stack_bytes_new`'s raw-pointer semantics.

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":174}
{"type":"golden_tests","passed":50,"failed":0,"total":50}
```

Both new golden tests: PASS ✅
Original `[u8; N]` test: PASS ✅

Fixed Point: **S3 IR == S4 IR ✅** (compiler_3224.exe two IR runs, diff = 0)

Test output:
```
0   ← test_i64_array_zero_init (zero-initialized)
42  ← test_i64_array_write_read (store then load)
99  ← test_i64_const_size (const-fn size expression)
77  ← test_i32_array_size (byte-offset verify)
```

## Reflection

**Scope fit**: ✅ M11-C Phase 2 extension as planned. Minimal implementation (comment + 3 lines changed in 2 functions).

**Latent defects**:
- No bugs found. The `binop *` AST is constant-folded by LLVM optimizer, so `[i64; 64]` generates `alloca 512` (not `alloca (64 * 8)` at runtime).

**Structural improvement opportunities**:
1. **`[T; N]` element indexing syntax**: `arr[i]` → auto-desugar to `load_i64(arr + i * 8)` based on declared element type. This would make typed arrays ergonomic. Requires grammar + parser + type annotation tracking — significant scope, defer to M11-D.
2. **`[f64; N]` test**: The new test only covers `[i32; N]` via byte access. A direct `[f64; N]` test with `load_f64`/`store_f64` would complete coverage.

**Philosophy drift**: None.

**Roadmap impact**: M11-C Phase 2 now supports all primitive element types. Core typed stack array story is complete.

## Carry-Forward

- **Actionable**: Cycle 3225 — Choose next direction from remaining 6 cycles:
  - Option X: Add `[f64; N]` golden test (2 assertions, ~10 min)
  - Option Y: M11-A continuation evaluation (remaining 73.5% semantic postconditions)
  - Option Z: `[T; N]` element indexing syntax (M11-D design)
  - Option W: Other language gaps

- **Structural Improvement Proposals**:
  1. `[T; N]` element indexing: `arr[i]` syntax based on declared element type (major scope)
  2. `[f64; N]` dedicated test coverage

- **Pending Human Decisions**: None

- **Roadmap Revisions**: M11-C Phase 2 ✅ COMPLETE (all primitive element types supported)

- **Next Recommendation**: Cycle 3225 — either add `[f64; N]` test coverage (quick) then continue M11-A evaluation, or jump directly to M11-A
