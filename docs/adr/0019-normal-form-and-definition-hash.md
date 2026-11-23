# ADR-0019: Normal form and definition hash

- Status: Accepted (autonomous, AUTONOMY.md §4). "The contract hash is the key of the incremental build" in §2 is superseded (ADR-0043: the key is the caller hash without `#[uses]`, the contract hash itself is unchanged). "Decimal" in the literal row of §1 is superseded (ADR-0048 §3: the normal form keeps the radix that was written, backlog 2N-02). The name dependence of the definition hash in §2 (writing references as paths) is superseded (ADR-0059: identity is the hash of the nameless normal form, names are an index)
- Date: 2026-10-24
- Related: DESIGN.md principles 1·2·8, ADR-0008·0011, backlog 1D-01

## Context
The maps (L0–L2) and the proof cache are keyed by per-definition hashes (DESIGN.md §6). The tool decides the file layout deterministically (ADR-0011). Both can be defined only if there is a normal form where "the same program is the same text".

## Decision

### 1. Normal form
The text that `bmb fmt` emits is the normal form. The normal form is determined only by the syntax tree and is independent of the whitespace, line breaks, comments and item order of the original text.

| Element | Rule |
| --- | --- |
| Indentation | 4 spaces. No line length limit (lines are not wrapped) |
| Units | `mod name {` on the line after `#[summary("..")]`. Sub-units first (by name), then definitions (by name). One blank line between items |
| Attributes | In the order `#[effects(..)]`, `#[uses(..)]`. The `uses` list is sorted and deduplicated |
| Signature | `fn name(a: T, b: U) -> R`. Without contracts, `{` on the same line |
| Contracts | `requires`·`ensures`·`decreases` each on its own line indented 4 spaces, items joined by commas with a trailing comma. `{` on the next line |
| Loops | From the line after `while cond`, `invariant ..,` `decreases ..,` then `{` on the next line |
| Blocks | `{}` if empty. Otherwise one line per statement, with the tail expression last. No `;` after a block-shaped statement (`if`, block) |
| Expressions | Only the parentheses that precedence requires. But `if`, a block or `forall` as the operand of an operator or a postfix operation is always parenthesized. One space on each side of a binary operator |
| Literals | Decimal, no suffix |
| Comments | Not in the normal form. A `//` comment is not part of the program (ADR-0016). Rationale is handled by the structured rationale syntax (ADR-0006) |

Properties of the normal form (guaranteed by tests): formatting twice is the same as formatting once. Parsing the normal form again gives the same check and proof results.

### 2. Definition hash
- The hash function is SHA-256. The notation is `sha256:` + 64 hexadecimal characters. In maps it may be shortened to the first 12 characters.
- The hash input is the **qualified normal form**. It is the same as the normal form, except that every name reference is written as a full path (`f(x)` → `unit::f(x)`, `#[uses(p)]` → `#[uses(unit::p)]`). So the same text has a different hash if it refers to something different in another unit.
- Each definition has two hashes.
  - **Body hash (`body`)**: attributes, signature, contracts and the whole body. The definition's own name is replaced by `_`. So the same definition under a different name or unit gives the same hash, and duplicates can be found (DESIGN.md §7.2). Self-recursive calls, however, stay as full paths.
  - **Contract hash (`iface`)**: attributes, signature and contracts only. It is the key of the contract-firewall incremental build (DESIGN.md §8.1). For a `spec fn` the contract hash equals the body hash, because a caller can unfold the body with `#[uses]`.
- The proof cache key is decided in stage 2. The starting point is the combination of the body hash, the contract hashes of the definitions it calls, and the body hashes of the `spec fn`s it unfolds.

### 3. File layout
- One top-level unit is one file: `src/<unit name>.bmb`. Sub-units are nested inside the same file.
- The content of a file is the normal form of that unit. So the same program gives the same set of files.
- File names and locations have no meaning. Read through any file layout, it is the same program (principle 7).

## Rationale
- Sorted item order and the rule of not wrapping lines make the normal form simple and deterministic. A line length that is easy for humans to read is a non-goal (DESIGN.md §3).
- Leaving the definition's own name out of the body hash makes it possible to find the same definition after a move or a rename.
- SHA-256 is implemented with the standard library only, without external dependencies, and tested with verification vectors.

## Consequences
- `bmb fmt` deletes comments. Comments written by hand by a human are not preserved.
- Item order loses its meaning. No feature that depends on the order of definitions in the source is added to the language.
