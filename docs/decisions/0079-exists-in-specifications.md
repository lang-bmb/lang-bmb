# ADR-0079: `exists` in specifications

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": backlog T0-25. Widens the language scope (ADR-0016))
- Date: 2026-11-01
- Related: ADR-0016 (language scope), ADR-0017 §4 (specification encoding), ADR-0023·0075 (ground instance retry), ADR-0045 (judgement criteria, ledger of conveniences given up), backlog T0-19, T0-25

## Context

- For existence-shaped contracts, agents first write Verus's `exists` quantifier. That is 11 of the 45 BVB sessions of the second pilot, all five existence-shaped tasks, and also qwen's BRB `t12` (`dev-docs/experiments/T0-19/results.md` §3).
- BMB had no `exists`. The compiler rejected it with E0103 "expected `;`, found `:`" and a wrong mechanical fix ("insert `;` before `:`").
- The shapes agents wrote instead (a recursive predicate, an index predicate trigger) meant the same but were shapes the judge could not decide (T0-19 (가)·(나)). (나) was solved by T0-24 (ADR-0075). The upstream cause, the absence of `exists`, remained.
- JUDGMENT §7: a restriction that is not in the ledger of conveniences given up (DESIGN §4.1) is a defect. `exists` is the standard Verus surface (DESIGN §1 "Verus-family specification syntax"), and making people write the same meaning as `!(forall|x| !P)` is a tightening that is not in the ledger.

## Decision

1. **Surface.** Specification contexts have `exists|x: T, ..| e`. The syntax, binding and context rules are the same as for `forall`. Bound variables have integer types (E0402) and are used only in specifications (E0503). There must be at least one `#[trigger]` (E0511), and the triggers together must mention every bound variable (E0512). All triggers form one multi-pattern. The diagnostic text uses the name of the quantifier.
2. **Syntax tree.** `ExprKind::Forall` is widened to `ExprKind::Quant { q: Quantifier, binders, body }` (`Quantifier::Forall | Exists`). The normal form is `exists|..| ..`, and the kind in the JSON AST (ADR-0073) is `"exists"`. The identity hash (ADR-0059) follows the normal form, so it tells `forall` and `exists` apart. `exists` becomes a reserved word (no identifier of this name exists in the corpus, the seeds or the library).
3. **Encoding.** It is SMT `exists`. The sorts of the bound variables, the range facts and the trigger patterns are the same as for `forall`. The range facts are the antecedent of `=>` in `forall` and a conjunct of `and` in `exists`. The decrease obligation of a recursive call inside a quantifier is "for all values of the bound variables" in both (2V-05). The ground instance retry (ADR-0075) already counts the variables bound by `exists`.
4. **Tools.** The trigger lint W0906, the call edges of the map, rename and find references, and specification weakening detection (`bmb weaken`) pass through the same node, so they apply unchanged.

## Rationale

- **Cause and the layer it belongs to:** the language (a restriction not in the ledger, JUDGMENT §7) and the tool (the wrong fix). Fixing the judge (candidate B) or teaching "`!(forall ..)` instead of `exists`" in the prompt (candidate D) absorbs the symptom in a different layer (T0-19 §5).
- **Standard practice:** both Verus and Dafny have `exists` in specifications and give it the same trigger rules as `forall`. The `exists` of SMT-LIB takes patterns (instantiated in negative position).
- **Why one widened node:** the two quantifiers have the same binding, trigger and context rules and differ in only two places of the encoding (the connective, the symbol). With separate variants every traversal has to handle both branches, and missing one becomes a defect that occurs with only one quantifier.
- **Confirmation (the T0-19 judge, the frozen harness as is):** for BVB task 808, the contract and loop invariant written with `r == (exists|j| j < arr.len() && #[trigger] arr[j] == k)` are proved and accepted by the hidden specification check (`dev-docs/experiments/T0-19/judge.py`, `accepted: true`).

## Consequences

- Tests: the pass corpus `tests/verify/pass/exists.bmb` (`exists` in a postcondition, a precondition and a loop invariant), the fail corpus `tests/verify/fail/exists.bmb` (E0802 with a counterexample, and an index proved from the range fact that comes from an `exists` precondition), the check corpus `tests/check/err/exists.bmb` (E0511·E0512·E0402·E0503). The normal form round trip, the JSON AST round trip and the determinism test run on the same corpus.
- Diagnostic text: the fix text of E0510 becomes "`forall` or `exists` body". E0402·E0503·E0512 on a quantifier get a fix hint (bind an integer index, move it into a specification function, make the term that mentions that variable a trigger too). So five lines of `tests/check/err/specs.jsonl`·`places.jsonl` change, and the ratio of errors with a fix hint (P3) goes 0.8722 → 0.8778. The descriptions of E0510·E0511·E0512·W0906 in the diagnostics table (`docs/diagnostics.md`) speak of both quantifiers.
- Measurement condition: the compiler tree changes, so this is a new condition (ADR-0070 (a)). The frozen prompts and sets are unchanged. The BVB porting rule (`bench/bvb/README.md` T3 "no `exists` in M1") is a record of the frozen set and is not changed.
- Reverting: take `exists` out of the parser and it is the previous surface (the `Quantifier` of the syntax tree may stay).
