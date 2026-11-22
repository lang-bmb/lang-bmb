# 05 Ownership and references

This chapter describes the ownership of values: values of which types are copied and which are moved, the rule that a moved value cannot be used again, moves in branches and loops, the points of release, and the rules of references. References are made only as call arguments and received only as parameters. They are not held in local variables, fields or results, so there is no lifetime notation (ADR-0015). The types themselves are in chapter 03, proof obligations in chapter 07.

## Copy and move

Whether a type is copy or move is decided by its structure (ADR-0047). It is not marked on the declaration.

| Type | Value use |
| --- | --- |
| integer types, `f64`, `bool`, `char`, `()` | copy |
| arrays, structs and enums that hold only copy types (`Option<i64>`, `[u8; 4]`) | copy |
| `Vec<T>`, `String`, `Box<T>` | move |
| structs, enums and arrays that hold a move type as a field, payload, element or type argument | move |
| a type parameter `T` without a `Copy` bound | move |
| a type parameter with a `Copy` bound | copy. A move type cannot be its type argument (E0428) |
| `std::process::Child` | move |

- A value use is the initializer expression of a `let`, the right side of an assignment, an argument passed by value, a method call whose receiver is `self: T`, the value of a block or of an `if` or `match` arm, the value of a `return`, and a field value of a struct construction. A value of a move type moves away at that position, and a value of a copy type is copied.
- A read is not a move: `v.len()`, `v[i]`, `b.f`, `*b` (a value of a copy type), a `&v` argument, `v.as_slice()`.
- Cloning is `x.clone()` (ADR-0141). It works for any type whose value has memory: `Vec`, `String`, `Box`, and the structs, enums and arrays that hold them. The result is the same value as the original and has its own memory (the buffer, the box and the elements are all made anew). The receiver is only read. The effect is `alloc` and it cannot be used inside a specification (E0503). In a proof the result is the same value as the original.
- If the type has a method named `clone` (a function of its own `impl` or a method of a trait it implements), that one comes first.
- A clone that is not made is E0404: a type that holds in its name a type parameter without a `Copy` bound (`Vec<T>`, `Pair<T>`: the value may have memory and the type is not known here), a type that holds `std::process::Child` (one owner per resource), a copy type (it is copied by value, so remove the `.clone()`).
- Corpus: `tests/run/values.bmb`, `tests/run/vecs.bmb`, `tests/run/boxes.bmb`, `tests/run/generic_moves.bmb`, `tests/check/err/boxes.bmb`.

## Use of a moved value

Using a moved local variable or parameter again is an E0413 error. Assigning to it again (`let mut`) brings it back to owned.

```
#[effects(pure)]
fn take(v: Vec<i64>) -> usize { v.len() }
#[effects(alloc)]
fn twice() -> usize {
    let v: Vec<i64> = Vec::new();
    let n: usize = take(v);
    n + v.len()
}
```

The `v` of `n + v.len()` is an E0413 error. The program below is accepted.

```
#[effects(alloc)]
fn again() -> usize {
    let mut v: Vec<i64> = Vec::new();
    let n: usize = take(v);
    v = Vec::new();
    n + v.len()
}
```

- A `match` that binds a move payload moves the scrutinee. The operand of `?` is moved too. A use after that is an E0413 error.
- If a method whose receiver is `self: T` is called twice, the second call is an E0413 error. Calling a function value with an `FnOnce` bound twice is the same.
- Specification contexts (`assert`, `assume`, invariants, `proof` blocks) read a moved value. They neither look at nor change the move state (ADR-0098, `tests/verify/pass/moved_specs.bmb`).
- Corpus: `tests/check/err/moves.bmb`, `tests/check/err/move_traits.bmb`, `tests/check/err/child.bmb`, `tests/run/move_traits.bmb`.

## Moving out of a place and partial moves

What can be moved as a whole is a local variable, a parameter, and a field of a local struct.

| Place moved out of | Verdict |
| --- | --- |
| a field of a local struct `b.v` | partial move. Using that field again is E0413, using the whole struct is E0433, the other fields can be used |
| an element `g[0]` | E0413. Take it out with `pop`, or read it |
| `*r`, `*b` (a value of a move type) | E0413 (a value behind a `&mut` reference is E0239) |
| a field behind a shared reference or inside a `Box` `r.v` | E0413 |
| a field behind `&mut` `m.v` | E0239. Take it out by replacing it with `std::mem::replace(&mut m.v, Vec::new())` |

- Assigning again to a field that was moved out makes the struct whole. A field of a struct that was moved as a whole cannot be assigned to (E0413).
- A `match` on a field of a local struct (`match l.e`) is also a partial move of that field only.
- `std::mem::replace(&mut p, v)` returns the old value of `p` and puts `v` in `p`. `std::mem::swap(&mut a, &mut b)` exchanges the values of two places. If the two places are inside the same variable it is an E0410 error. There is no `std::mem::take` and no `drop(v)` (E0219).
- Corpus: `tests/run/owned_fields.bmb`, `tests/run/mem_moves.bmb`, `tests/check/err/owned_fields.bmb`, `tests/check/err/mem_moves.bmb`, `tests/check/err/nested_vecs.bmb`.

## Moves in branches and loops

Where paths join, the state of a value must be one. This is because the points of release are decided by the program text (ADR-0031, ADR-0081).

| Join point | When only one path moved |
| --- | --- |
| `if` .. `else`, the arms of a `match` | accepted. The arm that did not move releases the value at its end, and after the join the value is in the moved state |
| `if` without `else` | E0414 |
| the right side of `&&` or `\|\|` | E0414 |
| the body of a loop (`while`, `loop`, `for`) | E0414. Accepted if it is assigned again within the same iteration |
| the exit and the head of a loop, where `break` and `continue` gather | E0414 if the states differ |

```
#[effects(alloc)]
fn branch(c: bool) -> usize {
    let v: Vec<i64> = Vec::new();
    if c { take(v) } else { 0 }
}
```

- Using a value in the moved state after the join is an E0413 error (whichever arm it came through).
- A path that ends in `return` or in an expression of type `!` is left out of the join.
- Corpus: `tests/run/branch_drops.bmb`, `tests/run/loop_exits.bmb`, `tests/check/err/moves.bmb`, `tests/check/err/loop_exits.bmb`.

## Release

An owned value is released at the following points. The points are decided by the move check and the program does not write them itself (`drop(v)` is E0219).

- The end of a block: the locals declared in that block that are still owned (for a partially moved struct, only the remaining fields). At the end of a function, also the parameters received by value.
- Assignment: when a place that holds an owned value (a variable, a field, an element, `*b`) is assigned to, the old value.
- `return`, `?`, `break`, `continue`: the owned locals of the scopes that the exit leaves.
- An `if` or `match` where only one arm moved: the end of the other arms.
- The value of an expression statement that discards its value. A temporary that was only read (`mk().x`, a `&mk()` argument) at the end of that region.
- A struct is released in field order, an enum in the payload order of the variant, a `Vec` frees its buffer after releasing its elements, and a `Box` frees its memory after releasing its value. There are no user destructors, so the program cannot observe this order (the only exception is `std::process::Child`, which closes its pipes). What the tests fix is not the order but "every allocation is freed exactly once and freed memory is not touched" (`run_corpus_frees_every_allocation_once` in `compiler/tests/codegen.rs`).
- Corpus: `tests/run/branch_drops.bmb`, `tests/run/returns.bmb`, `tests/run/owned_arrays.bmb`, `tests/run/vec_tree.bmb`.

## References only in parameters

A reference type is used only as the type of a parameter.

| Form | What it points to |
| --- | --- |
| `&[T]`, `&mut [T]` | the elements of an array, a `Vec` or a slice |
| `&str` | text |
| `&T`, `&mut T` | scalars (ADR-0054), structs and enums, `Vec<T>`, `String`, `Box<T>`, type parameters |

- Using a reference in the type of a local variable, a return type, or the type of a field or payload is an E0403 error. It cannot be used as a type argument either (E0403; `Vec<&T>` and `Box<&T>` are E0402 by the rules for element and content types).
- Lifetime notation (`'a`) is an E0208 error. References do not leave the function, so it is not needed.
- `&[i64; 3]` (a reference to an array) and `&mut str` are not types (E0402). An array is passed as `&[T]`.
- In the body, a `&T` or `&mut T` parameter is read with `*x` (scalars), its fields are read with `x.f`, and if it is `&mut` it is written with `*x = v` and `x.f = v`.
- Corpus: `tests/run/scalar_refs.bmb`, `tests/run/swap.bmb`, `tests/check/err/types.bmb`, `tests/check/err/data_types.bmb`, `tests/check/err/places.bmb`.

## Reference arguments

`&e` and `&mut e` are used only when they come directly in the argument position of a call. Any other position (the right side of a `let`, `*(&x)`, `Some(&p)`) is an E0409 error.

| Parameter | Argument |
| --- | --- |
| `&[T]` | `&arr`, `v.as_slice()`, `&v[lo..hi]` (integer elements), a received `&[T]` or `&mut [T]` parameter |
| `&mut [T]` | `&mut arr`, `v.as_mut_slice()`, a received `&mut [T]` parameter |
| `&str` | a literal, `s.as_str()`, `&s[a..b]`, a received `&str` parameter |
| `&T` | `&e` (any value will do), `&*b`, a received `&T` or `&mut T` parameter |
| `&mut T` | `&mut place` (a variable, a field `p.x`, an element `a[0]`, `*b`), a received `&mut T` parameter |

- `v.as_slice()`, `v.as_mut_slice()`, `s.as_str()` and string literals are also an E0409 error outside argument position (a literal also comes as an operand of a comparison).
- `&mut v[lo..hi]` is an E0409 error. Subslices are read-only.
- Passing a `Vec` or a `String` as `&v` or `&s` in the position of a slice or `&str` is an E0218 error. Passing a value in a reference position or a reference parameter in a value position, or mismatching `&` and `&mut`, is an E0401 error. A borrow expression written in a value position (`id(&x)`) is E0409.
- A method call `x.f(..)` passes `&x`, `&mut x` or `x` according to the declared receiver (ADR-0051). If `x` is already a reference it is passed as it is.
- Corpus: `tests/run/methods.bmb`, `tests/check/err/types.bmb`, `tests/check/err/subslices.bmb`, `tests/check/err/vec.bmb`, `tests/check/err/strings.bmb`.

## Writable places

The left side of an assignment, a `&mut` argument, the receiver of a mutating method (`push`, `pop`, `truncate`, `push_str`, `copy_within`), and the receiver of a method whose receiver is `self: &mut T` must be a writable place. Otherwise it is an E0408 error.

- A writable place is a `let mut` local, the value a `&mut` parameter points to, and the paths of fields, elements and `*b` below them.
- Not writable: a `let` local, a value parameter (a `mut` parameter is E0210: copy it into a `let mut` local), the value a `&` parameter points to, the variable of a `for`, a binding of a shared-reference `match`.
- Corpus: `tests/run/fields.bmb`, `tests/run/box_places.bmb`, `tests/check/err/types.bmb`, `tests/check/err/places.bmb`, `tests/check/err/ranges.bmb`.

## Aliasing

A variable passed as `&mut` in a call cannot appear in the other arguments of that call (E0410).

```
#[effects(pure)]
fn fill(a: &mut [i64], n: usize) -> () {}
#[effects(pure)]
fn same(a: &[i64], b: &[i64]) -> bool { a.len() == b.len() }
#[effects(pure)]
fn ok() -> bool {
    let mut x: [i64; 2] = [0, 0];
    fill(&mut x, x.len());
    same(&x, &x)
}
```

| Passed together | Verdict |
| --- | --- |
| `&x` and `&x` | accepted |
| `&mut x` and `&x`, `&mut x` and `&mut x` | E0410 |
| `&mut x` and a reference to a different variable | accepted |
| `&mut x` and a move of `x` (`keep(&mut w, w)`) | E0410 |
| `&mut x` and, in a by-value argument position, an expression that only reads `x` | accepted (ADR-0024) |

- An expression that only reads is one made only of variables, literals, indexing, unary and binary operations and method calls: `x.len()`, `x[0]`, `k + 1`. An expression that contains a call, a block or an `if`, or that reads a field (`p.x`), is an E0410 error.
- Aliasing is judged per variable. Different fields or different elements of the same variable are also taken to overlap.
- Corpus: `tests/check/err/types.bmb`, `tests/check/err/moves.bmb`, `tests/check/err/data_values.bmb`, `tests/run/swap.bmb`.

## Reference `match`

- The bindings of a `match` on a `&T` parameter are references that point to the payload. They are read with `*n` and cannot be written (E0408).
- The bindings of a `match` on a `&mut T` parameter are written with `*n = v`. Writing the scrutinee inside an arm (`*s = ..`) is an E0410 error: the `match` is borrowing it (ADR-0055).
- Moving a value of a move type out of a binding of a reference `match` is an E0413 error.
- Corpus: `tests/run/mut_match.bmb`, `tests/check/err/mut_match.bmb`.

## Closure captures

- A closure that uses an outer local uses `move` (E0262, ADR-0053). A capture is a value use at the point where the closure is made: a copy type is copied and a move type moves away. Using a local that moved away later is an E0413 error.
- The type of a captured local must be written (a parameter, a typed `let`). Capturing a name whose type is not written, such as the variable of a `for`, is an E0262 error.
- The bound decides the manner of the call: `Fn` only reads its captures, `FnMut` changes its captures, and with `FnOnce` the function value moves once it is called (the second call is E0413).

```
#[effects(pure)]
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { f(x) }
#[effects(alloc)]
fn caps(k: i64) -> i64 {
    let v: Vec<i64> = Vec::new();
    let a: i64 = apply::<_>(move |x: i64| -> i64 { x.wrapping_add(k) }, 1);
    apply::<_>(move |x: i64| -> i64 { if v.len() > 0 { x } else { a } }, k)
}
```

- Corpus: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.
