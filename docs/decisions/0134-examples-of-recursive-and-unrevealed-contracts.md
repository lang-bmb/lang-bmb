# ADR-0134: The examples tool answers recursive, quantified and unrevealed contracts; `bench/intent` re-frozen

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": the way the tool queries, and a re-freeze before the official run following the procedure set by ADR-0130. The verdict criteria and the prompt are not changed)
- Date: 2026-11-14
- Related: ADR-0130 (H-INTENT set), ADR-0133 (recursive function form), ADR-0108 (bounded search), ADR-0078, backlog T1-44g, `dev-docs/measurements/t1-44f.md`

## Context

- `bmb examples` produced no `allowed` row for 12 of the 90 items of `bench/intent` (10 `clean`, 2 `defect`). That difference was a shortcut that does not read the contract: "`ok` if there are no examples" (ADR-0130).
- Of the 94 executable functions with contracts in `lib/std`, only 34 produced rows. For 18, the contract calls a `spec fn` not in `#[uses]`, so no query was even made (`t1-44f.md`).
- Three causes: (1) the model search of the solver ends in `unknown` on the quantified axioms of recursive definitions, (2) in queries where the solver has to find the inputs too, quantified formulas over a whole array are not decided, (3) the tool read the contract not by its meaning but in the context of that function's proof (`#[uses]`).

## Decision

1. **The queries of examples also write recursive definitions as recursive functions** (the form of ADR-0133, the same step of `sat_of`).
2. **A query the solver did not decide is asked again with written-in inputs** (`examples::written`, only queries whose inputs are not bound, up to 8 per query). The n-th input is determined from the sequence of lengths 1·2·3·0·4 and a fixed order of 12 small integers. A scalar array is given not as element-wise equalities but as one array value (a chain of `store`): only then are quantified formulas over the given array decided. For an input outside the type or the `requires`, the query is `unsat`, so it moves on to the next one. The solver finds only the outputs for the given inputs.
3. **A `spec fn` that the contract calls is read as its definition regardless of `#[uses]`.** `#[uses]` is what the proof of that function sees; to callers and readers, the meaning of the contract is the definition of that `spec fn`. Only those whose definition cannot be given remain with the reason "by name only".
4. **The examples of `bench/intent` are frozen again.** It is before the official run, so it follows the procedure of decision 5 of ADR-0130. The items, intents, programs, prompt and judgement are unchanged; what changed is the `examples` text of 13 items, the `generated_with` of `set.json`, and the MANIFEST.
5. Types whose values are not visible (`Vec`·`Option`·those containing structs, 40 functions of `lib/std`) are outside the scope of this decision (backlog `T1-44i`).

## Consequences

All are values measured deterministically.

| The 90 of `bench/intent` | Before (`ba563ed4`) | After |
| --- | --- | --- |
| Items with no `allowed` row (`clean` / `defect`) | 10 / 2 | 0 / 0 |
| Items with no row at all | 4 / 1 | 0 / 0 |
| `undecided` > 0 | 10 / 3 | 4 / 3 |
| Average number of rows | 6.67 / 9.58 | 7.93 / 9.93 |
| Token ratio (b)/(a) | 1.571 | 1.662 |
| Items that "two `allowed` results for one input means a defect" gets right | 82 | 84 |
| `rlimit` sum of the summary rows | 402,837,669 | 720,250,384 |

- The order in which the 12 were resolved: 7 by the recursive function form, 3 by written-in inputs, 2 by array values.
- **It moved further from the token criterion** (1.571 → 1.662): because there are more examples. The criterion and its handling stay as in ADR-0132.
- **The surface-look rule fits better** (84/90, detections 44/45, false alarms 5/45). The detection rate of `spec+examples` is still read together with the value of this rule.
- `lib/std`: functions that produce rows 34 → 52 (36 with recursion and written-in inputs, 52 with reading as definitions). The remaining 42 are 40 for types, 2 with no rows.
- Independent check: a recursive sum contract under a quantified `requires` (`d::total`) was added to `tests/examples.rs` `shown_examples_hold_at_the_gate`. The gate proves assertions that write the values of the rows as literals.
- Cost: generating the set 23 seconds → 55 seconds. One query that is not decided asks 8 more written-in inputs.
