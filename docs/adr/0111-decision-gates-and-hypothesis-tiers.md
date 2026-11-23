# ADR-0111: Decision gates of the roadmap and hypothesis tiers

- Status: Accepted (approved by the maintainer 2026-11-11 "all approved, recommendations accepted". Gate B0 is supplemented by ADR-0116. Backlog 2H-08, maintainer instruction 2026-11-11 "review and establish the whole roadmap")
- Date: 2026-11-11
- Related: DESIGN.md §2 (premises E1~E3), ADR-0034 (mission targets), ADR-0040 (hypotheses and refutation conditions), ADR-0058 (tracks and hypothesis registry), ADR-0100 (freeze), ADR-0102 (E3 verdict criterion), ADR-0103 (evidence stage), ADR-0104·0105 (port twin, stage 3 scope), ADR-0037 (bootstrap), `dev-docs/reviews/roadmap-2026-11-11.md`

## Context

- Every hypothesis has a refutation condition (ADR-0040·0078). But how a refuted or unmet result changes the roadmap is not defined. Three times so far (G5 task unit not met, H-EFFECT refuted, H-SCALE refuted) the scope of the claim was narrowed or the verdict was moved to the next measurement (the port twin) (review §2.2 M1).
- That next measurement (3P-05) is possible after the freeze and the port of P0~P3. What happens if E3 (the criterion of ADR-0102 decision 2) is refuted or not judged then is not registered. The port is the largest investment of this project (stage0 S4 53,459 lines).
- The registry has 32 hypotheses and 0 are confirmed. The measurement machine is serial and has a daily limit. Measurements that judge the premises and measurements that pick tool defaults are in the same queue (review §2.2 M2).

## Decision

1. **The roadmap has decision gates.** A gate is a point on the critical path, and the defined measurement and audit results must be committed before passing it. For each gate, the action per result is registered before that measurement by this ADR (or an ADR that supplements it). The table of gates in `docs/ROADMAP.md` "critical path and decision gates" is the source of truth.

   | Gate | Position | What it judges | Results it rests on |
   | --- | --- | --- | --- |
   | A freeze | end of ⑤, 4D-01 | Whether the accepted language may be fixed | The 2H-01 results and 2H-02, the preconditions of ADR-0100 §5, the reference draft at the freeze commit (5A-01) and the rebaselined port map (4D-07) |
   | B economics | after P3 passes the differential tests, before P5 starts | Premise E3 | 3P-05 (H-TASK, G5-L, H-COMPREHEND, H-REFACTOR) and 2A-14e (H-AMORT), the port ledger (4D-08) |
   | C early fixed point | ⑦, 4E-02 | Whether BMB can write and run the front half of its own compiler | stage2 IR = stage3 IR, compile time and size of stage1 versus stage0 (the first real-scale values of G6·G9), the share of proved definitions in the ported code (S7) |
   | D full fixed point | end of ⑧, 4E-01 | The stage 3 pass criteria (3E) and principle 7 | stage2 = stage3, 3P-06 (size axis), the E3 re-judgement (if carried over) |
   | E ecosystem | after ⑨, 5E-01 | Whether principles 4·6 are demonstrated | The verdicts of gates B·D, the unproved area of the self-proof (4F) |

2. **Actions per result at gate B (registered before measurement).** The verdict criterion is ADR-0102 decision 2 as it is (G5-L ≤ 1x, or G5-L ≤ 2x and escaped defects at most 1/2 of Rust+tests). The verdict uses the reference model of ADR-0103 decision 3.
   - **Holds:** P4 and after continue as planned. It becomes the first case of moving the E3 family in the hypothesis registry to "confirmed".
   - **Refuted:** the start of the ports from P5 on stops (P4 and E1 in progress are finished: the fixed point is an engineering goal independent of E3). The failure attribution (language, compiler, tools, measurement design, JUDGMENT §3) is reported, and a direction ADR is raised as an item for approval. There are three candidates: (a) fix the attributed layer and measure once more on the same tasks, (b) rewrite E3 and principle 4 of DESIGN §2 by the refuted scope and revise the mission targets (G5·G7·G8), (c) continue self-hosting (stage 4) but limit the utility claims to the demonstrated axes (change safety, verdict trust, artefacts).
   - **Cannot be judged** (the comparison group has 0 defects so it does not discriminate, or the difference is within the interval of the judgement rule): it is carried over. A carried-over verdict is made once more at gate D with 3P-06.
3. **A verdict is carried over once.** A verdict on one premise (E1·E2·E3) or principle can be moved to "again at the next measurement" once. If it does not hold the second time either (refuted or cannot be judged), the claim is recorded as "could not be shown" and the corresponding statements in DESIGN, README and the utility report are revised. The verdicts already moved (G5 → G5-L, H-EFFECT, H-SCALE: all to the port twin) count as the first carry-over of this rule.
4. **Hypotheses have three tiers.** The tier is set by what is rewritten when the hypothesis is refuted. The table in `docs/ROADMAP.md` "hypothesis tiers" is the source of truth.
   - **Existence hypotheses:** if refuted, a premise of DESIGN §2 or a principle of §3 is rewritten. H-ORACLE, H-AMORT, H-TASK (E3), H-GAME, H-STABLE (the gate of principle 4), H-SCALE, H-COMPREHEND (principle 7), H-REVIEW, H-INTENT (E1).
   - **Differentiation hypotheses:** if refuted, a ledger row or an individual ADR is rewritten. H-ENCAP, H-EFFECT, H-ID, H-PAR, H-COMMUTE, H-SPLIT, H-MAINT, H-COMPAT, H-SPEC, H-NAME, H-FAMILIAR, H-REFACTOR, H-REUSE, H-SLICE.
   - **Tuning hypotheses:** they pick defaults of the tools and the surface. H-VIEW, H-GRAIN, H-EDIT, H-UNIT, H-REPAIR, H-SMALL, H-UNDECIDE, H-PROBE, H-HISTORY.
5. **The measurement queue is in tier order.** In the serial order of the measurement machine, measurements of existence hypotheses come first. A measurement of a tuning hypothesis goes ahead only when (a) the measurement machine is idle, or (b) its result sets a default that must be closed before the next gate. When a new hypothesis is registered, its tier is written with it, and for an existence or differentiation hypothesis also the gate at which it is judged. A tuning hypothesis can be kept only in its ADR instead of adding a row to the registry.

## Rationale

| Option | Verdict |
| --- | --- |
| **Decision gates + tiers + one carry-over (accepted)** | It connects the refutation conditions at the hypothesis level to decisions at the roadmap level. The actions are registered before the results, so narrowing the scope after the fact happens only within the rule. Stage gates and pre-registration of interim analyses are established methods (JUDGMENT S1) |
| As now (only per-hypothesis refutation conditions) | The consequence of a refutation depends each time on the judgement of the moment. There is no decision point in front of the largest investment |
| E3 verdict before the port | The judging ground is the port twin (ADR-0104). With synthetic tasks H-ORACLE failed to discriminate four times. It cannot be done, because it is circular |
| Gate B after E2 | It is after the whole port is done, so the result cannot change the plan |
| Date deadlines | Conflicts with "a stage closes by its pass criteria". A gate is defined by its position |

## Consequences

- `docs/ROADMAP.md` gets the sections "critical path and decision gates" and "hypothesis tiers".
- If accepted, 3P-05 becomes a prerequisite of the ports from P5 on (4D-04·4D-06). The order of the port map (P4 → E1 → P5) stays as it is, and the measurement goes in between.
- "Measurement does not set the order of the port" of 2H-02 (ADR-0105) stays as it is with respect to the order. What this ADR adds is the go or no-go verdict in front of P5.
- If gate B is refuted, the port can stop. That cost is limited to P0~P4 (about 42%, 4D-07 measures it again).
- A tier column is needed when a new row of the registry is made. The hypothesis tally of `scripts/progress.py` reads only the status column, so it runs as it is.
