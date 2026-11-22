# ADR-0016: Scope of the first milestone: language subset and tool queries

- Status: Accepted
- Date: 2026-10-24
- Related: all of DESIGN.md, ADR-0009·0012·0015, genesis §5·§6-1

## Context
The first milestone (M1) measures "the agent writes proved code without files and without tests" at a scale of a few dozen definitions (genesis §5). For that, the language subset that goes into the minimal end-to-end path (parsing → type and ownership checking → verification conditions → Z3 → LLVM IR) and the list of tool queries the agent uses must be fixed first.

There are three selection criteria.
1. Is it needed to make principle 4 (the proof gate) and principle 6 (through tools) *measurable*?
2. Can it show principle 9 (proof → check removal) in the IR at least once?
3. If the two above still hold without it, it is left out.

## Decision

### 1. Language subset

**Included**

| Area | Included items | Notes |
| --- | --- | --- |
| Units | `mod name { ... }` declaration | Independent of file paths. A one-line summary is mandatory for each unit (for L0) |
| Definitions | `fn`, `spec fn` | `spec fn` is opaque by default |
| Types | `i64`, `u64`, `usize` (64-bit), `bool`, `()` | Every local variable has an explicit type |
| References | Only `&[i64]`, `&mut [i64]` in argument position | Second-class references (ADR-0015). Cannot be stored or returned |
| Statements | `let`/`let mut` (type mandatory), assignment, `if`/`else`, `while` | One normal form (principle 8) |
| Arithmetic | `+ - * / %` (proof of no overflow and non-zero mandatory), `wrapping_add/sub/mul` | No `as` conversion |
| Indexing | `a[i]` (bounds proof mandatory), `a.len()` | The target of the check-removal demonstration |
| Calls | Direct calls resolved by path, self-recursion | No method lookup or auto-ref |
| Specification | `requires`, `ensures`, named result `-> (r: T)`, `invariant`, `decreases` (mandatory for loops and recursion), `old(x)`, `forall` + explicit trigger, `assert` | Contract clauses allow only calls to named predicates (`spec fn ... -> bool`) |
| Proof context | Declaration of the facts each definition uses (syntax undecided, for example `#[uses(p, q)]`) | Facts outside the declaration are not passed to Z3 |
| Effects | `#[effects(pure)]`, `#[effects(io)]` | Mandatory on every `fn` |
| Boundary | One `io` builtin (`print_i64`) | Counted as 1 for the unproved area and the area that needs tests |

**Excluded (after M1)**

- Structs and enums (including `Option`), generics, trait·impl
- Heap, `Vec`, strings, and all types where a move of ownership has meaning
- Escape hatches (`assume`, `external_body`, `unsafe`). The unproved area of M1 is only the `io` boundary
- The syntax of rationale declarations (ADR-0006), extension maps, optimization passes, self-hosting

Structs and enums are the first extension candidate right after the M1 measurement. The absence of `Option` is expected to be the first thing agent tasks run into.

### 2. Tool queries

Every output is normalized JSON. Keys are sorted, and there are no nondeterministic elements such as timestamps or paths. The same code must produce a byte-identical result.

**Read**

| Query | Map | What it returns |
| --- | --- | --- |
| `units` | L0 | Unit hierarchy, names, one-line summaries |
| `surface(def)` | L1 | Signature, contract predicate names, effects, proof status |
| `edges(def, direction, depth)` | L2 | Resolved call edges (forward and reverse, transitive) |
| `slice(def)` | — | Body + the contracts it depends on (+ the predicate definitions of the declared proof context). Size and ratio to the budget |
| `find(name)` | L0 | Definition location and hash. One definition per name |
| `contracts(predicate)` | Contract view | Where this predicate is ensured and where it is required |
| `status` | — | Proof status per definition, unproved area, area that needs tests |

**Write**

| Query | Action |
| --- | --- |
| `tx.begin` | Start a transaction |
| `tx.put(def, code)` | Add or replace a definition |
| `tx.remove(def)` | Delete a definition |
| `tx.commit` | Apply after checking structural and type validity. On failure, reject the whole transaction + structured diagnostics (ADR-0009) |
| `tx.abort` | Discard |
| `build` | Proof gate → LLVM IR → binary. unknown, timeout and a missing solver are failures |

**Diagnostics:** JSONL, stable error codes, multiple errors reported. If there is a counterexample, the concrete input values are given with it.

Excluded from M1: auto-fill, preserving refactorings, impact analysis in advance, reuse search (DESIGN.md §8.2). They are added if the measurement confirms the need.

### 3. Completion criteria

1. The proof gate is the default and cannot be turned off. A regression corpus shows that the build fails on unknown, timeout and a missing solver.
2. For the same code, the maps (L0–L2) and every query output are byte-identical across two runs.
3. The LLVM IR confirms that an index bounds check is gone because of a proved bounds fact. No `llvm.assume` is emitted from an unproved contract.
4. On the frozen task set (a scale of a few dozen definitions) the following are measured without retries and committed: navigation accuracy, tokens per query, first-attempt success rate, Rust-habit error rate, unproved area.

### 4. Detailed decisions

| # | Question | Decision |
| --- | --- | --- |
| O1 | What language is the compiler written in | Rust. Self-hosting after the semantics are stable (ADR-0012) |
| O2 | How Z3 is connected | SMT-LIB text passed to a process. Fixed `rlimit`, the query text is the cache key |
| O3 | How LLVM is connected | Generate textual IR and compile it with the LLVM tools (an item absorbed from v0). No bindings |
| O4 | How the tool layer is exposed | An MCP server + a CLI with the same functions. Both give the same JSON |
| O5 | Whether early `return` is allowed | By the normal form principle, only one tail expression. Reconsidered if the measurement shows large Rust-habit errors |
| O6 | Unit and value of the slice budget | Token count. The value is set after the M1 measurement (M1 only computes and reports it, it does not reject) |
| O7 | Syntax of the proof context | Attribute form (`#[uses(...)]`), which keeps "more verbose Rust" |

## Rationale
- Integers, slices and loops alone can show the proof gate, no overflow, bounds-check removal and termination. Structs and generics are not needed to measure the hypotheses of M1.
- With only the two effects `pure` and `io`, the computation of the area that needs tests (ADR-0003) can be shown.
- Leaving escape hatches out entirely fixes the trust base of M1 to the single `io` boundary, which makes the measurement of the unproved area simple.

## Consequences
- The M1 task set is limited to integer and array algorithms (search, sortedness checks, accumulation, recurrences and so on). This bias is stated in the measurement report.
- Accepting this ADR does not change DESIGN.md. It is a scope decision, not a design change.
