# ADR-0013: The surface is inherited, progress comes from giving up conveniences

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §4, genesis Q11

## Context
Why inherit the Rust/Verus surface? And what is the basis for saying BMB is "more advanced" than they are?

## Decision
- The main purpose of the succession of the surface is **LLM inferability**. It lets a model write a language it was not trained on with the knowledge it already has.
- The source of progress is **giving up what Rust and Verus kept for human convenience** and collecting the payback.
- Shape rule: "make the optional mandatory" rather than new syntax. New syntax only where Rust has no counterpart (effects, rationale, proof context).

## Rationale
In languages with little training data, LLM performance is 24–39 percentage points lower (genesis §4). Inheriting the surface reduces this loss. An independent syntax was something to be dropped in v0.

## Consequences
- Valid BMB code must look like "more verbose Rust".
- The Rust-habit error rate is measured, and a tightening whose inference loss is larger than its payback is reverted (DESIGN.md §4.1).
