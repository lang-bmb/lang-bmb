# Cycle 281: Fix Array-in-Loop Negative Value Codegen Bug
Date: 2026-03-30

## Inherited → Addressed
**Inherited**: Pre-existing array-in-while-loop native codegen bug with negative values.

**Root cause**: ConstantPropagationNarrowing stores negative values as i32 (`alloca i32`), but array init loads them as i64 (`load i64, ptr %name.addr`), reading 4 bytes of garbage from beyond the 4-byte alloca.

**Fix**: `bmb/src/codegen/llvm_text.rs:5868` — When loading a narrowed local (i32/i1) into an i64 array, load as i32 first then `sext` to i64.

## Scope & Implementation
- Fixed array element initialization from narrowed locals
- Added golden test: `test_golden_array_negative_loop.bmb`
- Verified full stress test with generics + arrays + negative values

## Review & Resolution
- All 6,199 tests pass
- Array sum with negatives: interpreter = native = 54 ✅
- Full stress test (generics + arrays + contracts): all 9 outputs match ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Check for similar narrowing issues elsewhere (function args, struct fields)
