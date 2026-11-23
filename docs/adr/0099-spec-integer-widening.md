# ADR-0099: Integer widening in specifications: branches and results

- Status: Accepted (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after them ADR-0100 approved"). Backlog 2N-08. The warning for a widening `as` in a specification is W0917
- Date: 2026-11-09
- Related: ADR-0022 (mixing integer types in specifications), ADR-0017 §4 (range guards of spec fn definition axioms), ADR-0028·0095 A4 (`x as T`), ADR-0095 (input aliases)

## Context

- With ADR-0022, specification arithmetic, comparisons and `spec fn` arguments mix integer types. But the branches of `if`·`match` among themselves, and a `spec fn` body against its declared result type, must be the same (E0401), and specifications have no conversions (E0513).
- So to use a `u8` byte as a `u64` result it had to be passed to an identity function: `spec fn wide(b: u64) -> u64 { b }` in `lib/std/sha256.bmb` and `if i < msg.len() { wide(msg[i]) } else ..` in `pad_byte` (found in T4-01). That identity function also had to be written in `#[uses]`.
- With ADR-0095 A4, `x as T` is the canonical conversion in executable code. Writing `msg[i] as u64` in a specification by the same habit is E0513.

## Decision (proposed)

1. **In specifications, integer branches and results are matched by widening.** If the range of the expected type (the declared result, the other branch) contains the range of the value's type (`u8` → `u16`·`u32`·`u64`·`usize`·`i16`·`i32`·`i64`, `i32` → `i64` and so on), they are regarded as the same type. If it does not (`u64` → `u8`, `i64` ↔ `u64`), it stays E0401.
2. **A widening `x as T` in a specification is an input alias.** Its meaning is the identity, `fmt` removes it and `bmb lint` reports it (a new W code). A narrowing `as` stays E0513 (specifications have no failure obligation).
3. Executable code does not change.

## Rationale (alternatives considered)

| Option | C1 soundness | Verdict |
| --- | --- | --- |
| **Widening only (recommended)** | The range is contained, so the range fact of the result type stays true. There is no need to audit whether some place assumes the result range | It covers the measured friction (`wide`), and the surface does not grow |
| Mixing all integers in specifications (branches and results are unbounded integers too) | The result of a `spec fn -> u8` may be out of range, so an audit of every place that assumes the result range comes first | Wide compared with the evidence (the measured case) |
| `as` in specifications as the canonical form | The meaning of a narrowing `as` (truncation or failure) would have to be newly defined for specifications. The meaning diverges from the `as` of execution (a checked conversion, ADR-0095 A4) | Two normal forms (mixed arithmetic and `as`) |
| As is (identity `spec fn`) | same | The identity function and its `#[uses]` entry remain |

The deciding criterion was whether it is the narrowest rule that keeps C1 without an audit.

## Consequences

- Implementation: the widening relation in the checker's specification-mode comparison of branches and results, the specification `as` alias and the W code, the corpora (widening branches in `tests/verify/pass`, narrowing E0401·E0513 in `tests/check/err`), removal of `wide` from `lib/std/sha256.bmb` (comparing verdicts and cost per definition).
- The encoder does not change (specification integers are already `Int`).
