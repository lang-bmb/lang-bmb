# ADR-0047: Generalizing owned types: `Box<T>`, recursive types, types that hold a `Vec`

- Status: Accepted (2026-10-26, approved by the maintainer "1/2 adopted, approved", backlog 2V-12)
- Date: 2026-10-26
- Related: ADR-0045, ADR-0015, ADR-0026 §1 (ban on recursive types), ADR-0027, ADR-0031 §1·§2 (exclusion of structs holding a `Vec`, E0414), DESIGN.md §5, review R6·R7, backlog 2V-07


## Confirmed at acceptance (2026-10-26)
- Diagnostic numbers (confirmed free): E0238 `Rc`·`RefCell`·`Cell`·`Arc`, E0239 move of a field behind `&mut` (mechanical fix `std::mem::replace`), E0433 use of the whole after a partial move. Direct recursion is the existing E0421, and use after move and "maybe moved" are the existing E0413·E0414.
- The ledger row of the §5 consequences ("move state") goes into DESIGN §4.1 together with this acceptance.
- Open question 1 (unifying data types without recursion into SMT datatypes too) is decided by V4·V7 measurement at implementation. 2 stays open.
- Implementation phase: ROADMAP 2O (owned types). It supersedes the exclusion of structs holding a `Vec` and of `Vec<Vec<T>>` in ADR-0031 §1.

## Context
ADR-0026 deferred recursive types with "the heap is 2G", and ADR-0031 (2G) added only `Vec<T>` and left structs holding a `Vec` and `Vec<Vec<T>>` out of scope. As a result, recursive data such as syntax trees and JSON values and context structs with several buffers cannot be written in the standard form, and the arena and parameter-spreading workarounds were rated "high" in self-hosting cost (`selfhost-gap.md` §4.1). The ledger has no payback item for this limit. SMT datatypes support recursion.

## Decision

### 1. Owned types
- **Move types:** `Vec<T>`, `Box<T>`, `String` (ADR-0050), and the structs, enums and arrays that hold these as a field, payload or element. Other values (scalars, data types and arrays that hold no move type) are copies as now. The classification is structural and deterministic (continuing the value semantics of ADR-0026, which does not use Rust's `Copy` derivation).
- `Box<T>`: `Box::new(v)`, reads `*b`·`b.f` (field projection is fixed by syntax), writes `*b = v`·`b.f = v`. The reference rule (ADR-0015) is unchanged: `&*b` only in argument position.
- Recursive types: structs and enums that hold themselves only under `Box` or `Vec` (`enum Expr { Num(i64), Add(Box<Expr>, Box<Expr>) }`, `struct Node { kids: Vec<Node> }`). Holding itself directly is E0421 as now.
- Nested containers: `Vec<Vec<T>>`, `Vec<S>` (`S` a move type), `Option<Box<T>>`.

### 2. Moves and freeing
- The rules of ADR-0031 §2 are widened to all move types: a value use is a move, use after move is E0413, "maybe moved" at a join point is E0414 (the free point is fixed by the text: no runtime drop flag).
- A partial move that takes a field out of a local struct is allowed, as in Rust. The field taken out cannot be used until it is assigned again, and the remaining fields are freed at the end of the block. A field behind `&mut` cannot be taken out, and `std::mem::replace(&mut x.f, v)`·`std::mem::swap(&mut a, &mut b)` are used (Rust standard. `mem::take`, which ADR-0031 rejected, comes after the `Default` trait (ADR-0046)).
- Freeing is recursive through per-type drop glue (the same as rustc). The order is the same as Rust (field declaration order, element order).

### 3. Proof encoding
- `Box<T>` is the value `T` itself (it is invisible in proofs).
- Recursive types and the types that hold them are SMT datatypes (ADR-0027 §3 applied to structs too). Data types without recursion keep the current field decomposition (ADR-0027 §1). The boundary and stability of the two encodings are measured with V7·V4 at implementation, and if merging into one is better, it is decided then.
- A `Vec<T>` whose element is a move type is `(Array Int ⟦T⟧, Int)` (⟦T⟧ is the sort of the element: a datatype, or the datatype of an `(Array, Int)` pair).
- Termination: a function on recursive data can use the structural size with `decreases x` (the data value). The encoder makes a size `spec fn` per type and gives the fact that the size of a sub-value is smaller (Dafny's datatype rank, Verus's `height`).

### 4. Code generation and runtime
- `Box<T>` is the address of a `malloc`ed `T`. The layout of a recursive type is cut by the pointer.
- Drop glue is a per-type internal function (`@"bmb_drop.<type>"`). Freeing a `Vec<S>` calls the glue for each element and then releases the buffer.
- Freeing deeply recursive data uses stack in proportion to the recursion depth (the same as rustc). The stack size is decided in 4C (system interface).

### 5. Rust habits to reject, and diagnostics (numbers fixed at acceptance)
| Habit | Handling |
| --- | --- |
| `Rc`, `RefCell`, `Cell`, `Arc` | Rejected, hint: an ownership tree (`Box`·`Vec`) or index handles |
| Move of a field behind `&mut` | The E0413 family, fix: `std::mem::replace(&mut x.f, v)` |
| Direct recursion `struct L { next: L }` | E0421 kept, fix: `Box<L>` |

"Structs holding a `Vec` and `Vec<Vec<T>>` are out of scope (E0402)" of ADR-0031 §1 is superseded when this ADR is accepted.

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **`Box`, recursive types and nested ownership by Rust's rules, freeing by drop glue at static points (proposed for acceptance)** | The Rust surface and semantics as is (S1). Keeps the 2G property that E0414 fixes the free point in the text without drop flags |
| Allow only arenas (`Vec<Node>` + `usize` handles) | A workaround (ADR-0045 §3). Handle validity spreads into proof obligations, and it is far from the standard expression |
| `Rc`/`RefCell` | Shared mutable state breaks frame conditions and alias analysis (the rationale of ADR-0015). Conflicts with the payback of the ledger's "lifetimes" |
| Allow "maybe moved" with drop flags | Rust allows it, but the free point depends on runtime state. Raising the payback of E0414 (deterministic freeing, no runtime flag) to a ledger item is proposed together |

## Consequences
- Syntax trees, JSON and S-expressions can be written as recursive enums, and a context as a struct with several `Vec` fields (two "high" self-hosting items are resolved).
- A row is added to the ledger of DESIGN §4.1: "Move state: Rust allows 'maybe moved' with drop flags → BMB requires the same state at a join point (E0414) → the free point is fixed by the text, no runtime flag".
- Fuzzer: the `Vec` fuzzer is widened to ownership trees (insert, remove, replace, `mem::replace`) and valgrind checks leaks and double frees.

## Open questions
1. Whether to unify data types without recursion into SMT datatypes too (one encoding, S3). Decided by measurement.
2. `Box<[T]>` and `drain`-style operations of `Vec`: separately when the need arises.
