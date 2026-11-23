# ADR-0058: Hypothesis-first track reset (T0–T5) and demand-driven language phases

- Status: Accepted (approved by the maintainer 2026-10-28: track reset "accepted", measurement path "CLI pilot first", proceed with ADR drafts for the three language candidates)
- Date: 2026-10-28
- Related: ADR-0021 (measurement procedure), ADR-0034 (mission goals), ADR-0037 (bootstrap), ADR-0040 (hypotheses), ADR-0045 (judgement criteria). Discussion record `dev-docs/reviews/direction-2026-10-28.md`

## Context

In the 4 days after the v2 reset, seven language phases (2B·2C·2G·2S·2T·2O·2K) were closed. But for everything that needs a model to measure, the data is 0.

- Mission goals G2·G3·G5·G8
- The six ADR-0040 hypotheses: H-SPEC, H-ORACLE, H-GAME, H-FAMILIAR, H-SCALE, H-REUSE

There is only one path to run a model, "user machine + paid API", and that path has been blocked since 10-24 (1E-10 🚫). As a result the first operating rule of genesis §3.5 ("a differentiating hypothesis is measured with a minimal experiment before building on it") is being broken. The layer it is attributed to is the process (JUDGMENT §3).

In the same discussion, under the premise that tools do the reading and writing, the need for human-oriented constructs (files, namespaces, names, visibility, inheritance, interfaces) was examined again. As a result nine new hypotheses were registered (discussion record §6.2).

## Decision

### 1. Tracks

The waves W0–W6 (BACKLOG execution plan) are replaced by the following tracks.

| Track | Purpose | End condition |
| --- | --- | --- |
| T0 Measurement path | Get model data flowing | First formal results of the frozen tasks (m1·m1b·BVB·BRB) committed, mission target values fixed (ADR-0034) |
| T1 Hypotheses | Judge the differentiators and the AI-native constructs | Every hypothesis in the hypothesis registry is one of confirmed, refuted, or on hold (with reason) |
| T2 Language completion | A language in which canonical programs can be written | The pass criteria of each phase |
| T3 Recognized benchmarks | Prove capability with a yardstick that outsiders accept | Freeze and measurement of correctness (BVB, HumanEval·MBPP ports) and performance (Computer Language Benchmarks Game) |
| T4 Libraries | Prove practicality with proved libraries that have recognized test vectors | Per library: everything proved, recognized vectors pass, speed and size table against the Rust crate |
| T5 Self-hosting | ROADMAP stage 4 | Stage 4 pass criteria |

**Wave mapping.**

| Old wave | New track |
| --- | --- |
| W0 (BCB) | T1 |
| W1, user track | T0 |
| Remaining language phases 2U·2N·2M of W4b | T2 |
| W5 | T5 |
| W6 | After T5 |

### 2. Measurement path: pilot first

By the maintainer's decision, pilot runs with the Claude Code CLI come first (`bench/pilot/`).

**What the pilot keeps**

- The frozen tasks and MANIFEST
- The system prompt (`prompt.md`)
- The only tool is `bmb mcp`
- The limit of 40 calls
- One session per task, no retries
- The same verdict as the harness

**What the pilot cannot keep**

- The CLI holds the agent loop, the cache and the inference settings.

**Rules**

- Pilot results are not ADR-0021 measurements. Every row carries `"pilot": true`.
- They are not used as the basis of claims in documents (C3).
- They are used only for judging direction and hypothesis priority.

The formal path (automated runs with the API key as a secret, `bench/run_models.py --budget`) is still the recommendation. It is switched to when the maintainer approves.

### 3. Demand-driven language phases

- A new phase of T2 is opened when one of two things exists.
  - A concrete demand from T3 or T4: a bench program or library to be written needs the feature.
  - A verdict of T1: a hypothesis points to changing the language.
- The defect of JUDGMENT §7 ("a canonical Rust program cannot be written") remains a defect regardless of demand.
- 2N, which is in progress, continues. Bit operations are needed by SHA-256 and CRC-32 of T4.

### 4. Hypothesis registry

- The registry is in the ROADMAP section "Hypothesis registry". The nine of ADR-0040 and the nine of the discussion record go in.
- There are four states: not measured, deterministic evidence, pilot, confirmed or refuted.
- A hypothesis that could not be judged for lack of progress stays not measured. Verdicts are not estimated.
- "Changes in the hypothesis registry" is added to the phase-end audit (AUTONOMY §3.2).

### 5. ADR drafts for the language candidates

The three language change candidates that came out of T1 are written as drafts in the "Proposed" state. Each is approved for acceptance separately.

- ADR-0059: hash references (H-ID)
- ADR-0060: type invariants (H-ENCAP)
- ADR-0061: `f64` (T3 demand)

## Rationale

The recommendation in JUDGMENT §10 form is in discussion record §7.1. In summary:

- The cause is in the process, so the process is fixed (§2.1).
- The canonical way is automation of the evaluation pipeline (S1).
- The decisive criterion is whether evidence keeps being produced (S5).

The pilot is a marked temporary measure that gives data for judging direction while waiting for the canonical path (JUDGMENT §2.5). The release condition is approval of the formal path.

The basis of the demand-driven phase rule is genesis §3.5. v0 widened features and ecosystem before verifying its core hypotheses. Opening only features that have demand ties the size of the language to the progress of measurement and benchmarks.

## Consequences

- ROADMAP gets the sections "Tracks (ADR-0058)" and "Hypothesis registry". The wave table remains as a historical record.
- BACKLOG gets track tasks (`T0-`·`T1-`·`T3-`·`T4-` IDs), and the backlog check (`scripts/check_backlog.py`) accepts those IDs.
- Pilot runs use the Claude Code usage of this account within the scope the maintainer decided. The formal path awaits approval.
- Reverting needs an ADR that supersedes this one (approved by the maintainer).
