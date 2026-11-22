# ADR-0114: The proof context of a closure is that of the function that wrote it

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": an implementation-level decision that applies the default context rule of ADR-0096 to the closures of ADR-0053. Within the scope of the maintainer's approval 2026-11-11 "recommendation accepted, autonomous execution/verification approved". To revert, delete `closure_uses` in `closures::lower` and the `call` branch of `verify::default_contexts`)
- Date: 2026-11-11
- Related: ADR-0053 (function values), ADR-0095 A1·ADR-0096 (default proof context), DESIGN §4.1 "proof automation", backlog 2V-20k

## Context

- A closure is lowered to a generated struct and a trait implementation (ADR-0053). The `requires`·`ensures` that the closure writes become the bodies of the generated `pre`·`post` spec fns, and the body becomes the generated `call`.
- The default context (ADR-0096) reveals the spec fns that the definition's own clauses call and does not go into their bodies. So `call` sees `pre`, but it sees the spec fns that the body of `pre` calls, that is the spec fns the closure wrote in its clauses, by name only. The diagnostic says "add `#[uses(small)]`", but an attribute cannot be attached to a closure (revealed by the 2V-20i fuzzer).
- The `#[uses(..)]` of the function that wrote the closure does not reach `call` either.

## Decision

1. **The clauses a closure writes are the clauses of `call` itself.** The default context of `call` includes the spec fns that the bodies of the generated `pre`·`post` call. It is the same rule by which an ordinary function sees the spec fns of its own clauses. It does not go into the bodies of those spec fns (ADR-0096 as it is).
2. **A closure inherits the `#[uses(..)]` of the function that wrote it.** The generated `call` has the `#[uses(..)]` of the enclosing function as it is. A closure inside a closure inherits from the `call` of the outer closure.
3. The `call` that arises when a function item is passed as a value is not affected (it follows the contract of that function itself).

## Rationale

| Option | Verdict |
| --- | --- |
| **Clauses are its own, `#[uses]` is inherited (accepted)** | In Rust and Verus a closure is part of the function that wrote it and is verified in the same context (JUDGMENT S1). Lowering is an implementation technique, not semantics: a clause the user wrote must not move one step away because of lowering (S3) |
| Syntax to attach `#[uses]` to a closure | It is new surface. Inheritance removes the need |
| As now | An undecided result or failure that the user has no way to fix (a compiler-layer defect, JUDGMENT §3) |
| Unfold the `pre`·`post` bodies as a closure | The same cost problem as r2c, which ADR-0096 dropped |

## Consequences

- `closures::lower` moves the `uses` attribute of the enclosing function to `call`, and `verify::default_contexts` adds the spec fns of the `pre`·`post` bodies to the generated `call`.
- If the `#[uses]` of the enclosing function is wide, the proof context of the closure is wide too (cost). It is the same choice as for the enclosing function.
- Corpus `tests/verify/pass/closure_context.bmb` (with the spec fns of the clauses only, with the inherited `#[uses]`).
