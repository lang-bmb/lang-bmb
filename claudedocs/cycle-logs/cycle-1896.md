# Cycle 1896-1897: Bootstrap nonnull Investigation + int_to_string Fix
Date: 2026-03-15

## Inherited → Addressed
- Phase 3 plan: Bootstrap nonnull attributes for String params/returns

## Scope & Implementation

### 1. Bootstrap nonnull Investigation
- **Finding**: Bootstrap compiler uses "String as i64" pointer model throughout
  - String params declared as `i64` (not `ptr`)
  - String values stored as `i64` (ptrtoint)
  - When calling runtime: `inttoptr i64 to ptr`
- **Attempted**: Changed `format_fn_params()` to emit `ptr nonnull` for String params
- **Result**: Breaks function body codegen — `inttoptr i64 %name to ptr` fails when `%name` is already `ptr`
- **Conclusion**: Adding nonnull to bootstrap user-function String params requires a full "String as ptr" migration of the entire codegen pipeline. This is a large architectural change, not a simple attribute addition.
- **Reverted**: All bootstrap nonnull changes

### 2. int_to_string Name Mapping Fix
- Fixed: `declare ptr @int_to_string` → `declare ptr @bmb_int_to_string` (text backend)
- Added: Name mapping `"int_to_string" => "bmb_int_to_string"` in codegen
- This prevents declaration conflict when bootstrap's own `int_to_string` is defined

### 3. Bootstrap Verification
- Stage 1: ✅ Builds and runs correctly
- Golden test (exit 42): ✅ Passes
- All 6,186 Rust tests: ✅ Pass

## Review & Resolution
- Bootstrap nonnull is NOT feasible as a simple attribute addition
- Requires architectural migration: "String as i64" → "String as ptr" across entire bootstrap codegen
- This is Phase 3's scope adjustment — SAE (Phase 3.2) can proceed independently

## EARLY TERMINATION
Phase 3.1 (bootstrap nonnull) is blocked by architectural constraint. Moving to other work.

## Carry-Forward
- **Bootstrap String Model**: The i64 pointer model prevents nonnull and other ptr-specific attributes. Migration to ptr model would touch ~2K+ lines in compiler.bmb
- Next Recommendation: Phase 3.2 (Bootstrap SAE) or return to Phase 2 LSP testing
