# ADR-0107: Function values in the map: `passes` edges, the position of generated definitions, `display`

- Status: Accepted (autonomous, AUTONOMY §4 "decision records": an addition within the backward compatibility rule of ADR-0020 §1. Eighth execution plan "2V-20f autonomously accepted ADR on the map schema")
- Date: 2026-11-10
- Related: ADR-0020 (map schema), ADR-0053 (closures and function values), ADR-0059 (identity), backlog 2V-20g·2V-20h

## Context

- Closures and function values are lowered to a trait, a struct and an `impl` before checking (ADR-0053, `closures::lower`). The map sees the lowered program. So three things were missing.
  - Map L2 has no edge that says "this definition passes this closure (or function item)". The definition that passes a closure and the closure body are not connected in the map (`calls` reaches only the receiving generic function).
  - The `file`·`line` of a generated definition are `<generated>` line 1.
  - The path is an internal name (`<u::__closure_0 as u::__fn_0_0>::call`). Diagnostics call it `u::f::{closure#0}` since 2V-20g.
- The internal path does not change. It is the key of the cache, `--def` and identity (ADR-0059), and it is a query argument.

## Decision

1. **L2 `passes`.** There is one edge for each place that passes a function value.
   - `from`: the definition that has the argument in its body. It is the innermost one. A closure inside a closure is passed by the `call` of the outer closure.
   - `to`: the `call` of the closure, or the passed function item itself (`u::inc`).
   - The keys are the same as for other edges: `level`, `from`, `to`, `kind`.
2. **The position of generated definitions.** `file`·`line` are the place in the source that the definition stands for.
   - The struct of a closure and its `call`·`pre`·`post`: the closure.
   - The struct and methods of a function item: the name of that function.
   - The trait and method of a bound: the name of the bound parameter.
   - ADR-0020 §1 defines the position as "the position of the definition name". A generated definition has no name in the source, and `<generated>` line 1 pointed nowhere. So the blank is filled without changing the meaning of the key.
3. **`display`.** The key `display` is added to L0·L1 rows. It is the name by which diagnostics call the definition (`Tables::shown`), and it is present only when it differs from the path. Examples: `u::f::{closure#0}`, `..::ensures`, `<u::inc as u::apply::F>::call`, `u::apply::F`.
4. **Queries and tools take the internal path.** `display` is for reading. Accepting it as a query argument is not in the scope of this ADR.

## Rationale

| Option | Verdict |
| --- | --- |
| **Add the edge, the position and `display` (accepted)** | Within the backward compatibility of ADR-0020 §1 (adding a key, adding a `kind`). Identity and cache keys stay as they are |
| Change the map path to the display name | The meaning of the key changes (`version` bump). The cache, `--def` and identity change with it |
| Connect the closure body with a `calls` edge | The passing definition does not call the closure. The receiving function calls it. Analyses based on `calls` (W0903 and others) go wrong |

## Consequences

- `closures::Lowered::passes`·`at`, `Tables::passes`·`at`. The map's `Index::build` inserts the edges and picks the positions. `display` in `Entry`·`TypeEntry`.
- Test: `function_values_pass_and_sit_where_written` in `tests/map.rs`.
- Whether queries such as `impact` follow `passes` is not decided. If a need shows up, it is a decision on the query side.
