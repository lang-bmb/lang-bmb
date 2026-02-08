# ISSUE: `let` bindings disallowed inside while-loop imperative blocks

**Status**: PARTIALLY RESOLVED (v0.89.4, Cycle 42 - MIR scope fix; parser grammar still pending)
**Date**: 2026-02-07
**Severity**: HIGH
**Component**: Parser / Grammar + MIR Lowering
**Found during**: Phase 1 Dogfooding (bmb-hashmap)

## Description

Inside while-loop body `{ }` blocks, `let` bindings cause parser errors. Only bare expression sequences (`{ expr; expr; 0 }`) are accepted. This forces complex algorithms to use recursive function style instead of imperative while-loop style whenever intermediate bindings are needed.

## Reproduction

```bmb
fn main() -> i64 = {
    let mut i = 0;
    while i < 10 {
        let x = i * 2;    // ERROR: Unrecognized token '='
        i = i + 1;
        0
    };
    0
};
```

**Workaround (current)**: Rewrite as recursive functions.

```bmb
fn loop_at(i: i64) -> i64 =
    if i >= 10 { 0 }
    else {
        let x = i * 2;    // OK in regular block
        loop_at(i + 1)
    };
```

## Impact

- Forces all while-loop bodies to be trivial expression sequences
- Complex hash table operations (Robin Hood probing, resize) had to be rewritten recursively
- Recursive style may cause stack overflow for large datasets without TCO
- Makes BMB feel like a functional-only language in while contexts, contradicting the imperative block design

## Expected Behavior

`let` bindings should work inside while-loop `{ }` blocks identically to regular blocks:

```bmb
while condition {
    let x = compute();
    let y = transform(x);
    store_i64(addr, y);
    0
}
```

## Suggested Fix

Review `grammar.lalrpop` for the while-loop body production rule. It likely uses a restricted expression-sequence rule instead of the full block rule that supports `let` bindings.

## Priority

HIGH - This is a language expressiveness gap that affects every non-trivial while loop. Per CLAUDE.md Principle 2, workarounds are defects in a programming language.
