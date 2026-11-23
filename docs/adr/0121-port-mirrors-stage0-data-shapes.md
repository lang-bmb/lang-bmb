# ADR-0121: The port follows the data shapes of stage0

- Status: Accepted (approved by the maintainer 2026-11-11 "all accepted"). It replaces the choice of the "arena access" alternative in port map §5.2
- Date: 2026-11-11
- Related: ADR-0037 (bootstrap), ADR-0047 (recursive types, `Box`), ADR-0113 (fixing of ported components), `dev-docs/measurements/selfhost-port-map.md` §5.2·§7.4, `dev-docs/experiments/port-shape/results.md`, backlog 4D-02

## Context

- The port map (4A-02) assumed an arena (`Vec<Node>` and indices) for the syntax tree of the port, and named its quantifier invariants as the largest risk of the self-proof (§5.2, §7.4). At that time the language had no `Box` and no recursive types.
- Now it has them (ADR-0047): the syntax tree of stage0 (`compiler/src/ast.rs`: `enum` with `Box`, `Vec`) can be written in the same shape.
- The two shapes were measured on the same small parser (`port-shape/results.md`). Both were fully proved with no escape hatch, and the output is the same. The proof cost (`rlimit`) is 2.6 times for the arena, the most expensive obligation is 10 times, and the spread of the cost over seeds is 3.09 versus 1.01. Line count and run time are at the same level.

## Decision

1. **The data structures of the port are written in the same shape as those of stage0.** Where stage0 is an owned tree (`enum`, `Box`, `Vec`) it is an owned tree, and where stage0 is tables and indices it is tables and indices. In each component, one type of the port corresponds to one type of stage0.
2. **The invariant of index structures is "the one that points comes later".** Where stage0 uses indices (tables, interning, the sorted arrays of the map), the single monotonicity that an element points only to elements before itself is written as the type invariant. It is not wrapped in checking accessor functions or escape hatches.
3. **The shape changes when measurement requires it.** If, in the port ledger (ADR-0122), the proof cost or the run time of a component exceeds the criterion of a gate, the shape change of that component is made on both stage0 and the port together (if before fixing, stage0 is changed first, ADR-0113 decision 3).
4. **Depth is bounded by fuel.** Recursion that walks the tree takes the nesting limit of stage0 (256) as fuel. The measure is one integer (it combines depth and rank).

## Rationale

| Option | Verdict |
| --- | --- |
| **Follow the shape of stage0 (proposed)** | The standard practice of porting is to move, not to redesign (JUDGMENT S1). The differential tests require the same output at component boundaries, and a fix after fixing changes both sides in one commit (ADR-0113), so the shapes must be the same for the fix to go to the same place. The measured proof cost is also lower |
| Redesign as an arena | It was possible in the experiment, but at 2.6 times the proof cost, and the capacity argument is carried in the contract of every function. The shape differs from stage0, so the correspondence of fixes breaks. The run time gain (16%) is small against the criterion of the gate (within 2 times stage0) |
| Decide per component at the time | The choice for the first component (the parser) binds all later components (the checker and the prover read that tree). It has to be decided in advance |

The limits of the experiment (one task, 340 lines, one session per arm) are in the results document. That is why decision 3 exists: if it comes out differently at large size, the ledger shows it.

## Consequences

- The "arena index validity" risk of port map §5.2 no longer applies to the syntax tree. The remaining index structures are tables, interning and the map. If accepted, correction notes are added to §5.2·§5.4·§7.4.
- Two frictions that the experiment revealed remain as work to resolve before the port: the length obligation of the output buffer (backlog `4D-13`), and the failure mode in which the solver dies of memory on a false obligation (backlog `T2-16`).
- For the memory of long-running paths such as the MCP server (port map §5.1), an owned tree gives per-request release as the release at the end of scope.
