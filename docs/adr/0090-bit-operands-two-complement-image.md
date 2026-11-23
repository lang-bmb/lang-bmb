# ADR-0090: In specifications the operand of a bit operation is the two's complement image at the width of its type

- Status: Accepted (autonomous, AUTONOMY.md §4: an implementation-level decision of the integer encoding). Supplements ADR-0048 §4 (encoding forms). The meaning of executable code and the query text for in-type operands are unchanged
- Date: 2026-11-05
- Related: ADR-0022 (specification arithmetic may leave the type), ADR-0048 §2·§4, backlog 2N-10

## Context

- Specification arithmetic may go outside the type (ADR-0022: `x + y` is the plain sum of two `u8` values). When such a value becomes the operand of a bit operation, the meaning depended on the form the encoder chose (found in 2N-03). The bitvector form (`&`·`|`·`^`, shifts by a variable amount, bit methods) reads the low `w` bits with `int2bv`, and the linear forms read the integer value as it is: `>>` by a constant amount is `div`, `!` is `-a - 1`·`MAX - a`, constant rotations and `wrapping_shr` are also `div`.
- So the same expression had two values. With `x = 66, y = 255, k = 1` of `u8`, `(x + y) >> k` (bitvector, 32, half of the low 8 bits 65) and `(x + y) >> 1` (`div 321 2` = 160) differ, and so do `!(x + y)` and `!((x + y) & 255)`. The previous encoder rejected both equalities with a counterexample. Values of executable code are always in the type, so this is not a false proof, but the meaning of a specification is not single.

## Decision

1. **Meaning.** In specifications, an integer operand of a bit operation (`&`·`|`·`^`·`<<`·`>>`·`!`) or a bit method is read as the two's complement image at the width `w` of its type (the value reduced to `w` bits and read with the signedness of the type). The result is always a value of the type. The shift amount is the value as it is, as now.
2. **Encoding.** The bitvector form already has this meaning (`int2bv`). The linear forms that read the value as it is (`>>` by a constant amount, `wrapping_shr`, rotations, `!`, `swap_bytes` at byte width) wrap the operand in `wrap_int` (`verify::image`). If the operand is syntactically a value of the type (a literal, a variable, a place, a method result, the result of a bit operation, `old`, `verify::in_type`), it is not wrapped: that query text does not change. The linear form of `<<` (already `wrap_int`) and the `mod` of an `&` mask (the low `k ≤ w` bits are the same in the image) are unchanged.
3. **Sub-proofs.** `by (bit_vector)` carries over the encoder's terms (ADR-0048 §4), so it follows the same meaning.

## Rationale

- **Cause and the layer it belongs to:** a different reading was used per form without the meaning having been fixed. The layer is the encoder (it is a decision that fixes the meaning of the language, but the meaning of executable code does not change).
- **Standard practice:** a fixed-width bit operation reads its operand as a bit string of that width (SMT-LIB `int2bv`; Rust's `as` conversion and `wrapping_*` use the same image). For in-type values it equals Rust's result.
- **Candidates.** (a) the integer as it is (infinite-precision two's complement): `&`·`|`·`^` are defined, but shifts by a variable amount require a nonlinear definition of `2^k`, and the bit methods (`count_ones`, rotations) have no meaning without a width. (b) the two's complement image (this decision): every operation has a meaning and the bitvector form is unchanged. (c) forbidding out-of-type operands (a checker error): the benefit of specification arithmetic (ADR-0022) is lost for bit operations only, and which operand is outside the type is known only by proving, so the checker cannot decide it. The deciding criteria are C1 (one meaning) and S1 (standard practice): only (b) gives every form the same meaning and is standard practice.
- **Confirmation:** `shift_image` (`>>` by a constant and by a variable amount) and `not_image` (`!` and a constant rotation equal those of the masked operand) in `tests/verify/pass/bit_vector.bmb` are proved after this decision, and the previous encoder gives counterexamples (`x = 66, y = 255, k = 1`; `x = 1, y = 255`). The interpreter of `soundness_bitvector` also reads the image in every bit operation.

## Consequences

- The meaning of specifications that used out-of-type operands changes. Executable code and queries for in-type operands are unchanged down to the text.
- A wrapped operand gets one more `mod` (only in such queries).
- Reverting: make `verify::image` return the operand as it is, and it is the previous form.
