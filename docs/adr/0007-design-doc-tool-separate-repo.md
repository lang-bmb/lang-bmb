# ADR-0007: The design document tool lives in a separate repo, Markdown first

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §1·§6, ADR-0005, genesis Q6

## Context
Where does the design document tool outside the language (ADR-0005) live, and in what form does it produce output?

## Decision
It lives in a separate repo. **Markdown is the first-priority** output.

## Rationale
Markdown can be read without a tool and diffs in git. With a separate repo, the language repo only has to keep the contract that is the map.

## Consequences
The design document tool is outside the scope of the first milestone (ADR-0016).
