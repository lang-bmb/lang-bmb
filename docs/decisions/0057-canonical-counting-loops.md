# ADR-0057: The normal-form fix for countable `while` is a lint warning plus `bmb fix`

- Status: Accepted (2026-10-28, Accepted (autonomous): AUTONOMY §4 "implementation-level decision within the scope of an existing ADR", backlog 2K-04)
- Date: 2026-10-28
- Related: ADR-0049 §2 ("a `while` counting loop that does the same job gets a machine fix that changes it to `for` in normal form"), DESIGN.md §3 principle 8·§4.1 ledger "representation", ADR-0021 (bench freeze), backlog 2K-01d·2K-04

## Context
ADR-0049 §2 fixed the normal form of a countable loop as the range `for` and said to provide a machine fix that changes `while` counting loops. It did not fix the severity of the diagnostic (error or warning). Other examples of normal-form rules (`match` arm order E0426, Rust habits E02xx) are errors of `check`. But `while` counting loops exist in frozen bench assets: the M1 seed workspace (`bench/m1/seed/src`, 2 places) and the baseline programs (`bench/baseline`, 9 places). Making it an error would mean the starting state of frozen tasks does not pass the check.

## Decision
1. **Diagnostic.** A countable `while` is W0907 (warning). `bmb lint` reports it (`lint::canonical_loops`, looks at syntax only), and the fix of the diagnostic carries the replacement source text. `check`, `verify` and `build` do not change.
2. **`bmb fix` applies it.** For a program that passes the check, `bmb fix` applies the fix of W0907 too, with the same round rule as other machine fixes (overlapping fixes go to the next round). Nested counting loops change one per round, from the outside.
3. **Conditions.** In `while i < hi [invariant ..] decreases d { ..; i = i + 1; }` directly after `let mut i: T = lo;`: only the last statement changes `i` (no assignment, `&mut` or mutating method), the body does not change a variable that `hi` reads and `hi` is only variables, constants, fields, arithmetic and `len()`, there is no `continue` to this loop, `i` is not used after the loop, and if both ends are literals then only when `T` is `i64` (a `for` with two literal ends is `i64`). A range `for` is encoded as exactly this unfolded form plus the head invariant and the measure `hi - i` (2K-01d, query identity test), so the changed program still passes proof with the user invariants left as they are (the added obligations are only the head invariant, which is always true, and the measure).
4. **Splice the source text.** The fix is made by joining pieces of the source text (comments between the `let` and the loop stay in front of the loop, and only the `decreases` line and the increment statement line are dropped). The normal-form printer discards comments, so it is not used.
5. **Migration.** The run programs, the proof corpus and `lib/bytes.bmb` are moved with `bmb fix`. The corpus that tests `while` itself (`loops`, `loop_exits`, `returns`), the failing corpus and the frozen bench assets stay as they are.
6. **When to revisit.** When the bench tasks are issued in a new edition (unfreeze), raising W0907 to an error of `check` is decided again.

## Rationale (alternatives considered)
| Alternative | Judgement |
| --- | --- |
| An error of `check` (E02xx, with fix) | The most direct for principle 8. The frozen M1 seed and baseline programs fail the check, which breaks the freeze of ADR-0021 |
| The normal-form printer (`bmb fmt`) changes it | The printer only does syntactic normalization. This transformation has to look at uses of the name (after the loop) and writes in the body, and `fmt --check` failing on the frozen assets is the same |
| **Lint warning + applied by `bmb fix` (accepted)** | Frozen assets pass the check as they are, and the machine fix is applied with one tool run. The cost that non-normal-form code can remain is exposed by the report of `bmb lint` |
