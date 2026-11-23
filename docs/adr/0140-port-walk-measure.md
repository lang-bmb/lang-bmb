# ADR-0140: Syntax tree walks in the port use the remaining depth as the measure

- Status: Accepted (autonomous, AUTONOMY.md §4: an implementation convention of the port that does not change the accepted language)
- Date: 2026-11-15
- Related: ADR-0047 §3 (structural measure), ADR-0100 (freeze), ADR-0121 (data shapes of the port), ADR-0122 (indicators of the ledger), `dev-docs/measurements/selfhost-port-map.md` §7.8, backlog 4D-25

## Context

- A recursive function must have a `decreases`. A structural measure (`decreases e`) can be used only on recursive enums and on structs that hold themselves in a `Vec` (E0401).
- The syntax tree of the port follows the shape of stage0 (ADR-0121): the `ExprKind` of `Expr { kind: ExprKind, span }` holds `Box<Expr>`·`Block`·`Vec<Expr>`, and `Block` holds `Vec<Stmt>`. Structs and enums hold each other in alternation, so no type can take a structural measure.
- The parser was not caught: the nesting limit of stage0 (256) became the measure as it was. What is caught are the functions that walk a finished tree (all of `fmt`, `plain_place`). The walks of the checker (`4D-03`) all have this form.
- The depth of the tree is not bounded by the nesting limit. Operator chains pile up to the left, and an expression of depth about 100,000 comes out within the limit (checked: 100 levels × 8 kinds of operator × 120, `dump parse` is the same in stage0 and the port).

## Decision

1. **A recursive function that walks the syntax tree takes the remaining depth `f: usize` as its last argument and is written with `decreases f`.** A recursive call passes `f - 1`, and the first statement of the function stops if `f == 0`.
2. **The value given at the start is an unreachable upper bound.** It is one of two: a true upper bound given by the input (inside the parser, the number of tokens: one level of the tree uses at least one token), or a depth the execution stack cannot survive (`1000000000`). The latter means the stack overflows before `f == 0` is reached, and stage0 ends the same way at that depth.
3. **The `f == 0` branch counts as an unreachable branch.** It is written in the friction of the ledger (decision 7 of ADR-0122), and no meaning is put on the value that branch gives (empty text, `false`).
4. **Widening the structural measure is not done now.** It is a change that increases the accepted programs (the exception procedure of ADR-0100), and a change to the prover is an amount the port has to carry over again. It is left as language work after the bootstrap (a follow-up of backlog 4D-25).

## Rationale

| Option | Verdict |
| --- | --- |
| **Remaining depth argument (accepted)** | It does not change the language. The minimal experiment (`dev-docs/experiments/port-shape/tree.bmb`) and the parser and `fmt` ports (33 functions) were all proven in this form. The cost is one argument and one unreachable branch each |
| Widen the structural measure to mutually recursive types | It is the way that fixes at the layer of the cause (the language), but it is a surface change during the freeze. It changes the proof encoding (size functions of recursive types) and the checker, and the port carries that change over again. It is not urgent: the workaround does not block proofs |
| Rebuild the syntax tree out of enums only | The structural measure holds. But the data shape diverges from stage0, which breaks the correspondence of the per-component differential tests (ADR-0121), and a `Block` holding `Vec<Stmt>` is still caught |

## Consequences

- The walks of the checker port (`4D-03`) are written with this convention. Backlog 4D-25 is closed by this decision.
- How to revert: when the structural measure is widened, delete the `f` argument and the `f == 0` branch and change `decreases` to the tree. The meaning of the functions stays the same (only the unreachable branch disappears).
- The list of termination proofs that lean on a runtime upper bound is kept per component by the porting ledger ("how to read" 2 of porting map §7.8).
