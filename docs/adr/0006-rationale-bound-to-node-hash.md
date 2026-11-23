# ADR-0006: Rationale lives in the code, bound to a node hash

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principle 3, §4.1 (comments), genesis Q5

## Context
Where should "why was it done this way" (the rationale) live? An external ADR or a free comment stays as it is when the code changes.

## Decision
Code-level rationale lives inside the code as a **structured declaration bound to a node hash**. When the bound node changes, its rationale is marked "unconfirmed".

## Rationale
Kept as an external document it drifts, and nobody can tell that it has drifted. Bound to a hash, the drift is detected mechanically.

## Consequences
- Free-text comments are replaced in the language by structured rationale (DESIGN.md §4.1).
- The rationale syntax is one of the few new constructs with no counterpart in Rust.
- This ADR directory is the decision record of the language *design*. It is at a different level from the rationale of BMB code.
