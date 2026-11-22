# ADR-0023: Ground-instance retry for undecidable obligations

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: ADR-0017 §4, ADR-0018, backlog 1B-14

## Context
When a false contract went through a predicate definition unfolded with `#[uses]` (a quantified axiom), Z3 could not build a model, ran up to the resource limit and returned `unknown` (`wrong_post` and `forgets_write` in `tests/verify/fail/contracts.bmb`). The verdict was failure (fail-closed), but with no counterexample it was hard for the agent to find the cause.

## Decision
- If an obligation is `unknown` and there are unfolded definitions, it is solved once more with **ground instances** in place of the quantified axioms. For each predicate application that appears in the obligation, `(=> range (= f(args) body[args]))`, the definition instantiated with those arguments, is added, and the applications that appear in the body are added the same way, up to 3 levels.
- On `unsat` it is accepted as a proof. The instances are logical consequences of the quantified axioms, so this is sound.
- On `sat`, it is reported as a counterexample only if every application received an instance and no quantifier remains in the instances. In that case a model of the instances is a model of the whole definition. Otherwise it stays undecidable (E0811) with an "unconfirmed counterexample".
- The retry is also recorded as is in the `--emit-smt` script, and the resource usage is the sum of the two attempts.

## Rationale
- No new fact enters the trust base. The instances come from axioms that are already assumed.
- Z3 can decide a query without quantifiers, so the counterexample can be confirmed (both cases above produced a counterexample with a few hundred units of resource).

## Consequences
- A predicate whose body keeps a quantifier (for example a sortedness predicate containing `forall`) may still not produce a confirmed counterexample.
- For nested applications deeper than 3, the instances are considered incomplete and are not used as a confirmed counterexample.
