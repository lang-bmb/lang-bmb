# ADR-0151: A line limit on the solver's answers

- Status: Accepted
- Date: 2026-11-25
- Related: ADR-0017 §5 (the solver process), ADR-0094 (loops that run until the input ends), ADR-0123 and ADR-0129 (the memory limit), DESIGN.md §6

## Context

- The solver connection reads one answer as lines until the parentheses balance and no string literal or quoted symbol is open, and it reads what a process still writes after `(exit)` to the end of its output. Both loops ran until the solver stopped writing. A solver that writes without end (a fault, or a process that is not Z3) kept the gate reading forever, and the memory of the answer grew with it.
- In the BMB port of the connection these are the only two loops without a measure. Under ADR-0094 each was proved for partial correctness and marked `termination: unproved`, and the mark spread over every definition that reaches them: 58 definitions of the port, the whole proof driver, the lint, the examples, the stability check, `lib::verify` and `main::main` (counted from the `termination` key of `bmb map L1 selfhost/*.bmb`).
- Unlike the input loops of ADR-0094, an answer of the solver has a shape the connection itself asks for. Every command is sent with `:print-success`, and the only answers of more than one line are the values of `get-value` (models). Over the gate's corpus (the files of `tests/verify`, the programs of the port's proof cases, the lint and the examples over them: 103,117 answers) the longest answer is 42 lines and 1,218 bytes. Statistics are never asked for, and Z3 writes an error on one line.

## Decision

1. **One answer takes at most 1,000,000 lines** (`ANSWER_LINES` in `compiler/src/smt.rs`, `answer_lines` in `selfhost/solver.bmb`). Blank lines before an answer count.
2. **At the limit the answer is undecided.** The read stops with the error `z3 answer too long: no complete answer in 1000000 lines`. It is a limit like the resource limit and the memory limit: the obligation is undecided (E0811) with the reason `answer limit reached, 1000000 lines`, never proved, and what follows an undecided query (the ground-instance retry, the bounded search) follows it.
3. **The process is retired.** Its output is in the middle of an answer, so it answers nothing more: both pipes are closed and it is waited for, so a process still writing ends at the closed pipe. The commands that follow in the same query get the same error without being sent, and the next `(reset)` starts a new process. A query asked again alone after the memory limit that meets the limit ends the same way.
4. **What a process writes after `(exit)` is read for at most the same number of lines**, then its pipes are closed and it is waited for. A real solver writes one line there.
5. Both compilers do the same, so the port stays byte-identical to the Rust compiler; the two read loops of the port are proved to terminate, and no definition of the port is marked `termination: unproved`.

## Rationale

- The value is a count of lines, because the loop reads one line at a time and its measure is the lines left; a byte limit would need a read that stops inside a line. 1,000,000 is more than 20,000 times the longest answer the gate gets, and an answer that long is at least a megabyte of model, far more than any counterexample shows. It is also small enough that reaching it costs a second: the differential case that reaches it on both sides runs in about that.
- Reaching it says nothing about the obligation, so the only honest verdict is undecided, as for the other limits. It is deterministic: the same output gives the same verdict on every machine.
- Alternative: no limit, termination left unproved. Rejected: a fault of the solver hangs the gate, and the mark hides, among 58 definitions, whether any other loop of the driver lacks a measure. ADR-0094 rejected a line limit for input whose length is the user's; the length of an answer is not, and the limit has a defined meaning.
- Alternative: a wall-clock limit only. The process already has one (`-T`, 660 seconds), but it is a timer inside Z3, which binds no other program and no fault of Z3's own, and a clock is no measure: the loop's termination would still be unproved, and a verdict by time depends on the machine.
