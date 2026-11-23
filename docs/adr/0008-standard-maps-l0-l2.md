# ADR-0008: Standard maps L0–L2 are fixed, additions are backward compatible

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principle 2, §7, genesis Q7

## Context
How much of the map goes into the specification? Third-party tools need a fixed contract to depend on, but fixing too much prevents evolution.

## Decision
- The standard navigation maps **L0 (table of contents), L1 (surface) and L2 (relations)** are fixed in the specification.
- Later levels may be added only in a backward-compatible way.
- Every other view is an extension map.
- A map does not contain code. It points to code by location and hash.

## Rationale
Third-party tools need a contract to depend on. A map holds only what navigation needs and stays small.

## Consequences
The nondeterministic elements that v0's `bmb index` had (recorded timestamps, unsorted lists) are not allowed. The same code must produce a byte-identical map.
