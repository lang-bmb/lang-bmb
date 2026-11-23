# ADR-0125: A way of working in which a parent agent manages subagents

- Status: Accepted (instructed and approved by the maintainer 2026-11-11 "in the parent-agent style that keeps managing subagents", "proceed with the recommendation"). It supplements the work loop of AUTONOMY.md §2
- Date: 2026-11-11
- Related: AUTONOMY.md (work loop, decision authority, reporting), ADR-0111 (decision gates), ADR-0113 (fixing of components), ADR-0122 (authoring cost of the ledger), HEALTH.md (gate), `scripts/ship.sh`

## Context

- This language is a language that agents write. The process of an agent working in BMB is itself the data of utility (session tokens and gate runs of the port ledger, ADR-0122).
- If one session writes everything directly, its context fills with the details of tasks, and it cannot do independent work side by side.
- If the same model writes and the same model reads and passes it, that is not verification.

## Decision

1. **The goal is at the level of a gate.** One goal is set at a time. The completion condition of a goal states only what is judged by output (the status of a backlog row, the exit code of the gate, the branch matching the remote). The ladder: the autonomous queue before the freeze → ports P0~P2 and gate B0 → P3 and gate B → the fixed point (the gates of ADR-0111).
2. **Roles.** The parent agent does task selection, instructions, verification, integration, commits, the gate and records. A subagent takes one task and one set of files and does not commit or push.
3. **Verification is execution.** The parent agent does not copy the numbers a subagent reports but measures them again (tests, measurement scripts, the gate). It does not read and pass.
4. **Parallelism is up to two with no overlapping files.** There is one author at a time changing the compiler (`compiler/`), and while another subagent is running it works in a separate build directory or work tree (changing the binary disturbs someone else's run).
5. **Fixed items of the instructions.** The files that may be changed, prohibitions (commits, files out of scope), the completion criterion, the report format: facts, mismatches between documents and implementation, friction and diagnostic codes, the number of gate runs and the runs that ended undecided (ADR-0122).
6. **Items for approval accumulate and do not stop the work.** On reaching an item of AUTONOMY §4 that needs approval, it gives a recommendation (JUDGMENT §10) and goes on to the next task. The cases in which it stops and reports are, in addition to AUTONOMY §6: a soundness defect found, the gate failing twice for the same cause, a measurement result that yields a hypothesis verdict.
7. **Records.** The tokens and gate runs of subagent sessions are written in the backlog evidence or the port ledger.

## Rationale

| Option | Verdict |
| --- | --- |
| **Parent agent + subagents, verification by execution (accepted)** | Independent work is done side by side, and the writing side and the judging side are separated. There is one gate at a time and the push is to one branch, so integration has to happen in one place |
| Separate cloud sessions | The work trees differ, so integration and the gate are separate per session. Used only for independent long runs (gate detection-power mutants and so on) |
| One session directly | The context fills with details and it is serial |

First cases: the `port-shape` experiment (one session per arm, the parent agent measures again with `measure.py`), the conformance test suite (`5A-02`, the chapters split in two).

## Consequences

- AUTONOMY.md gets a section "§8 Parent agent and subagents".
- The critical path is still tied to the maintainer's approval and the measurement machine. This way of working fills the work in between.
