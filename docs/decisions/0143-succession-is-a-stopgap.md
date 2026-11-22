# ADR-0143: The surface succession is a supplementary means below the model

- Status: Accepted (maintainer instruction 2026-11-16: the bold sentences of decisions 1~3. The elaboration is the author's interpretation)
- Date: 2026-11-16
- Related: ADR-0142 (the model of the language), DESIGN.md §1·§4, JUDGMENT.md §7, ADR-0040 (H-FAMILIAR), ADR-0100 (freeze), backlog T0-38·T0-42·T1-50

## Context

- The elaboration of ADR-0142 wrote the surface succession as the "first application" of the model and put it at the same height as the model. The definition in DESIGN §1 also called BMB a "Rust dialect".
- Maintainer instruction (2026-11-16): "The design philosophy ranks higher. Rust·Verus is no more than an effort to make up for the model's lack of prior training. If there is a means to grow out of it, that takes priority. If the project succeeds, a dedicated model could be made, or it could be adopted outside. But we cannot handle that, so we evaluate taking the current environment into account and do what we can."

## Decision

1. **The model is above and the succession is below.** The Rust·Verus succession is neither a definition nor a principle; it is a means of making up for the circumstance that models have not been trained on BMB. When it collides with the model (decision 1 of ADR-0142), the model wins. The first row of the table and the first item of the rationale of ADR-0142 are read in this sense.
2. **A means of getting out comes first.** If there is a means that does the same job without leaning on the trained distribution, it is used before the succession. The means that can be handled now are references and examples given in the context, notation filled in by tools, and diagnostics with a way to fix attached. Where that means is sufficient by real measurement at some place, the form at that place is set to the form that fits how the agent works and is not tied to the Rust form. The succession remains only at places where there is no means or it is insufficient.
3. **Evaluation is done in the current environment.** The value measured with the models usable now and the means that can be given now is the rationale. A dedicated model and outside adoption cannot be handled, so they are used neither as rationale nor as a premise of plans: a form that loses now is not adopted on the expectation that they will come, and what can be done now is not postponed because they are absent.

## Rationale

- Putting the succession at the same height as the model makes "it differs from Rust" a counter-rationale in itself. Lowering it to a means changes the question: is there a better way to make up for the lack of training at this place.
- Decision 3 narrows the "separated value" of decision 3 of ADR-0142. The means of separating must be something that can be given now (edition versus edition, an arm given a reference), and a hypothetical value after training is not a rationale.

## Consequences

- "Rust dialect" is taken down from the definition in DESIGN §1, and the ranking is written in the surface rules of §4 and JUDGMENT §7.
- The surface is frozen. A change that departs from the Rust form goes through the exception procedure of ADR-0100 and the maintainer's approval. No syntax changes right away by this decision.
- The list and measurement of the means of getting out are taken by `T1-50`. T0-38 (fine-tuning arm) is on the axis of a dedicated model, so it stays on hold.
