# ADR-0135: Induction lemma drafts come from `bmb fix --induction`, and only those the gate proves

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": the implementation of a tool defined by ADR-0062 §5. The language surface and the proof gate are not changed)
- Date: 2026-11-14
- Related: ADR-0062 §5 (automatic induction as a tool), DESIGN principle 6, backlog T1-24, `tests/run/max_list.bmb` (BVB `vc-vv0039`)

## Context

- ADR-0062 §5: lemmas that are determined mechanically are written by a tool. When `bmb fix` recognizes such a form at E0811, it proposes a `proof fn` draft and a call site.
- Until now `bmb fix` read only the diagnostics of the checker and did not run the prover. E0811 is a diagnostic of the prover.
- Whether a draft is right cannot be known from the form alone. Two predicates with the same parameter types can be unrelated.

## Decision

1. **The place is `bmb fix --induction`.** The prover runs only when the flag is present. The cost and output of `bmb fix` without the flag are unchanged. The diagnostics of `bmb verify` do not carry a draft: not every verification pays the cost of trying to prove candidates for each failed proof.
2. **Generate and filter.** The tool writes candidates in a fixed order and applies only the first candidate for which the gate proves the lemma and the target function together. A draft that is not proven is not shown. So the correctness of a draft is guaranteed by the gate, not by the recognition rule.
3. **One form is handled: recursive predicate `R` ⇒ predicate `Q` of the same parameters.** A `Q(args)` reached by unfolding the postcondition through the non-recursive predicate definitions of the unit, and a recursive predicate `R` of the same unit with the same parameter types whose `decreases` is a single unsigned parameter. The body of the lemma is `if p > 0 && R(.. p - 1 ..) { self call }`: it calls the induction hypothesis only when it can be called (when `R` also holds at the previous step). The call goes before the result of the function, and is written only when the result is a variable.
4. **Applying is the agent's choice.** Without `--write` it only reports (the same as other fixes).

## Not chosen

- **As the `replacement` of the E0811 diagnostic.** One fix is the text of one site, but a draft is two sites (the lemma, the call), and there is the cost problem of 1 above.
- **Narrow the recognition rule and propose without verification.** Wrong drafts go out, and the agent spends turns fixing them.
- **The "prefix extension" (joining interval sums) form.** The statement of the lemma is not determined mechanically from the target contract (which interval to join). It is opened separately when demand is measured.

## Consequences

- The program made from `tests/run/max_list.bmb` by deleting the lemma and the call (gate failure, E0811) is proven by one `bmb fix --induction --write` (2.3 seconds). The written lemma has the same statement as the hand-written one (`contains_has_index`) and only the condition of the body differs.
- For two unrelated predicates and for a false postcondition, it writes nothing.
- **What was checked is this one form.** How often it is right in real failed sessions was not measured. The measurement can be done by running this tool on the failure records of the official runs (sessions that ended in E0811) (backlog `T1-24a`).
- It is not exposed over MCP (the tool list is frozen, ADR-0021). The tool list of a new edition is decided together in `T1-20`.
