# ADR-0033: Send only the `spec fn` declarations each query reaches

- Status: Accepted (autonomous)
- Date: 2026-10-24
- Related: ADR-0018 §1, ADR-0032 (supersedes the "query header" row of the proof key), DESIGN.md §4.1·§8.1, backlog 2D-14

## Context
The query of every obligation carried every `spec fn` declaration of the program in its header (`verify::header`). So adding one `spec fn` or changing a signature changed every query text, and both the proof cache (key: query text) and the incremental cache (header hash in the key) missed everything (the limit in the consequences of ADR-0032). DESIGN.md §4.1 names not sending unrelated context to the solver as a means of proof stability.

## Decision
- The query header is only the options (seeds, `rlimit`) and the prelude (`verify::query_header`).
- For each obligation, only the `spec fn`s that the text sent after it (unfolded definitions, local declarations, assumptions, the negated goal) mentions by a symbol `|path|` are declared, in path order (`verify::needed_decls`). The unfolded definitions are already closed up to the definitions their bodies call (`relevant_axioms`), so one scan is enough. The ground instance retry (1B-14) selects separately by that retry text. The W0901 satisfiability query is the same.
- The "query header" row of the incremental key (ADR-0032) is replaced by the following two rows.

| Element | Reason |
| --- | --- |
| Hash of the query header (options, seeds, `rlimit`, prelude) | It goes into every query as is |
| The SMT declarations of every `spec fn` reached by starting from the qualified normal form of the definition and following the contract (signature, clauses, `#[uses]`) text for executable callees and the whole text for `spec fn`s | The symbols of a query come only from the definition itself, callee contracts and unfolded bodies (including the measures of the recursive component), so this covers everything a query can declare (an over-approximation) |

## Rationale
- Verdict unchanged: over the whole corpus (31 files of `tests/check`·`tests/verify`·`tests/run`, the m1·m1b seeds) the `bmb verify` output is byte-identical to before. Not only the verdicts but also the `rlimit` figures are the same (sum 12,455,584, maximum 5,000,190 as before). Z3 spends no resources on unused declarations. The m1·m1b oracle and weak agent run records also have the same verdict fields. `tests/stability.rs` passes.
- Query size: the `--emit-smt` script total of the m1b seeds 1,987,446 → 746,728 bytes, `spec fn` declaration lines 21,294 → 707.
- The incremental key uses the set reached by text, so it is wider than the real query. Too wide only causes unnecessary re-encoding, and too narrow gives a different query under the same key and breaks soundness. The audit mode of `tests/incremental.rs` (recompute even on a hit and compare script hashes) confirms 0 mismatches under the edits (including a new named `spec fn`).
- Alternative: put the per-query declaration set exactly in the key. It cannot be known before encoding (skipping the encoding is the goal). Rejected.

## Consequences
- Adding or removing an unrelated `spec fn` leaves the query text of other definitions byte-identical, and both the proof cache and the incremental cache hit (`adding_an_unrelated_spec_fn_reencodes_nothing` in `tests/incremental.rs`: 0 re-encodings and IR generations).
- A signature change of a `spec fn` re-encodes only the definitions that reach that declaration. This can be wider than the `names` edges (direct naming) of `impact` (when a callee contract calls it).
- In "the base (options, definitions, axioms, declarations)" of ADR-0018 §1, the declarations now differ per obligation. Obligation isolation is unchanged.
