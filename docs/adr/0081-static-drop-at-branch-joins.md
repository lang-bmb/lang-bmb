# ADR-0081: A value moved in only one branch is dropped statically at the end of the other branches

- Status: Accepted (approved by the maintainer 2026-11-01 "recommendation adopted, approval items approved": the recommendation of backlog 2O-07). Changes the join rule of ADR-0031 §2 ("maybe moved is E0414") at the joins of `if`·`else` and `match`. ADR-0047 §2 (moves of owned types) follows the same rule
- Date: 2026-11-02
- Related: ADR-0031 §2·open question 3, ADR-0047 §2, ADR-0049 §4 (drops at `return`), backlog 2O-06·2O-07

## Context

- ADR-0031 §2: at branch joins and loop heads a variable must be "definitely initialized" or "definitely moved", and "maybe moved" is E0414. The rule exists so that drop points are fixed statically and there are no run-time drop flags.
- It got in the way of a real program. `parse` in `tests/run/calc` (2O-06) discards the left tree `a` when there is no right operand, but that branch does not move `a`, so E0414 was raised. The workaround was `let unused: Box<Expr> = a;`. Rust accepts this program.
- Rust drops a "maybe moved" value with a run-time drop flag. BMB does not have that flag (the move state row of DESIGN §4.1).

## Decision

1. At the join of `if`·`else` and `match`, if a value (a local, a field path of it) is moved in some branch and still owned in another, it is **dropped at the end of each branch that owns it** and is regarded as moved after the join. The drop point is fixed by the program text (after the value of that branch has been computed and stored). There is no run-time flag.
2. A join whose branch does not have an end of its own stays E0414. That is an `if` without `else` (the path that skips the block), the right side of `&&`·`||`·`==>`, and a loop head (the back edge and the head). Covering a "maybe moved" at a loop head with a drop would need a different drop per iteration (ADR-0031 §2 as before).
3. Partial moves (2O-03) are the same. If one branch moves the field `s.a`, the other branch drops `s.a` at its end, and the remaining fields are dropped at the end of the block as now.

## Rationale

- **Cause and the layer it belongs to:** it is a restriction that fails to accept a program Rust accepts, and what the ledger (DESIGN §4.1) gave up is "run-time drop flags", not this program (JUDGMENT §7). The layer is the compiler (move checking and code generation).
- **Standard practice:** Rust puts the drop at the end of the scope, and if the state at that point differs by branch, a drop flag decides. Drop elaboration removes the flag at points where the state is statically known. BMB has no flags, so in this case, where the state is known per branch, the drop is pulled to the end of that branch. Then at the end of the scope it is "moved" on every path and no flag is needed.
- **The drop time is not observable:** BMB has no user `Drop` (ADR-0047), so pulling the drop to just before the join does not change anything the program observes. The proof does not model drops either, so it does not change.

## Consequences

- Code: `moves::Mv::join_branches` (the n-way join of `if`·`else` and `match`, branch-end drops in `MoveInfo::branch_drops`), `drop_branch` in `codegen` (drop after the value of the branch is stored).
- Tests: the run program `tests/run/branch_drops.bmb` (`if`·`else`, a three-arm `match`, one field), the workaround in `tests/run/calc.bmb` removed. Both are clean under valgrind (`--leak-check=full`) and the output is as expected. Check corpus: five E0414 of `if`·`else` and `match` disappear from `tests/check/err/boxes.jsonl`·`moves.jsonl`·`owned_fields.jsonl`, and the E0414 of an `if` without `else` (`no_else` in `moves.bmb`), `&&` and loops remain.
- Measurement condition: the compiler tree changes, so this is a new condition (ADR-0070 (a)). Only programs that were rejected with E0414 before are handled differently.
- Reverting: replace `join_branches` with the two-way `join` (E0414) and it is ADR-0031 §2 as before.
