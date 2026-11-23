# ADR-0097: Inherent `impl` blocks with different bounds

- Status: Accepted (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after them ADR-0100 approved"). Backlog 2T-11
- Date: 2026-11-09
- Related: ADR-0026 (data type surface, one `impl` block per type·E0308), ADR-0046 (generics and traits, proved once per definition with sort parameters), ADR-0015 (second-class references), ADR-0092 (standard library), finding 3 of `dev-docs/health/4B.md`, port map P0

## Context

- ADR-0026 has one inherent `impl` block per type (E0308 "one per type"). There is no `where` clause (E0240: bounds go next to the type parameter).
- So a bound that only one method needs becomes a bound of the whole type. `get` of `std::collections::BTreeMap` cannot return a reference (ADR-0015) and returns a copy of the value, so it needs `V: Copy`, and that bound goes onto the struct `BTreeMap<K: Ord, V: Copy>` and the single block, so that even `insert`·`remove`·`len`·`into_values` cannot take values that move (4B-04).
- **It gets in the way of the port.** Of the 313 `BTreeMap<..>` type annotations in stage0 (`compiler/src/*.rs`), 264 (84%) have a value that is not `Copy` (`String`·`Vec`·structs) (2026-11-09, the number of annotations counting integers, `bool`, `char` and references as `Copy`). Port P0 moves these maps to `std::collections`.
- Rust allows several inherent `impl` blocks on one type, with separate bounds per block (Rust Reference, "Inherent implementations"). Verus is the same.

## Decision (proposed)

1. **Several inherent `impl` blocks are allowed on one type. Each block must have a different bound list.** Two blocks with the same bound list are E0308 (one normal form: the block a method goes in is determined by the bounds that method requires).
2. **The bounds of a block include the bounds of the type declaration.** The parameter names and order are those of the type declaration, as now (ADR-0026).
3. **Method names are unique across the blocks.** The path is `unit::T::f` as now (the block does not appear in the path). An overlap is E0302.
4. **Proof and code generation do not change.** The type parameters of a method are those of its block, and it is proved once per definition over sort parameters (ADR-0046). A method call `x.f()` finds the method by name, and if the instance does not satisfy the bounds of that block it is the current bound error (E0428).
5. The identity hash is per definition, so it contains the bounds of the method (those of the block). Moving a method to another block changes the meaning, so the hash changes too.

## Rationale (alternatives considered)

| Option | C1 soundness | Verdict |
| --- | --- | --- |
| **Several blocks, one per bound list (recommended)** | unchanged (a definition with bounds fixed per method) | It is the standard Rust shape, so there are no habit errors. There remains one place where bounds go (next to the type parameter) |
| `where V: Copy` on the method | same | Rust accepts it too, but a second place to write bounds arises (it opens E0240). Two normal forms |
| As is (a free function `fn get<K: Ord, V: Copy>(m: &BTreeMap<K, V>, ..)`) | same | Method syntax is lost (ADR-0051), and for maps of values that move the workaround of using positions as values continues |
| `get` returns `&V` | — | It goes against second-class references (ADR-0015). It is not in the scope of this ADR |

The deciding criterion is S1 (standard practice): the same shape as Rust, one place for bounds.

## Consequences

- `std::collections::BTreeMap<K: Ord, V>`: `get`·`value_at` are in the `impl<K: Ord, V: Copy>` block, the rest in the `impl<K: Ord, V>` block. Existing users compile as before.
- This ADR does not create a way to **read** values that move (no reference returns, ADR-0015). That case keeps positions (indices into a `Vec`) as values and borrows with `&v[i]`, as now. If closures (ADR-0053, 2V-18) are adopted, a `with(k, f)` shape is a candidate (open question).
- Implementation: the checker (the E0308 condition, method lookup), the `impl` notation of map L1, `fmt` (block order as written), the corpora `tests/check/err`·`tests/verify/pass`, two blocks in the fuzzer `soundness_traits`.

## Open questions

1. Reading values that move: after measuring in how many places port P0 uses the position workaround, decide whether a closure-based `with` is needed.
