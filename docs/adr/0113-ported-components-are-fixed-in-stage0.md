# ADR-0113: Ported components are fixed in stage0

- Status: Accepted (approved by the maintainer 2026-11-11 "all approved, recommendations accepted": overall review R10, backlog 4D-09). It supplements ADR-0100
- Date: 2026-11-11
- Related: ADR-0037 (bootstrap, two compilers are not grown together), ADR-0100 (stage0 feature freeze: the subject is the accepted language), ADR-0105 (tool changes are cheaper before their port), HEALTH.md S4, the port map (`selfhost-port-map.md`)

## Context

- The subject of the freeze in ADR-0100 is the accepted language. The tools (map, queries, workspace, diagnostics, MCP) are not subject to the freeze.
- stage0 keeps growing: S4 (line count excluding tests) went from 42,712 to 53,459 (`progress.md`). After the freeze, changes on the tool side continue (responses to measurements such as ADR-0108~0110).
- If the stage0 side changes after a component has been moved to BMB, the port goes stale. The differential tests catch it, but once caught, both sides have to be fixed. That is the lesson of v0.

## Decision

1. **Once a component passes the differential tests, that stage0 component is fixed.** For each component of the port map (C0~C13), this applies from the commit at which the BMB port passes the differential tests of that component in the gate.
2. **A change to a fixed component changes both sides in one commit.** A commit that changes the files of that component in stage0 must carry the corresponding change of the port and pass the differential tests. Defect fixes are the same.
3. **Components not yet moved are free.** So it is cheaper to finish tool changes before the port of their components (P5·P9~P11) (ADR-0105), and if the failure attribution of gate B requires tool fixes, they are done before P5 starts (ADR-0111).
4. **An `accept` of the S4 budget states a per-component reason.** When the +2% budget that applies from 4D (HEALTH S4) is exceeded, the reason states which component grew and why, and the port status of that component.
5. **The gate does the detection.** The file list per component and whether it is fixed are kept in one file in `dev-docs/health/`, and the gate checks, in a commit that changes the stage0 files of a fixed component, the change on the port side and the pass of the differential tests. The implementation comes before the first component passes (within 4D-02) (backlog 4D-10).

## Rationale

| Option | Verdict |
| --- | --- |
| **Fix components in turn as they are moved (accepted)** | During a migration, the original is closed so that the moved side and the original do not diverge (established migration method). The room for tool changes that measurements require stays in the components not yet ported |
| Fix all of stage0 at the freeze | It blocks the tool fixes (the 2D-17 line) that the evidence stage and gate B will require |
| As now (only the language is frozen, S4 budget) | The budget can be exceeded with `accept`, and it is not tied to which components have been ported |

## Consequences

- The order of the port becomes the order of deadlines for tool changes: the query scope and cap (2D-17) before the map and queries (P5), the decision on the defaults of undecided information and the probe (ADR-0108·0109) before diagnostics and MCP (P9~P11).
- A stage0 defect fix after fixing costs double. That cost continues until 4G (stage0 retirement).
