# ADR-0082: Two types with the same fields have one identity (structural identity is kept)

- Status: Accepted (autonomous, AUTONOMY §4: a decision within the scope of ADR-0059. It does not change ADR-0059)
- Date: 2026-11-02
- Related: ADR-0059 §1 (identity, nominal types and traits), ADR-0063, ADR-0074 (`expect`), backlog T1-28, 2D-19

## Context

- ADR-0059 §1 puts only the trait `impl`s of a type into the text of that type. So two structs that implement no trait and have the same fields have the same `id` (`geo::Point`·`arrs::Point` in `tests/verify/pass/structs.bmb`, found in the 2D-19 alignment).
- Rust does not mix the two types (nominal). The question (T1-28): whether to put the inherent `impl` or the declaration location into the text to separate the identities, or to keep the same identity when the structure is the same. The tools that pair by `id` (`expect`, the check of `rename`, the pairing of `weaken`) have to handle this case.

## Decision

1. Structural identity is kept. Two types with the same fields (and trait `impl`s) have the same `id`. ADR-0059 is unchanged.
2. Tools that read `id` treat the identity as the value of a path and do not look up a definition by identity. That is how it is now:
   - `expect DEF ID` compares with the current identity of the path DEF. A change to another path with the same identity has no effect.
   - The check of `rename` compares identities as a multiset (a sorted list). Two of the same identity count as two.
   - `weaken` pairs by path first, and two definitions paired by identity have the same text, so the verdict is the same whichever one is paired.
   - `query duplicates` shows the groups of definitions with the same identity (information).
3. A test guards this behaviour: `equal_types_share_an_identity_and_the_tools_tell_them_apart` in `tests/workspace.rs` (the same identity, renaming one, and after changing one the `expect` of the other passes while the stale `expect` of the changed path is rejected).

## Rationale

- **Identity is meaning (ADR-0059 §1):** in BMB the values of two types with the same fields and no trait can do the same things (field reads and writes, construction, the datatype in the proof, the layout in code generation). The checker rejects a program that mixes the two types (nominal checking), so even with the same identity the meanings do not mix within one program. Inherent `impl` functions are definitions with their own paths and identities, so they are told apart without being put into the text of the type.
- **If the declaration location were included**, merely moving the same type to another unit would change the identity, which goes against the purpose of ADR-0059 (names and locations are metadata). If the inherent `impl` were included, an edit that adds one function would change the identity of the type and of every definition that uses it (the same problem as the contract firewall, the context of ADR-0063).
- **Rename capture:** a capture in which a name comes to refer to the other of two equal types is not seen by the multiset comparison. But that capture does not change the meaning (the operations on the values are the same, and if an inherent `impl` function is called, the identity of that function differs and the multiset changes).

## Consequences

- No code change. One test.
- Reverting: change ADR-0059 §1 to put the function identities of the inherent `impl` or the declaration path into the type text (needs approval).
