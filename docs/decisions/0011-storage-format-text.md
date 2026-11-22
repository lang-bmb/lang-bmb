# ADR-0011: The storage format is text, the tool decides the layout

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principles 1·7, genesis R1

## Context
If the agent accesses code only through tools, is there a reason for code to be text files (alternatives such as a structured DB)?

## Decision
The issue is closed. The storage format itself does not matter. **Code stays as text, and the tool decides the file layout deterministically.**

## Rationale
The agent accesses code only through tools, so storage hides behind the tool. Text can use existing infrastructure such as git and diff as is.

## Consequences
- A file path has no meaning. Module identity is the declared unit (v0's "file path = module" is dropped).
- The same code must produce the same file layout (normal form output).
