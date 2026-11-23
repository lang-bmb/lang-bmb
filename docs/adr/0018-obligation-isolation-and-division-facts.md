# ADR-0018: Proof obligation isolation and auxiliary division facts

- Status: Accepted (autonomous, AUTONOMY.md §4)
- Date: 2026-10-24
- Related: supplements ADR-0017 §3·§5, backlog 1B-09·1B-13

## Context
Two things surfaced during the implementation of 1B.

1. **Results varying with incremental state.** When the same query was solved 40 times in a row in one Z3 process with `push`/`pop`, only the 36th came out `unknown` (rlimit 200,000). `rlimit` applies per query, but the internal state left by earlier queries changes the result. In fact the `a / b + a % b` example was proved only after another obligation and failed on its own. Determinism is kept, but adding one obligation can change the result of another, unrelated obligation. This goes directly against proof stability (genesis §4).
2. **Nonlinearity of division.** `trem(a,b) = a − b·tdiv(a,b)` of ADR-0017 becomes a multiplication with a variable divisor, and not even the range obligation of `a % b` was proved.

## Decision

### 1. Obligation isolation
- For each obligation, after a `(reset)`, the base of that definition (options, definitions, axioms, declarations) is sent again, and only the assumptions of that obligation and the negated goal are asserted. Obligations do not share state.
- The script exported with `--emit-smt` contains the order that was actually executed (`(reset)` + base + obligation), as is.
- Even when a solver command is cancelled by the resource limit before `check-sat`, it is reported as E0811 (undecidable).
- The solver confirms the response of every command with `print-success`. An error is caught at the command that caused it.

### 2. Auxiliary division facts
Each time `/`·`%` is used, the following linear facts are assumed (`q` is the quotient, `r` is the remainder).

| Sign | Facts |
| --- | --- |
| Signed | `b = 1 ⇒ q = a`, `b = −1 ⇒ q = −a`, `|b| > 1 ⇒ 2|q| ≤ |a|`, `b ≠ 0 ⇒ |r| < |b|`, `b ≠ 0 ⇒ r = 0 ∨ sign(r) = sign(a)` |
| Unsigned | `a ≥ 0 ∧ b > 0 ⇒ 0 ≤ q ≤ a`, `b > 0 ⇒ 0 ≤ r < b` |

- These facts enter the trust base. So `compiler/tests/lemmas.rs` proves with Z3, at every gate, that each fact follows from the SMT definitions of ADR-0017. If a proof breaks, the gate fails.
- **Rule:** from now on, every auxiliary fact the verifier assumes comes with a machine-proof test in the same way. No special-purpose fact is added to make an example pass.

### 3. Side decisions
- `#[uses(..)]` accepts a predicate of another unit by its full path (`unit::name`).
- Even on `unknown`, if the solver gives a candidate model it is attached as an "unconfirmed counterexample". The verdict is still failure.
- Array literals are forbidden in a specification context (E0503). The meaning of an array value in an unbounded-integer specification has not been fixed.

## Rationale
- The cost of obligation isolation is parsing the base again for each obligation. The whole current corpus takes about 3 seconds. That is worth trading for stability.
- The division facts are all linear and proved, so they reduce the load on the solver without widening the trust base.

## Consequences
- Proof results are independent per obligation. In performance measurement the number of obligations becomes the main cost factor.
- Nonlinear arithmetic (for example the overflow of `a / b + a % b`) may still not be proved. It shows up as unproved area.
