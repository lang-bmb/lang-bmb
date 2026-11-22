# ADR-0078: Registration of six supplementary hypotheses (H-EFFECT, H-INTENT, H-REVIEW, H-SMALL, H-REPAIR, H-COMPAT)

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": recommendation (나) of §3 of the review `dev-docs/reviews/hypotheses-2026-11-01.md`). It adds rows to the hypothesis registry of ADR-0058 §4, so the pass criteria of T1 cover more
- Date: 2026-11-01
- Related: ADR-0034 (targets), ADR-0040·0058 (hypotheses), ADR-0030 (effects), ADR-0066 (measurement models), ADR-0076 (measurement condition), ADR-0077 (judging rule), backlog T1-42

## Context

- The maintainer's draft of twenty "supplementary hypothesis candidates" (appendix A of the review document) was checked against the design, the decisions and the backlog (§2 of the same document). Of the gaps the utility report (`dev-docs/reports/utility.md`) showed, three were not in the registry: the test area of stateful code, review in which a person looks only at the specification (DESIGN premises E1·E2(d)), and the gap between models.
- What was approved: six are registered. The evaluator of H-INTENT·H-REVIEW is a review model by default, and a human sample is for calibration. The verdict figures are frozen at registration, aligned with the G targets. The "relaxation" side of H-GRADUAL and H-PRIOR is reframed without changing DESIGN.

## Decision

### 1. Hypotheses registered (verdict criteria are frozen before measurement)

All comparisons are read with ADR-0077. A figure with repeats has a 95% interval, and a difference measured only once is a difference only when it exceeds the minimum difference D. As in ADR-0066, qwen is the first model and sonnet is the cross comparison.

| ID | Hypothesis | Measurement | Verdict criterion (frozen) |
| --- | --- | --- | --- |
| H-EFFECT | If I/O, state and external dependencies are exposed as effects (ADR-0030), the pure core that is proved separates from the shell that needs tests, and the remaining test area is small | On a set of tasks with input/output and state (BRB shape, the same hidden verdict as the Rust arm), the remaining test area of successful sessions (`test_needed_defs` / number of definitions) and the lines of agent test code | Confirmed: the remaining test area is at most 20% of the definitions and the lines of test code are at most 1/3 of the Rust arm (both models). Refuted: either one is exceeded. The G8 target for pure tasks (1/10) stands, and this is a separate target for code with effects (principle 4: tests of the trust boundary remain) |
| H-INTENT | Showing examples and counterexamples generated from the specification catches a specification that deviates from the intent better than a review that reads code and specification together | A set of tasks seeded with specifications changed to deviate from the intent (the natural-language intent is unchanged; the contract is consistent but means something else). Arm (a) review of code + specification, (b) review of specification + deterministically generated examples and counterexamples. The evaluator is a review model (qwen, sonnet as cross) | Confirmed: the detection rate of (b) is at least that of (a) (within D counts as equal), and the tokens read by (b) are at most 1/2 of (a). Refuted: the detection rate of (b) is lower than (a) by more than D |
| H-REVIEW | A reviewer who sees only the specification diff catches as many defects as a code diff review | A set of changes seeded with defects is split into the specification diff (the textual difference of the contracts and the `bmb weaken` verdict) and the code diff, and a review model reviews them | Confirmed: the detection rate is equal or higher (within D counts as equal), and the amount read (tokens) is at most 1/5. Refuted: the detection rate is lower by more than D |
| H-SMALL | Improvements to tools and diagnostics narrow the success-rate gap between a weak model and a strong model | The difference of the writing success rates of qwen and sonnet on the same frozen set. The baseline is the first run under the ADR-0076 condition (the originals that T0-13 runs again). The 2/24 of T0-10 has mixed conditions and is not used as the baseline | Confirmed: the gap on BRB (BMB arm) at a later commit (after tool and diagnostic improvements) is at most half the baseline gap, and the decrease is larger than D. Refuted: the gap stays the same or grows (including within D) |
| H-REPAIR | The counterexample and the repair suggestion (`fix`) of a failure diagnostic reduce repair iterations | Ablation: on m1b-write, (a) the current diagnostics, (b) an edition with the counterexample values and `fix` removed from the tool results (a harness filter; only what the model sees differs). qwen | Confirmed: the sum of rejected commits (`commits_rejected`, T1-40) and failed `verify` per successful session in (a) is at least 40% less than in (b). Refuted: not less (including within D) |
| H-COMPAT | When a contract changes, backward compatibility is decided by proof and the call sites that break are pinpointed exactly | Deterministic: on a history mixing compatible and incompatible contract edits (the BCB generator), the `bmb weaken` verdict and the call sites of `impact` are compared with the generator's ground truth | Confirmed: verdict accuracy 100% (undecided counts as a failure) and recall of affected call sites 100%. Refuted: there is even one wrong verdict or missed call site |

### 2. Not registered (reasons)

- **H-DECIDE** is not a claim about the language but an engineering property of the judge. It is kept as an indicator: every formal run reports the `undecided` ratio of the hidden specification check (T0-28).
- **H-REFINE** goes into the task set of H-SPLIT (division of labour with the contract fixed first) as a "refinement writing" arm (stage 3).
- **H-PARSE** becomes the next edition that adds external data boundary tasks to the task set of H-ENCAP (type invariants, ADR-0060) (T1-47).
- **H-GRADUAL** goes against principle 4 ("no proof, no binary") and a non-goal ("rapid prototyping"). Reframed within DESIGN it is "code that starts from an escape hatch with a rationale (ADR-0029) has its unproved area shrink as edits accumulate". It is looked at again after the escape hatch implementation (2E-02~2E-04).
- The "rule relaxation" of **H-PRIOR** changes the §4.1 ledger (ADR-0045) and principle 8, so it is not registered. The "automatic conversion" side is already covered by H-FAMILIAR, the syntax re-evaluation (2R-04) and the Rust habit diagnostics (2D-29).
- **H-CONC, H-COST** are language scope that is not in DESIGN (ADR-0016). **H-FFI** hangs on the ecosystem gate (CLAUDE.md: after principles 4·6 are demonstrated). **H-CAP** is reframed after effects (2F) as an extension of the H-GAME set, **H-PROV** at the multi-agent measurement (H-SPLIT), and **H-TRACE** after rationale (2E-04) as "the accuracy with which an agent links requirements to clauses". The "equivalence proof" of **H-MIGRATE** cannot be measured now because the prover has no Rust semantics. It can be measured if changed to hidden differential tests and contract strength.

### 3. Evaluator

The evaluator of H-INTENT·H-REVIEW is a review model. It is the two models of ADR-0066; qwen is used first and sonnet as the cross. The review prompt and the verdict (was the defect found) are frozen together with the task set. A human sample is used only to calibrate whether the review model's verdict agrees with a person's verdict, and its size and method are set in the freezing ADR of the task set.

## Rationale

- **There must be hypotheses that measure the premises:** DESIGN premises E1 (the object of review moves to intent and specification) and E2(d) (specification validity) are BMB's first premises, yet the registry had no hypothesis measuring them. H-INTENT·H-REVIEW fill that place.
- **How the figures were aligned with the G targets:** the test code target for pure code (G8 1/10) stands, and code with effects has a boundary where principle 4 leaves tests, so 1/3 is set separately. The token ratios (1/2, 1/5) are in the same unit as the "total tokens" axis of G5. All success-rate comparisons are read with the rule of ADR-0077, so that the variation of a single measurement is not mistaken for a difference.
- **Why H-COMPAT is deterministic:** the verdict needs no model. The existing evidence (2D-19: on two editions of the BCB history, the verdicts of 2,005 edited definitions all agree with the generator's ground truth) covers the verdict accuracy side, and only call-site recall remains (T1-46).

## Consequences

- Six rows in the registry ("hypothesis registry" in `docs/ROADMAP.md`). H-COMPAT is "deterministic evidence (partial)", the rest are "not measured".
- Backlog: task sets and tools T1-43 (H-EFFECT, after 2F-02), T1-44 (H-INTENT·H-REVIEW: the set of specifications deviating from intent, the example and counterexample generation tool, the review harness), T1-45 (H-REPAIR ablation harness), T1-46 (H-COMPAT call-site recall), T1-47 (H-ENCAP boundary parser edition), indicator T0-28 (undecided rate of the judge). For H-SMALL the original runs of T0-13 are the baseline.
- The pass criterion of T1 ("every hypothesis in the registry is confirmed, refuted or on hold") applies to these six too.
- Reverting: take the rows out of the registry and leave the status as "on hold (reason)" (the rows are not deleted).
