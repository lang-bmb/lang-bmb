# Cycle 286: Fix Nested Generic Enum Native Codegen (ptr↔i64)
Date: 2026-03-30

## Inherited → Addressed
No inherited defects.

## Scope & Implementation
**Bug found**: `Option<Option<i64>>` (nested enums) failed native compilation.

**Root cause**: Inner enum's ptr was stored as-is into outer enum's i64 slot. The `store i64 %_t0` failed because `%_t0` is `ptr`.

**Fix 1**: `llvm_text.rs:5800` — Added `ptr` case to EnumVariant arg store: `ptrtoint ptr → i64` before storing.

**Fix 2**: `llvm_text.rs:5769` — Changed EnumVariant arg type inference to use `place_types` (which knows about EnumVariant ptrs) instead of `infer_operand_type` (which only knows MIR types).

**Also found**: Parser doesn't handle `>>` in nested generics (`Option<Option<i64>>`). Use `Option<Option<i64> >` with space. This is a known lexer ambiguity (>> vs > >).

## Review & Resolution
- All 6,199 tests pass
- 19 golden tests: all PASS (interpreter = native)
- Nested enums work natively

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: `>>` lexer ambiguity for nested generics (like C++ pre-C++11)
- Next Recommendation: Continue hardening, test Result<Entry<T>, i64>, generic with strings
