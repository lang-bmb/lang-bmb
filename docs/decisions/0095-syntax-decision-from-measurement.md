# ADR-0095: Syntax decision from measurement (2R-06)

- Status: Accepted (approved by the maintainer 2026-11-08 "ADR-0095 adopted, 2N-07 approved"). Decision 1 (r2c) is superseded (ADR-0096: r2, which does not close over bodies)
- Date: 2026-11-08
- Related: DESIGN.md §4, ADR-0013, ADR-0036, ADR-0026, ADR-0028, ADR-0049, ADR-0057, `dev-docs/measurements/syntax-eval-1.md` (2R-02), `syntax-eval-2.md` (2R-04), `syntax-evidence.md` (2R-03)

## Context

ADR-0036 said that for each construct, borrowing, extending, varying or rejecting is decided by measurement. 2R-02 made a first evaluation of about 70 current constructs, 2R-04 compared the variation and rejection candidates offline (tokens, normal form, map size, proof cost, fix recovery), and 2R-03 counted the habits and diagnostics per construct in 1,205 real model sessions of the frozen harnesses. The synthesis is §2R-04h of `syntax-eval-2.md`.

Three things are large. (1) The shape of loop and contract clauses blocked about 40% of the sessions: repeated clause keywords (`invariant a, invariant b,`) 19.7%, inline expressions in contract clauses 16.6%, invariants inside the loop body 11.5%. (2) The shadowing rule (E0303) came up in 152 sessions, and 73 sessions ended on that error. (3) The `#[uses]` list is a hand-written context, and the rule r2c replaces it even in a large generated workspace.

## Decision

**A. Four variations that change the canonical form**

1. **Default `#[uses]` context = r2c.** The proof context of a definition is the closure of the `spec fn`s called by its own contracts, invariants and `assert`s and by the `requires`·`ensures` of its callees. `#[uses(..)]` is written only to add other facts, and in the canonical form the tool removes a list equal to r2c. r2c is computed on top of the checker's name resolution (including generics, traits, invariants and methods).
2. **Quantifier-free inline expressions in contract clauses (T2).** `requires`·`ensures` are a call of a named predicate or a quantifier-free expression. A clause containing a quantifier (`forall`·`exists`) must be a named predicate (the meaning of E0501 narrows to that). `requires`·`ensures` in L1 are normal form expression text.
3. **Shadowing narrowed to real hiding.** The same name is rejected only when an inner scope hides a name visible from outside (E0303). The same name in non-overlapping scopes (sibling blocks, bindings of different `match` arms) is accepted. A `let` that hides a parameter is still rejected.
4. **The canonical integer conversion `x as T` (between integers only, with a range obligation).** `T::try_from(x).unwrap()` is accepted as an input alias and moved to the canonical form. This is the recommendation of 2R-04c as is (ADR-0028 open question 1).

**B. Two input aliases that do not change the canonical form** (write surface policy, widening the fix rule of DESIGN §8.2)

5. **Repeated clause keywords.** `invariant a, invariant b,` (also `ensures`·`requires`) is accepted without an error and fixed to a list under one keyword ("The meaning of an input alias" below).
6. **Invariants inside the loop body.** The leading clauses of `while c { invariant a, decreases d, .. }` are accepted and moved to the head (the same mechanism).

**C. Recorded as decided** (not changed): the mechanical fixes of the write surface (2R-04e: 365 places, 0 changes of meaning), `self: &T` and method calls (ADR-0026·0051), range `for` (ADR-0049·0057), explicit triggers and the E0511 mechanical fix (2R-04g), rejection of the specification aliases `int`·`nat`·`@`, `decreases` required (a rule of the proof gate; the hint points to `for`).

**D. Outside this decision:** compound assignment is 2N-07 (approved the same day, in the same way as the input aliases of B below: `x op= e` is accepted and fixed to `x = x op e`), `impl` blocks with different bounds are 2T-11, moved values in specifications are 2T-12.

## Rationale

- A1: the 223 definitions of the m3 generated workspace and the 53 definitions of the m1b seeds are proved with r2c on all five seeds; `rlimit` is +0.8~2.3%, query bytes +1.5~2.0%, and no obligation exceeds the budget (1,000,000). The definitions whose key changes are 36 and 5, and they are proved again once (2R-04a). E0509 is 2.7% of the sessions. Computation on top of name resolution is a premise because, in `tests/verify/pass`, which has post-M1 surface (generics and so on), the approximate parser could not read the mentions and 30 definitions failed. This failure is not a counterexample to the rule.
- A2: inlining the 39 single-use predicates gives tokens −6.4~6.9% and L1 −24~27%. 35 are leaves, so the queries are the same (the macros of ADR-0067). The 9 quantifier predicates are units of triggers and reuse, so their names stay (2R-04b). With real models, inline expressions are 16.6% and E0501 is 245 sessions (2R-03).
- A3: of the 152 E0303 sessions, 46 are a `let` rebinding a parameter (still rejected) and 21 are two `let`s in one function. The same name in sibling scopes is common in Rust and is not ambiguous in BMB either. The single normal form (principle 8) is unchanged as long as the scopes of the names do not overlap.
- A4: −6.8 tokens per site. E0205 comes up in 49 sonnet sessions (2R-03).
- B5·B6: the two habits are other shapes whose meaning is uniquely determined (19.7%, 11.5%). They do not change the canonical form, so the map, the normal form and the proofs are unchanged, and it is the same shape as the rule of 2R-04e (a fix does not change the meaning).
- C: the measurement of each section supports the current state or is already reflected.

Alternatives considered: T1 of A2 (inlining every expression, quantifiers included) was rejected because triggers scatter inside the clauses and the unit of reuse disappears. T3 (current + an extraction tool) does not cover the model evidence (16.6%). "Allow all shadowing like Rust" for A3 was rejected because hiding a parameter (46 sessions) is on the side that changes the meaning (the original value disappears).

## Consequences

- Implementation phase (after adoption): A1 the r2c computation in the checker and the canonical cleanup of `#[uses]` (the tool removes), recomputation of the incremental keys. A2 the parser, checker, encoder, map (L1 expression text) and `query contracts`. A3 the checker (E0303 scope). A4 the surface replacement of ADR-0028 open question 1 and the round trip of both forms. B5·B6 new diagnostics and replacement text.
- Frozen bench assets (tasks, prompts) are not changed. The single-attempt success rate of the same tasks on the changed surface is measured in 2R-05 (A/B, same model). Most of the habits belong to the small model, so it is measured with that model.
- DESIGN §4 is revised in the same commit as the adoption (A1~A4).
- Adoption is the maintainer's approval (AUTONOMY §4: change of the language surface). Adopted 2026-11-08.

## The meaning of an input alias (set at adoption)

The aliases of B and A4 and the compound assignment of 2N-07 are the same mechanism as ADR-0051 §3 (path call → method call, W0911). The parser accepts the habit shape as the same AST as the canonical form, so `check`·`verify` emit no error, `bmb fmt` writes the canonical form from the AST, `bmb lint` emits at that place a warning (W09xx) with replacement text, and `bmb fix` applies it. The AST is the same, so the identity hash, the incremental key, the map and the proof are the same for the alias and the canonical form.

## Implementation record (2026-11-08)

- **B5·B6 (`2R-07`).** The parser accepts repeated clause keywords (`requires`·`ensures`·`invariant`; `decreases` has a different meaning and stays an error) and clauses at the front of the loop body. The replacement text is W0912 (delete from after the preceding comma up to the keyword, normal form `invariant a, b,`) and W0913 (move the clauses before the body, normal form layout). Alias sites are collected separately in `Diags::aliases` and are not in the output of `check`·`verify`; `bmb lint` reports them and `bmb fix` applies them.
- **2N-07.** `t op= e` (`+ - * / % ^ | & << >>`) is `t = t op e` (W0914, the right side in parentheses if needed). The canonical form reads the place twice, so the place must not call anything (a variable, a field, a dereference, an index without calls). Otherwise E0103 and a hint.
- **A3 (`2R-08`).** E0303 is only when a visible name (a parameter, an outer local, the result name) is hidden, and when the types differ even in non-overlapping scopes. The second condition is a constraint of the implementation: the locals table (`Tables::locals`) has one type per name per function, and the proof, code generation, move checking and the map read that table. With the same type, the same name in sibling scopes, two `for i` and different arms is accepted, and proof and execution are the same (`tests/run/scopes`).
- **A4 (`2R-09`).** `x as T` (between integers) is the same syntax tree as `T::try_from(x).unwrap()` (the identity hash is unchanged: not one hash changed while moving the 67 files of the corpus), and the precedence is the same as Rust (after unary, before `*`). The normal form is `x as T`, and a conversion on the left of `<`·`<<` is parenthesized. `T::try_from(x).unwrap()` is W0915 (parentheses depending on the place). The other fixes of the checker (E0212 and so on) write the alias, and W0915 moves it to the canonical form at the next turn.
- **A2 (`2R-10`).** E0501 is only for clauses containing a quantifier. Map L1 was already normal form expression text. The failure message shows the text of the clause instead of the predicate name. The fuzzer `soundness_specs` writes the quantifier-free clauses of odd-numbered functions inline.
- **A1 (`2R-11`), a difference from the decision: it does not close over bodies.** The decision text is r2c ("the closure of the `spec fn`s called") and 2R-04a also measured the closure. When the implementation measured the standard library (a corpus 2R-04a did not measure), the closure unfolded bit-level predicates that are deliberately kept opaque (`crc32::byte` behind `byte_result`, also `sha256`·`base64`), four definitions failed over the budget, and the library cost was +384%. The rule that does not go into bodies (r2: the `spec fn`s called by its own clauses, invariants, `assert`s and `proof` code and by the callees' contracts) gives library +0.58%, `tests/verify/pass` +0.19%, `tests/run` +4.39%, and the only verdicts that changed are the two fail corpus cases that tested opacity (those tests were moved one level below the body). r2 is a subset of r2c and has the same meaning as a written `#[uses]` (no transitivity), so it was implemented as r2. **Item for the maintainer to confirm**: whether to change the decision text to r2, or to add a new opacity marker and go with r2c (language surface). W0916 reports `#[uses]` entries that the default context contains, and `bmb fix` removes them.
