# ADR-0112: A/B comparisons are judged by the interval of the difference

- Status: Accepted (approved by the maintainer 2026-11-11 "all approved, recommendations accepted": overall review R3, backlog T0-39). It supplements ADR-0077 §3·§4 and ADR-0103 decision 2
- Date: 2026-11-11
- Related: ADR-0077 (repeated measurement, interval, minimum difference D), ADR-0086 (D per set), ADR-0103 (sequential repeats), ADR-0102 (G5-L), `bench/repeat.py`, `dev-docs/reviews/roadmap-2026-11-11.md` §2.2 M3

## Context

- ADR-0077 §3 is a rule for comparing the success rate of one arm with a target value (a run-level t interval, not narrower than the Wilson interval of the pooled trials). There is no rule for comparing two arms, so an interval was computed per arm and read by whether they overlap (2R-05). That two 95% intervals do not overlap is a much stricter condition than a 5% test of the difference.
- ADR-0103 decision 2 set the repeats at "up to 3". At 3 versus 3, even the most extreme result has a two-sided p of the exact permutation test of 2/20 = 0.10.
- The mean tokens per success are dominated by tail sessions (for sonnet `brb-write` the top 3 sessions are 75.7%, `utility.md` table F). The tasks are the same for both arms.

## Decision

1. **Two arms on the same task set are judged by the difference.** The difference in the authoring success rate (B − A) is given by the Welch 95% interval of the run-level success rates (degrees of freedom rounded down). This interval is made not narrower than the Newcombe interval of the pooled trials (the interval of the difference built from two Wilson intervals). The reason is the same as for the floor of ADR-0077 §3: with few runs, the between-run standard deviation underestimates the sampling uncertainty. If the interval does not contain 0, a direction is claimed, and if it does, it is "within variation". The two-sided p of the exact permutation test (over all relabellings of the runs' arms) is reported with it.
2. **Repeats are at least 4 per arm.** In an A/B comparison registered after this ADR, when repeats become necessary (ADR-0103 decision 2: the difference of the first run is within D), they are raised to 4 per arm. If the interval excludes 0 at fewer than 4 per arm, it is marked "provisional".
3. **Tokens are compared by the per-task ratio.** For each task, the ratio B/A of the mean session tokens of the two arms (the mean over runs if there are repeats) is computed, and the median over tasks, its exact order-statistic 95% interval and the geometric mean are reported. The mean tokens per success continue to be reported alongside (the target values of ADR-0034·0102 are in that unit).
4. **A comparison measured only once stays as in ADR-0077 §4** (minimum difference D, D per set is ADR-0086).
5. **Comparisons already judged are not judged again.** A new computation is marked "post hoc" and only reported alongside.

## Rationale

| Option | Verdict |
| --- | --- |
| **Interval of the difference + sampling floor + 4 per arm (accepted)** | A comparison of two means is made by the interval of the difference (established method, JUDGMENT S1). The floor is consistent with ADR-0077. At 4 versus 4 the minimum p of the exact test is 2/70 = 0.029, so a verdict can be reached |
| Interval overlap (now) | The power is low. It was never set as a comparison rule |
| Welch only | It leans on the standard deviation of 3 to 4 runs. If the runs happen to be equal, the interval shrinks to 0 |
| Judge by the permutation p only | At 4 per arm there are only a few possible p values, so the verdict is coarse. It is used in the report |
| A paired test that treats tasks as the sample | The tasks are a fixed set (ADR-0077 rationale). It is not used for the success rate, and is used as the per-task ratio only for tokens, where the difference between tasks is large |

- **Post hoc check (not a verdict):** the six numbers of 2R-05 (before 20·20·21, after 24·23·22, 25 tasks) give a difference of +10.7%p, a Welch interval of +2.2~+19.2%p, a Newcombe interval of −0.4~+21.8%p and a permutation p of 0.10. Under this rule too it is "within variation" (the floor narrowly contains 0). Overall review §2.2 M3 gave only the Welch interval. The grounds for changing the rule are not that result but the power of interval overlap and the limit of 3 versus 3.

## Consequences

- `bench/repeat.py diff --a FILES... --b FILES...` and its tests (`Diff` in `bench/test_repeat.py`, gate "bench repeated runs").
- The number of repeated runs goes from 3 → 4 per arm, so measurement machine time grows. It is used first for the comparisons of existence hypotheses (ADR-0111 decision 5).
- In the verdict of gate B (ADR-0111 decision 2), "the difference is within the interval of the judgement rule" is the "within variation" of this rule.
