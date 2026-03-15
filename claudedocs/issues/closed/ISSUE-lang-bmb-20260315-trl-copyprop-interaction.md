# TRL + CopyPropagation Interaction Bug: Incorrect Loop Termination

## Summary

When multiple tail-recursive functions coexist in the same compilation unit, the TailRecursiveToLoop (TRL) optimization produces loops that terminate early (after 2 iterations instead of N).

## Reproduction

```bmb
// File: test_repro.bmb
// This file has TWO tail-recursive functions: pobj_ms and jobj_find

fn pobj_ms(doc: i64, src: String, p: i64, obj: i64) -> i64 = {
    // ... parse one key-value pair ...
    if done { result }
    else if comma { pobj_ms(doc, src, next_pos, obj) }  // tail recursion
    else { result }
};

fn jobj_find(doc: i64, n: i64, key: String, src: String, idx: i64, cnt: i64) -> i64 =
    if idx >= cnt { -1 }
    else if match { value }
    else { jobj_find(doc, n, key, src, idx + 1, cnt) };  // tail recursion

fn main() -> i64 = {
    let msg = "{\"a\":\"1\",\"b\":\"2\",\"c\":\"3\",\"d\":\"4\"}";
    // ... parse and check count ...
    // Expected: 4 keys, Actual: 2 keys (native), 4 keys (interpreter)
};
```

## Key Findings

1. **Interpreter**: Correctly parses all keys (count=4)
2. **Native compilation**: Only parses 2 keys (count=2)
3. **Adding a dummy changing parameter** to pobj_ms fixes it
4. **Removing jobj_find** (the other tail-recursive function) fixes it
5. **The LLVM IR looks correct** — the bug is in MIR optimization, not codegen
6. **O0 and O3 both fail** — not an LLVM optimization issue

## Analysis

- TRL transforms each function independently with correct phi nodes
- The bug manifests only when BOTH functions are TRL-transformed in the same module
- CopyPropagation (29 applications in the test) is the likely culprit — it may incorrectly propagate values across TRL-generated loop iterations when multiple loops exist

## Workaround

Use direct string search (`str_find`) instead of JSON parser for critical dispatch paths. The JSON parser can be used in isolation (separate compilation unit) without issues.

## Priority

Medium — affects programs with 2+ tail-recursive functions that interact through shared data structures. The LSP server works with the workaround.

## Discovered

Cycle 1893-1894 (2026-03-15)
