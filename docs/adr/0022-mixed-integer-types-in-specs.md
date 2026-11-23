# ADR-0022: Mixing integer types is allowed in specifications

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: ADR-0017 §2, ADR-0016, backlog 1E-05

## Context
This surfaced while writing the bench programs. M1 has no integer conversions, and the checker did not allow `i64` and `usize` to be mixed in one operation, even in specifications. So in a loop that runs on a `usize` index there was no way to express the upper bound of an `i64` accumulator in terms of the index (`c <= k` could not be written). The only output is `print_i64`, so the accumulator could not be a `usize` either.

By ADR-0017 §2 specification arithmetic is already over unbounded integers, and in SMT everything is `Int`. Separating types in specifications is a restriction with no semantic basis.

## Decision
- **In a specification context**, different integer types (`i64`·`u64`·`usize`) can be mixed in arithmetic and comparison. The result of mixed arithmetic is an integer (the checker's internal notation is `i64`, the meaning is an unbounded integer).
- **In a specification context**, a different integer type can be passed as a `spec fn` argument. If the value is out of range, that call is outside the range guard of the definitional axiom and its value is not determined (ADR-0017 §4). So this is sound.
- **Executable code is unchanged.** Different types are an error (E0401). There are no conversions in M1.

## Rationale
- It is the same approach as Verus's promotion to `int`. It also fits the succession of the surface (ADR-0013).
- The meaning is already an unbounded integer, so no new proof obligation or trust base arises.

## Consequences
- The absence of conversions in executable code remains. The problem that `u64`·`usize` values cannot be printed is recorded as a basis for the stage 2 "checked conversion" candidate (ROADMAP 1C observation).
