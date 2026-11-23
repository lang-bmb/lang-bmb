# ADR-0098: Specifications read moved values

- Status: Accepted (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after them ADR-0100 approved"). Backlog 2T-12
- Date: 2026-11-09
- Related: ADR-0031·0047 (moves, E0413), ADR-0062 (`proof` code reads values and does not move them), ADR-0017 (values in specifications), ADR-0092 (standard library), finding 3 of `dev-docs/health/4B.md`

## Context

- Move checking (`compiler/src/moves.rs`) counts a mention in `assert`, a loop invariant or `decreases` as a use too. Speaking of a moved local variable there is E0413. Inside a `proof { .. }` block there is already an exception (ADR-0062: ghost code reads values and does not move them).
- Postconditions are not blocked, because the parameters in `ensures` are the values as they came in. So the same fact can be stated in `ensures` and cannot be stated in an `assert` inside the body. Reproduction (2026-11-09): in `fn put<T>(v: &mut Vec<T>, k: T)`, `k` in `ensures` passes, and `assert(..k..)` after `v.push(k);` is E0413 "`k` is used after it was moved".
- Because of this, 4B-04 wrote the shifting of move keys in `std::collections` as recursion instead of a loop (the invariant had to speak of the moved key).
- On the proof side the meaning already exists. A moved value can no longer be read or changed, so its logical value is the value at the time of the move. The encoder does not read move information (`moves::MoveInfo` is used only for the drop points of code generation).
- In Verus the standard way is to keep a snapshot in a ghost local variable for specifications (`let ghost`).

## Decision (proposed)

1. **Specification positions (`assert`·`assume`·loop invariants·`decreases`, and `proof` blocks as now) can read a moved local variable.** The value read is the value at the time it was last moved. If it was assigned again afterwards, it is the assigned value (the current SSA as is).
2. A mention in a specification position is not a move. A use in an executable position stays E0413·E0414.
3. Partial moves (field paths) are the same. The specification reads the value of the moved field.
4. There is no new surface. `let ghost` is not added.

## Rationale (alternatives considered)

| Option | C1 soundness | Verdict |
| --- | --- | --- |
| **Specifications read moved values (recommended)** | unchanged: a moved value cannot be accessed and so does not change, and the encoding already holds that value. The ownership rules of executable code are unchanged | There is no new surface. `ensures` and an `assert` inside the body read the same name with the same meaning |
| A ghost local variable `let ghost k0 = k;` (Verus) | same | A keyword and an erasure rule are newly created, and there are two forms for writing the same fact. It is a habit Rust users do not have |
| As is (work around with recursion) | same | The friction of having to turn a loop into recursion repeats in the port (4B-04) |

The deciding criteria are S1 (standard practice, no new surface) and the principle "one normal form".

## Consequences

- Implementation: no checking of the move state in specification positions in `moves.rs` (like `proof` blocks, which are `ghost` now). The corpora `tests/verify/pass` (an invariant that speaks of a moved key is proved), `tests/check/err` (E0413 in executable positions as before). An `assert` that speaks of a moved value in the fuzzer `soundness_specs`.
- The shifting of move keys in `std::collections` can be turned back into a loop (after comparing cost and verdicts).
- Code generation does not change (specifications are erased).
