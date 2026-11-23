# ADR-0083: A type invariant broken between iterations: the loop rule stays, in-place shifts use `copy_within`

- Status: Accepted (approved by the maintainer 2026-11-02 "recommendation adopted for the decision items": recommendation (D). ADR-0060 open question 3, backlog T1-26. Implementation T1-26a)
- Date: 2026-11-02
- Related: ADR-0060 §2 (assumptions and obligations of validity), ADR-0031 (`Vec` and slice operations), ADR-0016 (language scope), JUDGMENT §10

## 1. Problem

If the `insert` of the strictly sorted set (`tests/run/sorted_set`) shifts in place on `self.items` (`self.items[j] = self.items[j - 1]` while decreasing `j`), the same value is present twice between iterations and the validity obligation at the loop head (E0823) fails. For now it shifts on a copy (`let mut a = self.items`) and assigns once at the end. With a non-strict order (`<=`) the in-place shift would keep the invariant in every iteration. The loops of the ring buffer, `generic_stack` and the four of BRB-ENCAP (`bench/brb-encap`) were not blocked.

## 2. Cause and the layer it belongs to

ADR-0060 §2 assumes the validity of the locals the loop havocs (types with an invariant) at the head, and emits it as an obligation at entry and at the end of an iteration. So a shift that leaves the invariant broken between iterations cannot be expressed. The layer is the language (the validity rule), or the absence of a standard operation that lets this shift be written as one operation (the library).

## 3. Standard solution and source

- Rust: an overlapping in-place shift is written with `slice::copy_within(src, dest)` (standard library). A hand-written element-by-element loop is not the standard way.
- Verus·Dafny: the type invariant is not placed at the loop head automatically; it is written as a loop invariant (Dafny's `Valid()` is an explicit predicate). Automatic head validity is something BMB added as a convenience (ADR-0060; the `generic_stack` migration lost one loop invariant clause).

## 4. Comparison of candidates

| Candidate | C1 soundness | C2 determinism | S1 standard practice | S2 root cause | S3 consistency | S4 efficiency |
| --- | --- | --- | --- | --- | --- | --- |
| (A) as is (head validity + copy) | pass | pass | a copy is used in Rust too, but it is second best | the workaround remains | one rule | one line of copy, one line of final assignment |
| (B) a notation in which a loop `invariant` clause stands in for the validity of that value (head validity is turned off per value) | pass (the obligations at exits and boundaries remain) | pass | no precedent (new notation) | general | a new kind of loop clause | the notation has to be learned |
| (C) validity only at loop exits (no head assumption or obligation) | pass | pass | same as Verus·Dafny | general | one rule | every loop loses the head assumption it now gets automatically: every loop that relies on the invariant has to write a loop invariant clause |
| **(D) the rule of (A) + built-in `copy_within`** | pass (the contract of the operation is exact: the value at each position after the shift) | pass | the Rust standard library as is | this class (overlapping shifts) as one operation | the rule stays, one operation | the in-place code has the same shape as Rust: no loop and no copy |

## 5. Recommendation and the deciding criteria

(D). The criteria that decided are S1 and S4. The standard Rust way for the blocked code is not a loop but one `copy_within` call, and that call breaks the invariant only inside one operation, not between iterations, so the loop rule need not change. (C) matches standard practice but loses the head assumption every loop now gets automatically (S4). (B) is a notation without precedent.

- Surface: `a.copy_within(lo..hi, dest)` (arrays, `&mut [T]`, `Vec<T>`, `T: Copy`). Obligations: `lo <= hi <= len`, `dest + (hi - lo) <= len` (the same bounds as E0805). Result: position `dest + k` (`k < hi - lo`) of the new array is `lo + k` of the old array, the rest is unchanged. Code generation: `llvm.memmove` (already present).
- Unchanged: the head validity of ADR-0060 §2. The `insert` of `sorted_set` becomes one line of `copy_within` and two assignments.

## 6. Impact (information)

- The checker (one method), the encoder (array update term: one range quantification), code generation (`memmove`), the fuzzer (one shift in `soundness_vec`). Corpus: `tests/run/sorted_set`; the BRB-ENCAP reference solutions are frozen and stay as they are.
- Measurement condition: the compiler tree changes, so this is a new condition (ADR-0070 (a)). The operation is not in the language rules section of the frozen prompts, so if an agent writes it (Rust habit) it is now accepted.

## 7. Approval needed

A widening of the language surface (ADR-0016 scope, AUTONOMY §4). Until approval, (A) stands.

## 8. Workaround

The current copy workaround remains in backlog T1-26 as a "workaround". Condition for lifting it: the adoption and implementation of this ADR.

## Implementation (2026-11-02, backlog T1-26a)

- Surface: `lo..hi` in argument position is `ExprKind::Range` in the syntax tree and is accepted only as the first argument of `copy_within` (elsewhere E0435, a new diagnostic). Argument shape E0405, shared or immutable receiver E0408, move elements E0404, inside a specification E0503.
- Encoding: the two obligations of §5 (E0805). If the range is constant and has at most 16 elements, the new element array is a `store` chain of the old values; otherwise it is one new constant and one quantified fact per element leaf array (the trigger is a read of the new array). With only the quantified fact, the reachability check (W0905) in the two test functions with constant ranges had 2 undecided and rlimit 2,000,000; with the `store` chain it had 0 and 4,454 (health V12). The length is unchanged and the receiver enters the loop write set.
- Code generation: only unproved bounds are checked, then one `llvm.memmove`.
- Tests: `tests/verify/pass/copy_within.bmb` (array, slice, `Vec`, struct elements, a general contract), `tests/verify/fail/copy_within.bmb`, `tests/check/err/copy_within.bmb`, `CopyWithin` in `soundness_vec` (including out of bounds, checked against the interpreter), the gate detection mutants `v-copy-within-dest-bound`·`v-copy-within-unshifted`.
- The `insert` of `tests/run/sorted_set` shifts with `self.items.copy_within(p..n, p + 1)` instead of the copy workaround, and the `shifted` predicate is gone. The output is unchanged.

