# ADR-0118: The main documentation condition of the twin measurement is definition summaries versus `///`

- Status: Accepted (approved by the maintainer 2026-11-11 "all recommendations accepted", item D2. Backlog 3P-09, the re-decision of the main condition that ADR-0106 decision 5 deferred)
- Date: 2026-11-11
- Related: ADR-0104 (port twin), ADR-0106 (documentation condition: this ADR changes decisions 1·2), ADR-0117 (definition summaries), ADR-0077·0112 (minimum difference D, the interval of the difference), ADR-0103 decision 3 (reference model)

## Context

- ADR-0106 made the condition that strips the comments of the Rust arm the main condition because BMB had no channel for definition documentation, and wrote that the main condition is set again once a channel exists (decision 5).
- With ADR-0117, definition summaries exist. A `///` in the source becomes `#[summary("..")]` in the normal form and is carried in map L1.
- Real Rust code has `///`. Rust with all comments removed is not how Rust is written, so an advantage in that condition could only be written as "against Rust without comments" (ADR-0106 decision 2).

## Decision

1. **Main condition (`PAIR_DOCS=summary`, the default):** the BMB arm has definition summaries and the Rust arm has `///` with the same text. The other comments on the Rust side (`//`, `/* */`, `/** */`) are removed. Unit summaries and `//!` stay as in ADR-0106.
2. **What the same text means:** in a fixture, the `///` of a BMB definition (lines are joined with a space) is literally the same as the `///` of the Rust item of the same name. The Rust side may only add, after it, sentences that begin with `Requires ..`·`Ensures ..`. This is so that the Rust arm reads as text what the BMB arm reads as contracts. Other sentences are on both sides or on neither.
3. **Definitions that exist only in BMB:** `spec fn`·`proof fn` get no summary in the fixtures. The Rust arm has no corresponding item, so only one side would have more documentation.
4. **Auxiliary condition (`PAIR_DOCS=none`):** neither arm has definition documentation (the old main condition of ADR-0106 with the summaries on the BMB side removed as well). It shows whether the difference in the main condition comes from the documentation.
5. **Control condition (`PAIR_DOCS=all`):** the Rust crate is given as written. The BMB arm is the same as in the main condition.
6. **The qualification rule for claims (changes ADR-0106 decision 2):** utility claims are written from the results of the main condition. If the difference shrinks to within D in the control condition, the claim is qualified as "against Rust with doc comments only". The auxiliary condition is not used for the verdict, only for attributing the difference. The auxiliary and control conditions are one sonnet run on the same tasks and commit; the control condition reruns only the Rust arm, the auxiliary condition reruns both arms.
7. **The scope judgement stays as it is** (ADR-0106 decision 3): the Rust definition source is compared after removing all comments, and the BMB hashes do not read the summary (ADR-0117). A definition in which only the documentation was changed is not a changed definition.
8. **Gate:** `pair.py check` builds, for each fixture, the Rust crates of the three conditions and the BMB sources of the two conditions and compares them with the corpus, and `doc_parity` checks decisions 2·3 and "the number of `///` in the source = the number of definition summaries left in the normal form". The oracle and the weak control run in the main condition.

## Rationale

| Option | Verdict |
| --- | --- |
| **Definition summaries versus `///` as the main condition (accepted)** | Both arms see the same text. It is close to how the Rust arm is really written. The claim carries no "without comments" qualification |
| As now (the condition without documentation is the main one) | Even after the channel exists, only the Rust arm is in a condition worse than usual |
| Free comments for Rust as well (the control condition as the main one) | BMB has no channel for free comments (DESIGN §4.1). Information that only one side has enters the verdict |
| Forbid the `Requires`·`Ensures` sentences on the Rust side | The BMB arm reads that content as contracts. Forbidding it leaves only the Rust arm without knowledge of the preconditions |

## Consequences

- `bench/twin/pair.py`: `DOCS` (environment variable `PAIR_DOCS`), `strip_comments(.., docs)`, `strip_summaries`, `bmb_sources`, `item_docs`, definition-level `doc_parity`. `docs` in the result rows records the condition (`rust_comments` and `PAIR_RUST_COMMENTS` are removed: there is no formal run of the twin yet).
- The fixtures of the port twin (3P-02) put the same `///` on the port and on stage0. The existing doc comments of stage0 (Rust) are the starting point.
- The three conditions and decision 6 go into the pre-registration of 3P-05.
