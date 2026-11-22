# ADR-0015: Ownership and moves + second-class references

- Status: Accepted
- Date: 2026-10-24
- Related: DESIGN.md §5, ADR-0012, genesis Q13

## Context
Implementing a Rust-level borrow checker (lifetime inference included) independently is expensive. On the verifier side, aliasing complicates frame conditions.

## Decision
The memory model is **ownership and moves + second-class references**. A reference (`&`, `&mut`) can only be passed as a function argument. It cannot be stored or returned. The surface is Rust's `&`·`&mut` unchanged.

## Rationale
- Lifetime annotations and lifetime inference disappear from the language. The cost of an independent borrow checker goes down.
- Aliasing is restricted, so the verifier's frame conditions become simple.
- noalias can be attached to every `&mut`.

## Consequences
Structures that hold references (iterators, views and so on) must be replaced by handles or indices. This cost is measured by the Rust-habit error rate.
