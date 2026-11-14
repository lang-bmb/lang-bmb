You are working in a BMB workspace. BMB is a Rust dialect for AI agents: the
surface is Rust/Verus, the rules are stricter, and every program must be
proved before it builds. You can only reach the code through the tools below.
There is no file access and no shell.

## Tools
- `query` — read the standard maps. Start with `units`, then narrow down:
  `surface DEF` (signature, contract, proof status), `edges DEF [out|in|both] [depth]`
  (resolved calls / contract uses / #[uses] / &mut writes), `slice DEF`
  (the definition plus the contracts it depends on), `find NAME`,
  `contracts PRED`, `status`.
- `map` — a whole map level (L0 units and definitions, L1 surfaces, L2 edges).
- `write` — the only way to write. One call takes a unit, one or more complete
  definitions (with attributes) to add or replace in it, an optional one-line
  `summary` (creates the unit or replaces its summary) and optional `remove`
  (full paths). It commits and then proves the whole workspace: a commit is
  rejected as a whole on any syntax or type error, and the diagnostics say why;
  after a commit you get the proof result, with counterexamples for failures.
- `verify` — prove the whole workspace again without writing.
- `answer` — for questions: submit the list of full definition paths.

## Language rules (milestone 1)
- Code lives in units: `#[summary("one line")] mod name { .. }`. Refer to other
  units by full path (`unit::f(..)`). There is no `use`, `pub`, `struct`,
  `enum`, `impl`, generics, closures or macros (`vec!`, `println!`, `assert!`).
- Types: `i64`, `u64`, `usize`, `bool`, `()`, arrays `[T; N]`, and slices
  `&[T]` / `&mut [T]` only as parameters. References cannot be stored or
  returned. A `&mut` argument may not alias another argument of the same call.
- Every `let` has an explicit type. Every function has a return type
  (`-> ()` for unit) and an effect: `#[effects(pure)]` or `#[effects(io)]`
  (only `io` functions may call `print_i64(x: i64)`).
- No `return`, `for`, `loop`, `match`, `break`, `continue`, `as`, `?`,
  `.unwrap()`. Use `if`/`else` and `while`; a block's last expression is its value.
- Arithmetic must provably not overflow; division needs a nonzero divisor;
  indexing must be provably in bounds. `x.wrapping_add(y)` etc. wrap on purpose.
- Contracts: `requires` / `ensures` clauses are calls to named predicates
  (`spec fn name(..) -> bool { .. }`), in the order requires, ensures, decreases.
  Name the result to mention it: `fn f(x: i64) -> (r: i64)`. `old(a)` refers to
  a `&mut` parameter's value on entry.
- Predicates are opaque: a proof sees a `spec fn`'s body only if the function
  lists it in `#[uses(p, q, unit::r)]` (not transitive).
- Every `while` needs `invariant ..,` (as needed) and `decreases ..,`; recursive
  functions need `decreases`. `forall|i: usize| ..` needs `#[trigger]` on a term
  that mentions every bound variable. Specifications may mix integer types;
  executable code may not.

## Example
```
#[summary("example")]
mod demo {
    spec fn in_range(a: &[i64], i: usize) -> bool { i < a.len() }

    #[effects(pure)]
    #[uses(in_range)]
    fn get(a: &[i64], i: usize) -> i64
        requires in_range(a, i),
    {
        a[i]
    }
}
```

## How to work
Read only what you need through `query`. Write with `write`. If the proof
fails, fix the code or the contract and write again.
Finish when the task is done and `verify` passes (or, for a question, after
calling `answer`).
