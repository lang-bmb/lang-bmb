# ADR-0053: Closures and function values

- Status: Accepted (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after that ADR-0100 approved", 2V-18). Diagnostic numbers fixed: E0245 → E0262, E0246 → E0263, E0247 → E0264, E0248 → E0265, E0249 → E0266, E0608 unchanged. Implementation backlog 2V-20
- Date: 2026-10-26
- Related: ADR-0045, ADR-0046 (generics and traits), ADR-0047 (owned types), ADR-0015 (second-class references), DESIGN.md principles 5·6·8, §4.1 "lifetimes"·"polymorphism"·"call resolution"·"effects", review R14, backlog 2V-16

## Context
BMB rejects closures as "not supported in milestone 1" (E0207). The only ledger rationale is "lifetimes" (second-class references): a closure that captures references is a value that holds references. Move-capturing closures, closures with no captures, and function values have no ledger rationale (review R14). Canonical Rust that passes behaviour as an argument, such as a sort criterion (`sort_by`), a search predicate or a traversal, is now worked around by duplicating functions or selecting the behaviour with an enum.

As a precedent on the verification side, Verus attaches `requires`/`ensures` to closures and speaks about the contract of a function value in specifications with `f.requires(args)` and `f.ensures(args, r)`. ADR-0046 brought in generics and trait bounds, so the `Fn` family of bounds sits on top of that.

## Decision

### 1. Scope
| In | Out (reason) |
| --- | --- |
| Closure expression `move \|x: i64\| -> i64 { .. }`, with parameter and result type annotations | Reference capture (non-`move` capturing closure): ledger "lifetimes" |
| Closure with no captures `\|x: i64\| -> i64 { .. }` | Returning a closure `-> impl Fn(..)`: open question 1 |
| Built-in traits `Fn`, `FnMut`, `FnOnce` as bounds (`F: Fn(i64) -> i64`) | Function pointer type `fn(i64) -> i64`: dynamic call target (open question 2) |
| Passing a function item as a value (`apply::<_>(geo::norm, x)`) | `dyn Fn`, `Box<dyn Fn>`: same as ADR-0046 §6 |
| Closure contracts `requires`/`ensures`, `f.requires(..)` and `f.ensures(.., r)` in specifications | Argument-position `impl Fn(..)`: does the same job as a generic parameter (principle 8), changed by machine fix |

### 2. Capture and ownership
- A closure that captures variables is written with `move`. In Rust a closure without `move` captures by reference, so for the same text to mean the same thing (S1) `move` is needed. If it is missing, E0245 is issued and the machine fix adds `move`. The machine fix removes `move` from a closure with no captures (one form).
- `Copy` values are copied and other values move into the closure (ADR-0031 §2, ADR-0047). A closure value follows the same ownership rules as an unnamed struct with the captures as fields, and the drop point is fixed by the text.
- Which traits it implements is the same as Rust:
  - If it moves a capture out, it implements only `FnOnce`.
  - If it modifies a capture, it implements up to `FnMut`.
  - Otherwise it implements `Fn`.
  - The calls are respectively `f(x)` (`f: F`, `F: FnOnce`), `f(x)` (`f: &mut F`), `f(x)` (`f: &F`), and reference parameters follow the second-class reference rules as they are.
- For parameter and result types the annotation is canonical (ledger "types"). If omitted, E0246 is issued and the machine fix fills them from the expected bound (`F: Fn(i64) -> i64`) (principle 6).
- The types of closures and function items have no name (same as Rust). So the type argument in that position is written `_`, and that is canonical. This is the only exception to ADR-0046 §3 (explicit type arguments). That argument is determined deterministically by the value argument of the same call, and map L2 records the resolved closure node.

### 3. Closures do not escape
- A closure type has no name and there is no closure return (`impl Fn`), so a closure value exists only during the execution of the function that made it. A closure goes down as an argument, or is held in a local or a struct field inside the same function.
- This property is the basis of the termination rule of §4 and the capture facts of §5. If open question 1 is opened, both rules are decided again.

### 4. Proof encoding
- A type parameter with the bound `F: Fn(A..) -> R` is an uninterpreted sort (ADR-0046 §4). The uninterpreted predicates `req_F(f, a..)` and `ens_F(f, a.., r)` come with it. `f.requires(a..)` and `f.ensures(a.., r)` in specifications are these predicates. The arguments are listed as they are, without a tuple (ledger "representation").
- A call `f(a..)` inside a generic function:
  - The obligation is `req_F(f, a..)`.
  - The result `r` is a fresh symbol, and `ens_F(f, a.., r)` is a fact.
  - An `FnMut` call replaces the state of `f` with a fresh symbol. The relation between states is open question 3.
- The closure expression `move |a..| -> (r: R) requires P ensures Q { body }` is proved once at the point where it is made:
  - The premises are `P`, the path condition at that point, and the facts about the captured values. The captures of `Fn` and `FnOnce` do not change, so the facts at the point of creation still hold. The captures of `FnMut` can change, so only `P` is a premise.
  - The obligation is `Q`.
  - If `requires` or `ensures` is absent, each is true (same as Verus).
- When an instance is made with the closure type `C`, `req_C` and `ens_C` are that closure's `P` and `Q`. Here the captures are fields of the closure value. An `f.requires(..)` in the premises of the callee is unfolded to that definition and becomes an obligation at the call site.
- The `req` and `ens` of a function item value `g` are the `requires` and `ensures` of `g`.
- Termination: calls in the closure body go into the call graph as calls belonging to the enclosing function and are checked against the `decreases` of the enclosing function. The reason this is sound is §3 (a closure does not outlive the enclosing call). Where a function value `g` is passed, an edge is put from the enclosing function to `g`. A call of an `F` parameter has no edge. Monomorphized instances have no function pointers and no `dyn`, so every recursion through a function value shows up as a cycle formed by the edges above.
- Passing a function value `g` inside a function that is in the same cycle as `g` (a strongly connected component of the call graph) is E0249. The callee may call `g` with any argument, so `decreases` cannot be checked. The hint is to wrap it in a closure that has the measure condition as `requires` (`move |x: i64| -> i64 requires x < n { g(x) }`). Then it is checked by the closure rule above.

### 5. Effects
- The effect of a closure is computed from its body. No effect annotation is attached (expressions have no attributes).
- The effect of calling `F` inside a generic function must be within the effects that function declares. On the side that passes the function, it is checked that the effect of the closure or function value is contained in the declared effect of the callee. If it exceeds it, E0608.
- Effect polymorphism (effect parameters) is not added. Open question 4.

### 6. Code generation
- A closure is a capture struct and one function that takes that struct as its first argument.
- Generics are monomorphized (ADR-0046 §5), so every call is a static call and an inlining candidate. A function item value is a zero-sized type.

### 7. Map and tools
- In map L2 the calls of a closure body go in as edges of the enclosing function. A place that passes a function value is a "value use" edge.
- A closure is an unnamed node inside the unit and its node hash belongs to the enclosing function.
- Machine fixes:
  - Add or remove `move`.
  - Fill in parameter and result types.
  - Change argument-position `impl Fn(..)` to a generic parameter (E0248).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **Move-capturing closures + `Fn` family bounds + Verus-style contracts, monomorphization (proposed for acceptance)** | The Rust surface as is (S1) and compatible with the ledger "lifetimes". The call target is static (ledger "call resolution"). The verification precedent is Verus |
| Forbidden, as now | Exclusion with no ledger rationale (R14). A workaround that duplicates functions per behaviour or dispatches with an enum (ADR-0045 §3) |
| Allow reference capture | The closure becomes a value holding references and loses the payback of second-class references (ADR-0015), namely no lifetime annotations and simpler alias analysis |
| Function pointers, `dyn Fn` | The call target is a run-time value. The call graph and the termination check do not close statically (open question 2) |
| Derive the closure contract from the body automatically | When the body changes the contract changes with it, and the proof of the generic definition becomes unstable. Explicit contracts are the canonical way in Verus and Dafny |

## Consequences
- Diagnostics:
  - New codes E0245 (capture without `move`, machine fix), E0246 (closure type annotation omitted, machine fix), E0247 (closure return and function pointer type), E0248 (argument-position `impl Fn`, machine fix), E0249 (function value passed inside the same cycle), E0608 (function value whose effect exceeds the callee's declaration).
  - The closure message of E0207 goes away.
  - At acceptance the numbers are checked again to be unused in `diag.rs` and in the provisional numbers of the proposed ADRs.
- Sorting and searching in the standard library (4B) are written like `sort_by<F: Fn(&T, &T) -> Ordering>` (`Ordering` is a built-in enum).
- Fuzzer: put higher-order functions with captures and bounds into the data fuzzer. The encoding is checked differentially by replaying counterexamples of closure contracts.
- Implementation phase: after generics (2T). Its position on the self-hosting critical path is decided at acceptance.

## Open questions
1. Returning closures (`-> impl Fn(..)`): the "do not escape" of §3 breaks. The termination rule and the capture facts must be decided again (see Dafny's `reads`/`decreases` approach for function values).
2. Function pointers `fn(..)` (handler tables, etc.): a dynamic call target. Decided together with a way to attach contracts to the type (Verus's `FnSpec`).
3. The relation between `FnMut` states (reasoning about accumulated state): the first implementation makes the state a fresh symbol. How much is needed is measured in self-hosting.
4. Effect polymorphism: needed for a pure caller and an IO caller to use the same higher-order function. Usage frequency is measured after 2T.
