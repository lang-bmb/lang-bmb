# Generic Struct Field Access Bug

**Status: FIXED (v0.60.261)**
**Severity: High**
**Discovered: 2026-02-04**
**Fixed: 2026-02-04**

## Summary

Accessing fields of generic structs returned incorrect values. The first field (`fst`) returned the value of the second field (`snd`).

## Root Cause

In `bmb/src/mir/lower.rs`, generic struct parameters were not being registered in `var_struct_types`. This caused `field_index()` to return 0 for all fields.

**Location:** `lower_function()` - parameter type processing (lines 213-223)

**Problem Code:**
```rust
// Only handled Type::Named, not Type::Generic
if let Type::Named(struct_name) = &p.ty.node {
    ctx.var_struct_types.insert(p.name.node.clone(), struct_name.clone());
}
```

**Fix:**
```rust
// v0.60.261: Also handle generic struct types
} else if let Type::Generic { name: struct_name, .. } = &p.ty.node {
    if ctx.struct_defs.contains_key(struct_name) {
        ctx.var_struct_types.insert(p.name.node.clone(), struct_name.clone());
    }
}
```

## Reproduction (Before Fix)

```bmb
struct Pair<A, B> {
    fst: A,
    snd: B,
}

fn pair<A, B>(a: A, b: B) -> Pair<A, B> =
    new Pair { fst: a, snd: b };

fn fst<A, B>(p: Pair<A, B>) -> A = p.fst;
fn snd<A, B>(p: Pair<A, B>) -> B = p.snd;

fn main() -> i64 = {
    let p = pair(1, 2);
    let f = fst(p);  // Expected: 1, Actual: 2
    let s = snd(p);  // Expected: 2, Actual: 2
    println(f);      // Prints 2 (wrong!)
    println(s);      // Prints 2 (correct)
    0
};
```

**Before Fix Output:**
```
2
2
```

**After Fix Output:**
```
1
2
```

## Verification

The fix was verified with:
1. `test_generic_bug.bmb` - direct test case
2. `packages/bmb-core/src/lib.bmb` - generic `Pair<A, B>` now works correctly

## Changes

**File:** `bmb/src/mir/lower.rs`

1. Added `struct_type_params` to `TypeDefs` for generic struct type parameter tracking
2. Added `Type::Generic` handling in parameter registration
3. Added `Type::Generic` handling in `ast_type_to_mir_with_type_defs()` with proper monomorphization
4. Added helper functions: `substitute_type_vars()`, `type_to_suffix()`

## Impact

- Generic types like `Pair<A, B>`, `Option<T>`, `Result<T, E>` now work correctly
- Phase 1 standard library completion is unblocked
