# ADR-0005: The language is responsible for code and maps, no further

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §1, ADR-0004, ADR-0007, genesis Q4'

## Context
If the design document is derived (ADR-0004), must the language also be responsible for design documents and diagrams that are easy for humans to read?

## Decision
The language's responsibility ends at **code and maps**. Design documents and diagrams for humans belong to a tool outside the language that consumes the maps. That tool is also built by the project.

## Rationale
A map is for agent navigation and is a different thing from a design document for humans. Bundling the two into one artifact makes the map larger and conflicts with the principle of keeping maps small (ADR-0008).

## Consequences
The map must be a stable contract that third parties (including the design document tool) can depend on (ADR-0008).
