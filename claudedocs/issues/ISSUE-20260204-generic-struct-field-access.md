# Generic Struct Field Access Bug

**Status: OPEN**
**Severity: High**
**Discovered: 2026-02-04**

## Summary

Accessing fields of generic structs returns incorrect values. The first field (`fst`) returns the value of the second field (`snd`).

## Reproduction

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

**Expected Output:**
```
1
2
```

**Actual Output:**
```
2
2
```

## Non-Generic Works

The same code with concrete types works correctly:

```bmb
struct IntPair {
    fst: i64,
    snd: i64,
}

fn main() -> i64 = {
    let p = new IntPair { fst: 1, snd: 2 };
    println(p.fst);  // Prints 1 (correct)
    println(p.snd);  // Prints 2 (correct)
    0
};
```

## Analysis

The bug likely occurs in:
1. Generic struct monomorphization (bmb/src/types/generics.rs)
2. Field offset calculation for generic structs (bmb/src/mir/ or bmb/src/codegen/)

## Impact

- Generic types like `Pair<A, B>`, `Option<T>`, `Result<T, E>` are broken
- Blocks Phase 1 standard library completion

## Workaround

Use non-generic structs with concrete types.

## Files to Investigate

- `bmb/src/types/generics.rs` - Generic instantiation
- `bmb/src/mir/mod.rs` - MIR struct field access
- `bmb/src/codegen/llvm.rs` - LLVM IR struct generation
