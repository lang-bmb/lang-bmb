# ADR-0010: Extension map generators are pure BMB functions

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §6, ADR-0008, genesis Q9

## Context
Anyone, third parties included, can make an extension map (ADR-0008). How is their determinism guaranteed?

## Decision
An extension map generator is written as a **pure BMB function**.

## Rationale
Determinism becomes a property the language proves, not a convention (the effect system checks `pure`).

## Consequences
- Using extension map generators requires the language to be mature enough to handle map data. In the first milestone the compiler builds only the standard maps, directly.
- The `pure` judgement of the effect system is a premise of this ADR.
