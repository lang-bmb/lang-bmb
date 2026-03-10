# i64 Function Parameters Incorrectly Generated as i32

**Status: RESOLVED (v0.60.250)**

## Summary
Function parameters declared as `i64` in BMB were sometimes incorrectly generated as `i32` in LLVM IR, causing type mismatch bugs when storing/loading values.

## Root Cause
The `LoopBoundedNarrowing` optimization pass in `bmb/src/mir/optimize.rs` was narrowing i64 parameters to i32 when it couldn't detect their use in 64-bit store operations. The `is_used_as_i64_store_value()` function only checked for `MirInst::IndexStore` but not for `store_i64()` function calls.

## Fix (v0.60.250)
Added detection of `store_i64`/`bmb_store_i64` function calls to prevent incorrect narrowing:

```rust
// bmb/src/mir/optimize.rs lines 4422-4430
MirInst::Call { func: callee, args, .. } => {
    if (callee == "store_i64" || callee == "bmb_store_i64") && args.len() >= 2 {
        let value_derived = matches!(&args[1], Operand::Place(p) if derived.contains(&p.name));
        if value_derived {
            return true;
        }
    }
}
```

## Verification
All affected packages now work correctly:
- bmb-hash: ✅ Tests pass (999)
- bmb-ptr: ✅ Tests pass (999)
- bmb-sort: ✅ Tests pass (999)

## Original Report

### Reproduction
```bmb
fn test_func(m: i64, capacity: i64) -> i64 = {
    let _s = store_i64(m + 16, capacity);
    load_i64(m + 16)
};

fn main() -> i64 = {
    let m = malloc(24);
    test_func(m, 64)  // Previously returned garbage, now returns 64
};
```

### Generated LLVM IR (Before Fix)
```llvm
define i64 @test_func(i64 %0, i32 %1) #5 {  ; capacity was i32
entry:
  store i32 %1, ptr %inttoptr, align 4     ; stored 4 bytes
  %load = load i64, ptr %inttoptr2, align 4  ; loaded 8 bytes - TYPE MISMATCH
```

### Generated LLVM IR (After Fix)
```llvm
define i64 @test_func(i64 %0, i64 %1) #5 {  ; capacity is now i64
entry:
  store i64 %1, ptr %inttoptr, align 8     ; stores 8 bytes
  %load = load i64, ptr %inttoptr, align 8  ; loads 8 bytes - matches
```
