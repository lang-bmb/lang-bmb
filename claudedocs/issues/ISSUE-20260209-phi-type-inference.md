# Phi Type Inference Bug for Mixed ptr/i64 Values

## Summary
The Rust compiler's LLVM codegen (`llvm.rs`) incorrectly infers phi node types
when branches produce mixed pointer and integer values.

## Reproduction
```bmb
fn main() -> i64 = {
    let v = getenv("BMB_RUNTIME_PATH");
    let dir = if v.len() > 0 { v } else { "default" };
    println(dir.len());
    0
};
```

## Error
```
panicked at bmb\src\codegen\llvm.rs:2351:74:
Found IntValue but expected PointerValue variant
```

## Root Cause
In `infer_phi_type()` (llvm.rs:1870-1921), the algorithm tracks the "largest
integer type" by bit width. When encountering:
- Branch 1: `v` (a `ptr` from getenv)
- Branch 2: `"default"` (a string literal, also `ptr`)

One branch's value may already be in SSA as `i64` (from an earlier `ptrtoint`),
while the other is a fresh `ptr`. The phi type defaults to `i64` due to the
integer width priority logic, causing type mismatch when the result is used
as a pointer.

## Fix Proposal
In `infer_phi_type()`, if any operand has pointer type, the phi should be
pointer type (since strings are always pointers). Add explicit check:

```rust
// If any value is a pointer type, phi should be pointer
if let BasicTypeEnum::PointerType(_) = ty {
    return Ok(ty);
}
```

## Workaround
Pass string values through function parameters instead of using if-else
that produces mixed types. Used for the bootstrap build command.

## Severity
Medium - affects programs using getenv/system with conditional defaults.

## Files
- `bmb/src/codegen/llvm.rs:1870-1921` (infer_phi_type)
- `bmb/src/codegen/llvm_text.rs:944-952` (phi type widening match)

## Status: RESOLVED (v0.90.62, Cycle 432)

### Fix Applied
1. **llvm.rs**: Already fixed in v0.90 — pointer type takes priority via explicit check
2. **llvm_text.rs**: Fixed in v0.90.62 — reordered match arms so `ptr` takes priority
   over `i64` in both phi type inference (line 948) and binop type inference (line 907)
3. **Integration tests**: 5 string conditional branch tests added (integration.rs)
