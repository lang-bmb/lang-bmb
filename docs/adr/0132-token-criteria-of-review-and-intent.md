# ADR-0132: Handling the token criteria of H-REVIEW and H-INTENT

- Status: Accepted (approved by the maintainer 2026-11-13, recommendation (A) of backlog `T1-44h`)
- Date: 2026-11-13
- Related: ADR-0078 (hypothesis registration), ADR-0128 (`bench/review`), ADR-0130 (`bench/intent`), `dev-docs/measurements/t1-44f.md`, backlog T1-44f·T1-44h

## Context

- ADR-0078 put a token condition into the confirmation criteria of the two hypotheses: for H-REVIEW, the amount read on the spec side is 1/5 or less of the code side; for H-INTENT, the spec+examples side is 1/2 or less of the code+spec side.
- The ratios measured on the two frozen sets before any run are 1.752 and 1.571 (ADR-0128, ADR-0130).
- The measurement of `T1-44f`: the history of the repository has only four changes, and of the 94 executable functions with contracts in `lib/std`, 77 have a contract text larger than the body. Even assuming changes that rewrite the whole body, 1~12 functions reach 1/5, and even counting the examples as 0 tokens, 17 functions reach 1/2. Function-level material cannot make a set that reaches the two criteria.
- The cause lies in the hypothesis registration: when the criteria were set, the size ratio of contract text to body in proven code was not measured.

## Decision

1. **The criteria are not changed.** The confirmation and refutation rules of ADR-0078 stay as they are.
2. **The two sets judge only the detection rate.** The verdicts `bench/review` and `bench/intent` can give are "refuted" and "not confirmed". The runs on the measurement machine are not postponed.
3. **The token axis of H-REVIEW is measured on the history of the port.** Changes where the bodies of several functions change while the contracts stay the same appear after porting (`4D-02`). `T1-44f` is opened then.
4. **The token condition of H-INTENT is written in the registry as unsatisfiable on function-level items.** That the contract text is larger than the body is a property of the code, independent of how the examples are expressed. An item set of units larger than a function (the public contract of a module versus the whole implementation) is registered separately when material exists.

## Not chosen

- **Register the criteria again (B).** It is before any model run, but it is a change after seeing the values (operating rule: criteria are fixed before measurement).
- **Postpone the two hypotheses until after porting (C).** The detection rate axis can be measured with the current sets.

## Consequences

- The two hypotheses cannot become "confirmed" before porting. Premise E1 of gate D is written until then as the verdict of the detection rate and "token axis not measured".
- This handling is written in the two rows of the hypothesis table of `docs/ROADMAP.md`.
