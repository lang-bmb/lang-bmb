# ADR-0036: Syntax evaluation scale

- Status: Accepted (autonomous, AUTONOMY.md §4: decides only the evaluation method. The language and DESIGN.md are not changed)
- Date: 2026-10-25
- Related: DESIGN.md §2·§3 (principles 3·5·6·7·8·9)·§4·§4.1·§9 R2·R4, ADR-0013·0019·0021·0034, backlog 2R-01

## Context
DESIGN.md §4 set "the surface succeeds Rust and Verus" (ADR-0013). The purpose of succession was to reduce the cost of a language not trained on. Succession itself is not a goal (the maintainer's instruction 2026-10-25). BMB's goals and conditions differ from those of Rust and Verus. The user is an agent, and the agent reaches code only through tools. The canonical form is the normal form, and the cost of reading is measured by slice size. So for each syntax element, the gain of prior knowledge and a more effective new form must be compared on the same scale.

There is no such scale now. §4.1 "tension and response" only says "measure the Rust habit error rate and revert if the loss is large". The criteria, measurements and judgement rules are not set. Measurement is also partial. The habit metric of ADR-0021 counts only E02xx. Yet a good part of Rust and Verus habits come out under other codes. For example `y += 1`, `0 <= i < n`, `int`, `exists` are rejected with E0103·E0211·E0402 (2R-02 measurement).

## Decision

### 1. Evaluation unit and two surfaces
- The evaluation unit is a **construct**. A construct is one syntax element that does one job (for example: a contract clause, a loop head, an integer conversion). The list is made by 2R-02.
- For each construct, two surfaces are looked at separately.
  - The **write surface** is the text the agent emits (the `tx_put` input).
  - The **canonical surface** is the normal form (ADR-0019). Storage, hashes, slices and the map all use the normal form.
- The judgement is made on the canonical form. Accepting a different form on the write surface and moving it to the canonical form is recorded with an additional marker, **input alias**. An input alias holds only when all three conditions are met.
  1. It is deterministic.
  2. It preserves meaning.
  3. It is a total function. One alias form is moved to exactly one canonical form.

  An input alias does not increase the ambiguity of the canonical form (principle 8). Instead the writing tool takes on the labor of handling aliases (principle 6).

### 2. Criteria
| # | Criterion | What it asks | Offline measurement (deterministic) | Real model measurement |
| --- | --- | --- | --- | --- |
| C1 | Prior knowledge transfer | Does the agent write the right form on the first attempt with the Rust and Verus knowledge it was trained on | Is the form valid Rust or Verus (subset property). Whether there is a diagnostic code and a fix suggestion for the habit form. The false friend (§3 K4) check | Habit error frequency per construct (the mapping table of §5), one-shot success rate, form match rate of the first commit |
| C2 | Ambiguity and normal form | Is there one canonical form per job. Does interpretation not rely on context such as type inference or method lookup | Normal form idempotence (`fmt` fuzzer), number of forms that do the same job, isomorphism detection (`duplicates`) | — |
| C3 | Tool friendliness | (a) Can a tool fill in this notation deterministically. (b) Are the map edges fixed by syntax. (c) Can the habit form be fixed mechanically | (a) Derivation rate: the share for which deleting the notation and filling it in again by rule gives the same proof and check results. (b) Can L2 edges be computed without resolution. (c) Share with a `fix.replacement`, and meaning preservation | Share of tasks that turn into a success after an automatic fix |
| C4 | Verification friendliness | Is the proof encoding simple and stable. Does a counterexample read in the surface form | Number of obligations, `rlimit` sum and maximum of the corpus, `bmb stability`. Are counterexample names and values 1:1 with source names. Can a soundness fuzzer cover it | Share fixed after seeing a counterexample |
| C5 | Token cost | The cost of writing and the cost of reading | Lexical tokens and `ceil(bytes/4)` per construct (ADR-0021). The write corpus (task reference solutions) and the read corpus (seed normal forms), `slice` size | Total session tokens |
| C6 | Performance justification | Does this construct pass proved facts to code generation, or does it block performance | `bench/baseline` time ratio, number of `; proved` facts in the IR | — |

- The offline measurement tools are `bench/syntax/measure.py` (2R-02) and the existing CLI (`fmt`, `fix`, `uses`, `stability`, `query slice`, `query duplicates`). 2R-04 adds a converter.
- The offline value of C1 is a **prediction**. When the real model measurement (2R-03, 2R-05) arrives, it is replaced by that value. Documents write prediction and measurement apart.

### 3. Constraints and objectives (derived from DESIGN.md §2)
The criteria are not combined in a weighted sum. **Constraints filter first, and the remaining candidates are compared in objective order.** This is because DESIGN.md §2 set correctness as a constraint, performance as the objective function, and the agent as the user.

**Constraints K1~K4.** A form that violates even one cannot be canonical.

| Constraint | Content | Rationale |
| --- | --- | --- |
| K1 Verifiable | The meaning of the construct is encoded soundly in the proof gate and can be covered by a fuzzer | §2 correctness, CLAUDE.md "a feature that is not verified is not put in" |
| K2 Canonical uniqueness | On the canonical surface there is one form per job. Input aliases are allowed (§1) | Principle 8 |
| K3 Fixed by syntax | The map edges of call targets, names and effects are fixed by syntax and explicit declarations alone | Principles 6·7, §4.1 call resolution |
| K4 Behavior preservation | A form borrowed from Rust or Verus behaves the same as in Rust or Verus in every program BMB accepts. Tightening (reducing the programs accepted) is allowed. Attaching a different meaning to the same form is a false friend and is forbidden | §4 "a more verbose Rust", ADR-0013 |

**Objectives O1~O5.** Earlier ones take priority.

| Order | Objective | Main criterion | Rationale |
| --- | --- | --- | --- |
| O1 | The agent's one-shot correctness and spec quality | C1, C3(a)(c) | G2·G3 (ADR-0034). The user is the agent |
| O2 | Read tokens | C5 read | Principles 5·7, G4. A slice is read many times |
| O3 | Write tokens the tool does not absorb | C5 write minus the share filled in by C3(a) | Premise E3, G5. The write cost of notation the tool fills in is taken as 0 |
| O4 | Proof cost and stability | C4 | G2 (proof within time), §4.1 proof stability |
| O5 | Performance | C6 | §2 objective function, G6. The surface has almost no influence on performance. Only measured gains are counted, and a measured regression beyond ±5% is refused |

**Things with weight 0.** Human readability, ease of learning and aesthetics (DESIGN.md non-goals) and novelty itself get no weight. "It is cleaner" is not a rationale.

**Handling of derivable notation.** If a notation is derived 100% deterministically from other code (C3(a)), the agent puts no information into it. Such notation is made optional on the write surface. The tool fills it in. Whether to keep it in the canonical form is decided by the following criteria.
- If derivation needs information outside the slice (a global fixed point, the type checker, the prover), it stays in the canonical form. This is because a slice must be self-sufficient (principle 5).
- If it is derived from syntax inside the slice alone, it can be omitted in the canonical form too (O2).
- If the notation plays an interface role, it stays. An interface role means keeping a callee change from spreading to callers (the effect declaration of ADR-0030).

### 4. Verdict categories and rules
| Verdict | Meaning |
| --- | --- |
| **Borrow** | The Rust or Verus form is used as is. Making a notation that is optional in Rust mandatory ("making the optional mandatory", §4) is also a borrow if the form is the same |
| **Extend** | A new form for a concept that has no corresponding form in Rust or Verus |
| **Variant** | A Rust or Verus form exists but does not work as is, so a close variant is used |
| **Reject** | The form is not accepted. A habit diagnostic and a hint are provided |

The judgement procedure follows this order.
1. **Is the concept within BMB's scope?** If outside, it is not a syntax verdict. Only a scope diagnostic (the E0207 family) is provided.
2. **Does Rust or Verus have a form for this concept?** If not, go to the extension rule (5).
3. **Does that form keep K1~K4?**
   - If it does, it is a borrow candidate. If an alternative wins on O1~O5 by the evidence criteria of §5, it is a variant, otherwise a borrow.
   - If it violates them, see whether there is a close variant. If there is, variant, otherwise reject.
4. A **close variant** is a form that satisfies all of the following.
   - (i) It is within the Rust and Verus vocabulary. Better if it is valid Rust.
   - (ii) It is a local change within the same syntax node.
   - (iii) There is a meaning-preserving mechanical fix from the habit form to this form.

   A variant without (iii) leaves a C1 loss the tool cannot absorb. Such a variant is adopted only with real model evidence (§5 E2 or above).
5. **Extension rule.**
   - Use Rust's extension points before a new keyword or symbol. The extension points are the attribute `#[..]`, path-call functions, and Verus's clause positions.
   - The name is chosen from the closest trained analogue.
   - K1~K4 are kept.
   - If the notation is 100% derivable, it is not syntax but tool fill-in or a map fact (§3).
6. **A rejection comes with a diagnostic.** Each rejected habit form gets a diagnostic code and a hint. If there is a meaning-preserving fix, a `fix.replacement` too.
7. **Reverting.** Per §4.1 response 3, if the measured C1 loss is larger than the gain recovered, a variant or rejection is reverted to a borrow. The evidence is §5 E2 or above.

### 5. Evidence criteria and change procedure
**Evidence levels**

| Level | Content |
| --- | --- |
| E0 | Argument (design principles, cases). Only the starting point of a verdict, not grounds for change |
| E1 | Offline deterministic measurement (tokens, derivation rate, share with a fix, `rlimit`, normal form checks). The measurement code and results are committed |
| E2 | Per-construct aggregation of real model run logs (1E-10·2A-06, 2R-03) |
| E3 | A/B on twin task sets that differ only in syntax (2R-05). Same model, one shot, no retry (ADR-0021) |

**Change thresholds**

Changing an M1 construct that is already implemented has the thresholds below. The constructs of the proposed ADRs (0026·0028~0031) are not implemented yet, so they can be judged by E1 and argument. But if the direction lowers C1, the thresholds below apply all the same.

| Direction of change | Evidence required |
| --- | --- |
| Towards Rust and Verus (relaxation, input alias, reject to borrow) | E1 (K1~K4 confirmed kept) + one of: in E2 the habit diagnostic of that construct occurs at least once per 10 `tx_commit`, or in E1 the O2·O3 gain is 5% or more |
| Away from Rust and Verus (new form, added rejection) | A gain in one of O1~O4 in E1 + not inferior in E3 |
| When changing with O1 at stake (a one-shot success rate claim) | E3. On the twin sets, the number of tasks only the alternative solves must exceed the number only the existing form solves by 3 or more. One task is 7~10%p (m1 §2), so a smaller difference is treated as noise |

**Meaningful size (E1).** Exceeding one of the following counts as a meaningful size.
- 5% of the lexical tokens of the read or write corpus
- 10% of the `rlimit` sum, or `rlimit_max` of even one definition exceeding 50% of the fixed `rlimit`
- A derivation rate of 95% or more (the verdict that it can be filled in)

**Restraint on change**
- Verdicts change in bundles. They change only at a decision ADR (2R-06) or a phase measurement (2H·3E), not one construct at a time whenever.
- The same construct is not reopened without new E2·E3 evidence.

**Migration**
- When the canonical form changes, the old form stays as an input alias for one phase. Using the old form emits a diagnostic with a `fix.replacement`, and `bmb fix` moves it mechanically.
- Frozen task sets are not edited (ADR-0021). New syntax is measured with twin sets under new names.

**Meaning preservation of fixes**
- A mechanical fix that changes a value is a C3(c) violation and a defect. Such a fix cannot be an input alias.
- Example: the fix where E0104 turns `0x10` into `0` (found in 2R-02).

### 6. Measurement scope of habit errors
- The metric of ADR-0021 ("number of E02xx / `tx_commit`") is kept as is for continuity of measurement.
- Separately from it, 2R-03 aggregates a **per-construct habit metric**. It uses a table that maps the diagnostics of the harness logs (code + the problem token of the message) to the construct list of 2R-02. This table also counts habits outside E02xx.

| Habit | Diagnostic |
| --- | --- |
| Name reuse | E0303 |
| Inline contract | E0501 |
| Rust standard methods and types | E0404·E0402 |
| `T::MAX` | E0301 |
| `+=`·bit operations·`exists`·`open spec fn`·`@`·`#![auto]` | E0103 |
| Chained comparison | E0211 |
| Missing attribute or clause | E0601·E0506·E0511 |

- This table is the "Rust habit error rate" of R4 broken down per construct.
- The agent's output form (DESIGN.md §9 R2: text versus structured edit operations) is not a subject of this scale. This scale compares text surfaces.

## Rationale
- **Constraints first, objectives in order.** DESIGN.md §2 set correctness as a constraint, performance as the objective function, and the agent as the user. A weighted sum leaves room to offset a constraint violation with another gain. The influence of surface syntax on performance is small by measurement (1E-08: even the speed gain of check removal is at noise level). So O5 is used only as a veto and a tie-breaker.
- **Two surfaces.** The agent does not touch code directly (principle 6). So the written form and the stored form need not be the same. This moves "explicitness is a property of the artifact, and the labor is the tool's job" (principle 6) into a rule of the scale. The three conditions of an input alias keep canonical ambiguity at 0 (principle 8).
- **K4 behavior preservation.** Prior knowledge is a gain only when the same form has the same meaning. Tightening only reduces the programs accepted, so it does not harm knowledge. Changing the meaning makes knowledge produce errors instead.
- **Evidence thresholds.** At a scale where one task of difference is 7~10%p (m1 §2), a small A/B difference is noise. A syntax change shakes the whole of corpus, seeds, prompts and diagnostics (restraint on change). So relaxation towards Rust is made cheap, and change away from Rust expensive.

Alternatives considered:
- (1) A weighted-sum score table. There are no grounds for the weights and constraint violations get offset. Rejected.
- (2) Every verdict by A/B only. The cost is large and most constructs are settled by E1. Rejected.
- (3) Tying the write surface and the canonical form into one (the current state). Even habit errors the tool could absorb remain as agent failures. They are separated on the scale, and whether to really separate them is decided by 2R-06.

## Consequences
- 2R-02 (first evaluation)·2R-04 (offline comparison)·2R-06 (decision) use this scale. This ADR does not make the verdicts themselves. Language changes get the maintainer's approval in 2R-06.
- Writing the per-construct habit mapping table is added to 2R-03 (§6).
- To revert, change this ADR to `Rejected`. There is no code change.
