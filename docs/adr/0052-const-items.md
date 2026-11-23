# ADR-0052: `const` items

- Status: Accepted (approved by the maintainer 2026-11-09 "ADR-0097·0098·0099, 2V-18, and after that ADR-0100 approved", 2V-18). Diagnostic numbers fixed (the provisional numbers of the proposal were in use): E0242 → E0259, E0243 → E0260, E0244 → E0261, E0314 → E0318, E0434 → E0437. Implementation backlog 2V-19
- Date: 2026-10-26
- Related: ADR-0045, DESIGN.md principles 5·8, §4.1 "names"·"effects", ADR-0026 (item surface), ADR-0050 (strings), review R14, backlog 2V-14

## Context
BMB rejects `const` and `static` items as "not supported in milestone 1" (E0207) and advises "return the value from a `spec fn` or use a literal". But a `spec fn` is specification-only and cannot be called from executable code. So executable code has to write the same literal in several places, or make an `#[effects(pure)]` function that returns the value. Neither can be used for an array length (`[u8; N]`). This is an exclusion with no ledger rationale (review R14). In the self-hosting analysis (`dev-docs/measurements/selfhost-gap.md` §3.1) there are 46 `const`/`static` items; diagnostic code tables and buffer sizes are examples.

The ledger row "names" (DESIGN v1.2) already forbids immutable `static`, which does the same job as `const`, and global mutable state (`static mut`, row "effects"). The canonical form that remains is Rust's `const`.

## Decision

### 1. Surface
- An item inside a unit: `const NAME: T = e;`. The type annotation is required (ledger "types": every type is annotated).
- The namespace is the unit; in the same unit it is written `NAME`, in another unit by the full path `unit::NAME`. Within a unit one name has one definition (E0302).
- The associated constant `impl T { const NAME: U = e; }` is not part of this ADR. Associated constants are decided together with trait associated items (ADR-0046).
- `#[effects(..)]` is not attached. Reading a `const` is pure.

### 2. Types
- Only value types are accepted: integers, `bool`, `char`, structs and enums made of such values, fixed-length arrays `[T; N]`.
- After ADR-0050 is implemented, `&str` and `&[T]` pointing to static data are also accepted (`T` is a value type). The literals `"..."`, `b"..."` and `&[..]` do the same job, so this is not putting a reference in data. It does not conflict with second-class references (§5).
- Owned types (`Vec`, `String`, `Box`, ADR-0047) and data types containing references are not accepted. Rust does not allocate in `const` either.
- The rejection code is E0434.

### 3. Constant expressions
- The initializer consists only of:
  - literals;
  - other `const`s;
  - unary and binary operators and comparisons;
  - built-in integer methods (`wrapping_*`, `checked_*`, `saturating_*`, `T::try_from(x).unwrap()`, `as` conversion);
  - struct and enum construction;
  - array literals and repetition `[e; N]`;
  - field access and indexing;
  - `if` and `match`.
- User function calls are not accepted. Rust's `const fn` is not added. Open question 1 decides it after measurement.
- An initializer that is not a constant expression is E0242. The hint points to the part that is not a constant expression.
- A reference cycle among `const`s is E0314.

### 4. Evaluation
- The checker evaluates every `const` in topological order. The expression set above has no loops or calls, so evaluation always terminates and is deterministic.
- Evaluation failure is a compile error (E0243), as in Rust. Failures are overflow, division by zero, out-of-range index, and a failed `try_from(..).unwrap()`. The error shows the failed operation and the operand values.
- The value is fixed in the checking stage. So a `usize` constant can be used as the array length in `[T; N]` and as the `N` of the repetition `[e; N]`. Type equality is decided by the evaluated length (`[u8; 4]` and `[u8; FOUR]` are the same type).
- Normal form: an array length is only a named `const` or a literal. An expression in the length position (`[u8; N + 1]`) is not accepted. Rust also restricts general const expressions in the length position. In that case a separate `const` is defined.

### 5. Proof encoding
- A `const` is a transparent value (unlike a `spec fn`, whose definition is opaque by default. It is a value, not a definition). In each query every `const` used becomes one SMT constant, and the `v` of the defining equation `(define-fun unit__NAME () T v)` is the evaluated value. Array values are written as element equations.
- It can be used in both specifications and executable code. It is visible without being put in the proof context (`uses`).
- The initializer itself has no proof obligation. If evaluation succeeded, all arithmetic is in range.

### 6. Code generation
- A scalar is inserted as an LLVM constant at every use.
- Arrays, structs and slices become one `private unnamed_addr constant` global, read at the use sites. Used as a value it is copied; used as a slice the pointer and length are passed.

### 7. Map and tools
- Map L0 shows the constant, and L1 shows its type and evaluated value. L2 gets an edge for each use of the constant. When the value changes, the proofs that use the constant are invalidated. This becomes part of the proof cache key.
- The `const`/`static` hint of E0207 changes:
  - `const` is accepted.
  - Immutable `static` is rejected with E0244 and gets the machine replacement `static`→`const` (ledger "names").
  - `static mut` is rejected with E0244 with a hint only (ledger "effects": pass it as an explicit context).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **Rust `const`, checker evaluation, transparent value (proposed for acceptance)** | The Rust surface as is (S1). Usable as an array length. The value is visible in proofs, so there is no assertion cost. Verus also accepts `const` |
| `spec fn` or literal, as now | Not usable in executable code, so literals are duplicated and an array length cannot be named. Exclusion with no ledger rationale (R14) |
| An `#[effects(pure)]` function that returns the value | Two forms for the same thing (principle 8). Not usable as an array length. Visible in proofs only through its contract |
| Including `const fn` | Canonical Rust, but the checker would need an interpreter (termination is guaranteed by `decreases`). First measure whether the 46 self-hosting places need calls (open question 1) |
| The prover carries the obligations of the initializer | An array length needs its value in the checking stage. One evaluator that fixes both value and failure is simpler and the same as Rust |

## Consequences
- Diagnostics:
  - New codes E0242 (initializer that is not a constant expression), E0243 (constant evaluation failure), E0244 (`static`), E0314 (`const` cycle), E0434 (type not usable in `const`).
  - E0207 is no longer used for `const`/`static`.
- Fuzzer: add a variant that lifts the constants of the arithmetic fuzzer into `const` items. The agreement of checker evaluation and execution result is checked differentially.
- The DESIGN §4.1 ledger row "names" does not change (the ban on immutable `static` is already written).

## Open questions
1. `const fn`: decided after measuring, on the 46 self-hosting places and the 2B-10 measurement, the share of initializers that need a function call.
2. Tables containing references (code tables such as `&[(&str, &str)]`): this interlocks with second-class references, so it is decided separately after ADR-0050 is implemented (2U). Until then the canonical alternatives are integer codes with a `match` function, or an array of structs.
3. Associated constants (`const` inside `impl`): decided together with trait associated items (ADR-0046 implementation, 2T).
