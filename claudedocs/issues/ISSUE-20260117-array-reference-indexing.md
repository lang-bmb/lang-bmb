# Issue: Array Reference Indexing Not Supported

**Discovered**: 2026-01-17 during Zero-Overhead benchmark implementation
**Severity**: Medium (affects performance-critical code patterns)
**Component**: Type checker (`bmb/src/types/mod.rs`)
**Status**: ✅ RESOLVED (v0.50.26)

## Summary

Array indexing through references (`&[T; N]`) is not supported. Arrays must be passed by value, causing potential performance overhead for large arrays.

## Reproduction

```bmb
// This works: pass by value
fn get_element(arr: [i64; 10], idx: i64) -> i64
  pre idx >= 0 and idx < 10
= arr[idx];

// This FAILS: pass by reference
fn get_element_ref(arr: &[i64; 10], idx: i64) -> i64
  pre idx >= 0 and idx < 10
= arr[idx];  // Error: Cannot index into type: &[i64; 10]
```

## Impact

1. **Performance**: Large arrays must be copied when passed to functions
2. **Benchmark fairness**: BMB vs C comparison unfair (C passes pointer, BMB copies array)
3. **Philosophy conflict**: Zero-overhead safety goal compromised by unnecessary copying

## Root Cause

In `bmb/src/types/mod.rs:1998-2001`:
```rust
match expr_ty {
    Type::Array(elem_ty, _) => Ok(*elem_ty),
    Type::String => Ok(Type::I64),
    _ => Err(CompileError::type_error(...)),
}
```

The index operation only handles `Type::Array` directly, not `Type::Ref(Box<Type::Array>)`.

## Proposed Fix

Extend the type checker to support indexing through references:

```rust
match expr_ty {
    Type::Array(elem_ty, _) => Ok(*elem_ty),
    Type::Ref(inner) => match inner.as_ref() {
        Type::Array(elem_ty, _) => Ok(*elem_ty.clone()),
        _ => Err(...),
    },
    Type::String => Ok(Type::I64),
    _ => Err(...),
}
```

Also need corresponding MIR and codegen changes to emit pointer-based indexing.

## Philosophy Alignment

Per BMB's core philosophy:
- **Performance First**: This fix eliminates unnecessary array copies
- **Zero-Overhead Safety**: Contract-proven bounds check with reference semantics achieves true zero overhead

## Related

- BENCHMARK_MASTERPLAN.md Phase 1: Zero-Overhead Proof
- SPECIFICATION.md Section 2.1: Slice type `&[T]`

## Workaround

For now, use smaller arrays in benchmarks or accept the performance overhead. The benchmark code documents this limitation.

## Resolution (v0.50.26)

**Files Changed**:
- `bmb/src/types/mod.rs`: Extended index expression type checker to handle `Type::Ref(inner)`
- `bmb/src/interp/eval.rs`: Added dereference logic in `eval` and `eval_fast` for `Value::Ref`
- `bmb/tests/integration.rs`: Added 3 tests (`test_array_ref_index`, `test_string_ref_index`, `test_invalid_ref_index`)

**Implementation**:
```rust
// Type checker (types/mod.rs)
match &expr_ty {
    Type::Array(elem_ty, _) => Ok(*elem_ty.clone()),
    Type::Ref(inner) => match inner.as_ref() {
        Type::Array(elem_ty, _) => Ok(*elem_ty.clone()),
        Type::String => Ok(Type::I64),
        _ => Err(...),
    },
    Type::String => Ok(Type::I64),
    _ => Err(...),
}

// Interpreter (interp/eval.rs)
let derefed_val = match &arr_val {
    Value::Ref(r) => r.borrow().clone(),
    _ => arr_val,
};
```
