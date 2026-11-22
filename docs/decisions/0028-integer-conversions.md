# ADR-0028: Integer conversions and checked·saturating arithmetic

- Status: Accepted (approved adoption plan: at the start of 2C, 2026-10-24. 2B surface reflected, 2026-10-25). The surface (open question 1) was decided by ADR-0095 A4 (`x as T`, 2026-11-08)
- Date: 2026-10-24 (proposed), 2026-10-25 (accepted)
- Related: DESIGN.md §4.1 (arithmetic)·principles 4·8·9, ADR-0013, ADR-0017, ADR-0018, ADR-0022, ADR-0026, ROADMAP 2C, backlog 2C-01·2C-05, `dev-docs/measurements/syntax-eval-2.md` §2R-04c

## Context
M1 has no integer conversion in executable code. `as` is rejected with E0205 (`compiler/src/parser.rs`, the fix hint is "checked conversions come after milestone 1"), and mixing integers of different types is E0401. The spec side was resolved by ADR-0022, but executable code is unchanged.

Friction that showed up (evidence):
- **Output.** The only built-in output is `print_i64`, so printing a `u64`·`usize` value required a hand-written counting loop. `show` (`u64`) and `show_size` (`usize`) in `tests/run/search.bmb` run `c = c + 1; i = i + 1` `n` times, and because of the loop cost they even carried an unjustified upper bound `printable_size(n) = n <= 1000000` (measurement report `dev-docs/measurements/m1.md` §6, ROADMAP 1C observations).
- **Mixing.** There is no way to compute with a `usize` index and an `i64` accumulator together in executable code. ADR-0022 resolved only specs.
- **Rust habit.** `as` is counted under E02xx (Rust habits). When the real model measurement (1E-10) finishes, the E0205 frequency will be known. There is no number now.

To decide: whether a conversion failure is a proof obligation or an `Option`/`Result` (depends on 2B), the surface form, and the scope of `checked_*`·`saturating_*`.

## Decision (proposed)

### 1. Conversion: `T::try_from(x).unwrap()` with an obligation
- Conversion between integer types has **one form, `T::try_from(x).unwrap()`** (`T` is `i64`·`u64`·`usize`). The value equals `x`, and **that `x` is within the range of `T` is a proof obligation** (new code E0815). Where `unwrap` would panic in Rust, BMB has a proof obligation. It is the same translation as indexing `a[i]` (Rust: panic, BMB: bounds obligation).
- If the source range is contained in the target range (`u64`↔`usize`, both 64-bit in BMB), no obligation is generated. The pairs that produce an obligation are four: `i64→u64`, `i64→usize` (`x ≥ 0`), `u64→i64`, `usize→i64` (`x ≤ i64::MAX`).
- **Failure as a value:** `T::try_from(x).ok()` is an `Option<T>` (2B, ADR-0026): `Some(x)` if in range, `None` if not, no obligation. It is the same value in Rust (`Result::ok`). BMB has no error type, so a bare `T::try_from(x)` (`Result<T, TryFromIntError>`) is not accepted (E0412, the fix hint is `.unwrap()`·`.ok()`). Widening `unwrap` into an obligation operation on general `Option`·`Result` values is outside this ADR (ADR-0026 §5 rejects `.unwrap()` with E0229).
- Conversions are not used in spec context. Specs are already unbounded integers and can mix types (ADR-0022). A conversion in a spec is E0513.

### 2. Bit reinterpretation (optional scope)
- `x.cast_unsigned()` (`i64→u64`) and `x.cast_signed()` (`u64→i64`) are two's complement reinterpretations with no obligation (the methods of the same name in Rust 1.87). They are for wrapping computations such as hashes and random numbers. They are added if demand is measured (open question 3).

### 3. Checked·saturating arithmetic
- `saturating_add/sub/mul`: no obligation. The value is the mathematical result clamped to the type's range.
- `checked_add/sub/mul/div/rem`: return `Option<T>` (2B, the `Option` of ADR-0026). The value is `Some(v)` if the mathematical result is in range, otherwise `None` (for division, division by zero and `i64::MIN / -1` are `None`).
- `overflowing_*` (tuple return)·`pow`·`abs` are outside the scope of this ADR. `i64::abs` overflows at `MIN`, so if added it is decided separately as an operation with an obligation.

### 4. Output
- A built-in `print_u64(x: u64)` (effect `io`) is added. For `usize`, `print_u64(u64::try_from(n).unwrap())` (no obligation). The `show_size` loop above becomes the single line `print_i64(i64::try_from(n).unwrap())`, and if the value comes from a slice length, the obligation is proved directly from the length upper bound fact (ADR-0017 §3, 2^60 − 1).

### 5. Proof gate
- The encoding is `Int`, as in ADR-0017. A conversion is the identity, and the obligation is the single target range predicate `range_T(x)`. The result carries the target range fact.
- Saturation: `sat_T(v) = ite(v > MAX, MAX, ite(v < MIN, MIN, v))`, `v = a op b` (the mathematical value). `add`·`sub` are linear, `mul` is non-linear and can yield `unknown` (fail-closed, unproven area).
- `checked_*`·`try_from(..).ok()`: `ite(range_T(v), Some(v), None)`. `Option<T>` is the datatype of ADR-0027 §3.
- If a new auxiliary fact is added, a machine proof goes with it in `compiler/tests/lemmas.rs`, per the rule of ADR-0018 §2. The definitions above need no auxiliary fact.

### 6. Code generation
- A proved conversion has **no instruction** (all three types are LLVM `i64`). Line comment `; proved E0815`. If not proved, it is a range comparison + `llvm.trap` (`emit-llvm`) and `bmb build` refuses.
- Saturation: `llvm.{s,u}{add,sub}.sat.i64`, and `mul` is `llvm.{s,u}mul.with.overflow.i64` + `select`. Checked: `*.with.overflow`.
- `cast_*`: no instruction.

### 7. Map and hashes
- Conversions and saturation are built-in methods like `wrapping_*`, so they create no call edge. `print_u64` creates an `io` boundary edge like `print_i64` (included in the `io_defs` computation of `status`).
- The normal form is `T::try_from(x).unwrap()` as is. The hash definition (ADR-0019) does not change.

### 8. Rust habits to reject, and diagnostics (numbers confirmed free in `diag.rs` at the time of acceptance)

| Habit | Handling | Code |
| --- | --- | --- |
| `x as T` | The existing E0205 is kept. `fix.replacement` is changed to `T::try_from(x).unwrap()` so `bmb fix` fixes it mechanically. If truncation was intended, the proof fails with a counterexample (fail-closed) | E0205 (existing) |
| `T::from(x)`, `x.into()`, `x.try_into().unwrap()` | If the target type is known, suggest replacing with `T::try_from(x).unwrap()` | E0212 (new) |
| `T::try_from(x).expect("..")` | Replace with `.unwrap()` (one normal form) | E0213 (new) |
| `T::try_from(x)?` | The existing E0206 | E0206 (existing) |
| A bare `T::try_from(x)` (no `.unwrap()`·`.ok()` attached), `bool`↔integer conversion | Type error, fix hint `.unwrap()` (obligation)·`.ok()` (`Option`) | E0412 (new) |
| A conversion in a spec | Hint "specs can mix types (ADR-0022)", replacement that removes the conversion | E0513 (new) |
| A conversion that can go out of range | Proof failure + counterexample | E0815 (new) |

The wording of the E0404 fix hint for `checked_add` and the like is updated to match the phase (no change of meaning).

### 9. Tests and fuzzer
- Corpus: `tests/check/err` (each code of the table above), `tests/verify/pass|fail` (a proof and a counterexample per pair), `tests/run` (the version with the `show_size` loop removed, saturation boundary values).
- Code generation: a proved conversion leaves no instruction in the IR, and without the proof `icmp`+trap remains (the same shape as 1C criterion 2).
- Soundness fuzzer: conversions between the three types and saturating operations are added to the generator of the unsigned arithmetic fuzzer (1B-33, `compiler/tests/soundness_unsigned.rs`). The interpreter computes an out-of-range conversion as an error and saturation as `i128` clamping. The properties are as before: a proved function has no error on allowed inputs, a confirmed counterexample really breaks it, and the output of the two builds equals the interpreter's. 0 violations over 60 batches is 2C pass criterion 2.

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **`T::try_from(x).unwrap()` + obligation (proposed for acceptance)** | It is valid Rust, and a proved BMB program behaves the same as the same Rust program in every execution. It reuses the existing rule "panic site = proof obligation" (indexing), so there is no new concept. It does not wait for 2B |
| Attach an obligation to `x as T` | The strongest alternative. Rust habit errors disappear, and within range the value equals Rust's truncation. But the same token means "silently truncate" in Rust and "range proof" in BMB, so the intent is not visible on the surface. If float and character conversions arrive later, the single `as` has several meanings (principle 8). The habit cost is absorbed by the automatic replacement of `bmb fix`. **Judged again once the E0205 frequency and the absorption rate of the automatic replacement are measured** (open question 1) |
| `Option`/`Result` return only | Verbose, since it forces a branch even when a proof shows there is no failure, and 2B must come first. The path that handles failure as a value is kept alongside as a bare `try_from` after 2B |
| Allow `as` with wrapping (truncating) meaning | This is exactly the "`as` truncating conversion" that the arithmetic row of DESIGN.md §4.1 gave up. Rejected |
| Merge `u64`·`usize` into one | At odds with the Rust surface, and breaks the habit of index and length types. Rejected |
| General `print`/formatting | Needs strings and formatting, so the scope is large. The single `print_u64` resolves the M1 friction |

## Consequences
- No change to the trust base: the conversion obligation is expressed with the existing range predicates, and without a proof there is no build.
- One more built-in (`print_u64`), and one more `io` boundary in the area that needs testing.
- `T::try_from(x).unwrap()` is longer than `as`. The loss is measured by the Rust habit error rate (DESIGN.md §4.1 tension and response 3).

## Open questions (decided by measurement 1E-07 or by the maintainer)
1. ~~**Surface:**~~ Decided (ADR-0095 A4, accepted 2026-11-08): the canonical form is `x as T` between integers (the meaning is as in §1: value preserved, range obligation E0815). `T::try_from(x).unwrap()` is an input alias read as the same syntax tree (W0915, `fmt`·`fix` turn it into the canonical form), and `T::try_from(x).ok()` stays as is. Precedence is the same as Rust (`-x as u64` is `(-x) as u64`). Rationale: 2R-04c (−6.8 tokens per site), 2R-03 (E0205 in 49 sonnet sessions).
2. ~~2B order~~: 2B arrived, so `checked_*` was put in the scope of this ADR as `Option<T>`. The value path of `try_from` is `.ok()` (§1).
3. **`cast_signed`/`cast_unsigned`:** added only when the `m1b`/`m2` tasks show demand for wrapping conversion.
4. **`bool`↔integer, `abs`·`abs_diff`·`min`·`max`:** the maintainer decides whether to leave them out of this scope.
5. **Priority of 2C:** decided by 1E-07. The friction evidence so far is the single author experience (`show_size`), and whether real models get stuck in the same place is checked with the 1E-10 results.
