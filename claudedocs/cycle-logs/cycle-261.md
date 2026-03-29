# Cycle 261: Fix Generic TypeVar Inference for Nullary Variants
Date: 2026-03-30

## Inherited → Addressed
Continuing from Cycles 241-260 (generic enum support). No carry-forward defects.

## Scope & Implementation
**Bug found**: Inline `Option::None` passed to generic function caused "conflicting type inference" error.
- Root cause: `infer_type_args()` stored `TypeVar("T")` from nullary variant, then failed when concrete `i64` tried to override it
- Fix in `bmb/src/types/mod.rs:8773`: When existing inference is a TypeVar and new arg is concrete, concrete type wins
- Added golden test: `test_golden_generic_typevar_inference.bmb`

## Review & Resolution
- All 6,199 tests pass
- `unwrap_or(Option::None, 42)` now correctly infers `T = i64`
- No regressions

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: None
- Next Recommendation: Test generic bool support, nested generic structs, Result error patterns
