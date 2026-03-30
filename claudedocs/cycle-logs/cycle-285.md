# Cycle 285: Fix Struct-in-Generic-Enum Native Codegen (i64→ptr coercion)
Date: 2026-03-30

## Inherited → Addressed
No inherited defects.

## Scope & Implementation
**Bug found**: `Option<Entry<T>>` (struct inside generic enum) failed native compilation.

**Root cause**: Enum slots store values as i64. When extracting a struct (ptr) from an enum slot, the value is i64 but the phi expects ptr. Missing phi coercion for i64→ptr.

**Fix**: `bmb/src/codegen/llvm_text.rs:2154` — Added `("i64", "ptr") | ("ptr", "i64")` to phi coercion detection. Added `inttoptr` and `ptrtoint` to coercion instruction selection.

- Also tested: 3-field generic struct (Triple), mixed-type Config struct — all work
- Added golden test: `test_golden_generic_struct_in_enum.bmb`

## Review & Resolution
- All 6,199 tests pass
- Generic struct + enum patterns work in both interpreter and native
- No regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Test more complex generic patterns (nested enums, generic in generic)
