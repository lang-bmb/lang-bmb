# ADR-0063: Canonical names for query symbols: position numbers inside the query instead of identity hashes

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation accepted, approval items approved"). Drafted 2026-10-28. Supersedes the row "query symbols with an `id` prefix" of ADR-0059 §4. The implementation is T1-10e
- Date: 2026-10-28
- Related: ADR-0059 §4 (nameless keys and queries), ADR-0032 (keys of the contract-firewall incremental build), ADR-0043 (caller keys), ADR-0033 (query declarations), ADR-0018 (one query per obligation), backlog T1-10e, 1B-40 (symbol renaming and verdicts)

## Context

- ADR-0059 §4 proposed naming the definition symbols of proof queries with an `id` prefix (`|#5568cad7c114|`). The purpose is that the query text, that is the proof cache key, is invariant under names.
- The implementation (T1-10, ADR-0059 "Confirmed at implementation") found that this rule conflicts with the contract firewall (ADR-0032).
  - The `id` contains the body of the definition. Editing the body of a `spec fn` that is not unfolded (not in `#[uses]`) changes its `id`.
  - Then the text of every query that uses that symbol changes and misses the proof cache. Now the symbol is the name, so it does not miss.
  - Naming by the surface hash removes the body dependency, but two different `spec fn`s with the same signature become the same symbol, which is unsound.
  - Putting the callee's `id` in the incremental key breaks the firewall for the same reason.
- Queries now contain names in five places: definition symbols (`|u::f|`), local constants (`|x@0|`), callee results (`|u::g.result@0|`), field selectors, datatype constructors. And there are places where the encoder walks locals in name order (the merge of `Enc::branch`).

## Decision

1. **Query symbols get canonical names per query.** When a query is built, symbols are numbered per kind in order of first appearance (definitions `d0`, `d1`, ..; locals `v0`, ..; callee results, selectors and constructors the same way). This is alpha normalization, and the numbers have meaning only inside the query.
2. **The encoder's walks do not rely on names.** Places that walk locals and fields in name order change to declaration position order. Only then does encoding the same definition with only names changed give the same query text.
3. **The legend is outside the query.** A table canonical name → path and name is kept with the encoding result. Diagnostics and counterexamples recover names through the legend and show names as now. `--emit-smt` writes the legend to a separate file next to the query file.
4. **The incremental key has the same shape.** A reference inside the key is written as a position number inside the key, and for each number what that query reads is written: for an executable callee the nameless surface, for a `spec fn` that is not unfolded the nameless declaration, for a `spec fn` that is unfolded the nameless body, for a type the nameless definition. Then the key changes only when the query text changes.
5. **The rest of ADR-0059 stays.** The identity of a definition (`id`), names as an index, the `id` of the map and `bmb hash`, and `rename`·`expect` do not change. Only the one row "query symbols with an `id` prefix" of §4 changes.

## Rationale

1. **Problem:** following the symbol rule of ADR-0059 §4 gives invariance under renames but breaks the cache on body edits of a `spec fn` that is not unfolded (loss of the firewall).
2. **Cause and layer:** if the symbol put in the query text carries "what that definition is" (identity), things the query does not read (bodies not unfolded) also enter the key. The key must contain only what the query reads (ADR-0032 "everything the encoding reads"). The layer is the tool (encoder).
3. **Canonical solution:** alpha normalization of the query. It is a standard technique used in compiler hash-consing and cache keys (de Bruijn-style numbers), and there is already a measurement that SMT symbol names do not affect verdicts (1B-40: for each of 718 obligations, over 16 variants including 4 symbol renamings, verdicts changed in 0%).
4. **Candidates:**

| Candidate | Invariant under renames | Firewall | Sound | Cost |
| --- | --- | --- | --- | --- |
| (a) `id` prefix (ADR-0059 §4) | Yes | **Broken** (bodies not unfolded enter the symbol) | Yes | Small |
| (b) Surface hash prefix | Yes | Yes | **No** (two `spec fn`s with the same signature are one symbol) | Small |
| (c) Names as they are (now) | No | Yes | Yes | None |
| **(d) Canonical numbers per query + legend (recommended)** | Yes | Yes | Yes | Changes to symbol naming and walks across the encoder |

## Consequences

- On rename edits the definitions re-encoded and the proof cache misses become 0 (the cost-side verdict of H-ID in ADR-0059). Now they are 848 → 847 and 3,560 → 3,560 (`dev-docs/experiments/rename-ripple/results.md`).
- The query text changes, so the proof cache format version is bumped. V7 (solver stability) and V4 (rlimit sum) are measured again, and if they get worse than the baseline this ADR is reopened.
- Counterexample and diagnostic output goes through the legend, so it must show the same names as now. This is checked by the expected output of the error and failing corpus being unchanged.
- When accepted, "the query symbol row of §4 is superseded (ADR-0063)" is written in the status line of ADR-0059. The implementation is backlog T1-10e. It changes `verify.rs` widely, so it is done after the proof function implementation (T1-18) is merged.

## Implementation (2026-11-02)

- §1~§3 (backlog T1-48): `smt::Canon`, declarations and definitions in first-use order, locals in first-bound order, a merge threshold measured in atom count. Proof cache format v3. Proof cache misses over 493 renames 4,297 → 0. The symbol numbers are not the per-kind numbers of §1 (`d0`, `v0`) but one sequence without kinds (`$0`, `$1`, ..). The legend knows the kind.
- §4 (backlog T1-49): `incremental::nameless_reads`. A reference in the key is a number inside the key (`@k`), and for each number there is the nameless text of what that query reads (the nameless text is the identity printer `identity::Ids::read_text`·`method_read` run with a resolver that assigns key numbers). The store is also looked up by key, and the script digest of the audit is that of the canonical text. Incremental cache format v3. Re-encodings over 502 renames 1,005 → 0, audit mismatches 0 (`bmb verify --audit`).
- Metrics: the order change of §2 changed V10 by +10.1% and V12 from 265 → 277 (V5 0.1355 → 0.1804, under the limit). The verdicts are unchanged and it is an effect of order only, so it was accepted with the reason, and reverting it is backlog T2-14.
