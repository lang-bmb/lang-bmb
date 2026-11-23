# ADR-0124: The length of `push` is not a proof obligation but a fact the runtime maintains

- Status: Accepted (approved by the maintainer 2026-11-11 "proceed with the recommendation": the recommendation of `dev-docs/measurements/4d-13.md`). It supersedes the obligation of the `v.push(x)` row in the table of ADR-0031 and the length obligation of `String` `push`·`push_str` in ADR-0050
- Date: 2026-11-11
- Related: ADR-0031 (Vec and ownership), ADR-0050 (strings), ADR-0017 §3 (length bound facts), ADR-0100 (freeze: the accepted language), `dev-docs/measurements/4d-13.md`, backlog 4D-13

## Context

- `v.push(x)` and the `push`·`push_str` of `String` carried the proof obligation "the length is within the maximum length" (E0816).
- A function that walks a tree and writes to a buffer cannot discharge this obligation with a contract (both arms of the `port-shape` experiment put a runtime guard in front of `push`). The formatter and the code generation of the port have the same shape.
- The runtime already maintains this condition: the growth of `Vec` (`bmb_vec_grow`) does `abort` if the new byte size does not fit in `isize` or the allocation fails (ADR-0031 §4).

## Decision

1. **`v.push(x)` has no length obligation.** After `push`, the proof gets "the length before insertion < `LEN_MAX(T)`" as a fact. The length bound facts (ADR-0017 §3) hold as before.
2. **The `push`·`push_str` of `String` are the same.** The proof gets the new length ≤ `isize::MAX` as a fact.
3. **When it is exceeded, the program ends.** It is the same class as an allocation failure (`alloc` effect, `abort`). For `Vec` the existing check on the growth path does it, and for `String` "if the needed length > `isize::MAX`, `abort`" goes into the growth path of `bmb_str_append`. Both are checks that run only on growth.
4. **The obligation of `Vec::with_capacity(n)` (E0816) stays as it is.** `n` is a value the program gives.

## Rationale

| Option | Verdict |
| --- | --- |
| **Remove the obligation, make it a fact (accepted)** | It fixes at the layer of the cause (JUDGMENT §3): the obligation overlaps a check that the runtime already does. Rust's `Vec::push` does not ask the caller about exceeding capacity either (S1). Soundness (C1): if `push` returned, then `len + 1 <= cap` and `cap * size <= isize::MAX` |
| A specification function for size and auxiliary lemmas | Every walking function needs a size function and an inductive proof, and structures that share children have no bound |
| A library function with a guard | A comparison per byte remains, and it silently truncates at a length that cannot be reached |

Measurement (`4d-13.md`, temporary build): for the 11 units of the standard library the verdicts of the 218 definitions stay the same, obligations 1,750 → 1,697, `rlimit` −2.2%. `port-shape/tree.bmb` with the guards removed is fully proved.

## Consequences

- More programs are accepted (a `push` without a guard is proved). It is a change before the freeze declaration.
- E0816 remains only for the capacity of `Vec::with_capacity`. The description in the diagnostic table is corrected.
- In the proof corpus, 6 `push` obligations disappear, and the two definitions that failed on that obligation alone (`sf::unbounded`, `vf::unbounded`) are proved. Other verdicts stay the same.
- Code generation: the length check of `push` (which remained when it was not proved) goes away. In full-check mode as well, the check on the growth path does the same work.
- The length guards and capacity arguments left in the standard library and the corpus may be deleted, and if left, the proofs stay the same. Deleting them is done when that file is changed.
