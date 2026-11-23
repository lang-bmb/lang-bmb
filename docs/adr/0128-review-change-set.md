# ADR-0128: Freezing the H-REVIEW change set, and its limits

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": freezing the task set of a hypothesis defined by ADR-0078. The verdict criteria are not changed)
- Date: 2026-11-12
- Related: ADR-0078 (H-REVIEW·H-INTENT), ADR-0021 (freezing), ADR-0066 (measurement models), ADR-0077 (verdict rules), `bench/review`, backlog T1-44·T1-44d·T1-44f

## Context

- ADR-0078 measures H-REVIEW as "a review model reviews a change set with planted defects, split into a spec diff (contract differences and the `bmb weaken` verdict) and a code diff", and froze the confirmation criterion as "the detection rate is equal or higher and the tokens read are 1/5 or less". It left the freezing of the set, the review prompt and the judgement to the ADR of the task set.
- A defect that passes the proof gate is a defect of the specification (DESIGN premise E1). So the programs before and after a change must both be proven.

## Decision

1. **Set.** `bench/review`: 143 changes (71 defective, 72 sound) derived from the 45 frozen BVB tasks. There are no hand-written programs.

   | Kind | Before → after | Label | Count |
   | --- | --- | --- | --- |
   | `contract-weakened` | reference solution → weak contract | defect | 45 |
   | `contract-restored` | weak contract → reference solution | sound | 45 |
   | `contract-weakened+body` | reference solution → weak contract + a body mutant proven under it (with a witness input for an output the hidden contract rejects) | defect | 26 |
   | `body-only` | reference solution → a body mutant proven under the same contract | sound | 27 |

   A body mutant is a single-site mutation of `bmb mutate`, up to two from the front per task and kind (the rule was fixed first, then measured). The committed `changes/` is the frozen artifact, and the compiler commit used to generate it is in `set.json`.
2. **Two editions.** The spec edition is the diff of the contract text (signature, `requires`·`ensures`, the spec fns they call) and the output of `bmb weaken`; the code edition is the diff of the whole program. Both receive the intent sentence and the same frozen prompt.
3. **Judgement.** The answer is a single `{"verdict": "defect" | "ok", "reason": ..}`. A detection is `defect` on a defective change, a false alarm is `defect` on a sound change, and a malformed answer is counted separately and is not a detection. The confirmation and refutation rules are those of ADR-0078 unchanged, and D is the 30%p of ADR-0077. The false alarm rate is only reported.
4. **The only evaluators are model reviewers.** No calibration on a human sample is done (maintainer decision 2026-11-11, `2H-05` cancelled). The verdict of this set is a verdict limited to model reviewers.
5. **Gate.** `gen.py --check` (structure) and `test_review.py` (proof of the 143 programs, labels, determinism of the editions, two scripted reviewers, judgement). Regeneration (`--regen`) depends on the proving power of the compiler, so it is not put in the push gate.

## Rationale

- Sound changes are needed to tell apart a reviewer that always answers "defect". Changes where the body changes are needed to measure whether the spec edition works without looking at the body.
- `weak → weak + mutant` (a body defect with the contract unchanged) was not included. The contract diff is empty, so the spec edition cannot catch it in principle, and that gap favours the hypothesis. It is written in the README.

## Consequences

- **On this set the token criterion (1/5) cannot be reached (a value measured deterministically before any model run).** The token ratio spec/code is 1.752 overall, and about 1.58 on the 53 where the body changes. A task is one function, so the code diff is 87~228 tokens, and the output of `bmb weaken` alone is 85~210 tokens. So the verdict this set can give is "refuted" (the detection rate is lower by more than D) or "not confirmed". The criterion is not changed.
- Measuring the token axis needs a set with multi-line body changes: `T1-44f` (changes of real size, for example the history of `lib/std` or the port).
- The verdict of `bmb weaken` separates the labels in 122 of the 143 (21 undecided). The detection rate of the spec edition may be the result of reading the tool verdict, and the table is also given per kind.
- The H-INTENT set (`T1-44e`) is added to the same harness as data (the part list of an edition and `compare.token_ratio`).
- The real model runs are done on the measurement machine.
