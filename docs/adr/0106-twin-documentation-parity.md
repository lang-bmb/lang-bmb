# ADR-0106: The documentation condition of the port twin: the Rust arm without comments, with a control condition with comments registered alongside

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": a measurement condition of ADR-0104 decision 4 "fairness is fixed first". Eighth execution plan "3P fairness") ADR-0118 changed the main condition and the qualification rule (decisions 1·2) (definition summaries versus `///` is the main condition, the main condition of this ADR is the auxiliary condition `none`)
- Date: 2026-11-10
- Related: ADR-0104 (port twin), ADR-0103 (evidence stage, the reference model for claims), ADR-0019 (normal form), ADR-0006 (rationale in the node hash), DESIGN.md §2 table "comments", `bench/twin/README.md` "fairness", backlog 3P-02~05

## Context

- The workspace that `bmb import` produces is in normal form, so it drops `//`·`///` comments. What remains is the `#[summary]` of the unit, the contracts and the names. The Rust arm receives the crate as it is and sees all doc comments and line comments.
- The fixture `stk` has the same documentation written on both sides, but it is not visible to the BMB arm. stage0 (about 50,000 lines) has many comments. Backlog IDs, design rationale and explanations of invariants are in comments.
- Measured as it is, the difference cannot be split into a difference of language and tools and a difference in the amount of information. This holds especially for the comprehension tasks (H-COMPREHEND).
- BMB has no channel for free documentation on the definition unit. DESIGN.md §2 puts "structured rationale bound to the node hash" in place of comments, but for now that (`#[rationale]`) attaches only to escape hatches (ADR-0029). Carrying documentation on the BMB side (the normal form keeping `///`, or the map carrying definition documentation) is a decision about the normal form and the surface (ADR-0019). It is outside the autonomous scope, and has to be decided before the freeze (ADR-0100).

## Decision

1. **Measurement condition (main condition): the Rust arm receives a crate without comments.** `materialize` in `pair.py` applies `strip_comments` to every `.rs` file.
   - Removed: `//`·`///` lines and trailers, `/* */` (nested)·`/** */`.
   - Kept: the inner module documentation `//!`. It corresponds to the `#[summary]` of a BMB unit.
   - Strings, byte strings, raw strings, characters and lifetimes are left as they are.
   - Then the information both arms see is names, types, code and unit summaries. BMB's contracts are added to this. The contracts are the very difference that BMB claims.
2. **Control condition: Rust with comments (`PAIR_RUST_COMMENTS=keep`).** It runs sonnet once (the reference model of ADR-0103 decision 3) on the same tasks and commit. Only the Rust arm is rerun.
   - If BMB leads in the main condition but the difference shrinks to within the minimum difference D (ADR-0077) in the control condition, the claim is written with the qualification "against Rust without comments".
   - If it leads by more than D in the control condition as well, the claim is written without qualification.
   - This rule is registered by this commit before the first run of 3P-05.
3. **The scope judgement uses the source without comments.** The BMB normal form drops the comments that an agent writes. So the definition source of the Rust arm is also compared after removing comments (`rust_defs`). A definition in which only comments were changed is not a changed definition. This is the same in both conditions.
4. **The gate of the stripper.** `pair.py check` builds both the crate with comments and the stripped crate for each fixture. Both must match the output bytes of the corpus (the same program). The oracle and the weak control also run in the main condition. `Comments` in `test_pair.py` covers strings, raw strings, nested blocks, characters and lifetimes.
5. **The documentation channel on the BMB side is left as the maintainer's decision.** If approved, that edition is added as a third condition, and the main condition is set again then. This ADR does not wait for that decision.

## Rationale

| Option | Verdict |
| --- | --- |
| **Strip Rust comments + a control condition with comments (accepted)** | The information of the two arms is the same, so the difference can be attributed to the language and the tools. The bias on the side of weakening Rust has its size measured by the control condition |
| Each language as it is used (keep Rust comments only) | Realistic, but the difference in the amount of information is mixed into the result. It cannot be attributed |
| Carry documentation on the BMB side | A decision about the normal form and the surface (ADR-0019), so outside the autonomous scope. Decision 5 hands it to the maintainer |
| Only remove the documentation of the fixtures on both sides | It has to be done by hand every time for the stage0 port, and no gate catches an omission |

## Consequences

- `bench/twin/pair.py`: `strip_comments`, `strip_crate`, `RUST_COMMENTS` (environment variable `PAIR_RUST_COMMENTS`, default `strip`), `rust_comments` in the result rows, comment removal in the scope judgement, the build of both conditions in `check`.
- The "fairness" section of `bench/twin/README.md` points to this decision.
- The control condition (sonnet once, Rust arm) goes into the execution plan of 3P-05.
