# ADR-0055: `match` on references and reference bindings

- Status: Accepted (2026-10-27, Accepted (autonomous): AUTONOMY §4 "implementation-level decision within the scope of an existing ADR", backlog 2O-02)
- Date: 2026-10-27
- Related: ADR-0015 (second-class references), ADR-0026 §3 (`match`), ADR-0047 §1·§2 (owned types, recursive types), ADR-0054, backlog 2O-02

## Context
A function that reads recursive data (ADR-0047) takes `&Expr` and looks at the sub-values in each arm. In Rust this is written `match e { Expr::Add(a, b) => .. }` (default binding mode, RFC 2005: `a: &Box<Expr>`) or `match &*e`. `match *e { Expr::Add(a, b) => .. }` is a move that takes the `Box` out from behind a reference, so Rust rejects it too (E0507).

Until now BMB required the scrutinee of a `match` to be an enum value (a reference is E0423, fix `*r`). When there was no payload that moves, `match *r` was enough as a copy, but to bind an owned payload (`Box` since 2O-01) a way to read through the reference is needed. Also `&` could appear only in argument position (E0409).

## Decision
- A reference is accepted as the scrutinee of `match`: an expression of type `&E` (`match r`, `r: &E`) and `&expr` in scrutinee position (`match &n.next`, `match &**z`). In this case each binding is a reference to its payload: for payload type `P` the binding is `&P` (same as Rust's default binding mode). An integer payload is also `&i64` and its value is read with `*n` (Rust auto-dereferences in some operations but BMB does not: the meaning is the same and the set of accepted programs is narrower). Passing `&Box<E>` to an `&E` parameter also uses dereference (`f(&**a)`; Rust's deref coercion is not used).
- Reference bindings follow the second-class reference rules (ADR-0015): they cannot be held in locals, fields or results, and are only passed as arguments or read. A reference `match` does not move the scrutinee.
- The bindings of a `match` on `&mut E` (`match r`, `r: &mut E`, `match &mut place`) are `&mut P` and write the payload place in place (implemented in backlog 2O-04b, at first E0207). Using the scrutinee variable inside an arm is E0410 (Rust: second borrow). The proof rebuilds the variant value from the values of the bindings at the end of the arm and writes it to the scrutinee place.
- `match *e` and `match n.next`, which bind an owned payload by value from behind a reference or from a field, are E0413, and the machine fix changes them to a reference `match` (`match e`, `match &n.next`).
- Proof: a reference is the same as its value (ADR-0015), so the value of a binding is the payload value itself. Code generation: it is a shared reference, so the binding is a shallow copy of the payload and is not dropped.

## Rationale
| Option | Verdict |
| --- | --- |
| **Reference `match` with Rust's default binding mode (accepted)** | Rust surface and meaning as is (S1). The canonical form for reading recursive data |
| Implicitly bind owned payloads by reference in `match *e` | Accepts a program that Rust rejects (violates the principle that the surface is inherited and the meaning is only made stricter) |
| Allow only accessor functions for reading payloads | A workaround (ADR-0045 §3). Boilerplate for every recursive type |

## Consequences
- Functions that take recursive enums (evaluator, size, fold) are written with `match e` (`tests/verify/pass/recursive`, `tests/run/expr_tree`).
- Using a reference binding in a value position gets a `*x` hint on E0401.
