# ADR-0072: The probe of the satisfiability checks writes every definition outside recursion as a macro

- Status: Accepted (autonomous, AUTONOMY.md §4). Widens the form of the probe of ADR-0069 (vacuity check, precondition satisfiability check). The meaning is unchanged and only the form written to the solver changes. The probe of proof obligations (ADR-0067) and the full query (ADR-0064) are unchanged
- Date: 2026-10-31
- Related: ADR-0064 (macro form and pattern rule), ADR-0067 (probe of obligations), ADR-0069 (probe of the checks), backlog T2-10

## Context

- Even after ADR-0069, in the programs of `proofs_are_not_vacuous` (112) and `bench/game`, 576 reach points of the vacuity check were undecided (rlimit 393,554,937) and 62 functions of the precondition satisfiability check were undecided (143,739,379). That is larger than all proof obligations (V4·V8·V10).
- For each undecided query (113 and 17 distinct texts), one kind of assertion at a time was removed and it was solved again to split the causes (`dev-docs/experiments/T2-10`). Removing the quantified definitions of non-recursive `spec fn`s decides the most: 339 for the vacuity check (rlimit 168,506,093) and 58 for precondition satisfiability (122,734,703). The answer is mostly `sat` (reachable, satisfiable), but Z3 cannot build a model over quantified definitions (the same phenomenon as ADR-0064·0067). These definitions are not leaves (they call other `spec fn`s), so they were quantified in the probe of ADR-0067 too.

## Decision

1. In the probe of the vacuity check and the precondition satisfiability check (ADR-0069), the definition of a non-recursive `spec fn` is written as a macro even if it is not a leaf (`verify::Forms::SatProbe`, the `SAT_QUANTIFIED` marker of `Prog::axiom`).
2. But it is a macro only when every definition it calls in that query is a macro. If it calls a recursive definition (quantified axiom) or a definition that stayed quantified because a pattern contains its name, that definition and the definitions above it stay quantified (the fixed point of `axioms_in_forms`). The macro form, the pattern rule and the out-of-range value (`f!ext`) are the same as ADR-0064.
3. The use of the answer, the share (`PROBE_SHARE`), and the full query and retry after it are as in ADR-0069.

## Rationale

- **Cause and layer:** the solver cannot build a model over quantified definitions. The layer is the encoder. The meaning is the same (ADR-0064 decision 3: the models of the macro and the quantified axiom correspond to each other), so `sat` and `unsat` are the same answers as the full query. The only verdict change possible is undecided → decided.
- **Canonical way:** `define-fun` for non-recursive definitions is the canonical form of SMT-LIB (ADR-0064). Solving the same goal in several forms, shortest share first, is the same as the rationale of ADR-0067.
- **Why macros that reach recursion are excluded (measurement):** with all non-recursive definitions written as macros, 20 points of `lib/sha256.bmb` (`sha256::chained`: calls the mutually recursive `chain.*`) did not finish within rlimit 50,000 and ran for over 120 seconds (a search that rlimit does not count, the same shape as the vacuity check timeout ADR-0064 saw). Excluding definitions that reach recursion, the number decided for the same queries is the same (vacuity 285, precondition 58) and it is within 0.13 seconds per query.
- **The probe of proof obligations is not changed:** the probe of obligations also looks for proofs. Widening macros made non-linear proofs regress ("all leaves as macros" and "macro first" of ADR-0067). Whether to use the same form for counterexample queries (failing corpus) is measured separately (backlog T2-11).
- **Measurement (the same 112 programs, after T2-09 → this decision):** vacuity check rlimit 380,378,351 → 239,829,012, undecided points 566 → 281, refuted 0 → 0. Precondition satisfiability 143,739,379 → 20,619,118, undecided 62 → 4 (58 are satisfiable), unsatisfiable 0 → 0. Failing corpus: vacuity check 5,503,225 → 4,394,478, refuted points and the reported set unchanged. Time 177 seconds → 125 seconds, 92 seconds → 14 seconds. The causes of the remaining undecided (definitions involving recursion 164, bit operation arithmetic 56, etc.) are backlog T2-13. Details in `dev-docs/experiments/T2-10/results.md`.

## Consequences

- The verdicts of the two checks are the same or more are decided. The reports (W0901·W0905) are refutations (`unsat`) only, and the queries have the same meaning, so a new report or a vanished report arises only from whether it is decided, not from the meaning.
- The query text of proof obligations, the proof cache and V4·V7·V8·V10 do not change (obligations use `Forms::Probe`·`Forms::Full`).
- Tests: `verify::tests::satisfiability_probe_unfolds_definitions_outside_recursion`, `satisfiability_probe_keeps_recursion_quantified`, `proofs_are_not_vacuous`, `compiler/tests/lint.rs`.
- Revert: if `Prog::axiom` does not put `SAT_QUANTIFIED` on non-leaf definitions, it is ADR-0069 as it was.
