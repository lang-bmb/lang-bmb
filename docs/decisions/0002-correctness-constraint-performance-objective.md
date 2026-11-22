# ADR-0002: Correctness is the constraint, performance is the objective function

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §2, ADR-0001, genesis Q2·§3.4

## Context
v0 started from "Performance > Everything", and verification stayed opt-in to the end. Measured, verification of real code was 0/12.

## Decision
Correctness is a **constraint**: code that does not satisfy it is not built. Performance is the **objective function**, maximized inside that constraint.

## Rationale
If the two are priorities at the same level, performance pressure pushes verification out. That is what happened in v0. Split into constraint and objective function, a performance improvement cannot cut into correctness.

## Consequences
- The proof gate is on by default and cannot be turned off (ADR-0003).
- The performance advantage claimed by v0 is treated as void and is measured again. An advantage is justified only when a proof removes a check (DESIGN.md principle 9).
