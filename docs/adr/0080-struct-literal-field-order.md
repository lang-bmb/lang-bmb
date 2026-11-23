# ADR-0080: The fields of a struct literal are written in declaration order (the checker requires it and fixes it)

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": option (a) of backlog 2B-13). Supersedes "(the normal form puts them in order)" in the "construction" row of ADR-0026 §1 and "the fields of a literal are sorted in declaration order" of §6
- Date: 2026-11-01
- Related: ADR-0026 §1·§6 (data type surface and normal form), DESIGN §3 principle 8 (one expression), backlog 2B-04b, 2B-13

## Context

- ADR-0026 said that a literal writes "all fields in declaration order" and that the normal form puts them in order.
- The problem 2B-04b found: field expressions are evaluated in the order written. If the normal form sorts the fields, the evaluation order of field expressions with effects (`Pair { b: loud(2), a: loud(1) }`) changes and the meaning of the program changes. So the normal form did not sort, and the checker accepted any order. As a result one literal had several notations (a violation of principle 8). Two programs of the corpus (`tests/run/shapes.bmb`, `tests/verify/pass/structs.bmb`) wrote an order different from the declaration order.
- Candidates (2B-13): (a) the checker requires declaration order and gives the rewritten expression as a fix. (b) leave the written order as is and allow several normal forms. (c) sort only field expressions without effects.

## Decision

1. A struct literal writes its fields in declaration order. In any other order the checker emits **E0434** ("struct literal fields must be given in declaration order"), and the fix is the same expression written in declaration order (`replacement`, applied by `bmb fix`).
2. The normal form does not sort the fields of a literal. The declaration order is the written order and the evaluation order.
3. E0434 is not emitted for an expression that writes the same field twice (E0422), writes a field that does not exist (E0306), or uses `..base` (E0223). Fixing that error comes first.

## Rationale

- **Principle 8:** one expression for one thing. (b) leaves as many notations for the same value as there are permutations.
- **It keeps the meaning:** sorting changes the evaluation order of expressions with effects. (c) relies on a "no effect" judgement, and for expressions with effects two notations remain again. (a) lets the person or agent choose the order, and that order is the declaration order. The evaluation order is as it appears in the text.
- **Mechanical fix:** the expression to write is determined deterministically, so a fix is attached. An agent's Rust habit (free field order) ends with one diagnostic and a fix (principle 6: "notation that is mechanically determined is the tool's job").

## Consequences

- Code: E0434 in `check::Checker::struct_lit` (the table in `compiler/src/diag.rs`), the diagnostics document `docs/diagnostics.md`.
- Corpus: `tests/check/err/field_order.bmb` (two cases: an expression without effects and one with effects), the two programs fixed to declaration order (run output and proofs are unchanged).
- Frozen sets: the only literals in a different order were the two programs of the corpus. Checked with the checker: the corpus, `lib/`, `bench/game`, the four `bench/brb-encap` seeds (0 E0434). The BMB solutions of the BRB-encap task modules have no literals. The gate runs the reference and weak solutions of every frozen set with this compiler. If a model writes a different order, it now gets E0434 and the fix. The compiler tree changes, so this is a new measurement condition (ADR-0070 (a)).
- Reverting: take out E0434 and it is the same as the current (b).
