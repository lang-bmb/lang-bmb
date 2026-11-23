# ADR-0104: The large-scale task axis (comprehension, refactoring, task completion) and its measurement ground: the port twin

- Status: Accepted (approved by the maintainer 2026-11-09 "Ok, proposals and recommendations accepted / as the code gets larger, code comprehension, refactoring and so on, task completion - hard to measure but an important axis", decision D3 of the overall evaluation)
- Date: 2026-11-09
- Related: DESIGN.md §3 principles 6·7, ADR-0037 (bootstrap, differential tests of the port), ADR-0102 (G5-L), ADR-0103 (evidence stage), `dev-docs/measurements/selfhost-port-map.md` (P0~P11), backlog 3A-05·3A-06·3A-08, 2H-02

## Context

- Where BMB has to win is large code. The map, slices and impact queries (principles 6·7), definition-unit identity and incremental re-proof should be worth more the larger the code is. But all model measurements so far are at the scale of tens of definitions, so that utility cannot show (utility §4.2).
- Large-scale tasks are hard to measure. The correct answer (the answer to this question, whether this refactoring preserves behaviour, whether this feature went in correctly) has to be produced deterministically, there has to be a fair Rust comparison group with the same function, and the tasks have to carry less of the author's bias.
- What exists now: BNB (`bench/bnb`). Generator workspaces (1k~10k definitions), Rust twins and 27 navigation and modification tasks are frozen, and only the oracle arm has run. It is generated code, so it lacks the structure of real code (deep data structures, state, error paths).
- The stage 4 port (ADR-0037) moves stage0 (Rust, about 50,000 lines, S4) to BMB. For each component, differential tests (P0~P11) check that it emits the same bytes as stage0. That is, as the port advances, **a real Rust codebase and a BMB codebase with the same function come into being as a pair, and a behaviour judge common to both (the differential corpus) already exists.**

## Decision

1. **Three large-scale task axes are registered as hypotheses (registered before measurement).** The comparison group is the Rust codebase with the same function and the standard file tools (read, search, edit, `cargo test`).
   - **H-COMPREHEND (comprehension):** for questions about a large codebase (where this function is called, the impact of this change, the path on which this diagnostic code is emitted, where this invariant is maintained), the rate of correct answers is the same or higher, and the tokens read are at most 1/3. The correct answers are produced by deterministic tools (the map, `impact`, the call graph) and a sample checked by a person.
   - **H-REFACTOR (refactoring):** for behaviour-preserving refactorings (rename, signature change with call-site update, function extraction and inlining, unit split, data structure replacement), the success rate is the same or higher, and the total tokens are the same or fewer. Success is when all three hold: the output bytes on the differential corpus are the same, the proof gate (BMB) or the tests (Rust) pass, and nothing outside the requested scope changes (definition identity hashes for BMB, diff range for Rust).
   - **H-TASK (task completion):** for feature-addition and defect-repair tasks drawn from real history, the success rate (hidden judgement) and G5-L (ADR-0102) are the same as or better than Rust+tests. The tasks are made by **commit replay**: one commit of the stage0 history is reverted on both sides of the twin, only the problem statement of that commit (the commit message and the backlog item) is given, and the corpus tests that the commit added are used as the hidden judgement.
   - The existing hypotheses H-SCALE·H-SLICE·H-SPLIT·H-AMORT·H-EFFECT are also measured on the same ground.
2. **The main measurement ground is the port twin.** A component of stage0 and its BMB port are twins. The judge is the differential test of the port as it is. This is work the port already has to do, so there is no cost of building a separate codebase for measurement, and it is real code, so it does not have the structural bias of a synthetic benchmark.
3. **The progress of the port gives the size axis.** The first measurement is when the ports of P0~P3 (base library, lexer, parser, checker) pass the differential tests, the next is P0~P7, and the last is the whole. The same task templates are rebuilt at the three sizes to see the slope of H-SCALE.
4. **Judging power and fairness are fixed first.** At least 20 tasks per axis, 3 qwen repeats and 1 sonnet run (ADR-0103 decisions 2·3). The Rust arm has the same problem statement, the same model, the same call limit, and the standard agent tools. Tasks, problem statements and judgement are frozen before the first run (ADR-0021). The bias that both sides of the twin share the same authoring lineage (a port of stage0) is written in the report.
5. **BNB stays as an auxiliary.** It is the first model evidence before the port (ADR-0103 decision 1.5), and a control whose size can be changed freely (1k·3k·10k). The stage 3 plan for a synthetic large codebase (3A) is rescoped once the port twin measurement stands (2H-02).
6. **Stage 3 and stage 4 merge on this ground.** The stage 3 pass criteria (navigation and proof cost at the scale of thousands of definitions, definition-unit merge of edits by several agents) are judged by the port twin measurement. The priority of the remaining stage 3 phases is set by 2H-02 from the results of 2H-01 (ADR-0103).

## Rationale

| Option | Verdict |
| --- | --- |
| **The port twin (accepted)** | Real code, a Rust comparison group with the same function, and a common behaviour judge come as by-products of the port. Commit replay gives a real task distribution |
| Synthetic large codebase only (extend BNB) | Size is easy to control, but the structure of the generator is the result. Refactoring and tasks have no realism |
| Port a public Rust project to BMB as the twin | The realism is the same, but the porting cost is paid for measurement only. The judge also has to be built new |
| Do not measure the large-scale axis | The claims of principles 6·7 and E2 (the navigation bottleneck) stay without evidence |

Difficulties and responses: for producing correct answers (comprehension tasks), the results of the deterministic tools are the candidate answers and a person checks a sample. The behaviour-preservation judgement (refactoring) is done by the differential corpus. Task realism (tasks) is handled by commit replay. Out-of-scope changes are judged exactly by identity hashes in BMB, and approximated by the diff range in Rust.

## Consequences

- H-COMPREHEND·H-REFACTOR·H-TASK in the hypothesis registry (ROADMAP). "Comprehension, refactoring and tasks in large code" in the utility axes of utility report §1.
- Backlog 3P (port twin measurement): a multi-file Rust arm harness, a comprehension task generator, refactoring tasks and a scope judge, a commit replay generator, the first measurement per size. The first measurement is after the port passes P3.
- The component order of the port map (P0~P11) does not change. Measurement does not set the order of the port.
