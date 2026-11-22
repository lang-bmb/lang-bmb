# ADR-0061: Floating point `f64`: IEEE 754 meaning, opaque encoding by default and bit-precise proof inside `by (bit_vector)`

- Status: Accepted (approved by the maintainer 2026-10-28 "ADR-0059 to 0062 accepted")
- Date: 2026-10-28
- Related: ADR-0045 (judgement criteria), ADR-0017 (integer encoding), ADR-0028 (conversions), ADR-0048 §2·§3 (`by (bit_vector)`), ADR-0046 (`Eq`·`Ord` bounds), ADR-0019 (normal form), direction discussion `dev-docs/reviews/direction-2026-10-28.md` §7.2 T3, §8 Q4

## Confirmed at acceptance (2026-10-28)
- Implementation phase: ROADMAP M2-d (representation extension). The order is backlog T2-02 (encoding cost experiment, F1 of the Consequences section) → T2-03 (F2~F7). The default of §5 (opaque symbols) is fixed by the measurement of T2-02, and if the measurement points the other way it is changed by a new ADR.
- The `fp.*` interpretation inside `by (bit_vector)` of §5 depends on 2N-03 (sub-proof syntax). Before 2N-03, work proceeds up to F2·F3·F5 and the opaque encoding.
- Diagnostic numbers: E0815 uses the existing number (the range obligation of ADR-0028). The number of the new literal diagnostic is chosen from the unused numbers of `diag.rs` at implementation.
- Open questions 1~4 stay as they are. Each is opened when demand appears.

## Confirmed by measurement (2026-10-28, backlog T2-02)
- The default of §5 is kept. The measurement is `dev-docs/experiments/T2-02/`. On obligations where floating point is unrelated to the goal the two encodings are the same, and when a floating-point comparison decides the path, always-bit-precise (candidate (a)) fails to decide within the limit (the mandelbrot shape). Opaque proves the same obligation with 4% of the limit.
- The sat model of an opaque query may not agree with IEEE. A counterexample is shown only when recomputing the operations with the model's values agrees (G1). The implementation is T2-03 (F4).

## Confirmed at implementation (2026-10-28, backlog T2-03)
- **Literals (§2):** `Tok::Float` (the bits of the value) and `ExprKind::Float`. `1.`, `.5`, `1e5` (no point) and `1.0e` (no exponent digits) are the new diagnostic E0109, and the first three get a fix that replaces them with the corrected literal. `1..n` and `1.max(2)` are unchanged (if what follows the point is `.` or a name, it is not floating point). The suffixes `f64` and `f32` are E0104 (fix removes them), and a literal that rounds to infinity is E0406 (the same rejection as Rust's `overflowing_literals`). Upper-case `E` and the `_` separator are accepted.
- **Normal form (§2):** Rust `{:?}` produces notations with no point in the mantissa, like `1e-9` and `1e16`. As is, the normal form would not be a literal again, so `.0` is put in the mantissa (`1.0e-9`, `lexer::float_text`). Otherwise it is the same as `{:?}`. On 2,000 random bit patterns and the boundary values, normal form → lexer returns the same bits (`fmt.rs`).
- **Expected type (§2):** a floating-point literal is `f64` everywhere (the same as Rust's default). An integer literal in an `f64` position is E0401 with a fix that changes it to `1.0`. `x as f64` is E0205 with the fix `f64::from(x)`.
- **Constants:** Rust's `f64::MIN` was added to the six constants (the same path as `T::MIN`·`T::MAX`, `check::F64_CONSTS`). A constant that does not exist is E0305 with the list.
- **Checker (§1, §3):** `f64` is a scalar (arrays, `Vec`, fields, payloads, `Box`, generic arguments, `&f64`/`&mut f64` parameters) and has no `Eq` or `Ord` (E0428, NaN hint). `%`, bit operations, libm methods, `to_bits`, `total_cmp` and `f64` → integer are all rejections with a hint (E0401·E0404·E0412).
- **`==` in specifications:** `==` on scalar `f64` is the same IEEE comparison as in code (`f64.eq`). The structural `==` of struct and array values (specification-only, ADR-0026 §2) is SMT equality per field, so on `f64` fields it is identity of the value (NaN = NaN, `-0.0` ≠ `0.0`). It is a specification-only equivalence, so it is sound.
- **Encoding (§5):** sort `Float64`, literals are `(fp ..)` with the exact bits (NaN is `(_ NaN 11 53)`). The operations are `f64.add`·`f64.sub`·`f64.mul`·`f64.div`·`f64.neg`·`f64.sqrt`·`f64.abs`·`f64.min`·`f64.max`·`f64.from` (integer)·`f64.lt`·`f64.le`·`f64.eq`·`f64.is_nan`·`f64.is_finite`, and `>`·`>=` are `lt`·`le` with the operands swapped (the same meaning in IEEE). A query declares only the symbols it uses (`verify::f64_decls`): the text of a query with no `f64` does not change. The interpretation is one definition in the same table (`verify::F64_FUNS`) used in two places (sub-proofs, counterexample search). `f64.from` inside a sub-proof is `to_fp RNE` of a signed bitvector (the same as LLVM `sitofp`/`uitofp`), and the logic is `QF_FPBV`.
- **IEEE check of counterexamples (G1, found in T2-02):** when a general query is sat, all `f64.*` applications outside quantifiers in the asserted expressions are collected, the model values of operands and results are asked for (`get-value`), and they are recomputed with Rust `f64` arithmetic and compared (`verify::f64_model_check`; NaNs are equal to each other, the rest by bits). Applications inside quantifiers and definition bodies, and `min`/`max` of two zeros with different signs (Rust does not fix the sign), count as cannot be checked. If they do not agree, the model is discarded and the same query is solved again with interpreted definitions (`f64_interpreted`, same rlimit). That model also has to pass the same check to be shown as a counterexample. If the interpreted query is unsat, it writes "holds in IEEE arithmetic" and is E0811 (undecided, no counterexample, `by (bit_vector)` hint). Even if the interpreted query proves it, that is not used as a proof (the §5 default is kept). Measurement: the opaque model of `x + y == y + x` had `f64.eq` false on two zeros, and the IEEE search found `y = NaN` (rlimit about 2,800,000, 0.5 seconds). The rlimit of this search is not added to the rlimit sum of the definition (it is not a proof cost). Counterexamples of sub-proofs go through the same check.
- **`print_f64` (§6):** Rust panics if the precision is greater than `u16::MAX` ("Formatting argument out of range", checked on rustc 1.97). So `decimals <= 65535` is the call precondition obligation E0801 (a run-time check if not proved). The runtime prints `NaN` for NaN and otherwise uses `printf("%.*f\n")`, and it is put in the module only when the program calls it (the IR of other programs is unchanged).
- **Code generation (§6):** `min` and `max` are `llvm.minnum`/`maxnum`. rustc 1.97 emits `nsz llvm.minimumnum`. Both handle NaN the same way and Rust does not fix the sign of two zeros, so the differential tests and the fuzzer leave that case out. `is_finite` is `fabs` then `fcmp olt inf`, `is_nan` is `fcmp uno`. If `f64::from` of a 64-bit source is not proved, the check `|x| <= 2^53` remains.
- **Run programs (F7):** n-body (n = 1000) and spectral-norm (n = 100) are proved and match the bench's expected output for small arguments. The official Rust submissions are blocked by the network, so the comparison is with Rust reference programs written from the public algorithms (`bench/game/ref/`). The exactness obligation of `f64::from(index)` in spectral-norm is proved with the premise `i, j <= 10000`.
- **Cost:** the `f64` corpus (`tests/verify/pass/floats`, `tests/run/floats`·`mandel`) is small (rlimit of thousands to tens of thousands per definition). Found during implementation: a query in which a spec fn not revealed by `#[uses]` takes `(Array Int Float64)` ran to the rlimit limit and then answered on retry (backlog T2-04).

## Context

- BMB has no floating point. The parser does not accept `f32` or `f64` as type names (the reserved primitive type list in `compiler/src/parser.rs`). Literals are decimal integers only (`lexer.rs`), and code generation puts every scalar in an `i64` register (ADR-0048 §1).
- The track reset (T3) replaces the 7 in-house performance benchmarks with recognized benchmarks. Of the programs of the Computer Language Benchmarks Game, n-body, spectral-norm, mandelbrot and fasta use `f64`. Only binary-trees and fannkuch-redux can be written in the language now. Under the new track rule ("a language phase opens on a concrete demand of T3 or T4") that is exactly the demand for this ADR.
- Canonical Rust programs that use floating point cannot be written in BMB now. By JUDGMENT §7 this is a language defect. The ledger (DESIGN §4.1) has no row forbidding floating point.

## Precedents (JUDGMENT S1 order)

| Source | Content | Check |
| --- | --- | --- |
| Rust | `f64` is IEEE 754 binary64, rounding is round-to-nearest-even (RNE). Arithmetic does not panic (overflow → ±inf, `0.0/0.0` → NaN). Only `PartialEq`·`PartialOrd`, no `Eq`·`Ord` (NaN). The total order is `total_cmp`. Integer ↔ floating point is `as`. Floating point → integer `as` saturates since 1.45 (out of range gives the end value, NaN gives 0). Only `f64: From<i32/u32/i16/u16/i8/u8>` (exact conversions) exists, no `From<i64>`. NaN bit patterns (payloads) are not guaranteed | The cast section of Rust by Example. NaN payloads: RFC 3514 (original not checked) |
| Verus | Supports IEEE operations with the SMT floating-point theory but **uninterpreted by default** and interpreted only inside `assert .. by(bit_vector)`. "Rust floating point = IEEE" is not a default assumption but an optional module (`assume_ieee_float`) | [PR #2229](https://github.com/verus-lang/verus/pull/2229) (merged 2026-03), [#2232](https://github.com/verus-lang/verus/pull/2232) (draft). Search summaries, original not checked |
| Dafny | No built-in IEEE type. `real` is the mathematical reals, and used as a floating-point model it misses rounding and underflow | Search summary (citing the DafnyMPI paper) |
| Why3·Frama-C | Why3 `ieee_float` is a complete theory including infinities and NaN. The floating-point model of Frama-C WP conforms to IEEE but itself states that "support for proving properties with automatic provers is very limited" | [Why3 stdlib](https://why3.lri.fr/stdlib/ieee_float.html), Frama-C WP changelog |
| SMT-LIB·Z3 | `(_ FloatingPoint 11 53)` (Float64), operations with an RNE argument such as `fp.add`·`fp.mul`·`fp.div`·`fp.sqrt`·`fp.lt`. Z3 solves by bit-blasting. It can be tens to thousands of times slower than real arithmetic on the same expression | SMT-LIB FP theory, a 2019 comparison study (search summary, original not checked) |
| Others | CopilotVerifier is also uninterpreted by default and turns on IEEE interpretation by a setting. CBMC is bit-precise | Search summaries |

## Decision

### 1. Type

- **There is one type, `f64`.** `f32` is added by the same rules when demand appears. The target programs of T3 all use `f64`. The parser keeps issuing E0402 for `f32` and advises changing to `f64`.
- `f64` is a scalar copied by value (`Copy`). It goes into arrays, slices, `Vec`, struct fields, enum payloads and generic arguments by the same rules as integers.
- `f64` does not have the built-in bounds `Eq`·`Ord` (2T-02). Same as Rust. It cannot be used for sorting or binary search, and `total_cmp` is added when demand appears.

### 2. Literals and normal form

- Shape: `digits.digits` with an optional exponent `e[+-]digits` (`1.0`, `0.5`, `6.02e23`, `1.0e-9`). Digits on both sides of the point are required. `1.` and `.5` are not accepted. So it does not overlap with `0..n` (range). There is no suffix, and it is a floating-point literal only when the expected type is `f64` (the same rule as integer literals).
- The value is the decimal notation correctly rounded to the nearest binary64 (the same as Rust `str::parse::<f64>`).
- **The normal form** (ADR-0019) is the shortest round-trip notation of that value (the same as Rust `{:?}`. The compiler computes it with the Rust standard library). `0.10` and `1e-1` become `0.1`. The same value gives the same text and the same hash.
- `f64::INFINITY`·`f64::NEG_INFINITY`·`f64::NAN`·`f64::MAX`·`f64::MIN_POSITIVE`·`f64::EPSILON` are associated constants (the same path as `T::MIN`·`T::MAX` of 2N-06).

### 3. Operations

| Operation | Meaning | Obligation |
| --- | --- | --- |
| `+`·`-`·`*`·`/`, unary `-` | IEEE RNE | None. A total function that does not panic in Rust |
| `%` | Rust's `%` (fmod, truncating). Differs from SMT `fp.rem` (IEEE remainder) | Left out of the first scope. Added as an opaque function when demand appears |
| `==`·`!=`·`<`·`<=`·`>`·`>=` | IEEE comparison (with a NaN involved only `!=` is true) | None |
| `x.sqrt()`·`x.abs()`·`x.min(y)`·`x.max(y)`·`x.is_nan()`·`x.is_finite()` | Rust standard meaning. `sqrt` is correctly rounded in IEEE | None |
| `sin`·`cos`·`exp`·`ln`·`powf` etc. | libm. Correct rounding is not guaranteed, so a bit-precise encoding is impossible | Left out of the first scope (open question 2) |

- Judgement on hidden failure paths (the ledger row "arithmetic"): integer overflow splits in Rust into panic (debug) and wrapping (release), so it was made an obligation. Floating-point arithmetic has one meaning in Rust and is a total function. So no implicit obligation is added. If finiteness is needed, write `is_finite()` in the contract and prove it with `by (bit_vector)` (§5).

### 4. Conversions

- **Integer → `f64`:** `f64::from(x)`. Rust has `From` only for integers of 32 bits or less. BMB accepts it for every integer type.
  - If the source is 32 bits or less there is no obligation (it is exact).
  - If the source is `i64`·`u64`·`isize`·`usize`, the exactness obligation `|x| <= 2^53` is emitted (E0815, the same number as the range obligation of ADR-0028). It is an integer fact, so the current prover solves it (`i + j` of spectral-norm, the integer random numbers of fasta, etc.).
  - The difference from Rust (no `From<i64>`) is written as one row in the ledger (§Consequences).
- **`f64` → integer:** left out of the first scope. Saturating `as`, the only safe path in Rust, is a "silently lossy conversion" that the ledger forbids. None of the four target programs uses it. The surface is decided after 2R-06 (surface decision for `as`) (open question 1).
- **Bit reinterpretation (`to_bits`·`from_bits`):** left out of the first scope. NaN payloads are not guaranteed in Rust and SMT-LIB has only one NaN. So a fact about `to_bits(NaN)` could be unsound whichever way it is fixed.

### 5. Proof encoding

- **The sort is bit-precise:** an `f64` value is SMT `(_ FloatingPoint 11 53)`.
- **Operations and comparisons are uninterpreted by default:** in general obligation queries `+`, `<`, `sqrt` etc. are uninterpreted function and predicate symbols (`f64.add` etc.). A path condition (`if x < 4.0`) enters as an atom of that symbol.
  - Consequence: the integer, index and move obligations of a function mixed with floating point are solved at the current cost, without bit-blasting.
  - An uninterpreted function yields only facts that are true in every interpretation, so it is sound (C1). Same operation, same arguments → same value (congruence) holds because IEEE RNE is a deterministic function. The only thing that does not hold is NaN payloads, and §4 made them unobservable by leaving out bit reinterpretation.
- **Bit-precise proof is done inside `assert(e) by (bit_vector);`:** the sub-proof of ADR-0048 §3 is used as is. Verus PR #2229 does the same thing with the same syntax.
  - Only in that query are the symbols interpreted as `fp.add RNE` etc.
  - The proved `e` is used in the outer query as a fact over the uninterpreted symbols.
  - The basis of soundness is that code generation emits IEEE RNE as is (§6). The reason not to make the "IEEE assumption" an optional module like Verus: BMB has one compiler, and the compiler itself guarantees that LLVM output with fast-math and contraction off is IEEE.
- **The real (`Real`) encoding is not used:** it ignores rounding and yields false facts (violates C1).

### 6. Code generation and determinism

- **Representation:** `f64` is LLVM `double`. It is a slot kind separate from the register model (`i64`). In memory it is 8 bytes.
- **Operations:** `fadd`·`fsub`·`fmul`·`fdiv`·`fneg`·`fcmp` (ordered comparisons `olt`·`ole`·`ogt`·`oge`·`oeq`, `!=` is `une`), `llvm.sqrt.f64`·`llvm.fabs.f64`, and `min`·`max` are `llvm.minnum`/`llvm.maxnum`, which match the Rust meaning.
- **Determinism (C2):**
  - No fast-math flags are attached.
  - `contract` (FMA contraction) is not attached. Without the IR flag, clang `-O2` does not contract either.
  - On x86_64 (SSE2) and aarch64 the same IR gives the same bits. This is checked by adding floating-point run programs to the determinism test (V6).
- **Output:** the built-in `print_f64(x: f64, decimals: u32)`. Fixed-point output the same as Rust `println!("{:.N}", x)`. The implementation is libc `printf("%.*f")` (glibc correctly rounds the exact value). The runtime dependency stays libc (S3 = `[]`). `sqrt` is an LLVM intrinsic, so libm is not needed. It is differentially tested against Rust formatting, including exact halfway cases (ties).

### 7. Fuzzer

- The independent interpreter (`tests/soundness*.rs`, Python helper) uses binary64 as is. CPython `float` is C `double` and `+ - * /` and `math.sqrt` are IEEE RNE. Values are compared by `struct.pack('<d')` bits, except NaN, which is compared with `isnan`.
- The generator mixes finite values, signed zeros, infinities, NaN and subnormals. The output of the two builds (checks removed, all checks) is compared with the output of the interpreter.
- Mutants: `fadd`↔`fsub`, `olt`→`ole`, missing restriction of interpretation to assertions (interpreting `fp.lt` outside `by (bit_vector)`). All three go into gate detection power (`dev-docs/measurements/gate-power`).

## Rationale (JUDGMENT §10)

1. **Problem:** canonical Rust floating-point programs cannot be written in BMB. Four recognized performance benchmarks of T3 are blocked.
2. **Cause and layer:** the language (JUDGMENT §3 "the canonical solution cannot be expressed in the language"). The ledger has no rationale for forbidding it, so it is a defect (§7).
3. **Canonical solution:** Rust `f64` meaning (S1 a) + the "opaque by default, bit-precise locally" of verification tools (S1 b: Verus PR #2229, CopilotVerifier).
4. **Comparison of candidates:**

| Candidate | C1 Sound | C2 Determinism | C3 | C4 | S1 Canonical | S2·S3 | S4 Goal (cost) | S5 Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| (a) Always bit-precise (interpret `fp.*` in every query) | Pass | Pass | Pass | Pass | CBMC | No sub-proof concept needed | Every query involving floating point is bit-blasted. Risk of worse V5 (limit headroom) and V8 (bench proof cost) | Cost not measured |
| **(b) Opaque by default + bit-precise inside `by (bit_vector)` (recommended)** | Pass (§5) | Pass | Pass | Pass | Verus·CopilotVerifier | One `by (bit_vector)` handles bit operations and floating point together (S3) | Current cost if floating point is unrelated to most obligations | Cost not measured |
| (c) Real (`Real`) encoding | **Violation** (ignores rounding) | — | — | — | Dafny `real` (a precedent that says not to use it as a floating-point model) | — | Cheapest | — |
| (d) Floating point as an escape hatch outside proof | Pass | Pass | Pass | A restriction outside the ledger (§7) | — | Depends on the escape hatch (2E, not accepted) | Cheap | — |

5. **Recommendation and decisive criterion:** (b). (c) is out by C1. (d) is out by JUDGMENT §7 (an encoding being hard is not a reason to leave out a feature). (a) and (b) part on S1 (the precedent of verification tools is b) and S3 (the same sub-proof as bit operations). But there is no S5 evidence (cost) yet, so before acceptance a measurement task (F1 below) measures the rlimit of (a) and (b) and the result fixes the default. If (a) is within the fixed V5 rule (0.2) on the target programs and the corpus, (a) is simpler.
6. **Impact (information):** lexer, parser, normal form, checker, encoder, code generation (new slot kind), fuzzer, diagnostic table. One built-in, `print_f64`.
7. **Approvals needed:** widening the language scope (AUTONOMY §4). Widening `f64::from` to 64-bit sources adds a ledger row (DESIGN change, approval).

## Consequences

- **Recognized benchmarks that become writable (T3):**
  - With this ADR alone: n-body (array of structs, `sqrt`), spectral-norm (`Vec<f64>`, `f64::from`, `sqrt`)
  - Together with 2N-02 (bit operations): mandelbrot (PBM bit output, `print_bytes`)
  - Together with 2U and byte output: fasta (integer LCG + cumulative probability comparison, `f64::from(u32)`)
- **DESIGN §4.1 ledger:**
  - Add to the row "conversions": "integer → `f64` is `f64::from` only. A 64-bit source has an exactness obligation (`|x| <= 2^53`). No `f64` → integer and no bit reinterpretation".
  - Add to the row "arithmetic": "floating point is an IEEE total function, so no obligation".
- **Open questions:**
  1. The surface of `f64` → integer conversion (after 2R-06).
  2. libm functions (`sin`·`exp` etc.). A bit-precise encoding is impossible, so whether to keep them only as uninterpreted symbols and accept the libm dependency (S3).
  3. `f32`.
  4. `total_cmp` and sorting.
- **Proposed tasks (numbers are fixed in the backlog):**
  - F1 Encoding cost experiment: in the `dev-docs/experiments/` format, measure the rlimit and verdicts of (a) and (b) on drafts of n-body and spectral-norm and on synthetic queries. The same method as 2B-02, and the result fixes the §5 default.
  - F2 Lexer, parser, normal form: literals, shortest round-trip normal form, associated constants, diagnostics (malformed literal shapes, with hints).
  - F3 Checker: types, operations, comparisons, methods, `f64::from` and the E0815 obligation. Rejection of `Eq`·`Ord` bounds.
  - F4 Encoder: opaque symbols by default, `fp.*` interpretation inside `by (bit_vector)` (depends on 2N-03), passing and failing corpus (counterexample values in shortest decimal notation).
  - F5 Code generation: `double` slots, operations and intrinsics, `print_f64`. Differential test against Rust `{:.N}`.
  - F6 Fuzzer: floating-point generator, Python bit-precise interpreter, three mutants.
  - F7 Run programs and recognized benchmarks: write n-body and spectral-norm in BMB and prove them. Compare output agreement, time and size with the official Rust submissions of the Benchmarks Game. End audit (HEALTH §5).
