# ADR-0145: Development practices are also targets of payback

- Status: Accepted (approved by the maintainer 2026-11-22 "recommendation accepted": decisions 1~4, the experimental build (`T1-53`) goes after gate B). The maintainer's direction (2026-11-22: "Beyond the level of language, syntax, compiler and tools, doubt development practices and patterns too and take payback from them. Explore fully the possibility of opening a new development paradigm")
- Date: 2026-11-22
- Related: DESIGN.md §4.1·§8·§9, ADR-0041 (trade-off ledger), ADR-0059 (identity hash), ADR-0060 (type invariants), ADR-0065 (commutation relation), ADR-0102 (lifetime unit), ADR-0142 (the model is how the agent works), `dev-docs/reviews/practices-2026-11-22.md`, `dev-docs/reviews/collaboration-2026-10-28.md`, backlog `4E-03`

## Context

- DESIGN §4.1 and §8 give up what Rust·Verus kept for human convenience and take the payback in the compiler and tools. The list of paybacks ends there.
- Branches, PRs and merge queues, CI that runs on every change, version numbers, deployment that restarts, canaries and rollbacks are practices that make up for properties the language cannot give: the independence of two edits, the impact range of a change, whether a new version keeps the old promises, whether new code fits the state that was running.
- BMB already has several of those properties: read sets (incremental proof key), the commutation verdict (ADR-0065, 0 soundness violations), the direction of a contract change (`bmb weaken`, 483/483), the contract firewall, identity determined by content, no global mutable state, invariants attached to types. Payback on the practice side is in neither the design nor the hypothesis registry (the collaboration discussion went as far as H-COMMUTE·H-SPLIT).
- There is also unfavourable evidence. `3P-05a`: on questions that read the text of definitions and on refactoring, the BMB arm spends more than the Rust arm. Payback from practices does not reduce the cost of one edit.

## Decision

1. **Development practices are made targets of payback.** A "practice level" table is added to DESIGN §8: the practice, the lack it makes up for, the property of BMB that removes that lack, the form of the payback, the state of the evidence. A row without evidence is written as "not measured" and is not claimed. Open research R5 "the basis of collaboration and deployment" is added to DESIGN §9.
2. **Three hypotheses are registered.** H-TRUNK (committing concurrently at the unit of definitions to a single trunk has a smaller verification cost of integration than splitting into branches and merging, and 0 integration failures), H-LIVE (a definition replacement proven by refinement, applied without a restart, keeps the observed behaviour within the old contract, and the majority of real edits are such replacements), H-RACE (choosing by measurement among the proven ones of competing implementations of the same contract gives better performance and equal or fewer total tokens per lifetime unit). The refutation conditions are in §8 of the review document.
3. **The verdict and the upper bound are measured first. No machinery is built.** A server that accepts concurrent commits, a runtime that swaps the definitions of a running program, and a coordinator of competing implementations are decided separately after deterministic experiments (the shape of the edits of the real history, a replay of the throughput of the trunk) have shown the upper bound.
4. **Order.** This exploration does not go ahead of gate B and the porting from P5 on. The deterministic experiments run alongside them.

## Rationale

| Option | Verdict |
| --- | --- |
| **Put practices in the ledger and measure them as hypotheses (recommended)** | The properties that are the basis of the payback already exist and some have been measured. If they are not in the ledger, practices remain "outside the language" and no payback is taken. As hypotheses they can be refuted |
| Build the machinery (server, swapping runtime) first | It goes against the operating rule (a differentiating hypothesis is measured by a minimal experiment before anything is built on it). Decisions that collide with closed world optimization and lowest runtime dependency (ADR-0041) would be made without evidence |
| Leave it outside the language (the current state) | This was the recommendation of the collaboration discussion (the server is a product decision). The judgement is still right. But if the list of what payback can be taken and the hypotheses are also left outside, the exploration itself does not happen |

- Payback stands only on what is proven. For the `io` shell, performance, resources and configuration outside the code, the practices remain as they are (outside the guarantee scope of DESIGN §3).

## Consequences

- If accepted: DESIGN §8.3·§9 R5, three rows of the hypothesis registry of ROADMAP, backlog `T1-51` (the shape of edits), `T1-52` (replay of the throughput of the trunk), `T1-53` (soundness of replacement: scope decided after acceptance).
- Even before acceptance the deterministic experiment `T1-51` runs (autonomous scope: measurement, no design change).
