# If/Else Early Return Pattern Codegen Bug

**Status: OPEN**
**Priority: MEDIUM**
**Category: Compiler Bug**

## Summary
When `main` uses `if cond { ...; 0 } else { ...; 0 }` pattern with substantial work in each branch, the compiled binary returns incorrect results from the else branch. The interpreter produces correct output.

## Reproduction
```bmb
fn main() -> i64 = {
    let n = read_int();
    if n < 2 {
        let _p = println(0);
        0
    } else {
        // ... substantial work ...
        let _p = println(count);  // prints wrong value
        0
    }
};
```

Interpreter: correct. Compiled (bmb build --release): wrong value from else branch.

## Workaround
Use mutable state + single return:
```bmb
fn main() -> i64 = {
    let n = read_int();
    let mut result = 0;
    if n >= 2 {
        // ... work ...
        result = count
    } else { () };
    let _p = println(result);
    0
};
```

## Impact
- Multiple AI-Bench problems had to be restructured to avoid this pattern
- Affects any BMB program using if/else as expression in main with side effects
- Not a bootstrapping blocker (CLAUDE.md Rule 6 — Rust bug fix only if blocking bootstrap)

## Root Cause (Suspected)
LLVM IR codegen for if/else expression in main may incorrectly handle PHI nodes or branch targets when both branches contain I/O side effects + return value.

## Acceptance Criteria
- [ ] Root cause identified in codegen (llvm.rs or llvm_text.rs)
- [ ] Fix or workaround documented in BMB Reference
- [ ] Regression test added

## Context
Discovered during AI-Bench problem creation (Cycles 2270-2285). Documented but not fixed per Rule 6.
