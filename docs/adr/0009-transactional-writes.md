# ADR-0009: Writes are transactions, proof is a build gate

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §6, genesis Q8

## Context
If the agent writes code only through tools, writes need a unit and a validity criterion. Requiring proof on every edit makes large refactorings impossible.

## Decision
- The unit of writing is a **transaction commit**.
- At commit time, **structural and type validity is mandatory**.
- **Proof is enforced at the build gate**. An unfinished proof is allowed during editing, and that state is shown in the map.

## Rationale
During a large refactoring, proofs necessarily break for a while. Structure and types are cheap to check, so they are required on every commit.

## Consequences
The code store can hold a state where "the types are right but the proof is not done". No binary comes out of such a state.
