# ADR-0120: Candidate tool surfaces are cleaned up following the hypothesis verdicts

- Status: Accepted (approved by the maintainer 2026-11-11 "approved, autonomous execution/verification approved". Backlog 4D-11, a supplement to ADR-0113·0116)
- Date: 2026-11-11
- Related: ADR-0073 (three edit surfaces), ADR-0109 (batch probe), ADR-0111 (tuning hypotheses), ADR-0113 (fixing of ported components), ADR-0116 (shrink survey before the freeze), ADR-0100 (the subject of the freeze is the accepted language), `dev-docs/measurements/surface-cut.md`

## Context

- The result of the shrink survey before the freeze (4D-11): there is nothing to reduce in the language surface. All accepted features are verified, and the only features with dedicated code are closures and constant items.
- On the tool side there are three surfaces that came in as candidates of tuning hypotheses: JSON syntax tree editing (`bmb ast`, MCP `read_ast`·`edit_ast`: `ast_json` 2,397 lines), file diff editing (MCP `edit_file`: `udiff` 252 lines), the batch probe (`bmb probe`, MCP `probe`: 339 lines). The hypotheses are H-EDIT and H-PROBE.
- Candidate surfaces were put in so that measurement picks one. If they stay after the pick, what is to be ported (about 3,000 lines) and the surface to maintain grow.
- The tools are not subject to the freeze (ADR-0100 decision 1). But once a tool is ported and fixed, both sides have to be changed together (ADR-0113).

## Decision

1. **The language surface is frozen as it is now.** This is the conclusion of the shrink survey.
2. **The candidate surfaces of tuning hypotheses are cleaned up following the verdict.** When a hypothesis is judged, the tools of the surfaces not picked and their code are deleted. The verdict on H-EDIT (T1-41) keeps one of the three edit surfaces, and the verdict on H-PROBE (T0-37) keeps or deletes the batch probe.
3. **The deadline is before the port of the tools starts.** The port of the tools is from P5 on and after gate B (ADR-0111). Candidate surfaces with no verdict by then are taken out of the porting scope and retired with stage0 (4G). If they are needed again after retirement, they go into the BMB compiler by a new decision.
4. **Tools not tied to a hypothesis are not in scope.** `bmb weaken` (contract adjustment, the audit of H-GAME, the commutation relation), `lint`, `uses`, `fix`, the map and queries, and the workspace are product features.
5. **An ADR that adds a new candidate surface also states the condition for deleting that surface.** This is added to the rule for registering a tuning hypothesis (ADR-0111 decision 5).

## Rationale

| Option | Verdict |
| --- | --- |
| **Clean up following the verdict, deadline before the tool port (accepted)** | The registered measurements are not thrown away, and the surfaces not picked do not create porting and fixing cost |
| Delete now | The sonnet data are unfavourable to the JSON syntax tree surface (`brb-ast` 22/24 and 6.91 million tokens versus `brb-diff` 24/24 and 2.72 million tokens), but the qwen runs remain. Deleting before the verdict breaks the pre-registration |
| Port everything | Two of the three candidates are surfaces that will not be picked. Porting them writes twice the code that will be deleted |
| Take out language features such as closures | The only reason to take them out is the porting volume. Workload is not a criterion (JUDGMENT §5) |

## Consequences

- In the estimate of the tool components (C10·C12) of the port map, the 2,988 lines of candidate surfaces are left as "settled after the verdict".
- "Derive the removal task for the surfaces not picked" is added to the completion criteria of `T1-41` and `T0-37`.
- Among the preconditions of the freeze declaration (4D-01), the shrink survey (ADR-0116 decision 1) closes.
