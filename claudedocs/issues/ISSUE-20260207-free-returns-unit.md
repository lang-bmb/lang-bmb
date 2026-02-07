# ISSUE: `free()` returns unit instead of i64, causing confusing type errors

**Date**: 2026-02-07
**Severity**: MEDIUM
**Component**: Interpreter / Builtins
**Found during**: Phase 1 Dogfooding (bmb-hashmap)

## Description

The `free()` builtin function returns `()` (unit type) instead of `i64`. This causes confusing `expected (), got i64` type errors when `free()` is used in if/else branches alongside i64 expressions, which is a very common pattern in manual memory management.

## Reproduction

```bmb
fn cleanup(ptr: i64, should_free: i64) -> i64 = {
    if should_free > 0 {
        free(ptr)         // Returns (), not i64
    } else {
        0                 // Returns i64
    }
    // ERROR: expected (), got i64 (branch type mismatch)
};
```

The error message is misleading because it doesn't point to `free()` as the source of the unit type.

## Current Workaround

```bmb
fn free_and_zero(ptr: i64) -> i64 = {
    let _f = free(ptr);
    0
};
```

Every library using manual memory management must define this wrapper.

## Impact

- Confusing error messages that don't identify the root cause
- Every memory-management library needs a `free_and_zero` wrapper
- Inconsistent with `malloc`, `store_i64`, `load_i64` which all return i64

## Suggested Fix

Change `free()` to return `i64` (0) for consistency with other memory builtins. In `bmb/src/interp/eval.rs` line ~2199, change `Ok(Value::Unit)` to `Ok(Value::Int(0))`.

## Priority

MEDIUM - Affects all manual memory management code. The workaround exists but adds unnecessary boilerplate to every library.
