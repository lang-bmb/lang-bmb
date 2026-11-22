# ADR-0085: Truncating division toward 0 from one `div`·`mod` pair of the same operands

- Status: Accepted (autonomous, AUTONOMY.md §4: an encoding decision within the scope of ADR-0017). Supplements the defining formulas of `tdiv`·`trem` in ADR-0017 (the same functions when the divisor is not 0)
- Date: 2026-11-04
- Related: ADR-0017 (integer semantics and SMT encoding), ADR-0018 (auxiliary division facts), ADR-0064·0067 (definition axioms and probes), backlog 1B-47·T1-14, JUDGMENT §8

## Context

- In the pilot BVB `vc-va0093-max-balance` (2026-10-28) the proof passed but the specification strength was undecided (T1-14). The agent's contract (floor division written with truncation toward 0, as in `0 - (9 - n) / 10`) and the hidden contract (`(n % d + d) % d` written with a divisor parameter `d`) are the same expression on the domain -128..127.
- Solving the same implication again at HEAD on 2026-11-04 (`tests/verify/pass/const_division.bmb`) was still E0811 undecided. The probe takes 6,439,133 and the whole query 11,211,090 rlimit, over the limit of 5,000,000.
- The cause is the defining formulas of ADR-0017. `tdiv(a,b) = sign(a)·sign(b)·div(|a|,|b|)` creates the two terms `(div a b)` and `(div (- a) b)`, and the solver keeps a separate quotient-remainder pair for each `div` term. `trem(a,b) = a − b·tdiv(a,b)` is a product when the divisor is a variable (a parameter of a definition axiom), so it is nonlinear.

## Decision

The two definitions in the solver prelude (`verify::PRELUDE`) are changed.

```
tdiv(a,b) = ite(a ≥ 0 ∨ mod(a,b) = 0, div(a,b), ite(b > 0, div(a,b) + 1, div(a,b) − 1))
trem(a,b) = ite(a ≥ 0 ∨ mod(a,b) = 0, mod(a,b), mod(a,b) − |b|)
```

- `div`·`mod` are the Euclidean division of SMT-LIB (`a = b·div + mod`, `0 ≤ mod < |b|`). When the divisor is not 0, the two formulas are the same functions as the definitions of ADR-0017. This was confirmed with z3 (assuming a difference from the old formulas under `b ≠ 0` gives `unsat`).
- Only one `div`·`mod` pair arises per operand pair, and `trem` has no product with the divisor.
- When the divisor is 0, both formulas are the division by 0 of SMT-LIB (an uninterpreted value). The old formulas additionally produced facts such as `tdiv(a,0) = −tdiv(−a,0)`, and these facts have no basis (at run time E0804 prevents a 0 divisor, and the value of division by 0 in a specification is not fixed). The new formulas produce no such facts.

## Rationale

- **Measurement:** the probe of the same implication goes 6,439,133 → 456,403 and the whole query 11,211,090 → 621,942, and it is proved within the limit (verdict E0811 → proved). The health indicators (`health.py`) went down: V4 7,428,098 → 7,225,434 (before the regression test of this ADR was added), V8 24,254,414 → 23,914,744, V10 12,178,127 → 12,021,529; V5 0.1355 → 0.1244, V11 (satisfiability check) 282,057,550 → 283,986,695 (+0.7%, within budget), and there are no violations. The baseline was accepted at the value with the regression test included (V4 7,898,495). In the fail corpus (`tests/verify/fail`, 39 files) the diagnostics and counterexamples are unchanged apart from the `rlimit` values.
- **Rejected:** (가) the original plan of backlog 1B-47, namely new quotient and remainder symbols and linear constraints per constant divisor. The cost of this task comes not from direct division by a constant divisor but from definition axioms with a parameter divisor, so the original plan does not reach it. The solver already does the same thing (one quotient-remainder per pair) by sharing `div`·`mod`. (나) a guard that makes it equal to the old formulas at a 0 divisor (`ite (= b 0) 옛식 새식`). As a function it is exactly the same, but with a variable divisor the guard remains and the probe costs 7,912,698, more than the old formulas. All the guard protects is the facts without basis, so it was dropped. (다) making the leaf `spec fn` with a parameter divisor a macro. The whole query is 9,291,538, over the limit.

## Consequences

- Code: `PRELUDE` in `compiler/src/verify.rs`. The proof cache key is the query text (including the prelude), so it does not collide with old entries even without raising the format version.
- Tests: `tests/verify/pass/const_division.bmb` (the implication of the pilot contracts), the `rlimit` update of the expected output of the fail corpus, all soundness fuzzers pass (`cargo test`).
- Reverting: restore the two lines of `PRELUDE` to the formulas of ADR-0017.
