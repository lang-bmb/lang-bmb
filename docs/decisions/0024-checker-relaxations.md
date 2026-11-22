# ADR-0024: Three checker relaxations

- Status: Accepted (approved by the maintainer 2026-10-24, "start all of them")
- Date: 2026-10-24
- Related: ADR-0015 (aliasing), ADR-0016 (M1 scope), ADR-0017 (specification integers), ADR-0022 (mixed specification integers), compiler audit (2026-10-24)

## Context
The compiler audit confirmed three rules by which the checker rejects semantically safe programs. All three widen what the language accepts, so the maintainer's approval was needed (AUTONOMY.md §4).

1. `f(&mut a, a.len())`, `f(&mut a, a[0])` were rejected with E0410 (aliasing). A scalar argument is computed as a value before the call, so it cannot overlap with `&mut a`.
2. Negation `-x` of an unsigned value in a specification was rejected with E0411. Specification integers are mathematical integers (ADR-0017), and `0 - x` is already allowed.
3. A literal larger than `u64::MAX` in a specification was rejected with E0406. Specification integers have no upper bound.

## Decision
1. Another argument that mentions the same array as a `&mut` argument is allowed if its parameter is not a reference and the argument is a read-only expression (no call, `&mut`, block or `if`). Other cases (a reference argument, an argument containing a call, and so on) remain E0410.
2. In a specification context, negation of a `u64`·`usize` value is accepted as `i64` (a specification integer). In executable code it remains E0411.
3. Integer literals in a specification context have no size limit. In executable code it remains E0406.

## Rationale
- None of the three changes the execution semantics or the proof encoding. The prover computes arguments as values from the left and encodes specification integers as SMT `Int`.
- 1 is wider than Rust (Rust rejects `f(&mut a, a[0])` as a borrow error). In BMB an array argument is a pointer and a scalar argument is copied before the call, so it is safe. It is a relaxation in a direction that does not conflict with Rust habits.

## Consequences
- Regression examples: `tests/run/relaxed.bmb` (proof and execution). The cases where rejection is kept are in the existing `tests/check/err` corpus.
- To revert, delete the `reads_only` branch and the two specification-mode branches in `check.rs`.
