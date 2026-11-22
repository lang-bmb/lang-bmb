# ADR-0049: Early exit (`return`·`break`·`continue`·`?`) and `for`·`loop`

- Status: Accepted (2026-10-26, approved by the maintainer "1/2 adopted, approved", backlog 2V-12)
- Date: 2026-10-26
- Related: ADR-0045, DESIGN.md §4.1 ("failure", "expression"), ADR-0031 §2 (free points), 2R-04f (`for` offline comparison), review R9, backlog 2V-09


## Confirmed at acceptance (2026-10-26)
- Diagnostic numbers: E0241 `?` with a different error type (confirmed free). E0203 (`return` forbidden)·E0204 (`for`·`loop` forbidden)·E0206 (`?` forbidden) retire together with the implementation (`diag::RETIRED`, no reuse). E0206 is not used with another meaning.
- Open question 1 (`if let`·`while let`) stays as is, because its rationale is the ledger's "expression" (one normal form, `match`).
- The normal form of counted loops (range `for`) is written into the ledger's "expression" row together with this acceptance.
- Implementation phase: ROADMAP 2K (control flow).

## Context
BMB has no `return`·`break`·`continue`·`?`·`for`·`loop` (E0206, E0207 "`return` is not supported in milestone 1" and so on). So a conditional early return is written with nested `if` and flags, and a counted loop with `while` + `decreases`. In the self-hosting analysis, the 518 sites of `?`·`return`·`break` are rated "high" in cost. What the ledger forbids is only the implicit `From` conversion of `?` (it hides the failure path), not early exit itself. Verus and Dafny support early return and `break`, and VC generation for them is a standard technique.

## Decision

### 1. Early exit
- `return e;`·`return;` (unit), `break;`·`continue;` inside a loop, the label `'outer: while ..` with `break 'outer;`·`continue 'outer;` (the same as Rust).
- `e?`: if `e: Option<T>` and the function result is `Option<U>`, it returns `None`, and if `e: Result<T, E>` and the result is `Result<U, E>` (the same `E`), it returns `Err(e)`. If the error types differ it is rejected (ledger "failure": no implicit `From`), and the hint is `match` or an explicit conversion function.
- `loop { .. }` and `break e;` with a value (only in `loop`, the same as Rust). `loop` also takes `invariant`·`decreases`.

### 2. `for`
- The normal form of a counted loop is `for i in lo..hi { .. }` and `for i in lo..=hi { .. }` (the type of `i` is the integer type of `lo`·`hi`). It takes an `invariant` clause. `lo <= i < hi` is an automatic fact inside the body and `decreases` is automatic (`hi - i`, principle 6).
- `while` remains for other loops. A `while` counted loop that does the same job gets a mechanical fix that turns it into `for` in the normal form (principle 8: one form per job).
- The iterator forms `for x in v.iter()`·`for x in &v` are not provided (an iterator is a structure that holds a reference: the payback of ADR-0015. One normal form of the ledger's "expression"). The hint is `for i in 0..v.len()`.

### 3. Proof encoding
- `return`: under the path condition at that point, `ensures` (with the return value substituted) is an obligation and the path ends. The result at the end of the function is the join of the remaining paths.
- `break`: the loop exit state is the join of the normal exit (invariant ∧ ¬condition) and the state at each `break` point (the symbolic state of that iteration). `continue` carries the invariant preservation obligation at that point.
- `?`: the same as its expansion into `match` (the `None`/`Err` arm is a `return`).
- `for`: the same as the `while` encoding with the range fact and the automatic `decreases` added. The measurement of 2R-04f, that the unfolded SMT queries are identical, goes into the completion criteria.

### 4. Ownership and freeing
- The owned locals of the blocks an early exit passes through are freed on that exit edge. The free point is still fixed by the text (ADR-0031 §2). An exit edge is also a join point (the E0414 rule applies per edge).

### 5. Code generation
- A branch to the exit block and freeing per edge. `?` is a tag comparison and a branch.

### 6. Diagnostics
- E0206 (`?` forbidden) and the "`return`·`for`·`loop`·`break` is not supported" messages go away with acceptance. The code E0206 is not reused as is; whether to use it for "`?` with a different error type" or to use a new number is decided at acceptance (changing the meaning of an existing code is an approval item).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **All of Rust's early exits + the range `for` normal form (proposed for acceptance)** | The Rust surface as is (S1), with verification precedent. Only the `From` of `?` is left out, per the ledger |
| Nested `if` and flags as now | A workaround. Lines and tokens grow and Rust habit errors occur (the E0206 frequency is to be measured in 2B-10·1E) |
| Iterator `for` | An iterator that holds a reference conflicts with second-class references (ADR-0015). The range `for` does the same job |

## Consequences
- The normal form is written into the "expression" row of the ledger of DESIGN §4.1: a counted loop is a range `for`, and the rest are `while`/`loop`.
- 2R-04f (`for` offline comparison) becomes measurement input for this ADR.
- Fuzzer: `break`·`continue`·`return`·`?` are added to the loop fuzzer.

## Open questions
1. `while let`·`if let` (left out by ADR-0026): whether `?` and `match` are enough is decided after the 2B-10 measurement.
