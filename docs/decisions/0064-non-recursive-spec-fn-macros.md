# ADR-0064: The definition of a leaf `spec fn` that binds an array is written as a macro instead of a quantified axiom

- Status: Accepted (autonomous, AUTONOMY.md §4). Supplements the definition axiom row of ADR-0017 §4 for leaf `spec fn`s that bind an array. The meaning is unchanged and only the form written to SMT changes · proof obligations put a probe that writes every leaf as a macro in front of this form (supplemented by ADR-0067)
- Date: 2026-10-29
- Related: ADR-0017 §4 (opaque `spec fn`, definition axiom), ADR-0023 (ground instance retry), ADR-0061 (`f64`), backlog T2-04

## Context

- ADR-0017 §4 puts in the definition of an unfolded `spec fn` as `∀ args. range(args) ⇒ f(args) = body` (trigger `f(args)`). The form is the same whether recursive or not.
- T2-04 (found while implementing `f64`): when a definition axiom binds an `(Array Int Float64)` parameter, Z3 4.15.4 could not finish a query that has a counterexample. The minimal reproduction is one axiom and five assertions. With `Int` elements it is `sat` at rlimit 240, with `Float64` it is `unknown` at 382,936 (`incomplete (theory array)`). The ground instance retry (ADR-0023) then gave the answer, so the verdict was right, but hundreds of thousands of rlimit were thrown away per failing obligation.
- The same phenomenon exists on the integer side. This is the reason ADR-0023 came about ("when a false contract goes through an unfolded predicate definition, Z3 cannot build a model"), and there remained a case where one counterexample of the failing corpus runs up to 5,000,000 (`bf::or` in the 2N-02 report).

## Decision

1. **The definition of a leaf `spec fn` that binds an array is a macro.** This is when all three conditions hold.
   - It does not recurse (it is not in the recursive component `Tables::recursive`. A generic instance is judged by its generic).
   - Its body does not call another `spec fn` (leaf).
   - The definition binds a variable of an array sort (slice or `Vec` parameter, `(Array ..)`).

   The form is `(declare-fun f!ext (sorts..) result)` and `(define-fun f ((arg sort)..) result (ite range(args) body (f!ext args..)))`. If the range is `true`, only the body is put, without `f!ext`.
2. **Everything else stays a quantified axiom.** And even a macro candidate uses the quantified axiom if the patterns of that query (`:pattern`, those made by user `#[trigger]` and those in the bodies of unfolded definitions) contain that symbol. A macro is expanded, so it cannot go into a pattern (when Z3 expands a macro in a pattern no usable trigger is left). The axiom text contains both forms (`Prog::axiom`), and the choice is made when the query is assembled (`relevant_axioms`).
3. **The meaning is the same.** The quantified axiom says `f = body` inside the range and says nothing outside the range. The macro is the body inside the range and the value of the uninterpreted `f!ext` outside the range. The models of the two definitions correspond to each other. So what is proved and what has a counterexample are the same. The meaning at out-of-range arguments (backlog 2N-10) is not changed.
4. **Query assembly:** a symbol defined as a macro is not declared (`needed_decls`). A definition comes after the macros it uses (the dependency order of `relevant_axioms`). A `spec fn` that is not unfolded is only declared, as now (opaque).

## Rationale

- **Cause and layer:** the solver cannot build a model over a quantified definition that binds an array-sort variable (Z3 `reason-unknown`: `incomplete (theory array)`). The layer is the encoder. The meaning of the language does not change.
- **Canonical way:** for a non-recursive function definition the canonical form is SMT-LIB's `define-fun`. Z3's `macro_finder` did not recognize our axiom, which carries a range condition, as a macro (`unknown` on the same query, rlimit 383,042).
- **How it was narrowed (measured in the same work):**
  - All non-recursive ones as macros: failing corpus rlimit −89%. But nested macros multiply the size of terms, and the vacuity check of `lib/sha256.bmb` exceeded the gate time limit (rlimit hardly counts the simplification of expanded terms).
  - Leaves only: failing corpus −85%, vacuity check 138 seconds. But three BVB reference solutions hit the limit. Two (`vb-clover-is-prime`, `vb-mbpp-3-is-non-prime`) were because a leaf predicate used in a `#[trigger]` was expanded as a macro and the trigger disappeared (one was a proof failure). Fixed by the pattern rule (decision 2). In the remaining one (`gcd_exec` of `vc-va0053-aspect-pairs`), just unfolding the scalar predicate `both_nonneg` took a non-linear query from 226,723 to the limit (retry 53,006).
  - Only leaves that bind an array (this decision): the minimal reproduction is fixed (below), and the three BVB cases above stay at the baseline. Failing corpus rlimit is −8%.
- **Measurement:**
  - Minimal reproduction (one definition axiom with an `(Array Int Float64)` parameter): `unknown` 382,936 → `sat` 1,397.
  - Failing corpus rlimit sum: 137,479,381 → 126,061,811 (−8%). Diagnostic codes and positions are the same. In one place (`fill_short` of `fail/generics.bmb`, a case where the loop invariant forgets the written elements) solver undecided (E0811) changed to a failure with a counterexample (E0802).
  - The proof corpus and V4·V7·V8·V9 are in this commit's entry of `dev-docs/health/baseline.json`.
- **What is left:** making scalar predicates macros too greatly reduces the cost of queries that have a counterexample (failing corpus −85%). A way to get that gain without hurting non-linear proofs is measured separately in backlog T2-05 (for example: re-solve with macros only when the first attempt is `unknown`).

## Consequences

- The query text changes, so the proof cache is in effect emptied once (the key is the query text). The cache format is unchanged.
- Ground instance retry (ADR-0023) has work left only for recursive definitions. Instances of macro symbols are equations already true from the definition, so they do no harm.
- Revert: delete the macro branch in `Prog::axiom`. The storage format and the language do not change.
