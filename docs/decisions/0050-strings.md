# ADR-0050: `String`·`&str`·`char`

- Status: Accepted (the length obligation of `push`·`push_str` is superseded by ADR-0124. 2026-10-26, approved by the maintainer "1/2 adopted, approved", backlog 2V-12)
- Date: 2026-10-26
- Related: ADR-0045, ADR-0044 (bytes and byte strings, partly superseded), ADR-0015, ADR-0047 (owned types), review R10, backlog 2V-10


## Confirmed at acceptance (2026-10-26)
- Diagnostic numbers: E0820 character boundary and UTF-8 invariant obligation (confirmed free). E0234 (rejection of `"..."` in an expression) retires together with the implementation. The `b'a'` replacement of E0101 and the `String`·`str`·`char` replacement of E0402 go away.
- Open question 1 (the character view `Seq<char>`) is for when the need arises. 2 (character traversal) is decided in the 4B standard library.
- Implementation phase: ROADMAP 2U (strings). After owned types (2O).

## Confirmed at implementation (2026-11-05, backlog 2U-01~05)
- **Lowering:** the prover and the code generator see text as bytes (`Tables::lowered`: `String` is `Vec<u8>`, `&str` is `&[u8]`, `char` is `u32`). So the existing byte encoding, layout and freeing are used as they are, and the checker keeps the types apart (type names in the map and diagnostics are unchanged). Operations whose meaning differs from bytes (`push` of a `char`, comparison) are separated by the checker's `Tables::text` (expressions whose value is text).
- **Invariant (§3):** every way of making text is a built-in operation (literals, `String::new`, `push_str` of text, `push` of a `char`, `from_utf8` after its check, a piece at boundaries), so it holds by construction. So the obligation at the creation point is a theorem and is not emitted. The fact at the point of use is not added yet either: nothing in the proofs of this stage needed it. As the price, a fact that follows from the invariant, such as "after an ASCII byte is a boundary", is proved with an `is_char_boundary` check (`tests/run/strings`·`escape`). Whether to add it as a fact is 2U-06.
- **Character traversal (2026-11-08, 4B-02 (b)):** open question 2 was answered with `std::str::decode_utf8(s, i) -> Option<Decoded>` (the character and the next position, a contract exactly equal to the well-formed sequences of Table 3-7). Without iterators (ADR-0049), traversal is `while` over that result (`chars` in `tests/run/std_str`).
- **Facts at the point of use (2026-11-08, 2U-06):** they are facts only where a boundary is asked about. The obligation (E0820) of a piece `&s[a..b]` assumes, for that one obligation only, what UTF-8 says about the four bytes right before each end (allowed bytes, after an ASCII byte or a whole character is not a continuation byte, the number of continuation bytes after a lead byte, no four continuation bytes in a row, the lead byte of a continuation byte is within three before it, a lead byte is not a continuation byte). The targets are the text the piece is taken from and text the definition received (a text parameter, a call result, `&mut` text after a call: the checker's `Tables::text_sigs`), and for received text also at its end (the last character is whole: if the prefix matches, what follows is a boundary). There is no quantified fact (`utf8_window` in `verify/text.rs`). So the boundaries after an ASCII byte, after a whole character and before a lead byte are proved without a runtime check (`tests/verify/pass/utf8_facts`: skipping leading spaces, splitting at `:`, stripping a prefix. The `is_char_boundary` checks of `tests/run/strings`·`escape` were deleted), and the counterexample of a refuted boundary is valid UTF-8 (before, it was a value that cannot be text, such as `[128, 128]`). First, one quantified fact was put at every place where text comes in, and measured: instances chained from byte to byte, so a refuted boundary hit the limit without a model (the `push_str` length obligation), or used half the limit (an rlimit spread of 12 times over 8 seeds), and without the backward-looking clause, stripping a prefix was 1.8M. The current form has a seed spread of 1.02, and `escape` is cheaper than before even with the checks deleted (1,536,353 → 1,491,315). Assertions and contracts do not receive this fact (`text_begins` in `tests/verify/fail/utf8_facts`): when demand appears, the same local fact for that obligation. Mutation `v-utf8-char-ends-before-ascii`.
- **Pieces:** `&s[a..b]` is E0820 (`a <= b`, both positions Rust's `is_char_boundary`). The value is the declared function `bmb_shift` and one axiom, and that axiom enters only the queries that use it. `lambda` was used first, but Z3 gave up with "incomplete (theory array)" (`tests/run/escape.bmb`; the real cause was a false invariant, but it could not produce a counterexample).
- **`push(c)`:** for a constant `c` the exact UTF-8 bytes, otherwise the exact length (`len_utf8`) and the ASCII byte, and for the rest only the case split of lead byte (192..=244) and continuation bytes (128..=191) are facts. The `div`·`mod` of the exact encoding made queries heavy when together with the quantified facts of text. The case split is enough for boundary proofs.
- **Comparison:** in code, all six comparisons exactly, with a fresh constant for the longest common prefix and its defining facts; in specs, `==` is slice equality and order is a quantified expression over the common prefix.
- **`from_utf8`:** the proof does not model whether it succeeds (a new condition). The result is the same bytes for both `Ok` and `Err`. The runtime validation follows the rules of Rust's `str::from_utf8`.
- **Rust habits:** `&s`·`s` in a `&str` position is E0218 (the title widened to `String`, towards `s.as_str()`), `"..."` in a `&[u8]` position and `'a'` in a `u8` position are E0401 with the `b".."`·`b'a'` fix (E0234 retired), `to_string`·`to_owned` become the single `String::from` (E0404 hint), traversal such as `chars()` is bytes and `while` (open question 2 is 4B), and `+`·`format!`·`s[i]`·`char` arithmetic are rejections with a hint.
- **Allowed scope:** `Vec<String>`·`&[String]` work and `[String; N]` does not, like `Vec`. `char::from_u32` is also used in specs (a pure function). Spec functions do not take `String` (`&str`, the same rule as `Vec`).
- **Runtime:** `memcmp` (libc) is used for comparison. Appending also accepts bytes that come from its own buffer (found again by the offset before growing: `s.push_str(&s[a..b])`).

## Context
ADR-0044 rejected `String`·`&str`·`char` and replaced them with `Vec<u8>`·`&[u8]`·`u8`. The rationale was the cost of bringing the UTF-8 invariant into proofs and "compiler strings are practically ASCII". Cost is not a criterion (ADR-0045 §5), and strings are a standard Rust type and the area where an LLM has the most prior knowledge (S1, the Rust habit error rate of S4). Verus specifies `String` with a `Seq<char>` view.

## Decision

### 1. Types
- `String`: owned, a valid UTF-8 byte sequence. A move type (ADR-0047).
- `&str`: a borrowed string. Per the reference rule (ADR-0015), only in argument position.
- `char`: a Unicode scalar value (`0..=0x10FFFF` excluding the surrogate range). Literals `'a'`, `'\u{1F600}'`.
- A string literal `"..."` is `&str` (argument position, the same rule as `b"..."`).

### 2. Operations (names and meanings of the Rust standard library)
- Creation and extension: `String::new()`, `String::with_capacity(n)`, `s.push_str(t)`, `s.push(c)`, `s.clone()`.
- Views: `s.len()` (bytes), `s.as_str()`, `s.as_bytes()` (`&[u8]`, argument position), `s.is_char_boundary(i)`.
- Conversion: `String::from_utf8(v: Vec<u8>) -> Result<String, Vec<u8>>` (Rust has `FromUtf8Error`; the same information, with the bytes handed back), `String::from(t: &str)`, `char::from_u32(x) -> Option<char>`, `u32::from(c)`, `c.len_utf8()`.
- Comparison: `==`·`<` in executable code (byte lexicographic order, the same as Rust).
- Pieces: a piece cannot return a reference (ADR-0015), so it is handled as `(start, end)` byte positions, and a function that takes a piece is passed `&s[a..b]` in argument position (boundary obligation: `is_char_boundary`).
- Output: built-in `print_str(s: &str)` (`io`). The macros `print!`·`println!`·`format!` do not exist, per the ledger (metaprogramming).

### 3. Proof encoding
- A `String`/`&str` value is the byte sequence `(Array Int Int, Int)` with the invariant `utf8(bytes, len)` (a recursive `spec fn`) attached. The invariant is an obligation at the creation point (for built-in operations preservation is a theorem), and a fact where the value is used (the same way as integer range facts).
- `char` is `Int` with a range fact (scalar value).
- The view for specs `s.view()`: a character sequence (`Seq<char>`, the same as Verus) is open question 1.

### 4. Code generation
- `String` has the same layout as the `Vec<u8>` header, and `&str` is (pointer, length). `from_utf8` is a validation loop (the runtime check is a failure path carried in the result type, so it is not an obligation).

### 5. Relation to ADR-0044
- `u8`, `Vec<u8>`, `b'a'`, `b"..."`, `print_bytes`, `lib/bytes.bmb` stay as they are (bytes as bytes).
- What changes: the rejection of `"..."`·`'a'`·`String`·`&str`·`char` (E0234, the `b'a'` replacement of E0101, the `String` replacement of E0402) goes away. The code E0234 is retired without changing its meaning (no reuse).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **`String`·`&str`·`char` with the UTF-8 invariant (proposed for acceptance)** | The Rust surface as is (S1). The invariant is an obligation at the creation point + a fact at the point of use, and proofs of byte operations cost the same as now |
| Bytes only (the current ADR-0044) | An expressiveness defect (ADR-0045 §7). All the Rust string code an agent writes is rejected |
| Unverified `String` (no invariant) | The correctness of `as_bytes` and boundary operations cannot be proved (C1) |

## Consequences
- Code that handles text, such as the lexer, diagnostic messages and JSON, takes the Rust form.
- Implemented after owned types (ADR-0047) and generics (ADR-0046) (`String` is a move type, comparison is `Ord`).
- Fuzzer: UTF-8 boundaries and the `from_utf8` failure path are added to the byte fuzzer.

## Open questions
1. The character view in specs (`Seq<char>`): added when a task appears that needs character-level specs.
2. Character traversal: there are no iterators (ADR-0049), so a library function such as `decode_utf8(s, i) -> (char, usize)` is decided in 4B.
