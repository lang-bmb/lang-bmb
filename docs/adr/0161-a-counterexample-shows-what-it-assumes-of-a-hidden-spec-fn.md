# ADR-0161: A counterexample shows what it assumes of a spec fn seen only by name

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0017 (counterexamples from the model), ADR-0023 (ground instances), ADR-0090 (spec values are not held to their declared types), ADR-0095 (the default proof context), ADR-0108 (bounded search)

## Context

- A spec fn that a proof sees only by name (not in its context) is an uninterpreted function in the query. When the query is satisfiable, the failure was reported with the inputs as a "counterexample" and a `fix` naming the function ("the proof sees `off` only by name; ...").
- The inputs are a counterexample only under the values the model gave that function, and those were not shown. They are often no counterexample under its definition: with `spec fn nonneg(x, n) = off(x, n) + 1 > 0` and `off` hidden, the model takes `off(x, 38) = -1` (a `usize` spec fn may take a negative value in a query, ADR-0090), and reported `counterexample: x = [] (len 0), n = 38`, while `off([], 38)` is 0 and the postcondition holds.
- The verdict is right: without the definition the obligation does not follow. Only the message misled.

## Decision

1. When the query of a refuted obligation has spec fns seen only by name and the counterexample has inputs to show, the message reads `counterexample with `f`, `g` unknown: <inputs>` in place of `counterexample: <inputs>`.
2. It goes on with `, assuming f(a, b) = v, ..`: the applications of those functions in the model's query, read from the model with `get-value`. They are the applications outside quantifiers in the asserted facts, inner ones first, then those of the ground instances of the query's definitions (ADR-0023) whose symbols the query has, where a quantified definition applies the function; each once, at most three, in this order. An argument that is an input is written by its name (a slice by its name alone), any other by its value; a value that is not an integer or a boolean is `..`. Without such an application, or when the model cannot be read, the `assuming` part is left out.
3. Nothing else changes: verdicts, codes, the `fix`, the queries and the scripts are as before. The values are those of the pinned solver for the query, so the same input gives the same bytes. A contract comparison (`bmb weaken`) keeps its own wording.

## Rationale

| Option | Verdict |
| --- | --- |
| **Say the function is unknown and show the values the model assumed (adopted)** | The reader sees why the inputs fail and whether that value is one the definition can take; deterministic, and no extra query |
| Only the wording "with `f` unknown" | Honest, but leaves the reader to guess the value that made the query fail |
| Print Z3's whole interpretation of the function (`get-model`) | Its shape (`ite` chains, an else value, `lambda`) depends on the solver and on the query, and its arguments are solver values with no names |
| Confirm the counterexample by evaluating the definition | A different verdict path (the definition may be recursive or quantified); not needed for an honest message |
