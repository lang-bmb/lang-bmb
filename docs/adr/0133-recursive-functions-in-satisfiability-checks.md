# ADR-0133: Recursive definitions as recursive functions in satisfiability checks

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": the encoding of advisory queries. The queries and verdict criteria of the proof gate are not changed)
- Date: 2026-11-14
- Related: ADR-0069 (probes of satisfiability checks), ADR-0072 (macros in satisfiability check probes), ADR-0075 (ground instance retry), ADR-0131 (advisory `rlimit`), backlog T2-13

## Context

- The vacuity check (W0905) and the precondition satisfiability check (W0901) look for a model of the path condition. A point that is not decided is not reported, so undecided is a loss of detection power (health V12).
- The probe writes definitions outside recursion as macros (ADR-0072). Recursive definitions remain as quantified axioms, and Z3 ends the model search of such queries in `unknown` (`incomplete`).
- Measurement (`279413e0`, the 167 programs of V11·V12): 216 undecided (204 reach points, 12 preconditions). Taking out 101 queries of the 12 programs with the most undecided, changing the definitions to `define-funs-rec` and asking Z3 as is, 69 or more were `sat` in about 20ms.

## Decision

1. **For a satisfiability check that the probe did not decide, if the query reaches recursive definitions, a query that writes all the reached definitions as one `define-funs-rec` is solved first within the probe share** (`verify::rec_base`). The order is probe → bitvector translation (reach points only) → recursive function form → full query → ground instance retry.
2. **The meaning is the same as the quantified definition.** The body of each function is `(ite type-range-of-the-parameters body (f!ext args))`: inside the range the body, outside it the uninterpreted `!ext` (the same rule as the macro form). So both `sat` and `unsat` are taken as answers.
3. **The queries of proof obligations stay as they are.** The recursive function form is used only for queries that look for a model.
4. **It is not yet turned on for the queries of `bmb examples`.** Turning it on changes the output (9 of the 90 frozen examples of `bench/intent`), and that set is before its official run, so it has to be re-frozen. It is done once, together with other changes, in `T1-44g`.

## Consequences

The same 167 programs, on the same machine, before (`d55d73fb`) and after:

| | Before | After |
| --- | --- | --- |
| Undecided reach points | 204 | 78 |
| Undecided precondition checks | 12 | 8 |
| `rlimit` sum (V11) | 191,944,190 | 118,117,292 |
| Reports | 0 | 0 |

- No program has more undecided results.
- No reports appeared: the 130 newly decided are all `sat` (reached).
- Test: `tests/lint.rs` `recursive_definitions_leave_no_satisfiability_check_undecided` (a recursive contract that needs a model is decided, and a contract made contradictory by a recursive definition is reported as W0901).
