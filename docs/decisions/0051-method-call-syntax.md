# ADR-0051: Method call syntax as the canonical form

- Status: Accepted (2026-10-26, approved by the maintainer "1/2 accepted, approved", backlog 2V-12)
- Date: 2026-10-26
- Related: ADR-0045, ADR-0026 §1·§3 (method call syntax forbidden), DESIGN.md principles 5·8, §4.1 "call resolution", 2R-04d (input aliases for method calls), review R11, backlog 2V-11


## Confirmed at acceptance (2026-10-26)
- Diagnostic number: E0313, the same method name in two or more bound traits (requires the path form, confirmed unused). The form `T::f(&x, ..)` gets no diagnostic; the normal form (`bmb fmt`) rewrites it to method syntax.
- Accepted ahead of 2R-06 (grammar decision) (approved by the maintainer). 2R-06 decides the remaining constructs.
- The ledger row "call resolution" is fixed together with this acceptance.
- Implementation phase: ROADMAP 2M (method syntax). After generics (2T).

## Confirmed at implementation (2026-11-07, 2M-01)
- Resolution order: the inherent `impl` functions of the receiver type (for a reference, the type with one level stripped), otherwise a trait that has `f`. The trait candidates are the bounds for a type parameter, and for a concrete type the traits that have an `impl` for that type (same as Rust. The decision's "bound trait" was read to include the traits a concrete type implements). Built-in method names of built-in types (`len`, `push`, integer and `f64` methods, etc.) come before traits (Rust's inherent-method priority).
- Diagnostics: E0313 (two or more traits, points to the path call), new E0314 (an associated function with no receiver, or a function with its own type parameters, called as a method; path call wording). Type argument syntax on methods (`x.f::<U>(..)`) is not added: such functions are called by path. E0227 is retired.
- An inherent `impl` of a generic type fixes its type arguments by unifying the receiver type with the `impl` head (no E0237: the receiver type is fixed by the annotated type).
- Method: the checker rewrites each resolved call to the path call `T::f(adjust(x), ..)` (`check::check_resolved`) and checks again until no method call is left to resolve. Moves, recursion, effects, proof, code generation and the map do not need to know methods separately, and the diagnostics are those of the last check, so no trace is left of the receiver being seen twice in the first check. The rewritten call carries `method: Some(adjustment)` on `ExprKind::Call`, and output (slices, views, diagnostics) writes it as `x.f(..)`. `bmb fmt` writes the parsed program.
- Adjustment in map L2: the target of a `calls` edge is the resolved function, and the adjustment is told by the receiver type of that function (the signature in the slice `deps`) and the `writes` edge of `&mut self` (`arg` 0). No separate field is put on the edge: the edge set has one edge per (caller, target, kind), so calling the same target both as a method and by path would split it in two.
- Normal form (2026-11-07, 2M-02): `bmb fmt` re-checks the rewritten program and writes `x.f(..)` only when every call resolves to the original function, and also checks that the identity hash of each definition is unchanged (`check::methodize`). The decision's "no diagnostic" was read to mean that `check` does not reject path calls: for in-place migration there is, like W0907 (ADR-0057), a lint warning W0911 and its fix (`bmb fix` applies it, splicing the source text so comments survive). `check`, `verify` and `build` do not change. The corpus and `lib/` were moved with W0911 (9 run files and 3 proof files, 85 places; `lib/` 0 places), and the failing corpus (expected output contains positions) and the frozen bench assets (0 path calls) stay as they are.
- Workspace `rename` renames method calls that resolve to the target (`refs::Rewrite::method`). Identity (ADR-0059) is computed on the resolved program, so method call sites also reference the target's `id`.

## Context
ADR-0026 forbade method call syntax for user functions and allowed only `Point::norm1(&p)`. The rationale was principle 5: if the resolved call target is not in the text, the slice is not self-contained. But built-ins use method syntax: `v.len()`, `v.push(x)`, `x.wrapping_add(y)`, `u8::try_from(x).unwrap()`. That is two rules for the same thing (S3). Also, BMB annotates every local type and forbids auto-deref chains and Deref coercion, so the type of the receiver expression is determined deterministically by the signatures inside the slice, and so is the call target. Built-in methods already work that way.

## Decision
- `x.f(a, ..)` is `T::f(adjust(x), a, ..)`. `T` is the type of `x` (for a reference, the pointee type) and `f` is a method of the inherent impl of `T` or of a bound trait (ADR-0046).
- The receiver adjustment is **one level** and is fixed by the declared receiver of `f`: `&x` for `self: &T`, `&mut x` for `self: &mut T` (`x` is a writable place), `x` for `self: T` (move or copy). If `x` is already `&T`/`&mut T` it stays as is. There are no other auto-reference or dereference chains and no Deref coercion (the rest of the ledger row "call resolution" stays).
- The canonical form is method syntax. The form `T::f(&x, ..)` is changed by a machine fix (principle 8). Associated functions (no receiver) stay `T::f(..)`.
- Map L2 records the resolved target (`unit::T::f`) and the adjustment. The slice includes the impl signatures of the receiver type (principle 5 kept).
- If the same name is in both the inherent impl and a bound trait, the inherent impl wins as in Rust, and ambiguity between two traits is an error (requires the path form).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **Method syntax canonical, one-level receiver adjustment (proposed for acceptance)** | Rust surface (S1). The target is deterministic from annotated types and signatures, so the payback of principle 5 is kept in full. Built-ins and user definitions follow the same rule (S3) |
| Current state (path calls only for user functions) | Two forms for the same thing (violates principle 8), Rust-habit errors (the input aliases measured by 2R-04d) |
| Rust's full auto-ref/deref | If receiver adjustment chains, target resolution depends on Deref implementations. Loses the payback of the ledger row "call resolution" |

## Consequences
- Fix the DESIGN §4.1 ledger row "call resolution": "auto-deref chains and Deref coercion forbidden. The one-level receiver adjustment of a method call is fixed by the signature".
- The exclusion "method call syntax (forbidden by §4.1 call resolution)" in ADR-0026 §1 and the call form in the associated function row of §3 are superseded.
- Move the `T::f(&x)` calls of the corpus and library by machine fix. Accepted together with 2R-06 (grammar decision).
