# Cycle 282: Audit Narrowing Issues Across Codegen
Date: 2026-03-30

## Inherited → Addressed
Audited all `load i64, ptr %*.addr` patterns for similar narrowing bugs.

## Scope & Implementation
- Found ~25 hardcoded `load i64` from `.addr` locations
- Most are in builtin functions (vec, hashmap, malloc, free, store_i64) where args are always i64 (pointers/sizes) — NOT at risk from narrowing
- malloc already has special narrowing handling (sext i32→i64)
- Array init was the only user-facing narrowing issue (fixed in Cycle 281)
- Enum variant args were already fixed (Cycle 264, load from .addr with sext)

## Review & Resolution
- No additional narrowing bugs found
- 6,199 tests pass

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Run benchmarks natively to find more codegen issues
