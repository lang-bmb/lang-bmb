# ADR-0094: Loops that run until the input ends

- Status: Accepted (2026-11-06, approved by the maintainer "proceed, leaving an unproved mark". Backlog 4C-04)
- Date: 2026-11-06
- Related: ADR-0016 (`decreases` required for loops and recursion), ADR-0030 (effect `nondet`, test-needed area §4), ADR-0049 (`loop`), ADR-0091 (system interface), port map P11 (MCP server)

## Context

- 4C-02 (a) brought in standard input. A line loop that reads to the end (`loop { match std::io::read_line(&mut l) { Ok(0) => break, .. } }`) has no measure, because its termination depends on outside input. ADR-0016 requires `decreases` on every loop, so it was blocked with E0506.
- Self-hosting needs this shape: the request loop of the MCP server (P11), line-oriented tools. Until now one had to read the whole input and loop over the bytes, or put a limit on the number of lines (`tests/tools/wc.bmb`).
- Verus does not require a termination proof for loops in executable code (only in specification and proof code). Rust has no notion of termination.

## Decision

1. **A `loop` in a `nondet` definition may omit `decreases`.** Only a `loop` (not the statements `while`·`for`) inside an executable function whose declared effects include `nondet`. Elsewhere it is E0506 as before, and the E0506 hint for `loop` points to this way.
2. **It is proved as partial correctness.** The invariants, obligations and postconditions of that loop are proved as usual, and only the measure obligation is absent (if it terminates, it is correct).
3. **A mark is left.** A definition with such a loop, and the definitions that call it (transitively), get `"termination":"unproved"` in their L1·`status` rows (the key only when it applies, ADR-0020 §1), and `test_reasons` contains `nonterminating` (the test-needed area of ADR-0030 §4). LLVM `willreturn`·`mustprogress` are not attached to such definitions (a condition for when 2F-04 of ADR-0084 handles it).

## Rationale (alternatives considered)

| Option | Verdict |
| --- | --- |
| **Omit the measure only for a `loop` in a `nondet` definition + a mark (adopted)** | It opens only where termination really depends on the outside (definitions that read input). The proof remains as partial correctness, and the unproved area is visible in the map |
| Keep the rule (read everything and loop, a limit on the number of lines) | The server loop (P11) cannot be written. A limit is an arbitrary value and creates a false termination |
| A fuel argument (`fuel: u64`) | A meaningless argument is attached to every definition, and what to do when the fuel runs out is yet another behaviour to decide |
| Optional measure in every executable loop | The total function guarantee (DESIGN.md §4.1) is lost. The proof gets weaker even for loops unrelated to outside input |

## Consequences

- A loop that reads to the end with `std::io::read_line` can be written. Test `input_loops_run_until_the_end` in `compiler/tests/system.rs`.
- The map, `status` and `tests` count definitions with unproved termination. These definitions enter the unproved area of the self-proof (4F).
