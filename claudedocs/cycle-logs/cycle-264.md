# Cycle 264: Fix Generic Enum Native Compilation (3 Codegen Bugs)
Date: 2026-03-30

## Inherited → Addressed
No defects from Cycle 263.

## Scope & Implementation
Generic enums (Option<T>, Result<T,E>) failed native compilation due to 3 codegen bugs:

**Bug 1**: EnumVariant place_types missing — `llvm_text.rs:1552`
- EnumVariant alloca produces `ptr`, but wasn't registered in `place_types`
- Fix: Added `MirInst::EnumVariant` case → `"ptr"` in `build_place_type_map`

**Bug 2**: Switch discriminant not loaded from enum pointer — `llvm_text.rs:7258`
- `switch i64 %enum_ptr` used ptr as i64 directly
- Fix: When discriminant is `ptr` type, load i64 from pointer before switch

**Bug 3**: Enum FieldAccess used `%struct.` (empty struct) GEP — `llvm_text.rs:5608`
- Enum variant extraction used struct-typed GEP with empty name
- Fix: Detect enum field (empty struct_name), use `i64` GEP with offset+1 (skip discriminant)

**Bug 4**: EnumVariant args not loaded from .addr for locals — `llvm_text.rs:5747`
- Local variables stored to `.addr` but enum variant emission read them directly
- Fix: Load locals from `.addr`, sext i32→i64 for enum slots

Files changed:
- `bmb/src/codegen/llvm_text.rs` — 4 fixes in codegen
- `tests/golden/test_golden_generic_native_enum.bmb` — new golden test

## Review & Resolution
- All 6,199 tests pass, no regressions
- Native compilation: `Option<T>`, `Result<T,E>` with monomorphized generic functions work
- Existing contract golden tests verified (no regression in non-generic enum codegen)

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Generic Pair struct native compilation has pre-existing issues (aggregate return type mismatch)
- Next Recommendation: Fix generic struct native compilation, then test with optimization (opt -O2)
