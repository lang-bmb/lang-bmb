# ADR-0142: The language is modelled on how the agent works

- Status: Accepted (maintainer instruction 2026-11-16: the bold sentences of decisions 1~3. The elaboration under them is the author's interpretation, and the protocol is set by `T0-42`)
- Date: 2026-11-16
- Related: DESIGN.md §1·§2·§4, ADR-0001 (user = agent), ADR-0040 (premise E3, H-FAMILIAR), ADR-0045 (judgement criteria), ADR-0100 (freeze), ADR-0111 (hypothesis grades), the decision criterion of the seventeenth plan of the backlog, T0-38·T0-42

## Context

- Maintainer instruction (2026-11-16): "Hangul was made by modelling the human organs of speech. A language for AI should be made by modelling the working mechanism of AI. We make, by real measurement, a language that AI agents want to write. A new language must be compared taking into account that the model has not been trained on it."
- ADR-0001 defined the user as the agent, but did not write down where the form of the design is drawn from. "A language the agent wants to write" existed only in the seventeenth plan of the backlog (first applied by ADR-0141).
- The measurements have training mixed in. In BRB (T0-10·T0-11), BMB versus Rust was 2/24 versus 24/24 for qwen and 21/24 versus 24/24 for sonnet (15.6 times the tokens), and the difference between models was larger than the difference between arms (`dev-docs/measurements/t0-10.md` §6). In this difference the share of the design and the share of not being trained were not separated (T0-38 on hold).

## Decision

1. **The model of the design is how the writing side works.** The form of the language and the tools is justified by which property of how the agent works it is fitted to. Human habit is not a rationale in itself. There is one way human habit becomes a rationale: the fact that it is in the trained distribution of the model.

   | Property of how it works | Design fitted to it |
   | --- | --- |
   | It writes on a trained distribution | The surface is the Rust·Verus succession (DESIGN §4) |
   | Context is finite and there is no memory between sessions | Map, slice budget, rationale bound to the code (principles 2·5·7) |
   | Output is probabilistic and mistakes are mixed in | Proof gate, counterexamples and structured diagnostics (principles 4·6·8) |
   | The same meaning can be emitted in several forms | One expression and one normal form, notation that is determined mechanically is done by tools (principles 6·8) |
   | It reads and writes through tool calls | Tools are the only path (principle 6) |

   The table is a correspondence, not a demonstration. For the state of demonstration, the hypothesis registry (ROADMAP) is canonical.

2. **"Wants to write" is judged by real measurement.** The self-report of an agent is material for forming hypotheses, not a rationale. The rationale is the friction of the porting ledger (number of workarounds, hand-made helper code, gate runs and undecided counts, session tokens) and the success rate and tokens of the benches.

3. **Comparisons take not being trained into account.** The difference from a trained language is the sum of the cost of the design and the cost of not being trained.
   - The design is not judged by a difference in which the two are not separated. A feature is not removed or turned back to the Rust form only on the result of losing to a trained language, and nothing is adopted only for the reason that it is the Rust form.
   - The first-rank rationale for a design decision is a comparison in which both arms are equally unfamiliar: edition versus edition of BMB (ablation measurement), a control that is unfamiliar to the same degree (Verus).
   - A comparison with a trained language reports the unseparated value (the actual gain and loss on current models, the verdict of premise E3) and the separated value apart. The protocol of the means of separating is set by `T0-42`.

## Rationale

- The surface succession is not an exception to this decision but its first application: the trained distribution is also how it works. But the succession comes down from an axiom to a hypothesis (H-FAMILIAR). If a separated measurement points to another form, that form wins.
- Why self-report is excluded from rationale: it is the same as the operating rule (claims by code path and measurement), and an agent's statements of preference do not reproduce.
- Premise E3 put the cost of not being trained into the price paid. That verdict (the gain and loss now) is left as it is, and only the value used for design judgement is separated. Mixing the two into one number leads to discarding a right design because of a temporary cost, or to calling a defect of the design a temporary cost.

## Consequences

- A fix that changes the accepted language still goes through the exception procedure of ADR-0100 and the maintainer's approval. This decision sets the direction of proposals and the kind of rationale.
- Two judgement questions are added to JUDGMENT §7. The ADR of a new feature or surface states the property it is fitted to and the measured rationale.
- The separating measurements need model runs. `T0-42` sets only the protocol, and the runs follow the queue and approval. The hold on T0-38 (fine-tuning arm) stays.
