# 07 Proof gate

This chapter covers the verdicts the proof gate (`bmb verify`) gives on a program that has passed the check: the kinds of proof obligation each definition produces, what a caller assumes about its callee, the proof context (`#[uses(..)]` and the default context), the verdicts (proved, failed with a counterexample, undecided), determinism, and how `build` and `emit-llvm` use the verdicts. The diagnostics here are E08xx. The syntax for writing specifications is in [06 Contracts and specifications](06-contracts-and-specs.md), and the trust level that escape hatches give a verdict is in [08 Effects and escape hatches](08-effects-and-escape-hatches.md).

## Unit of obligation

- The unit of proof is the definition. An executable function and a `proof fn` produce obligations by symbolic execution of the body, and a recursive `spec fn` has a termination obligation. A generic definition is proved once, with its type parameters left as unspecified types (ADR-0046, `tests/verify/pass/generics.bmb`).
- Each obligation is solved separately, from a clean solver state. The assumptions are the facts gained on the path up to that point: the function's `requires`, the ranges of the parameter types, the conditions passed, earlier `assert`s, the `ensures` of calls, the invariants of loops.
- After the diagnostics, `bmb verify` prints a status line for each definition: `{"def":..,"status":"verified"|"failed","obligations":n,"failed":k,"rlimit":..,"rlimit_max":..}`. A definition with no obligations is `verified` with `obligations` 0 (the last lines of `tests/verify/fail/loops.jsonl`). The line adds keys only where they apply, in this order: `"trust"` (below), `"escapes":n` for a definition with escape hatches (its `assume` statements, one more for `#[external_body]`), whether the proof needed them or not, `"termination":"unproved"` for a definition that has a `loop` without `decreases` or calls one (ADR-0094), and `"backstop"` (below). A program is proved without trusted facts and with its termination proved when every line is `verified` and none has `trust`, `escapes` or `termination` (ADR-0160, `tests/verify/fail/verdict_rows.jsonl`). The map gives the same definitions its `trust`, `escapes` and `termination` keys; there a definition with no obligations has `status` `none`.

## Kinds of obligation

| Kind of obligation | When it arises | Failure diagnostic |
| --- | --- | --- |
| Precondition of a call | At every place that calls an executable function, a `proof fn`, a function value (`f(a)`) or a trait method through a bound: each `requires` clause of the callee | E0801 |
| Postcondition | At every exit of the function (tail expression, `return`, `?`): each `ensures` clause. The `ensures` of a `proof fn` is the same | E0802 |
| Overflow | `+`, `-`, `*` and unary `-` in executable code, a division of the form `i64::MIN / -1`, a shift amount outside the width | E0803 |
| Division by zero | The divisor of `/` and `%` in executable code | E0804 |
| Index and range | `a[i]`, a subslice `&a[lo..hi]`, the range of `copy_within` | E0805 |
| Invariant on entry | At the place where a loop is entered: each `invariant` clause | E0806 |
| Invariant preserved | At the end of the loop body (including `continue`): each `invariant` clause | E0807 |
| Loop decrease | At the end of the loop body: the `decreases` value decreases and is not negative | E0808 |
| Decrease of a recursive call | At a place that calls an executable function or a `proof fn` of the same recursive component: the measure decreases | E0809 |
| Assertion | `assert(e);` | E0810 |
| Termination of a `spec fn` | At a recursive call inside the body of a recursive `spec fn`: the measure decreases | E0813 |
| Range of a conversion | At `x as T` in executable code: the value is within `T`. To `f64`: the 64-bit integer is at most 2^53, so the conversion is exact | E0815 |
| Capacity of a `Vec` | The `n` of `Vec::with_capacity(n)` is within the maximum length. `push` and `push_str` have no obligation: if the length exceeds the maximum the program ends (the same level as an allocation failure), and the proof gains as a fact that the length is within the maximum after `push` (ADR-0124, `tests/verify/pass/unbounded_push.bmb`) | E0816 |
| Refinement of a trait contract | For each function of a trait impl: the trait's precondition implies the impl's precondition, and the trait's precondition and the impl's result imply the trait's result | E0817 |
| Bit-vector subproof | The separate query of `assert(e) by (bit_vector);` | E0818 |
| Range of a ghost argument | An argument of a `proof fn` call is within the parameter type | E0819 |
| Character boundary | Both positions of a string range `&s[a..b]` are character boundaries and `a <= b` | E0820 |
| Type invariant | At the places where a value of a type with `#[invariant(p)]` leaves: a call argument, a return, a `&mut` parameter at an exit, a loop head | E0823 |

- Corpus (code: file): E0801, E0802, E0809, E0810, E0813: `tests/verify/fail/contracts.bmb`. E0803, E0804: `tests/verify/fail/arith.bmb`, `tests/verify/fail/widths.bmb`. E0805: `tests/verify/fail/bounds.bmb`, `tests/verify/fail/subslices.bmb`. E0806, E0807, E0808: `tests/verify/fail/loops.bmb`. E0815: `tests/verify/fail/conversions.bmb`. E0816: `tests/verify/fail/vecs.bmb`. E0817: `tests/verify/fail/trait_contracts.bmb`. E0818: `tests/verify/fail/bit_vector.bmb`. E0819: `tests/verify/fail/proofs.bmb`. E0820: `tests/verify/fail/strings.bmb`. E0823: `tests/verify/fail/invariants.bmb`. The `.jsonl` next to each file holds the expected diagnostics.
- Specification expressions and ghost code have no overflow, division-by-zero or index obligations, because their arithmetic is over mathematical integers (ADR-0017). Wrapping arithmetic (`wrapping_add` and so on) and checked arithmetic have no overflow obligation either (`tests/verify/pass/arith_methods.bmb`).
- Whether the arms of a `match` cover every variant is not a proof obligation. The checker decides it (E0426, chapter 04).
- A fact already in the path condition is not emitted as an obligation. Code that only moves a value with a type invariant has no E0823 obligation (`tests/verify/pass/invariants.bmb`).

## What the caller assumes

A call proves the callee's `requires` (E0801) and assumes only its `ensures` (ADR-0017). It does not look at the callee's body. About the result of a function with no contract, the caller knows only the range of the type and the type invariant. For a `&mut` argument, only what the `ensures` says remains after the call.

```
#[summary("the caller sees only the contract")]
mod m {
    #[effects(pure)]
    fn one() -> i64 { 1 }

    #[effects(pure)]
    fn two() -> (r: i64)
        ensures r == 2,
    {
        one() + one()
    }
}
```

`bmb verify` reports E0802 (`postcondition `r == 2` may not hold in `m::two``) and E0803 (`` `+` may overflow `i64` ``) on `two`. With `-> (r: i64) ensures r == 1,` written on `one`, both are proved.

The same rule holds for loops: the code after a loop knows only the invariants and the condition under which the loop exited. A call through a bound uses the contract of the trait, a call on a concrete type uses the contract of that impl function, and a call of a function value uses `f.requires` and `f.ensures` (`tests/verify/pass/trait_contracts.bmb`, `tests/verify/fail/fn_values.bmb`).

## Proof context

The definition of a `spec fn` is unfolded only when it is in the proof context; otherwise the solver sees that function by name only (ADR-0017 §4). The context of a definition is the union of the following two.

- **Default context (ADR-0095 A1).** The `spec fn`s called by the definition's own contract clauses, loop invariants, `assert`s and `proof` code, and the `spec fn`s called by the `requires` and `ensures` of its callees. It does not go into the `spec fn`s that the bodies of those `spec fn`s call in turn (`tests/verify/pass/default_context.bmb`).
- **Names written in `#[uses(..)]`.** They only add and are not transitive: `#[uses(a)]` unfolds the body of `a` and does not unfold the `spec fn`s that body calls. One from another unit is written `unit::name`, one from a trait impl `<T as Tr>::name` (`tests/verify/pass/uses_impl_path.bmb`). A name that is not a `spec fn` is a check error (chapter 06, E0509).
- **Closures (ADR-0114).** The `spec fn`s called by the clauses a closure writes are in that closure's default context, and a closure inherits the `#[uses(..)]` of the function that writes it (`tests/verify/pass/closure_context.bmb`). This rule does not apply when a function item is passed as a value (it follows that function's own contract).
- The predicate of a type invariant follows the same rule. If its definition is needed, write it in `#[uses(p)]`.

```
#[summary("proof context")]
mod m {
    spec fn small(n: u64) -> bool { n < 10 }
    spec fn bounded(n: u64) -> bool { small(n) && n > 0 }

    #[effects(pure)]
    fn f(n: u64) -> (r: u64)
        requires bounded(n),
        ensures r < 20,
    {
        n + n
    }
}
```

`bounded` is unfolded because a clause of `f` calls it, but `small` is inside that body and so is a name only. `bmb verify` reports E0802 (`counterexample: n = 10`) and an E0803 diagnostic, and the `fix` of the diagnostics points to `#[uses(small)]`. With `#[uses(small)]` written before `f`, it is `verified`. The tools that fill in and remove `#[uses]` are in `compiler/README.md`.

## Verdicts

The verdict on one obligation is one of three.

- **Proved.** Only when the solver answers `unsat` for the negation.
- **Failed with a counterexample.** On `sat`, a diagnostic is reported with the code of that obligation (E0801 to E0823), and the input values are written at the end of the message: `counterexample: a = -9223372036854775808, b = -1`. If there is no input to show (the failure comes from the definition's own values or from values a call or a loop leaves open), it is reported without a counterexample. If an opaque `spec fn` is involved, the `fix` names it.
- **Undecided.** If the solver answers `unknown` or a resource limit is exceeded (the per-obligation `rlimit`, the memory limit of the solver process: ADR-0123; an answer that has not ended after 1,000,000 lines: ADR-0151), it is reported as not decided (E0811). The diagnostic also carries the code of the original obligation, as in `"obligation":"E0802"`, and gives the solver's reason. If there is a counterexample that was not confirmed, it is written as `unconfirmed counterexample` (`tests/verify/fail/nonlinear.bmb`, `tests/verify/fail/proofs.jsonl`).
- **The gate could not run.** If the solver is missing, is not the pinned version or ends with an error, that fact is reported (E0812). This code has no case fixed in the corpus (a diagnostic that depends on the environment).

Undecided (E0811) and the gate not being able to run (E0812) are failures (`docs/DESIGN.md` §6: unknown, timeout and a missing solver are all failures). The `status` of a definition is `failed` if even one obligation is not proved.

```
#[summary("counterexample")]
mod m {
    #[effects(pure)]
    fn div(a: i64, b: i64) -> i64 { a / b }
}
```

The output of `bmb verify` is E0803 (`` `/` may overflow `i64` in `m::div`; counterexample: a = -9223372036854775808, b = -1``), E0804 (`divisor may be zero in `m::div`; counterexample: a = -9223372036854775808, b = 0`), and `{"def":"m::div","status":"failed","obligations":2,"failed":2,..}`.

## Determinism

- The solver is Z3 at a pinned version, and the resource limit `rlimit` is the same for every obligation. Both values are in `scripts/toolchain.env` and are built into the compiler. The random seed is fixed at 0. A verdict is cut off by `rlimit`, not by time.
- A wall-clock safeguard (300 seconds per query) stops a query whose units tick slowly. A query it ends is undecided with a reason that says the verdict depends on the machine, and the status line counts such queries (`"backstop":n`). The units such a query had spent are the clock's, so they count 0 in `rlimit` and `rlimit_max`, and a proof with such a query is not cached (ADR-0152).
- For each obligation the solver state is reset and the whole base, options included, is sent again. So the verdict on one definition does not depend on whether another definition was proved first.
- Diagnostics are sorted by position and duplicates are removed. The same input gives the same bytes. The `rlimit` values of the status lines are also fixed in the failure corpus (`tests/verify/fail/arith.jsonl` and others).
- The symbols of a query are canonical names in order of first appearance, so the query text of a program that only renames definitions or locals is the same (ADR-0063).

## `build` and `emit-llvm`

- `bmb build` makes an executable only when every definition is `verified`. If even one fails, it ends with `build refused: the program is not fully proved` and writes no file (exit code 1). Undecided (E0811) and the gate not being able to run (E0812) are included here.
- A definition that became `verified` by relying on an escape hatch (`assume`, `#[external_body]`) gets `"trust":"assumed"` or `"external"` on its status line and does not block `build`. That definition and the definitions it calls do not have their runtime checks removed (chapter 08, `tests/run/escapes.bmb`).
- `bmb emit-llvm` writes IR in which the checks of proved obligations are removed and unproved obligations remain as runtime checks (trap). If there are failures it reports the diagnostics and the exit code is 1. A proof assumes the `requires` established by the caller and the `ensures` proved by the callee, so a failed definition and the definitions connected to it by calls (both the calling side and the called side) keep the checks of proved obligations too (`usable_proofs` in `compiler/src/lib.rs`).
- `bmb emit-llvm --all-checks` does not run the proof and leaves every execution obligation as a runtime check. Since it does not trust contracts, a program that is not proved still runs as it is, and the exit code is 0. No E08xx diagnostics and no status lines are printed (the command list in `compiler/README.md`).
- What becomes a runtime check is an execution obligation: overflow, division by zero, out-of-range index, conversion, shift amount, the capacity of `with_capacity`. Contract obligations (`requires`, `ensures`, `assert`, invariants, measures) are not executed. An unproved contract does not remain as a check; the execution obligations that relied on that contract remain as checks (`tests/conformance/07/emit_all_checks_skips_the_proof.bmb`).
- `assume` is not executed either. A false `assume` shows up as a trap, because the checks that would have been removed by relying on it remain (chapter 08, `tests/conformance/08/false_assume_ends_in_trap.bmb`).
- Only a proved obligation removes a check (`docs/DESIGN.md` §3 principle 9). Programs fixed down to their execution output are in `tests/run/` (`tests/run/collatz.bmb`).

The proof cache, incremental proof, proving only the definitions chosen with `--def`, the stability check that varies the seed, and the contract lint are tools. They are in `compiler/README.md`.
