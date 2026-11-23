# ADR-0004: The design document is derived, not written

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principles 1·2, ADR-0005, genesis Q4

## Context
The founding statement had "humans and AI communicate through the design document". If a human or an AI writes the design document separately, the question is who is responsible for keeping it in sync with the code.

## Decision
The design document is not written. It is **derived** deterministically from the code (the only source of truth).

## Rationale
A written design document always drifts from the code. A derived one cannot drift.

## Consequences
The information that goes into the design document (intent, rationale, contracts) must be structured inside the code (ADR-0006, principle 3).
