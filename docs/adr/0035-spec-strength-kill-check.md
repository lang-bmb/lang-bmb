# ADR-0035: The kill definition of the spec strength metric (BSB)

- Status: Accepted (autonomous, AUTONOMY.md §4: an implementation-level decision of the measurement procedure)
- Date: 2026-10-25
- Related: ADR-0021 (measurement procedure), ADR-0025 (`m1b`), ADR-0034 (G3), backlog 2A-10

## Context
G3 (spec quality) is measured as "the share of hidden mutants the agent's spec rejects". Putting the mutant body into the agent's function as is and proving again also breaks the invariants and auxiliary contracts of the reference solution, so almost every mutant comes out "killed" even when the spec is weak. Measuring the discriminating power of the spec itself needs a different definition.

## Decision
1. **Representation of a mutant:** a mutant is represented by its observed output on a witness input. The reference solution and the mutant are built with `bmb emit-llvm --all-checks` and run on the same inputs, and an input on which the outputs differ is recorded as a witness (at most 2).
2. **Kill:** if the agent's `requires`·`ensures` alone prove "the witness input is allowed, and on that input the mutant's output cannot occur", that mutant is killed. Each witness is an independent call, so this means "the mutant is excluded on at least one witness". If the workspace is not proved or the signature is wrong, the kills of that task are 0.
3. **Scope:** the W0904 mutation operations are applied to the bodies of the target definition and of the definitions it (transitively) calls.
4. **Discarded mutants:** not distinguished on the fixed inputs (treated as equivalent), differing only by a trap or a hang, a different output that still satisfies the hidden reference contract (another allowed answer), frontend failure. The mutants kept are confirmed by the same method to be killed by the hidden reference contract (the kill rate of the reference contract is 100%).
5. **Frozen:** the mutant files are in `bench/m1b/mutants/` with a separate MANIFEST, and `bsb_gen.py --check` confirms the determinism of regeneration. The `m1b` MANIFEST is not changed.

## Rationale
- The body substitution approach measures the proof structure, not the spec. The observation-based definition answers the same question as nl2postcond·POSTCONDBENCH, "does the spec reject a wrong implementation", with BMB's proofs.
- The discard rules remove from the denominator the mutants that need not be killed (equivalent, another allowed answer), so the metric does not sway with task design.

## Consequences
- 186 mutants (426 generated). Reference contract kill rate 100%, weak control 24.2%. The gate checks both values.
- Limits: the mutation operations are only the W0904 set (no variable replacement or statement deletion), and equivalence is a test on fixed inputs, not a proof.
