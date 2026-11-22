# ADR-0054: Scalar reference parameters `&T`·`&mut T`

- Status: Accepted (2026-10-27, Accepted (autonomous): AUTONOMY §4 "implementation-level decision within the scope of an existing ADR", backlog 2T-08)
- Date: 2026-10-27
- Related: ADR-0015 (second-class references), ADR-0016 (M1 scope), ADR-0026 §2, ADR-0046 §1·§2, backlog 2T-07a·2T-08

## Context
ADR-0015 decided second-class references, which put references only in argument position, and M1 (ADR-0016) accepted only `&[i64]` and `&mut [i64]` of those. 2B (ADR-0026) added `&Point` and `&mut Point`. A reference parameter pointing to an integer or `bool` (`x: &mut i64`) was E0402.

But generics (ADR-0046) accept every value type as a type argument, so the instance `get::<i64>` of `fn get<T: Copy>(x: &T)` already has an `&i64` parameter. Code generation could not handle this and panicked (2T-08, since 2T-01). Also, to impl a trait with the receiver `self: &Self` for an integer type (ADR-0046 §2), the signature of the impl function has to be `self: &i64`, and that was E0402. That is two rules: the same type works through generics and is rejected when written directly.

## Decision
- Parameter types accept `&T` and `&mut T` (`T` is an integer type or `bool`). The second-class reference rules (ADR-0015: argument position only, cannot be stored or returned) and the aliasing rule (E0410) stay as they are. A spec fn does not accept `&mut` (existing rule).
- The meaning is the same as Rust. The body reads with `*x` and, for `&mut`, writes with `*x = v`. The previous value in a contract is `*old(x)` (same as Verus).
- Code generation: the parameter is a pointer to the caller's slot and the body reads and writes it like a local variable slot. If the argument is a local variable or a scalar reference parameter, that slot is passed; if it is a field or element (memory representation, `u8` and `bool` are 1 byte), the value is put in a temporary and passed, and for `&mut` written back after the call. Because references are second-class there is no other path that sees that place during the call, so write-back and direct reference are observationally the same.
- The proof encoding does not change. The existing handling of `&mut` arguments (after the call the place becomes a fresh constant, only the callee's `ensures` is assumed) fits scalar places as is.

## Rationale
| Option | Verdict |
| --- | --- |
| **Allow scalar reference parameters (accepted)** | The Rust surface as is (S1). Generic instances and directly written code follow the same rule (S3). A trait's `&self` receiver can be impl'd for integer types |
| Generate code for generic instances only and keep rejecting the direct form | Two rules for the same type. Trait impls for integer types are blocked |
| Forbid `&T` instantiation at scalar types | Narrows ADR-0046's "type arguments are value types" (needs approval). Generic algorithms such as sorting cannot be used on integers |

## Consequences
- Output parameters (`fn parse(s: &[u8], out: &mut u64)`) can be used. Until now multiple results were returned as a struct. Both are valid and each has its own normal form.
- Regression tests of 2T-08: `tests/run/scalar_refs` (locals, fields, elements, generic instances, trait impls for integer types), `tests/verify/fail/scalar_refs` (counterexample for a place changed after the call).
