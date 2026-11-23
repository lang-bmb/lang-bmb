# ADR-0048: Complete integer types, bit operations, integer literal formats

- Status: Accepted (2026-10-26, approved by the maintainer "1/2 adopted, approved", backlog 2V-12) · The meaning of spec operands in §4 is complemented by ADR-0090: an operand outside the type is its two's complement image at that width
- Date: 2026-10-26
- Related: ADR-0045, ADR-0017 (integer encoding), ADR-0028 (conversions), ADR-0044 §1 (`u8`), review R8, backlog 2V-08


## Confirmed at acceptance (2026-10-26)
- Diagnostic numbers: the obligation that a shift amount is at or above the bit width has the same meaning as the existing E0803 (overflow), so that is used as is. E0818 `by (bit_vector)` sub-proof failure (confirmed free).
- Open question 1 (`i128`·`u128`) is for when the need arises. 2 (register width of narrow integers) is decided at implementation by measuring IR size and performance.
- Implementation phase: ROADMAP 2N (integers, bits).

## Confirmed at implementation (2026-10-28, backlog 2N-02)
- §2 "the shift amount is `u32`, the same as Rust": Rust's `<<`·`>>` operators accept every integer type as the amount (`impl Shl<i64> for u32` and so on). `u32` is the amount type of methods such as `wrapping_shl`·`checked_shl`·`rotate_left`. The implementation follows the rationale "the same as Rust", accepts every integer type for the operator's amount, and gives a literal amount the type `u32`. For a signed amount, E0803 also blocks negative values (the same as a Rust debug build).
- §3 normal form: the radix is kept, and `_` separators, leading zeros and hexadecimal letter case are not in the normal form (layout, like whitespace). The rule is the literal row of `compiler/README.md`.
- §4 cost: a query with bitvector definitions was expensive because Z3 solved bit by bit even when not needed (the range obligation of one 64-bit variable shift, rlimit 735859). An obligation is first solved with a query that abstracts the bitvector terms into integer constants and keeps only the result range facts (if proved, done; a proof of the weaker query is a proof), and otherwise goes to the exact query. Executable code declares the operand bitvectors as constants and gives their values (it finds a counterexample tens of times faster than `int2bv`: the counterexample of a 32-bit `|`, rlimit 216385 → 4469). The measurement and code path are in the bit operations entry of `compiler/README.md`.

## Confirmed at implementation (2026-10-28, backlog 2N-03)
- **What the sub-proof sees (§4):** it follows the Verus rule. The query sees only the free variables of the assertion `e`, and each variable ranges over all values of its type (a bitvector of width `w`). It does not see path facts, `requires` or earlier assertions. A needed fact is written inside `e` as `p ==> q` (Verus's `assert(e) by (bit_vector) requires p;` is a second notation for the same job and is not accepted: one notation, DESIGN principle 8). A ghost `let` (`proof` block) is also a free variable and its definition is not visible. The outer query assumes the proved `e` at the values of that place, but prefixed with "if the free variables are within their types" (`(=> (and ranges..) e)`): variables of executable code are always within their types so there is no cost, and nothing false is assumed about a ghost `let` that can be outside its type by spec arithmetic (`x + 1` being 256) (`ghost_range` in `tests/verify/fail/bit_vector`).
- **Ghost contexts:** `assert(e) by (bit_vector);` appears with the same meaning wherever `assert` appears (executable code, `proof` blocks, `proof fn` bodies, ADR-0062). Inside a spec it is E0503, like `assert`. Verus's `by { .. }` (block form) and other prover names (`nonlinear_arith`, `compute`) are the new diagnostic E0516 (for a name, a mechanical fix that deletes `by (..)`). What is not a term of the assertion (elements, fields, quantifiers, `old`, values behind a reference, data values) is rejected by the checker with E0515 (hint to bind it with `let`).
- **Translation (§4):** what is translated is not the syntax but the term the encoder made from `e` (a fresh symbol per free variable) (`compiler/src/subproof.rs`). So the sub-proof proves exactly the fact the outer query assumes: the linear form of a constant shift, the `mod` of an `&` mask, the `-a - 1` of `!` and the other forms the encoder chose are all carried over as they are. Each integer subterm gets an interval and is moved to a two's complement bitvector of a width that holds all its values, so `+`·`-`·`*`·`div`·`mod` by a positive constant·comparison·`bv2nat`·`int2bv` are exact (even when spec arithmetic goes outside the type). Exactness is confirmed by `bit_vector_translations_are_exact` in `tests/lemmas.rs`, which evaluates the encoder's terms at sample values. The query logic is `QF_BV`.
- **spec fn:** a spec fn the assertion calls is unfolded by the same translation as the definition axiom (`spec_block`). Only those that are not recursive, not generic, and take and return only integers and `bool`, and only those whose definition is exposed by `#[uses]` (one not exposed gets E0818 and a `#[uses]` hint: `bmb uses` fills it in). The definition axiom holds only when the parameters are within their types, so an integer argument whose interval does not show it to be within its type becomes an assumption of the sub-proof and is prefixed to the outer fact (`spec_guard` in `tests/verify/fail/bit_vector`). Each argument is named once with `define-fun` so nested unfolding (`byte` is eight layers of `bit`) does not grow.
- **The place for other theories:** the sort of a free variable (`subproof::Sort`) and the head symbols of the translator are the connection points. The `f64` of ADR-0061 §5 adds the `(_ FloatingPoint 11 53)` sort, a branch that interprets `f64.*` symbols as `fp.*`, and a logic that holds both theories (not implemented).
- **Cost (measured):** the CRC-32 table lemma `byte(c, b) == byte((c ^ b) & 0xff, 0) ^ (c >> 8)` is proved at rlimit 104,291 (with the mixed encoding it did not finish at 5,000,000, T4-04). A sub-proof failure is E0818 with a counterexample of the free variables (values of their types), and if it cannot be translated, E0818 with the reason.
- **Methods (§2):** `rotate_left`·`rotate_right`·`wrapping_shl`·`wrapping_shr` (the amount is `u32`, the remainder modulo the width), `count_ones`·`count_zeros`·`leading_zeros`·`trailing_zeros` (`u32`), `swap_bytes`. Rotations and wrapping shifts by a constant amount are the linear form (§4), and the rest are a term wrapping one named bitvector function (`bmb_rotl32` and so on) in `bv2nat`, plus result facts (range; counts are `0..=w` and extreme only at 0). Code generation is LLVM `fshl`/`fshr`·`ctpop`·`ctlz`/`cttz` (defined at 0)·`bswap`. `to_be_bytes`/`from_be_bytes`/`to_le_bytes`/`from_le_bytes` involve array values and are done separately (backlog 2N-09).
- **Congruence of the abstract query (§4 cost):** the table-style `t[i] ^ (c >> 8)` (code) and the lemma's `byte(i, 0) ^ (c >> 8)` (spec) are terms of the same shape with leaf values written differently. The congruence facts of 1B-49 were only between reads identical down to the leaf values, so representatives of the same shape (leaves read only by width) also get "equal if the other leaves are equal" (at most 64 per shape, `MAX_SHAPE_PAIRS`). It is true in any model, so the abstract query is still weaker. `table_step` went 5,000,000 (failure) → 5,859, and the whole of `lib/sha256.bmb` is +0.4%.

## Context
BMB's integer types are only `i64`·`u64`·`usize`·`u8`, and there are no bit operations (`&`·`|`·`^`·`!`·`<<`·`>>`) or hexadecimal and binary literals. From the decision to encode integers as SMT `Int` (ADR-0017), the encoding difficulty that bit operations become non-linear was left as a missing feature (ADR-0045 §7). The self-hosting analysis recommended leaving SHA-256 (32-bit wrapping, rotation, xor) to an escape hatch (`selfhost-gap.md` §4.1). The precedent in verification tools is to send bit operation obligations to the bitvector theory (Verus `by (bit_vector)`, Dafny `bv` types).

## Decision

### 1. Integer types
- All of Rust's fixed-width integers: `i8`·`i16`·`i32`·`i64`, `u8`·`u16`·`u32`·`u64`, `usize`·`isize` (64-bit target). 128-bit is open question 1.
- Arithmetic, comparison, `wrapping_*`·`checked_*`·`saturating_*`, the division obligations and conversions (ADR-0028) have the same rules, with only the range differing. The default type of a literal is fixed by the expected type (the current rule).

### 2. Bit operations
- The operators `&`·`|`·`^`·`!` (bitwise negation of an integer)·`<<`·`>>`. The operand types must be the same (the shift amount is `u32`, the same as Rust).
- If the shift amount is at or above the bit width, an overflow obligation (the E0803 family, the same as the panic condition of a Rust debug build). `wrapping_shl`·`wrapping_shr` have no obligation.
- Methods: `rotate_left`·`rotate_right`·`count_ones`·`leading_zeros`·`trailing_zeros`·`swap_bytes`·`to_be_bytes`/`from_be_bytes` (array return is after 2V-04)·`to_le_bytes`/`from_le_bytes`.
- `>>` is an arithmetic shift on signed types (the same as Rust).

### 3. Literals
- The prefixes `0x`·`0o`·`0b` and the `_` separator; the suffix type (`0xffu32`) is not used and the expected type is used (the same as the current rule: one normal form). The normal form keeps the radix written (a notation that carries meaning).

### 4. Proof encoding (mixed)
- Integers remain `Int` (ADR-0017). A bit operation term has the form `(bv2nat (bvand ((_ int2bv w) a) ((_ int2bv w) b)))`, and only that term goes through bitvectors (signed types use the two's complement mapping). The encoder gives result range facts (`0 <= a & b <= min(a, b)` and so on) along with it, like the division facts of 1B-42.
- Bit-level properties are proved separately in the bitvector theory with `assert(e) by (bit_vector);`, as in Verus, and the result is used as a fact. `e` is a closed expression that holds only bit operations, comparison and arithmetic. It is new syntax, but it succeeds the Verus surface (DESIGN §4: forms that come from Rust and Verus).
- Shifts and rotations by a constant amount are expanded into multiplication and division (linear obligations kept).

### 5. Code generation
- LLVM integer operations as they are. Whether narrow integers are their own width in memory and `i64` in registers (the current `u8` approach) or their own LLVM width is decided at implementation by measuring IR size and performance.

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **All Rust integer types + bit operations + the mixed encoding and `by (bit_vector)` (proposed for acceptance)** | The Rust surface as is (S1), with Verus and Dafny precedent. SHA-256 becomes proved BMB code |
| Bit operations as an escape hatch (`external_body`) | Puts what can be proved into the trust base (ADR-0045 §7). The escape hatch area (the 4F criterion) grows |
| Re-encode all integers as bitvectors | The cost of linear arithmetic obligations rises greatly (the rationale of ADR-0017). The mixed encoding pays the cost only where needed |
| Imitate with arithmetic (`x / 2^k`) | A workaround. xor and and become bit loops |

## Consequences
- The SHA-256, hashes and byte assembly (`to_be_bytes`) of self-hosting are written in the standard form. `u32`·`u16` come into being.
- A bitvector path and the `by (bit_vector)` sub-proof enter the encoder. Cost and stability are measured at implementation.
- Bit operations and the new widths are added to the unsigned arithmetic fuzzer.

## Open questions
1. `i128`·`u128`: not needed for self-hosting (`selfhost-gap.md` §4.1). They are Rust standard types, so they are added, but the timing is when the need arises.
2. Register width of narrow integers (§5).
