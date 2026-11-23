# ADR-0069: The vacuity check and the precondition satisfiability check also solve the probe first

- Status: Accepted (autonomous, AUTONOMY.md §4). Decides the work decision 4 of ADR-0067 left as backlog T2-06. The probe of proof obligations (ADR-0067 decisions 1·2) is put on the two checks in the same form with the same share. The meaning is unchanged and only the order of asking the solver changes
- Date: 2026-10-30
- Related: ADR-0023 (ground instance retry), ADR-0064 (macros for leaves that bind an array), ADR-0067 (probe of proof obligations), backlog 1B-44 (vacuity check), 2D-08 (W0901), T2-06

## Context

- The vacuity check (`verify::vacuous_ends`, W0905, gate test `proofs_are_not_vacuous`) solves the path condition at every loop exit and body end with rlimit 1,000,000 and reports only `unsat` (refutation). The precondition satisfiability check (`verify::preconditions_satisfiable`, `bmb lint` W0901) solves `requires` and reports if `unsat`; `sat` is satisfiable and `unknown` is undecided.
- For both queries the answer is mostly `sat` (the same shape as counterexample queries). ADR-0067 removed with the probe the cost of counterexample queries running to the limit because they cannot find a model over the quantified definitions of scalar leaves, but the two checks solved only the form of ADR-0064 (ADR-0067 decision 4).

## Decision

1. The queries of the two checks also solve the probe of ADR-0067 first. The probe is the same query with every leaf `spec fn` written as a macro, and the limit is `Z3_RLIMIT / 100` (`PROBE_SHARE`, now 50,000). If the probe is the same text as the full query there is none (`verify::probe_base`).
2. If the probe is `unsat` that is the answer (refuted, unsatisfiable), and if `sat` that is the answer (reachable, satisfiable) (same meaning, ADR-0064 decision 3). Otherwise the current query is solved with the current limit (vacuity check 1,000,000, precondition satisfiability check `Z3_RLIMIT`, then ground instance retry).

## Rationale

- **Cause and layer:** the same as ADR-0067. On queries whose answer is `sat`, Z3 cannot build a model over quantified scalar definitions. The layer is the encoder.
- **Canonical way:** trying the same goal with several encodings, shortest share first (rationale of ADR-0067). The same rule as proof obligations is used (one way for one thing, JUDGMENT S3).
- **Measurement (seed 0, Z3 4.15.4, the rlimit sum is the sum of `(get-info :rlimit)` per query, the verdict is the output of the check):** the corpus is all the programs of `proofs_are_not_vacuous` (`tests/verify/pass`, `tests/run`, `bench/baseline`, the m1·m1b·BVB seeds, each of `lib/*.bmb`, `tests/lib_run` and `bench/game` together with the libraries, 112 in all) and the failing corpus (36 of `tests/verify/fail`).

  | Check | Corpus | Now (ADR-0064 form) | Probe 1% (this decision) |
  | --- | --- | --- | --- |
  | Precondition satisfiability | Proof corpus etc., 112 | 189,088,728, undecided 71 | 143,739,379 (−24%), undecided 62 |
  | Vacuity check | Proof corpus etc., 112 | 471,936,726, undecided 665 | 393,554,937 (−17%), undecided 576 |
  | Precondition satisfiability | Failing corpus, 36 | 118,102 | 18,145 (−85%) |
  | Vacuity check | Failing corpus, 36 | 15,598,503, refuted 18, undecided 15 | 5,544,862 (−64%), refuted 18, undecided 5 |

  - **The verdicts did not get worse.** What is reported (`unsat` of W0901, the refuted points of the vacuity check) is the same on both corpora. For precondition satisfiability, 9 undecided changed to satisfiable (`sat`) (`arith_ok::sub_guarded`, `inv::widen`, two of `rg::count`, `arrs::repeated`, `bits::shl8`·`shr16`, `num::clamp`·`num::mid` of the m1b seed), and nothing decided changed to undecided. Of the undecided points of the vacuity check, 89 (proof corpus) and 10 (failing corpus) were decided as `sat` (reachable).
  - **Sensitivity to the share (precondition satisfiability / vacuity check, proof corpus):** 20,000 gives 143,830,664 / 389,624,944, 200,000 gives 143,739,379 / 417,959,579. As with the probe of obligations, 1% is in the flat region, so the same constant is used.
- **The increase of `lib/base64.bmb` (ADR-0067, 3,651,389 → 5,535,770) is not recovered.** Of the increase of 1,884,381, 1,501,708 is 29 obligations the probe did not decide (50,000 share each), and 382,673 is 123 obligations the probe proved but at a higher cost than the quantified form (1,659,664 vs 1,276,991). Options that change the order for proof obligations were measured by solving each stage of the `--emit-smt` script separately (the answers and rlimit of the stages are the same as the compiler: the sum of the current form matches the values of V4, the failing corpus and the libraries). Solve the quantified form first with share L, then the probe if undecided, then the full query: with L = 50,000 the libraries go 10,673,150 → 9,982,418 (`base64` 4,611,207) but V4 goes 7,387,374 → 8,590,834 (+16%) and the failing corpus 15,960,213 → 17,358,807 (+9%). L = 1,000~20,000 makes both V4 (8,103,786~8,564,596) and the libraries (10,880,878~11,846,031) worse. The option of reducing the probe share is the same as the sensitivity of ADR-0067 (20,000: libraries 10,498,991, V4 7,526,897). No option restores `base64` without losing the gain of T2-05, so the order for proof obligations (ADR-0067) stays. The cause of the macro form costing more on the queries of `base64` (size of expanded terms, etc.) was not investigated.
- **What the probe does not lose:** if the probe does not decide, the current query is solved with the same text and the same limit. Per query the cost grows by at most the probe share.

## Consequences

- The verdicts of the two checks are the same or more are decided. The reports (W0901·W0905) did not change on this corpus. Tests: `verify::tests::precondition_check_probes_scalar_leaves` (a `requires` that was undecided with the quantified definition is judged satisfiable), `proofs_are_not_vacuous`, `compiler/tests/lint.rs`.
- The remaining cost is mostly queries that neither the probe nor the full query decides (576 undecided of the vacuity check at the limit of 1,000,000 each, 62 undecided of precondition satisfiability). No metric sees this cost (backlog T2-10).
- Revert: if the two checks do not call `probe_base`, only the current query is solved. The storage format and the language do not change.
