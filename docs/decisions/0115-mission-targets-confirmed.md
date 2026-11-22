# ADR-0115: Mission target values confirmed, and the judgement rules

- Status: Accepted (approved by the maintainer 2026-11-11 "recommendations A1~A7 accepted", item A4. Backlog 2A-16). It closes "the target values are provisional" of ADR-0034
- Date: 2026-11-11
- Related: ADR-0034 (mission targets), ADR-0041 (G8·G9), ADR-0040 (G7), ADR-0077·0086·0112 (interval and judgement of the difference), ADR-0102 (the two layers of G5), ADR-0103 decision 3 (the reference model for claims), 2A pass criterion 4

## Context

- ADR-0034 left the target values provisional and set that they are confirmed once by the results of the first formal measurement (1E-10). The formal measurements (T0-10·T0-11), the repeats (T0-12), the write surface (T0-13) and part of the evidence stage are done.
- In the meantime the judgement methods were set separately: the reference model (ADR-0103), the interval (ADR-0077), the difference of two arms (ADR-0112), the two layers of G5 (ADR-0102). These rules are not attached to the table of target values.
- Some targets have a shape that cannot be reached because the comparison group is at the ceiling. In BVB, Verus is 43/45 with sonnet, and "+10%p" is arithmetically impossible.

## Decision

1. **The target values are confirmed as the values in the mission target table of `docs/ROADMAP.md`** (G1~G9, G5 is the two layers of ADR-0102). The values are not lowered or raised after seeing the results. The next change is only at the end of a stage (2H, 3E) (ADR-0034 as it is).
2. **The reference model for the verdict is the frontier cross-check model** (ADR-0103 decision 3, currently `claude-sonnet-5-5`). The values of the default run model (qwen) are reported alongside in the same table, and the gap is tracked by H-SMALL.
3. **The verdict uses intervals.** A target value for one arm uses ADR-0077 §3 (met, not met, open), a comparison of two arms uses ADR-0112 (the interval of the difference), and a comparison measured only once uses the minimum difference D of ADR-0077 §4 and ADR-0086.
4. **Cannot-discriminate is recorded separately.** In a comparison target, if the success rate of the comparison group is at least 90% or at most 10%, that measurement "cannot discriminate". It is neither met nor not met, and the target stays open. It is measured again on a set or a metric that discriminates (tokens, mutation score, lifecycle unit).

## Rationale

| Option | Verdict |
| --- | --- |
| **Confirm the values as they are + judgement rules (accepted)** | Targets are not revised after seeing the results (JUDGMENT C3, the same principle as ADR-0102). Only the missing judgement method is filled in |
| Calibrate to the measured values | It is the "first calibration" that ADR-0034 allowed, but with the current results a calibration in either direction is an adjustment after the fact |
| Change the comparison targets at the ceiling to "equal" | A relaxation with no grounds. Leaving them as cannot-discriminate and measuring by another metric is more honest |

## Consequences

- The "provisional" sentence of the ROADMAP mission target section is replaced by this decision. 2A pass criterion 4 is met.
- The verdict on the current values (2026-11-11): G1·G6·G9 met, G4 accuracy and token ratio at 10,000 definitions met (sonnet), G2 m1b met, G2 BVB cannot discriminate (Verus 43/45), G3 met (sonnet 186/186), G5 task unit not met, G5 defects cannot discriminate (0 hidden failures for Rust), G7·G5-L not measured, G8 direction agrees (pure core).
