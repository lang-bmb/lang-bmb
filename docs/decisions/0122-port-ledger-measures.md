# ADR-0122: Definitions of the port ledger metrics

- Status: Accepted (approved by the maintainer 2026-11-11 "all accepted"). It defines the numbers of ADR-0116 decisions 2·3
- Date: 2026-11-11
- Related: ADR-0116 (gate B0), ADR-0111 (gates B·C), ADR-0113 (fixing), HEALTH.md S7, `dev-docs/measurements/port-ledger.md`, `scripts/components.py`, backlog 4D-08·4D-12

## Context

- ADR-0116 set that gate B0 looks at "the multiplier without contracts and the multiplier with proofs", "the share of proved definitions" and "undecided obligations are the majority of the authoring cost", but did not set how to measure them.
- If "the multiplier without contracts" is measured by writing two editions separately (an edition without contracts, an edition with proofs), the value depends on the order in which the editions are written and on who writes them, and the cost doubles.
- "Undecided obligations" and "authoring cost" have different units (a number of obligations, tokens).
- The status `verified` of `bmb verify` includes definitions that lean on escape hatches (`trust` is `assumed`·`external`).

## Decision

1. **Line counts are lines of the normal form (`bmb fmt`), and specification lines are separated by machine.** Specification lines are lines that only proof reads: `spec fn`·`proof fn` items, `proof { }` blocks, and lines that begin with a clause (`requires`·`ensures`·`invariant`·`decreases`), `assert(`, or the attributes `#[uses]`·`#[invariant]`·`#[trigger]` (`spec_lines` in `scripts/components.py`). The rest are executable lines.
2. **There are two multipliers.** Executable multiplier = executable lines / stage0 lines, total multiplier = port lines / stage0 lines. The stage0 lines follow the rule of port map §7.1 (before the first `#[cfg(test)]`). "The multiplier without contracts" of ADR-0116 is read as the executable multiplier, and "the multiplier with proofs" as the total multiplier. Two editions are not written.
3. **A proved definition is a definition proved without trust facts.** Only those whose `status` is `verified` and that have no `trust` key are counted. The denominator is all definitions for which a status line is emitted. This share is S7 and the "share of proved definitions of C2" of gate B0. Escape hatches (`#[external_body]`, `assume`) are counted separately.
4. **The proof cost is the obligation count and the sum of `rlimit`.** Both come from the status lines of `bmb verify`. The whole estimate of gate B0 multiplies the measured values of the component (obligations per line, `rlimit` per line) by the remaining stage0 lines and the total multiplier.
5. **The authoring cost is three session records.** The port of a component is written in a delegated session, and the records of that session are moved into `ledger` of `components.json`: session tokens (the sum reported by the harness), gate runs (the number of times `bmb verify` was run), runs that ended undecided (among those, the number of times the only remaining error was E0811).
6. **"Undecided obligations are the majority of the authoring cost" of ADR-0116 decision 3 is measured as "runs that ended undecided are the majority of the gate runs".** It is a ratio of two numbers with the same unit.
7. **Friction records the diagnostic code and the handling.** One line states what was hit, the diagnostic code and the handling (backlog ID, workaround, left as it is). Completeness defects of the judge go to the backlog.

## Rationale

| Option | Verdict |
| --- | --- |
| **Separate one edition by machine (proposed)** | It reproduces (the same file gives the same value). On the two files of the minimal experiment (`dev-docs/experiments/port-shape/`), the machine classification equalled the values counted by hand |
| Write the edition without contracts and the edition with proofs separately | The cost doubles, and the executable code changed for the sake of proof (guards, data structures) is mixed into the difference of the two editions |
| Undecided obligations against tokens | The units differ, so "majority" is not defined |

The executable multiplier can be larger than the multiplier of an "edition written without proofs": executable code put in for the sake of proof (such as length guards) is counted as executable lines. That code is written in the friction column.

## Consequences

- The columns of the port ledger change: specification lines, executable multiplier, total multiplier, obligations, `rlimit`, gate runs, runs that ended undecided.
- The `ledger` keys of `components.json` change to `gate_runs`·`undecided_runs` (there is no port yet, so there are no values to move).
- If accepted, the completion criterion of `4D-12` is read by these definitions.
