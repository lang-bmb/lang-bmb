# ADR-0096: The default proof context does not close over bodies (r2)

- Status: Accepted (2026-11-09, approved by the maintainer "recommendation adopted". Backlog 2R-11)
- Date: 2026-11-09
- Related: ADR-0095 decision 1 (superseded by this ADR), ADR-0019 (definition hash), ADR-0032·0043 (incremental keys), ADR-0059 (locality of the identity hash), `dev-docs/health/2R.md`, `dev-docs/measurements/syntax-uses/default-context-2026-11-08.jsonl`

## Context

Decision 1 of ADR-0095 set the default proof context of a definition to r2c: the **closure** of the `spec fn`s called by its own contracts, invariants and `assert`s and by the `requires`·`ensures` of its callees (down to what the bodies of those `spec fn`s call). The evidence (2R-04a) was the m3 generated workspace and the m1b seeds.

The implementation (2R-11) measured the standard library, which 2R-04a had not measured. The library deliberately hides bit-level predicates behind other predicates (`crc32::byte` behind `byte_result`, and `sha256`·`base64` in the same shape). The closure unfolded them, four definitions (`base64::encode`, `crc32::table`·`update_with`, `sha256::compress`) failed over the budget, and the proof cost of the library was +384%.

## Decision

1. **The default proof context (r2) is the `spec fn`s called by name in the definition's own clauses, loop invariants, `assert`s and `proof` code and in the `requires`·`ensures` of its callees.** What the bodies of those `spec fn`s call does not enter (no transitivity). The context of a `spec fn` itself is only its clauses (not its body).
2. **`#[uses(..)]` is written only to add other facts.** Entries the default context already contains are reported by W0916 and removed by `bmb fix` (the "tool removes" of ADR-0095 as is).
3. **Opacity is one level.** A `q` called by the body of a predicate `p` is only a name to a definition that calls `p`. To unfold it, write `#[uses(q)]`.
4. **The identity hash contains the written `#[uses]` list (the syntax as is).** The default context depends on the callees' contracts, so putting it into the hash would make a callee's contract change the caller's identity (ADR-0059 locality). The incremental key (ADR-0032·0043) uses the actual context through `verify::context_of`.

The remaining decisions of ADR-0095 (2~6) are unchanged.

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **r2, not closing over bodies (adopted)** | Library +0.58%, `tests/verify/pass` +0.19%, `tests/run` +4.39%, 0 verdict changes (the two fail corpus cases that tested opacity were moved one level below the body). It is a subset of r2c and has the same meaning as the hand-written `#[uses]` (no transitivity). |
| r2c, the closure (the text of ADR-0095) | 4 library definitions fail, +384%. Restoring the intended opacity needs new surface (an opacity marker) |
| r2c + an opacity marker (`#[opaque]` or the like) | If the marker is forgotten, the proof cost explodes silently (the failure appears far away). It goes against the principle that opaque is the default (DESIGN §2 "definitions are opaque by default") and adds surface before the freeze |

The raw data is `dev-docs/measurements/syntax-uses/default-context-2026-11-08.jsonl`.

## Consequences

- The implementation stays as it is: `verify::default_contexts`, `Tables::context`, `verify::context_of` (proof, incremental key, `weaken`), the `uses` edges of the map, W0916 (`lint::default_uses`). Tests `tests/verify/pass/default_context`, `tests/verify/fail/contracts`·`bit_vector`, `uses_entries_of_the_default_context_are_removed` in `compiler/tests/fix_cmd.rs`.
- A proof that has to unfold two levels below the body writes `#[uses]` (`bmb uses` fills it in).
- The status of ADR-0095 changes to "decision 1 is superseded by ADR-0096".
