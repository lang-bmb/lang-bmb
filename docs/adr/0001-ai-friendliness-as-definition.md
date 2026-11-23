# ADR-0001: AI-friendliness is a definition, not a goal

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §2, genesis Q1

## Context
The founding statement had both "performance and stability first" and "AI-friendliness first". With two top priorities there is no criterion for choosing when they conflict.

## Decision
AI-friendliness is removed from the list of goals and raised to a definition: **user = agent**. Only correctness (the constraint) and performance (the objective function) remain as goals.

## Rationale
Two top priorities are the same as no priority. Once the user is defined, AI-friendliness does not compete with the other goals. It becomes the premise of every design judgement.

## Consequences
Ease of learning for humans and convenience of hand-coding become non-goals (DESIGN.md §3). What favours the agent can be adopted even if it is inconvenient for humans.
