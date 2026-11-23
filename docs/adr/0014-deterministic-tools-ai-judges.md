# ADR-0014: Deterministic tools fill the gap, the AI only judges

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md principle 6, §8.2, genesis Q12

## Context
A strict language increases the labour of writing. The AI still makes mistakes and has a context limit. Who fills this gap?

## Decision
Deterministic tools fill it.
- They make navigation easy: map queries, slices.
- They assist writing: the tool fills in notation that is mechanically determined.
- They point out mistakes: counterexamples, vacuous contracts, weak specifications, change impact.

The AI takes only judgement (intent, design, choice of specification).

## Rationale
Explicitness is a property of the artifact, and its labour belongs to the tool. Giving the AI work that can be computed deterministically only adds mistakes and tokens.

## Consequences
- No nondeterministic judgement such as an LLM goes inside the tool chain.
- Suggestions that use SMT run under a fixed resource limit (Z3 `rlimit`), not a time limit.
