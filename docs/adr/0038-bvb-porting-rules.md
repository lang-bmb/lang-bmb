# ADR-0038: BVB (porting public Verus tasks) rules

- Status: Accepted (autonomous, AUTONOMY.md §4: an implementation-level decision of the measurement procedure)
- Date: 2026-10-25
- Related: ADR-0021, ADR-0025, ADR-0034 (BVB), backlog 2A-08·2A-09

## Context
ADR-0034 decided to move public Verus tasks to BMB and compare with the original Verus using the same model. Porting carries a large risk of selection bias and conversion errors, so the rules are fixed.

## Decision
1. **Sources:** Verus-Bench (MIT, commit pinned) and the Verus part of Vericoding (MIT, commit pinned). Problem statements are written anew, and specs and reference code are kept with copies of the licenses (`bench/bvb/SOURCES.md`, `licenses/`). VeriContest has no public data and is excluded.
2. **Selection:** only tasks within the M1 expressive scope (integers, slices, loops, recursion, named predicates). For Verus-Bench all 150 are reviewed, and for Vericoding a fixed sample in sha256 order after a shape filter. The reason for exclusion is recorded in a table per task. Exclusions that favor BMB (2 for which no BMB solution was found because M1 has no lemmas or `assert forall`) are stated.
3. **Conversion rules T1~T7:** slice mapping, integer widening with a range precondition attached, `exists` → `!forall !`, `Seq` recursion → recursion on prefix length, Euclidean division, removal of vacuous clauses, and so on. The rules are in `bench/bvb/README.md`, and the conversion is reproduced with `port.py`.
4. **G1 (given definitions):** if the original spec uses a recursive spec function or a named predicate for triggers, that function is given identically to both sides (the BMB seed unit `given`, the Verus starting file). If the agent changes it, the judgement rejects.
5. **Judgement:** on both sides, verification success + implication of the hidden original spec (spec strength check). Same model, same cap, one shot.
6. **CI:** on the BMB side, reference solutions 45/45 and weak control 0/45 are a gate, and the original Verus side is a separate CI job that installs a pinned Verus version.

## Consequences
- 45 tasks (Verus-Bench 30, Vericoding 15). On the original Verus too, reference solutions 45/45 and weak control 0/45.
- Limits: in both languages the spec strength check is sensitive to triggers. The hidden contract mitigates this by placing the trigger on the element term.
