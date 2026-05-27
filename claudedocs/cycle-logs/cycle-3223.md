# Cycle 3223: M11-C Phase 2 — [u8; N] Implicit Stack Array Syntax
Date: 2026-05-28

## Re-plan

**Inherited scope**: Cycle 3222 Carry-Forward — investigate M11-C Phase 2 (`[u8; N]` type
annotation parser support).

**Trigger**: ⚪ NONE — proceed with implementation. Design analysis showed it's achievable
as a parser-only change with no MIR/codegen changes needed.

## Scope & Implementation

### Feature: `let name: [u8; N];` implicit stack allocation

New syntax: `let buf: [u8; N];` (without explicit `= value`) auto-generates `stack_bytes_new(N)`.

```bmb
// Before (still works):
let tape = stack_bytes_new(tape_size());

// After (new ergonomic syntax):
let tape: [u8; tape_size()];  // equivalent — zero-initialized stack array
let tape: [u8; 30000];        // with literal size
let mut buf: [i64; 64];       // mutable variant
```

### Implementation Design

Parser-only change. The `[u8; N]` annotation already parsed but N was discarded (skipped).
Now `parse_block_let_array_type_aware` captures N:

1. Detects `[elem_type; N]` pattern after `:` in `let` binding
2. Parses N as a full expression (supports literals AND function calls)
3. After `]`, checks for:
   - `=` → explicit value, use `parse_block_let_value` normally  
   - `;` → implicit, auto-generate `(call <stack_bytes_new> N_expr)`
4. Falls back to old skip behavior for non-standard array types

### Key Bug Found During Implementation

Initial attempt used `"(call @stack_bytes_new ..."` — WRONG AST format.
The bootstrap compiler's AST uses:
- `(call <user_fn_name> args...)` — angle brackets for ALL function names
- `lower_call_args_sb` ADDS the `@` prefix when emitting MIR

Fix: `"(call <stack_bytes_new> ..."` — angle bracket format ✓

### Files Changed

| 파일 | 변경 내용 |
|------|-----------|
| `bootstrap/compiler.bmb` | `parse_block_let_array_type_aware` + `parse_block_let_after_stack_array` (새 함수 2개) |
| `bootstrap/compiler.bmb` | `parse_block_let_skip_type`: `[` → `parse_block_let_array_type_aware` (1줄 변경) |
| `tests/golden/test_golden_stack_array.bmb` | 새 골든 테스트 (4 assertions) |
| `tests/golden/test_golden_stack_array.bmb.out` | 예상 출력 |
| `tests/bootstrap/golden_tests.txt` | 새 테스트 등록 (2863번째) |

### Grammar Note: Rust Interpreter Incompatibility

`let x: [u8; N];` syntax is NOT supported by the Rust LALRPOP parser (`bmb run`).
Golden tests use the bootstrap compiler binary, which IS updated. This is consistent
with Rule 6 (Rust compiler frozen, new features in bootstrap only).

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":141,"failed":0}
```

Fixed Point: **S3 IR == S4 IR ✅** (compiler_3223b.exe two runs, diff = 0)

Test output:
```
0   ← test_zero_init (zero-initialized)
99  ← test_write_read (store then load)
77  ← test_const_size (const-fn expression size)
0   ← test_explicit_init (explicit = stack_bytes_new still works)
```

## Reflection

**Scope fit**: M11-C Phase 2 as designed. Concise implementation (2 new functions + 1 line change).

**Latent defects**: 
- `let x: [u8; N];` is only for bootstrap compiler — Rust interpreter will reject it.
  This is by design (Rule 6), but users who use `bmb run` vs `bmb build` may be confused.
  A compiler warning or better error message could help.
- Fallback to old skip behavior for non-`[elem_type; N]` patterns means complex array types
  like `[String; N]` or `[f64; N]` don't get auto stack alloc (could be added later).

**Structural improvement opportunities**:
- `[f64; N]` pattern: 8 bytes per element → `alloca [N x double] + memset`
- `[i64; N]` pattern: 8 bytes per element → `alloca [N x i64] + memset`
- Currently the helper generates `stack_bytes_new(N)` which allocates N BYTES, so for
  multi-byte elements the caller needs to multiply: `[u8; N * 8]` for i64 array

**Philosophy drift**: None — proper language feature implementation.

**Roadmap impact**: M11-C Phase 2 ✅ COMPLETE.

## Carry-Forward

- **Actionable**: Cycle 3224 — Choose next direction from remaining cycles (7 more)
  - Option A: Extend `[T; N]` syntax for non-u8 elements (i64/f64)
  - Option B: Add compiler warning when `stack_bytes_new` used in `@inline fn`
  - Option C: Other language gaps (closure, generics)
  - Option D: M11-A continuation evaluation
- **Structural Improvement Proposals**:
  1. `[i64; N]` type annotation: auto-generate `stack_bytes_new(N * 8)`
  2. `[f64; N]` type annotation: auto-generate `stack_bytes_new(N * 8)`
  3. Compiler warning when `stack_bytes_new` in `@inline fn` body
- **Pending Human Decisions**: None
- **Roadmap Revisions**: M11-C Phase 2 ✅ COMPLETE
- **Next Recommendation**: Cycle 3224 — Extend element type support OR add @inline warning
