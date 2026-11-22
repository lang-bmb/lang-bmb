# ADR-0017: Integer semantics and SMT encoding

- Status: Accepted (autonomous, AUTONOMY.md §4) · §3·§5 are supplemented by ADR-0018 · integer types and bit operations are supplemented by ADR-0048 · the definitional axiom of §4 is supplemented by ADR-0064 for leaf `spec fn`s that bind an array, and by ADR-0067 for every leaf in the probes of proof obligations (a macro with the same meaning) · the defining formulas of `tdiv`·`trem` are supplemented by ADR-0085 (the same function when the divisor is not 0)
- Date: 2026-10-24
- Related: DESIGN.md §4.1 (arithmetic)·principles 4·9, ADR-0016, backlog 1B-01

## Context
Implementing 1B (the proof gate) requires fixing the meaning of executable arithmetic and specification arithmetic, and the way to carry them into SMT.

v0 `verify-vc` first modelled integers as unbounded integers, which disagreed with `i64` wrapping and was unsound. It recovered soundness by moving to 64-bit bitvectors (`legacy/v0:tests/verify-vc/EXPECTED.md`, T-BV). The cause was that v0's execution semantics was **silent wrapping**. v0 also added a separate `len < 2^62` axiom to remove the counterexample that an index operation such as `pos + 1` can overflow at extreme values.

## Decision

### 1. Executable arithmetic
- `i64`·`u64`·`usize` are 64-bit machine integers.
- The value of `+ - * / %` is the **mathematical result**. That the result lies within the range of the type is a proof obligation (no overflow).
- `/`·`%` truncate toward 0, as in Rust. That the divisor is not 0 and that the operation is not `i64::MIN / -1` or `i64::MIN % -1` are proof obligations.
- `wrapping_add/sub/mul` are operations modulo 2^64. They have no obligation.
- There are no other conversions or bit operations in M1.

### 2. Specification arithmetic
- Arithmetic in a specification context (contracts, invariants, `decreases`, `assert`, `spec fn` bodies) is over **unbounded integers**. There is no overflow and no obligation. This is the same as Verus's promotion to `int`.
- When an executable value enters a specification, it carries the range fact of its type.
- No range fact is attached to the result of a `spec fn`. The return type is used only for type checking.

### 3. Encoding
- Integers are encoded as SMT `Int`, and `bool` as `Bool`. Bitvectors are not used.
- Range facts are asserted at the point of use for parameters, values read from array elements, and call results. Ranges are not asserted in bulk with a quantifier.
- Truncating division is defined with SMT's Euclidean `div`·`mod`: `tdiv(a,b) = sign(a)·sign(b)·div(|a|,|b|)`, `trem(a,b) = a − b·tdiv(a,b)`. Unsigned types use `div`·`mod` as is.
- Wrapping operations use `mod 2^64`, and for `i64` the result is then moved into the signed range.
- A slice or array is a pair of `(Array Int Int)` (the elements) + `Int` (the length). A write is a `store`.
- **Length upper bound:** the length of a slice is assumed to be at most `isize::MAX / size_of(T)` (2^60 − 1 for `i64`·`u64`·`usize` elements, 2^63 − 1 for `bool`). In the Rust memory model the size of an allocation does not exceed `isize::MAX` bytes, so this is a sound fact. It replaces v0's `len < 2^62` axiom with a fact that has a rationale.

### 4. `spec fn`
- Each `spec fn` is an SMT uninterpreted function. The definitional axiom is `∀ args. range(args) ⇒ f(args) = body`, and the trigger is `f(args)`.
- The definitional axiom is included only when the name is in the `#[uses(..)]` of that proof. Unfolding is not transitive. If the body of A calls B, B must also be listed to unfold B.
- A recursive `spec fn` gets its definitional axiom only after it is proved that, under the range assumptions, the `decreases` value decreases at each recursive call and is at least 0. The axiom of an unproved `spec fn` is not included anywhere.

### 5. Query shape and determinism
- One executable function is one SMT-LIB file. Each proof obligation is `(push)` `(assert (not obligation))` `(check-sat)` `(pop)`.
- Names are generated deterministically from the source path and the obligation location. The same input gives the same bytes.
- Options: `(set-option :smt.random_seed 0)`, a fixed `rlimit` per obligation. The value is `Z3_RLIMIT` in `scripts/toolchain.env` (initial value 5,000,000, adjusted after measurement).
- Only `unsat` is a successful proof. `sat` is a failure with a counterexample. `unknown`, resource exhaustion and a solver error are failures with the cause.

## Rationale
- **In v2, Int is sound.** That the result of every executable operation is in range is a proof obligation, so in a program whose proof passes the machine value and the mathematical value are equal. The reason v0 used bitvectors (the mismatch between wrapping and unbounded integers) does not exist in v2.
- **Int is solved better.** Range obligations are linear inequalities, and quantifiers over array indices fit well with Z3's integer arithmetic. The combination of bitvectors and quantifiers is the main cause of instability (genesis §4, proof brittleness).
- A nonlinear obligation such as a multiplication can come out `unknown`. This is treated as a failure (fail-closed) and shows up as unproved area.
- When bit operations enter the language (after M2), this decision is reviewed again.

## Consequences
- The implementation from 1B-02 on follows this semantics. `Z3_RLIMIT` is added to `scripts/toolchain.env`.
- Because `#[uses]` is not transitive, proof context declarations can become long. This is a candidate for the auto-fill tool (DESIGN.md §8.2).
- The existing example `tests/check/ok/search.bmb` needs its `#[uses]` reinforced at the proof stage.
