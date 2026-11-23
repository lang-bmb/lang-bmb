# ADR-0089: An obligation the probe could not decide is first solved by the ground instance retry at the probe's share

- Status: Accepted (autonomous, AUTONOMY.md §4). Supplements the order of decisions 2·3 of ADR-0067 (probe, whole query, retry → probe, early retry, whole query, retry). The text and meaning of the queries and the rules for using each answer (ADR-0023) are unchanged
- Date: 2026-11-04
- Related: ADR-0023 (ground instance retry), ADR-0067 (probe), backlog 1B-46, 1B-53

## Context

- The cost of a false obligation came from the order (the 1B-46 diagnosis). The fail corpus `geo::off_diag` (E0802, a confirmed counterexample) is probe `unknown` 50,186, whole query `unknown` 5,000,178 (the limit), ground instance retry `sat` 493. To get one counterexample one whole limit is thrown away, and the cheap attempt is at the very end.
- In the first attempt (2026-11-04), accepting a complete `sat` of the early retry produced a false confirmed counterexample in `digest` of `lib/sha256.bmb`. The cause was that the ground instances of a `spec fn` with a struct result were built for the first field only, and 1B-53 fixed it. After that the same experiment has 0 false counterexamples.

## Decision

1. **Order.** If there is a probe, its answer is neither `unsat` nor `sat`, and there are quantified definitions, the ground instance retry is solved before the whole query with the same limit as the probe (`Z3_RLIMIT / 100`). `unsat` is a proof, and a `sat` when the instances are complete (`complete` of `ground_instances`) makes that model the counterexample (the rule of ADR-0023 as is). Otherwise the whole query is solved as now, and the full-limit retry when that is `unknown`.
2. **Script.** In `--emit-smt` this attempt appears as `; retry of obligation k with ground instances at the probe's share`, containing the limit command, as one step of that obligation (the `retry of obligation` step of `scripts/solver_stability.py`).
3. **Proof cache.** The retry mark of an entry is two bits (`cache::EARLY_RETRY`, `cache::LATE_RETRY`, format `bmb proof cache v4`). A proof that came from the cache emits the same script as the run that stored it (ADR-0067 consequences, 2V-03). A file in the old format is read as an empty cache.

## Rationale

- **Cause and the layer it belongs to:** the counterexample query cannot build a model over quantified definitions and runs to the limit (the same cause as ADR-0064·0067). The ground instance query has no quantifiers and is decided cheaply. The layer is the order in which the encoder asks the solver.
- **Standard practice:** trying the same goal with several encodings, shortest share first (the Sledgehammer slices and Why3 strategies in the rationale of ADR-0067). Putting the cheap attempt before the expensive one is the ordering rule of that method.
- **Measurement (seed 0, Z3 4.15.4, rlimit 5,000,000, same day and same machine, current first):**

  | Indicator | Current | This decision |
  | --- | --- | --- |
  | Fail corpus (sum of `rlimit` in the status lines of `tests/verify/fail`) | 15,994,677 | 11,148,987 (−30%) |
  | V4 | 7,898,495 | 7,999,335 (+1.3%) |
  | V8 | 23,914,744 | 22,810,907 (−4.6%) |
  | V10 | 12,021,529 | 12,806,234 (+6.5%) |
  | Library (each of `lib/*.bmb` alone) | 9,020,409 | 9,805,114 |
  | V7 · V12 | 0 · 297 | 0 · 297 |

  - Only two lines of the fail corpus change: `geo::off_diag` 5,050,957 → 50,779 (the same counterexample `a = 1`), `nl::f` 5,301,037 → 5,351,273 (nonlinear, the early retry cannot decide it either). Diagnostic codes, locations, counterexample values and definition states are unchanged.
  - The increase on the proved side is the obligations the early retry could not decide and on which it only spent its share (50,000). No proof is lost: if it cannot decide, what follows is the same text and the same limit as the current order.
- **Candidates:** (a) the current order (the retry at the very end). (b) accepting only `unsat` from the early retry: tried, and the fail corpus gets slightly more expensive (+50,236, +493) with no gain. The cost of a false obligation is on the counterexample side. (c) this decision. The deciding criterion is S2 (the time an agent waits): in an agent's writing loop a false obligation comes up at every first attempt, and one counterexample goes from one whole limit (current) to two probe shares. The cost on the proved side is within the budget (10%) of V4·V10.

## Consequences

- What is proved now is still proved, and a counterexample is confirmed only by the same rule (a model of complete instances). The soundness of that rule rests on ADR-0023 and the test of 1B-53 (`every_field_of_a_struct_result_has_ground_instances`).
- A proof with quantified definitions that the probe cannot decide spends at most one more probe share (50,000) per obligation.
- The proof cache format changes to v4 (the first run proves again).
- Reverting: take out the early retry block of `verify_pass` and it is the current order.
