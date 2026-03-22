# Ownership & Borrowing in BMB

BMB uses a Rust-inspired ownership model to guarantee memory safety at compile time. This tutorial covers the core concepts.

## Key Idea

Every value has exactly one owner. When the owner goes out of scope, the value is cleaned up. References let you use values without taking ownership.

## References

### Immutable References (`&T`)

Use `&` to borrow a value without taking ownership:

```bmb
fn sum_array(arr: &[i64; 5], idx: i64, acc: i64) -> i64
    pre idx >= 0
= if idx >= 5 { acc }
  else { sum_array(arr, idx + 1, acc + arr[idx]) };

fn main() -> i64 = {
    let arr: [i64; 5] = [10, 20, 30, 40, 50];
    let total = sum_array(&arr, 0, 0);
    println(total);
    0
};
```

Multiple immutable references can coexist:

```bmb
let x: i64 = 42;
let r1: &i64 = &x;
let r2: &i64 = &x;   // OK — multiple immutable borrows
```

### Mutable References (`&mut T`)

Use `&mut` to borrow a value and modify it:

```bmb
let mut x: i64 = 42;
let r: &mut i64 = &mut x;
*r = 100;             // modify through reference
```

Only one mutable reference at a time:

```bmb
let mut x: i64 = 42;
let r1: &mut i64 = &mut x;
// let r2: &mut i64 = &mut x;  // ERROR — already mutably borrowed
```

### The XOR Rule

At any given time, you can have either:
- **Any number** of immutable references (`&T`), OR
- **Exactly one** mutable reference (`&mut T`)

Never both. This prevents data races and aliasing bugs at compile time.

## Raw Pointers (`*T`)

For low-level systems programming (linked lists, trees, FFI), BMB provides raw pointers:

```bmb
struct Node {
    value: i64,
    next: *Node
}

fn new_node(val: i64) -> *Node = {
    let n = malloc(16) as *Node;
    set n.value = val;
    set n.next = null;
    n
};
```

Pointer operations:
- `null` — null pointer literal
- `ptr as i64` — cast pointer to integer
- `addr as *T` — cast integer to pointer
- `ptr.field` — auto-dereferences (`(*ptr).field`)

## Contracts + Ownership = Zero Overhead

BMB's ownership model combines with contracts to eliminate runtime checks entirely:

```bmb
fn safe_get(arr: &[i64; 5], idx: i64) -> i64
    pre idx >= 0 and idx < 5
= arr[idx];
```

The `pre` condition proves bounds are valid at compile time. No runtime check needed. The `&` reference proves the array is valid. Together: **zero-overhead safe access**.

## Lifetime Elision

For functions with a single reference parameter, lifetimes are inferred automatically:

```bmb
// Lifetime inferred — return borrows from arr
fn first(arr: &[i64; 5]) -> &i64
    pre arr.len() > 0
= &arr[0];
```

For multiple reference parameters, use explicit lifetime annotations:

```bmb
fn longer<'a>(x: &'a String, y: &String) -> &'a String = x;
```

## Summary

| Concept | Syntax | Purpose |
|---------|--------|---------|
| Immutable reference | `&T`, `&value` | Read-only borrow |
| Mutable reference | `&mut T`, `&mut value` | Read-write borrow |
| Dereference | `*ref` | Access referenced value |
| Raw pointer | `*T` | Low-level memory access |
| Contracts | `pre`/`post` | Compile-time safety proofs |

BMB's philosophy: **Performance > Everything**. The ownership model isn't about safety for its own sake — it's about enabling the compiler to generate the fastest possible code by proving properties at compile time.
