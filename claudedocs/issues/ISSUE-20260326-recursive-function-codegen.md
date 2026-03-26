# Recursive Function with Mutable Parameters Codegen Issue

**Status: OPEN**
**Priority: MEDIUM**
**Category: Compiler Bug**

## Summary
Recursive functions that modify parameters via vec_set and then recurse produce garbage output in compiled binary. The interpreter is correct.

## Reproduction
```bmb
fn heapify(v: i64, n: i64, root: i64) -> i64 = {
    let mut largest = root;
    // ... find largest child ...
    if largest != root {
        // swap
        let _r = heapify(v, n, largest);  // recursive call
        ()
    } else { () };
    0
};
```

Compiled output: garbage values (e.g., `-8070449075963733112`).

## Workaround
Convert recursive functions to iterative using while loops:
```bmb
fn sift_down(v: i64, n: i64, start: i64) -> i64 = {
    let mut root = start;
    let mut done = 0;
    while done == 0 {
        // ... find largest, swap, update root ...
    };
    0
};
```

## Impact
- Heap sort had to be rewritten as iterative
- Any recursive function with vec mutations may be affected
- Not blocking bootstrapping (bootstrap compiler uses different patterns)

## Root Cause (Suspected)
Stack frame management for recursive calls may not preserve vec handle correctly, or LLVM optimization of recursive + side-effect patterns causes issues.

## Acceptance Criteria
- [ ] Minimal reproduction case
- [ ] Root cause identified
- [ ] Fix or documentation in BMB Reference

## Context
Discovered during AI-Bench problem 86_heap_sort creation (Cycle 2286-2305).
