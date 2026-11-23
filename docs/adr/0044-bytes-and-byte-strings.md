# ADR-0044: Byte `u8`, byte string literals, byte output and the bytes library

- Status: Accepted (2026-10-25. Following the approval to start 2S and the instruction to adopt the recommendation, backlog 2S-03). The rejection of `"..."`·`'a'`·`String`·`&str`·`char` in §5 is superseded by ADR-0050 (the byte API stays as is)
- Date: 2026-10-25
- Related: DESIGN.md §4.1·§5, ADR-0015 (second-class references), ADR-0016, ADR-0017 §3, ADR-0022, ADR-0028, ADR-0031, ROADMAP 2S, backlog 2S-01, measurement `dev-docs/measurements/selfhost-gap.md` (4A-01)

## Context
In the self-hosting gap analysis (4A-01), the first required feature that no phase covers was bytes and strings (`selfhost-gap.md` §3 (a)). The current compiler uses `String` in 651 places, `&str` in 294, `u8` in 89, and 2,212 string literals (45KB), and its output is practically all text built by appending the default `Display` (integers, strings) (the table in the same document). Bytes are needed from the lexer on.

BMB has no byte type now. The character literal `'a'` is E0101 (no fix), and `"..."` in expression position is a general syntax error (E0103). Output is only `print_i64`·`print_u64` (integers, one per line). With 2G (ADR-0031) `Vec<T>` arrived, so a variable-size buffer already exists.

The recommendation of the gap analysis is not to create a new string type but to use `Vec<u8>` + (start, length) handles, and to work with bytes without a character type (the number of UTF-8 characters is obtained by counting the bytes that are not continuation bytes).

## Confirmed at acceptance (2026-10-25)
- The diagnostic number E0234 was confirmed free in `compiler/src/diag.rs`.
- The open questions stay as proposed: no automatic inclusion (4B), and the sorted map and input are outside this ADR.

## Decision

### 1. `u8`
- The integer type `u8` (range 0..=255) is added. Arithmetic, comparison, `wrapping_*`·`checked_*`·`saturating_*` and the division obligations follow the same rules as `u64`, and only the range differs (overflow E0803).
- Conversion keeps the form of ADR-0028. `u8::try_from(x).unwrap()` has the obligation `0 <= x <= 255` (E0815), and from `u8` to `u64`·`usize`·`i64` there is no obligation. An integer literal must be 0..=255 when the expected type is `u8` (the existing literal range diagnostic).
- It is 1 byte in memory (LLVM `i8`). It can be the element of an array or `Vec`, a struct field, or an enum payload. `Vec<u8>` is the byte buffer.
- In specs it mixes with other integers (ADR-0022).

### 2. Byte literals and byte string literals
- `b'a'` is a `u8` value. The escapes are `\n` `\r` `\t` `\\` `\'` `\"` `\0` `\xHH` (the same as Rust).
- `b"..."` is `&[u8]`. References are second-class (ADR-0015), so it is used **only in call argument position** (elsewhere E0409, the same rule as `v.as_slice()`). The elements are ASCII characters and the escapes above (the same as Rust: non-ASCII is `\xHH`).
- Putting it into a buffer is the library's `bytes::extend(&mut v, b"...")` (§4).
- Proof: a literal is an array whose length is constant and whose elements are each known (a `store` chain). Code generation: a private constant global of the module, and as an argument (pointer, length).

### 3. Byte output
- Built-in `print_bytes(s: &[u8])` (effect `io`): writes the bytes as they are to standard output (no newline appended). It corresponds to Rust's `std::io::stdout().write_all(s)`. Like `print_i64`·`print_u64`, it is an `io` boundary edge of the map.

### 4. The library `bytes` (written and proved in BMB)
- `lib/bytes.bmb` in the repository, unit `bytes`. It is not bundled with the compiler: the program that uses it passes the file along (`bmb build lib/bytes.bmb app.bmb`). Packages and automatic inclusion are decided in 4B.
- Functions (all proved together with contracts): `eq(a, b) -> bool`, `starts_with(s, p) -> bool`, `find(s, p) -> Option<usize>` (first position), `extend(v: &mut Vec<u8>, s: &[u8])` (Rust `extend_from_slice`), `push_u64(v: &mut Vec<u8>, x: u64)`·`push_i64(v: &mut Vec<u8>, x: i64)` (decimal), `parse_u64(s) -> Option<u64>` (decimal, `None` on overflow).
- Splitting and pieces are done with handles: `find` gives a position and a piece is (start, length). There is no function that returns a piece as a value (references cannot be returned).

### 5. Rust habits to reject, and diagnostics (numbers confirmed free in `diag.rs` at the time of acceptance)

| Habit | Handling | Code |
| --- | --- | --- |
| `"..."` in expression position (`&str`) | Mechanically replaced with `b"..."` (non-ASCII as `\xHH`) | E0234 (new) |
| `'a'` (`char`) | The mechanical replacement `b'a'` is added to the existing E0101 (when ASCII) | E0101 (existing) |
| The types `String`, `&str`, `char` | The existing E0402, hint: `Vec<u8>`, `&[u8]`, `u8` | E0402 (existing) |
| `format!`, `println!` | The existing E0202, `bytes::push_u64`·`print_bytes` added to the hint | E0202 (existing) |
| Slice methods such as `s.len()` | As is (`&[u8]` is a slice) | — |

### 6. Tests and fuzzer
- Corpus: `tests/check/err` (the table above), `tests/verify/pass|fail` (`u8` arithmetic and conversion obligations, literal content), `tests/run` (byte output, programs that use the library).
- The library `lib/bytes.bmb` is fully proved in the gate.
- Fuzzer: `u8` values and conversions are added to the unsigned arithmetic fuzzer (1B-33, 2C-04), and `Vec<u8>` and byte string arguments to the `Vec` fuzzer (2G-05). This is the "the fuzzer covers it" of the 2S pass criteria.

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **`u8` + `Vec<u8>` + byte strings in argument position (proposed for acceptance)** | There is only one new value type (`u8`), and the rest uses the `Vec` of 2G, slices and the rules of ADR-0015 as they are. The proof encoding is the existing (array, length) pair |
| `String`·`&str` (UTF-8 invariant) | The valid UTF-8 invariant must be brought into proofs or checked at run time. The compiler's strings are practically ASCII and byte comparison (4A-01), so little is gained. Rejected |
| A `char` type | The only place the compiler uses characters is column number computation (`chars()` in 14 places). It can be done with bytes. Rejected |
| The library as built-in methods (`v.extend_from_slice(s)` etc.) | The proof encoding and code generation grow per method. Written and proved in BMB, the trust base does not grow, and it becomes the first case of 4B (libraries) |
| Line-based output such as `print_str` | Output assembly is by default a buffer + one write (4A-01: `format!`·`push_str`). One function that writes bytes as they are is enough |

## Consequences
- Output widens from integer lines to a byte stream. Executable programs can produce text.
- A library file exists for the first time (`lib/`). There is no automatic inclusion, so the user passes it (reviewed again in 4B).
- `u8` arithmetic often produces overflow obligations (255). `wrapping_*`·`checked_*` already exist.

## Open questions
1. Whether to include the `bytes` library in programs automatically (4B).
2. Byte string keys of a sorted map (4A-01 (b)) are outside this ADR.
3. Standard input and file reading (4A-01 (c)) are outside this ADR. Without input, the lexer is tested with literals.
