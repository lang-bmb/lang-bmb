# ADR-0119: The mutant edition of H-ORACLE (oracle strength)

- Status: Accepted (approved by the maintainer 2026-11-11 "approved, autonomous execution/verification approved". Backlog T0-40. The judgement rule is registered by this commit, before the computation). "The BMB reference contracts kill all mutants" of decision 7 was not true (the contracts of the reference solutions are not complete specifications: 29/96): the gate compares the kill results of the reference solutions with recorded values (`bench/brb/oracle_strength_ref.json`). The judgement rule (decision 4) stays as it is
- Date: 2026-11-11
- Related: ADR-0040 (hypotheses and refutation conditions), ADR-0034·0115 (G5, the cannot-discriminate rule), ADR-0111 (H-ORACLE is an existence hypothesis, one carry-over), ADR-0112 (the interval of the difference), ADR-0103 decision 3 (reference model), backlog 2H-04 (`final_sources`), 2A-10 (the judgement method of BSB), `dev-docs/reviews/roadmap-2026-11-11.md` §2.2 M5

## Context

- The current verdict on H-ORACLE ("a proof pass misses fewer hidden defects than a test pass") is a comparison of the rate of "self-check passed & hidden failure". It failed to discriminate four times (T0-07 two models, T0-10, T0-11, T0-08 sonnet): if the model produces no defects on the Rust side, the denominator is 0.
- That verdict leans on whether the model produces defects. What the hypothesis asks about is the strength of the oracle: given a defective implementation, does the self-check of that arm (contracts + gate, or tests) catch it.
- The 24 tasks of BRB have 96 Rust mutants as hidden judgement material, and the hidden cases kill all of them (`harness.py adequacy`, gate). There are no equivalent mutants.
- The result rows hold the final files of the session (`final_sources`, 2H-04): the tests the agent wrote for the Rust arm, the contracts the agent wrote for the BMB arm. It can be recomputed without paid runs.
- Whether a contract kills a mutant is already judged by BSB (`bench/m1/bsb.py`): if `bmb verify` proves, looking only at the contract, that the contract does not allow the output of the mutant on a witness input, it is killed.

## Decision

1. **Metric: mutant kill rate.** For each task and each Rust mutant of that task, it looks at whether the self-check of the session catches the mutant.
   - **Rust arm:** in the final crate of the session, the task function is replaced by the mutant and the agent's tests run (`cargo test`, the same proptest settings and fixed seed as the session). If they fail, it is killed. If it does not build, that mutant is excluded for that session (the count is reported).
   - **BMB arm:** a witness is an input among the hidden cases on which the mutant gives a result different from the reference solution (the judge already computes this). If `bsb.kill_check` proves that the agent's final contract excludes the output of the mutant on one of the witnesses, it is killed. If the mutant ends in a panic or overflow on the witness, it counts as killed by the gate: that the final source is fully proved means there is no such execution. Undecided is not killed.
2. **Sessions in scope.** Only sessions that passed the self-check (`final_check_ok`). Tasks are paired, and only tasks for which both arms have a session in scope are used for the verdict. The number of dropped tasks is reported per arm.
3. **Data (fixed before the computation).** Reference model (sonnet): Rust arm `bench/brb/results/2026-11-04-claude-sonnet-5-5-rust.jsonl`, BMB arm `bench/brb-write/results/2026-11-04-claude-sonnet-5-5-bmb.jsonl` (the canonical surface, ADR-0101). Auxiliary (qwen): `bench/brb/results/2026-11-03-qwen3.8-27b-rust.jsonl` and `bench/brb-write/results/2026-11-02-qwen3.8-27b-bmb.jsonl`. `brb-hard` is added under the same rule once its tasks have mutant material.
4. **Verdict (registered by this commit).** The between-arm difference of the per-task kill rate (killed / number of mutants) is averaged over tasks (BMB − Rust). The p of the paired exact permutation test (sign flipping, two-sided) and the 95% interval of the difference (t interval of the paired differences) are given together.
   - **Holds:** for the reference model the interval is above 0 and p < 0.05, and for the auxiliary model the sign of the difference is the same.
   - **Refuted:** for the reference model the interval is below 0 and p < 0.05.
   - **Cannot discriminate:** the mean kill rate of the Rust arm of the reference model is at least 0.9 (the rule of ADR-0115). In this case the only conclusion is that a harder mutant set is needed.
   - **Cannot be judged:** otherwise. The verdict on H-ORACLE moves to the port twin (3P-05). This is the one carry-over of ADR-0111 decision 3.
5. **Sensitivity (not used for the verdict, reported with it).** (a) the kill rate without the mutants that show only by panic or overflow, (b) the Rust kill rate with proptest off, (c) the table per mutant.
6. **Known biases and their direction.**
   - A kill on the BMB side is looked at only on the witnesses of the hidden cases. If the contract excludes the mutant on another input, it is not counted. This is to BMB's disadvantage.
   - A kill on the Rust side is a real execution, so it is exact.
   - The mutants are written in Rust, and only their behaviour (input → output) is used for the BMB arm. Whether a BMB implementation of the mutant can be proved is not asked: if the contract excludes that output, it cannot be proved, by soundness.
   - The 96 mutants were picked by the task author as "common defects". They are not the distribution of the defects that models really produce. So this edition measures the strength of the oracle and does not measure the defect rate (G5).
7. **Tool and gate.** `bench/brb/oracle_strength.py` (not subject to the freeze: an analysis tool that reads stored results). The gate checks with the sessions of the reference solutions (oracle agent): the BMB reference contracts kill all mutants, and the kill rate of the Rust reference tests is fixed at the value that the reference tests of the task determine. The contracts and tests of the weak control (a solution that passes the self-check but is caught by the hidden judgement) do not kill the mutant that corresponds to its own defect.

## Rationale

| Option | Verdict |
| --- | --- |
| **Kill rate with the existing mutants of the tasks (proposed)** | The mutants are already frozen and there are no equivalent mutants. Both arms get the same defect set. There are no paid runs. Measuring the strength of a test set by mutation analysis is an established method (JUDGMENT S1) |
| Operator-based mutation in both languages | There is no mutation tool for Rust, so an external tool (pinned, S2) or a new tool is needed, and equivalent mutants have to be sorted out |
| Keep the current verdict | It discriminates only if the model produces defects. It failed four times |
| Only harder tasks (T0-08) | It already failed to discriminate with sonnet. It leans on task difficulty in the same way |

## Consequences

- If accepted, `oracle_strength.py` and its tests are written and the result is committed as `dev-docs/measurements/t0-40.md`. The evidence column of H-ORACLE in the hypothesis registry and the utility report are updated.
- Even if the verdict is "holds", the scope is "function-unit tasks, the task author's defect set". The verdict on the lifecycle unit (H-TASK) stays with gate B.
