# ADR-0117: The definition summary `#[summary("..")]` and its input form `///`

- Status: Accepted (approved by the maintainer 2026-11-11 "recommendations A1~A7 accepted", item A3. Backlog 3P-08, the surface decision that ADR-0106 decision 5 handed to the maintainer)
- Date: 2026-11-11
- Related: DESIGN §4.1 "comments"·§7.1 (L1), ADR-0006 (rationale in the node hash), ADR-0019 (normal form), ADR-0008·0020 (map L1, additions are backward compatible), ADR-0095 (input aliases), ADR-0106 (documentation condition of the twin), ADR-0100 (freeze)

## Context

- A unit has a one-line summary `#[summary("..")]`, and it is carried in map L0. A definition (function, type, trait, constant) has no place to carry documentation. The normal form removes comments (DESIGN §4.1: structured rationale in place of free comments).
- In the port twin measurement (ADR-0104) the Rust arm can have `///` documentation and the BMB arm cannot. ADR-0106 made the condition that strips the comments of the Rust arm the main condition, and handed the channel on the BMB side to the maintainer because it is a surface decision.
- The BMB sources in the repository already have sixteen lines of `///` in front of definitions (`tests/tools`, `bench/twin/fixtures`). Until now they were dropped as comments.
- If it goes in after the freeze (ADR-0100), two compilers are changed.

## Decision

1. **A definition can have one `#[summary("..")]`.** It attaches to functions (including trait method declarations), `struct`·`enum`, `trait` and `const`. The argument is one string (E0704). Two or more is E0605. It is the same attribute as the `#[summary]` of a unit.
2. **`///` is its input form.** Consecutive `///` lines are joined with one space into one string. The normal form (`bmb fmt`) writes `#[summary("..")]` as the first attribute of the definition. `//` and `////` stay comments. A `///` in a place that is not in front of a definition is rejected as an attribute in a place where an attribute cannot stand (E1002 and others).
3. **No hash reads the summary.** Identity, the body and contract hashes and the incremental proof key are computed from the text without the summary. Changing a summary does not invalidate other definitions or the proof cache (the same rule as the rationale `#[rationale]`, ADR-0029 §3).
4. **It is carried in the map L1 row under the key `summary`.** It appears only for definitions that have a summary (ADR-0020 §1: additions are backward compatible). It does not go into the `signature` of types and traits. The JSON AST (`edit_ast`) exchanges it as the `summary` key for functions and constants, and as `attrs` for types and traits.
5. **Proof and checking do not read the summary.** A summary is prose and not a contract. What enforces intent is contracts and rationale (principle 3).

## Rationale

| Option | Verdict |
| --- | --- |
| **Definition-level `#[summary]`, `///` is the input form (accepted)** | It fits with the one device that units already have (JUDGMENT S3). The ban on free comments (ledger) stays as it is: a summary is one line bound to a definition and part of the map. `///` is the standard Rust notation, so it is accepted as the habit has it and moved to the canonical form (the same way as the input aliases of ADR-0095) |
| The normal form keeps `///` comments as they are | If comments become part of the normal form, "one representation" breaks, and a separate channel is needed to carry them in the map |
| Carry it only in map L1 and not in the source | The map is a derivative of the code (principle 2). The map cannot have what the code does not |
| Do not have it | In the twin measurement only the BMB arm has no definition documentation. Whether it has an effect cannot even be measured |

## Consequences

- The accepted language widens: `#[summary]` and `///` in front of a definition. A `///` in front of a definition, a comment until now, becomes a summary, and a `///` inside a body becomes an error.
- The third documentation condition of the twin (ADR-0106 decision 5) is set up: the condition in which the BMB arm has definition summaries and the Rust arm has `///`. Which one is the main condition is decided after the oracle check of that condition (backlog 3P-09).
- The effect (the difference in tokens and correct-answer rate in H-COMPREHEND) is measured per condition in 3P-05. A lint that demands writing summaries is not added before that result.
- Corpus `tests/verify/pass/summaries.bmb`, `tests/check/err/summaries.bmb`, test `summaries_are_kept_by_the_normal_form_and_read_by_no_hash` in `compiler/tests/fmt.rs`.
