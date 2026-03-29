# Cycle 266: Fix Enum Return Use-After-Free + Verify Optimized Compilation
Date: 2026-03-30

## Inherited → Addressed
Inherited from Cycle 265: generic_combinators test failed with garbage value in native compilation.

## Scope & Implementation
**Root cause**: Enum variants constructed inside functions that return them used `alloca` (stack).
Returning a ptr to stack memory = use-after-free.

**Fix**: Added escape analysis for enum variants in `llvm_text.rs:5736`.
- `check_struct_escapes()` (existing) detects if dest appears in return phi or call args
- If enum escapes → `malloc` instead of `alloca`
- If enum is local-only → `alloca` (zero overhead)

File: `bmb/src/codegen/llvm_text.rs:5743` — escape-based allocation for EnumVariant

## Review & Resolution
- All 6,199 tests pass, no regressions
- All 9 generic golden tests: interpreter = native output (verified byte-by-byte, only \r\n vs \n difference)
- opt -O2 compilation: all tests pass
- Memory safety: no more stack-escaped pointers for returned enums

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: Returned enum uses malloc (heap), not aggregate return — future optimization: pack small enums into {i64, i64} registers
- Next Recommendation: Add generic with while loops, recursive generic functions, larger programs
