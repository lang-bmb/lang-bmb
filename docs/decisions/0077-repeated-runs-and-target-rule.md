# ADR-0077: Repeated measurement and the rule for judging against targets

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": backlog T0-12). Makes a declared exception to "once per (set, model, commit)" of ADR-0066 §5 · the scope of §2 and the minimum difference of §4 are supplemented by ADR-0086 (repeats for BVB·BRB, D per set) · the comparison of two arms is supplemented by ADR-0112
- Date: 2026-11-01
- Related: ADR-0021 (measurement procedure, single-attempt success rate), ADR-0034 (targets), ADR-0066 §5, ADR-0070, ADR-0076, backlog T0-12, 2A-16

## Context

- A formal measurement is one session per task and runs once per (set, model, commit) (ADR-0066 §5). A set has 10 to 45 tasks.
- A single-attempt success rate of that size varies from run to run, because the model samples. Without knowing the size of the variation, questions such as "does m1b 19/25 (76%) exceed the G2 target of 60%" or "is the write surface 2 tasks better than the original" cannot be answered (limitation 1 of utility §4).
- qwen3.8-27b is an open-weights model on the gateway of this environment, so repeats cost nothing (ADR-0066 §2).

## Decision

1. **Repeated measurement.** m1 and m1b are run **5 times** with `openai:qwen3.8-27b` at the same commit and under the same condition (`run_models.py --repeat K`, K = 1..5). Each run goes through the whole set with one session per task, and nothing is selected or discarded between runs. The results of all five runs are committed and reported together. The file name gets `-repeatK` and the result header gets `repeat`. This is not a task retry. Each run is an independent single-attempt measurement, and the reported figures are those of all runs.
2. **Scope.** Repeats are declared only for m1·m1b on qwen (`run_models.REPEAT_SETS`). sonnet (cross comparison, paid) and the other sets (BVB, BRB, variant editions) run once, as in ADR-0066 §5. The first repeated run is done under the condition of ADR-0076, before the original runs that `T0-13` makes (backlog plan 2026-11-01). If another set comes to need repeats, a new ADR widens the scope.
3. **Judging against targets.** The success rate of a set with repeats is reported as the mean of the per-run single-attempt success rates and a 95% interval. The interval is the run-level t interval (mean ± t(0.975, N−1) · s / √N). If the runs are equal and s is 0, the uncertainty of the sample size would vanish, so the interval never gets narrower than the Wilson interval of the pooled N × n trials (`bench/repeat.py`). Compared with a target: if the lower end of the interval is at or above the target it is **met**, if the upper end is below the target it is **not met**, otherwise it is **undetermined**.
4. **Comparing figures measured only once.** A difference between measurements without repeats (other sets, other models, variant editions) is claimed as a difference only when it is larger than the **minimum difference** D. D is the larger of the ranges (maximum − minimum) of the writing-task success rate seen in the repeated measurements, rounded up in percentage points. A difference of D or less is reported as "within run-to-run variation" and no direction is claimed. Until the repeat results exist, the rule of utility §4 (a difference of 1 or 2 tasks is not used for judgement) stands.
5. **Task stability.** Tasks that succeeded in all five runs, tasks that failed in all, and tasks that varied are reported separately. The list of varying tasks is input to failure attribution (the T0-16 method) and to the H-SMALL baseline.
6. **Fixing the mission targets (2A-16) is done after this result.** The targets are set together with this judging rule.

## Rationale

- **Why N = 5:** it is the smallest number at which the t interval with 4 degrees of freedom (t = 2.776) gives a usable width. The cost is 52 sessions per run (m1 15, m1b 37), 260 sessions for five. At about 5 to 10 minutes per session, with parallel sessions (INF-18), it fits in a day. With 5 runs per task, "always succeeds, always fails, varies" can be told apart.
- **Why a run-level interval:** the tasks are a fixed set (not a sample), and the variation comes from the model's sampling. Trials of the same task are not independent of each other, so a binomial interval applied directly to the pooled trials is too narrow. The t interval, which treats the per-run success rates as independent observations, captures that correlation. The Wilson lower bound keeps the interval from shrinking to 0 when the runs happen to be equal.
- **Relation to the single-attempt success rate principle:** CLAUDE.md·ADR-0021 "report the single-attempt success rate without retries". A repeat does not redo the same session; it makes an independent single-attempt measurement several times, and all runs are reported without selection, so it is within this principle.
- **Rejected:** (가) comparing a single figure directly with the target without repeats. With 25 tasks one task is 4%p, so the verdict depends on the sample. (나) repeating sonnet too. ADR-0066 §3 set the cross comparison to once, and it is paid. Differences in the cross comparison are read with the minimum difference of §4.

## Consequences

- Code: `--repeat` in `bench/run_models.py` (`REPEATS`·`REPEAT_AGENT`·`REPEAT_SETS`, `repeat_problem`), `condition_fields` in `bench/m1/harness.py` (`BMB_REPEAT` → `repeat` in the header), the summary `bench/repeat.py summary`.
- Tests: `test_repeats_are_declared_for_qwen_on_m1_and_m1b` in `bench/test_run_models.py`, `bench/test_repeat.py` (interval, verdict, summary, rejection of mixed runs). Gate job "bench repeated runs (ADR-0077)".
- The runs are made in an environment that reaches the measurement gateway (backlog T0-12). When the results exist, limitation 1 and §6 of utility §4 are revised, and the verdicts of the hypothesis registry are read with §3·§4.
- First repeat (2026-11-02, backlog T0-12, `dev-docs/measurements/t0-12.md`): qwen, commit `086bfcd4`, five times. **D = 30%p** (m1 writing range 6/10~9/10; m1b 20%p). G2 verdict: m1b writing met (lower bound 74.1%), m1 writing undetermined (lower bound 59.3%).
- Reverting: delete `--repeat` and it is ADR-0066 §5 as before.
