# ADR-0003: Compute and minimize the area that needs tests

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principle 4, genesis Q3

## Context
The founding statement's "almost no test code is written" had to be turned into a principle. "Zero tests" is literally false because of external boundaries such as IO.

## Decision
The principle is written as "proof replaces tests". The area that needs tests is **computed and minimized by the compiler**. "Zero tests" stays only as a direction.

## Rationale
External boundaries (IO, time, randomness and so on) are outside the scope of proof. The wording chosen can be measured and does not overstate.

## Consequences
- An effect system is needed. Effect declarations are the basis for computing the area that needs tests.
- The unproved area and the area that needs tests are aggregated in the map and become measurement axes (DESIGN.md §9 R4).
