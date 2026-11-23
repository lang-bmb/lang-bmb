# ADR-0046: Generics and traits

- Status: Accepted (2026-10-26, approved by the maintainer "1/2 adopted, approved", backlog 2V-12)
- Date: 2026-10-26
- Related: ADR-0045 (judgement criteria), ADR-0026 §1 (user generics and traits deferred), ADR-0027, DESIGN.md principles 5·6·8, §4.1 "polymorphism"·"types", review `dev-docs/reviews/2V-01-judgment-retro.md` R5, backlog 2V-06


## Confirmed at acceptance (2026-10-26)
- Diagnostic numbers (confirmed free, avoiding `compiler/src/diag.rs` and the provisional numbers of the proposed ADR-0029·0030): E0235 blanket impl, E0236 `dyn Trait`, E0237 omitted type arguments (mechanical fix), E0311 overlapping impl or ambiguous trait method, E0312 impl in the unit of neither the trait nor the type, E0428 bound not satisfied, E0429 number of type arguments, E0817 impl method does not refine the trait contract.
- Open questions: 1 (const generics)·2 (associated types)·4 (structural `Eq`·`Ord` derivation) stay open. 3 (default method bodies) is included (Rust standard, proved from the trait contract alone).
- Implementation phase: ROADMAP 2T (generics).

## Confirmed at implementation (2026-11-08, 2T-09)
- `Vec<T>` (2026-11-08, 2T-10): a type parameter can be the element of a `Vec` (before that it was E0402, so the generic containers of §Consequences could not be written). So that a type that cannot be the element of a `Vec` (an array or a data type holding an array, 2O-03b) does not enter that parameter, a parameter that a definition (the fields of a struct or enum, the signature·`let`·type arguments of a function, every function of an `impl`) uses as `Vec<T>` has an implicit bound `elem`, checked per instance with E0428 (`check::elem_params`). `elem` is left out of the bound comparison between trait methods and impl functions (it comes from the `Vec` in each body).
- `String` has built-in `Eq`·`Ord` (2026-11-08, 4B-04): Rust's byte lexicographic order. The table of ADR-0092 §2 set `BTreeMap<K: Ord, V>` and a map with byte string keys (the stage0 port) as scope, and that key is `String`. It is separate from open question 4 (structural derivation for data types). A generic definition is still proved once with an uninterpreted total order on the sort. In a `String` instance the comparison is the order of the text (the encoder lowers text to `&[u8]`, so it sends the comparison of an instance whose type argument is that to `text_cmp_spec`·`text_cmp_code`: `verify::text::is_lowered_text`), and `==` is equality of the bytes within the length. The total order axioms hold for the byte lexicographic order whose equality is defined by the bytes within the length (antisymmetry with that equality), so the generic proof is sound in the instance too. Code generation does the comparison of the instance with `bmb_str_cmp` (`codegen::text::text_targ`). Tests `tests/verify/pass/text_ord` (`"B" < "a" < "ab" < "é"`), `fail/text_ord` (the opposite claim, less than itself).
- The `Self` of a trait follows the same ownership rule as the other type parameters of §2: its bound is that one trait and there is no `Copy`, so `self: Self` and `Self` values move. In the first implementation (2T-07a) the checker silently attached a `Copy` bound to `Self`, so a type holding `Vec`·`String`·`Box` could not implement a trait (found in the 2M-03 audit). This is an implementation correction within this decision, not a new decision. The spec predicate that passed `Self` to the `T` of `T: Copy + Tr` (`scored<T: Copy + Score>`) was moved by removing `Copy` (Rust's `Self` is not `Copy` either). A super bound (`trait Tr: Copy`, Rust's supertrait) is decided separately when demand appears.

## Context
ADR-0026 deferred user generics and traits "to the generics phase", but that phase is not in the roadmap. So a workaround entered the recommendations: containers such as a sorted map and algorithms such as sorting and searching must be duplicated per type or built in (review R5·R12). In Rust, generics and traits are the foundation of the whole standard library, and the precedent on the verification side (Verus) also proves a generic definition once. What the ledger (DESIGN §4.1 "polymorphism") forbids is blanket impls and implicit trait resolution, not generics themselves.

## Decision

### 1. Scope
| In | Out (reason) |
| --- | --- |
| Type parameters `<T, U>` of functions, structs, enums and inherent `impl` | Const generics (an array length parameter is open question 1) |
| Trait bounds `T: Ord`, `T: Ord + Copy`, `where` clauses | Higher-ranked bounds, `?Sized` |
| `trait Name { fn ..; spec fn ..; }`: method signatures and contracts | Associated types and associated constants (open question 2), default method bodies (open question 3) |
| `impl Trait for Type`, `impl<T: B> Trait for Wrap<T>`: explicit impls that do not overlap | Blanket impls such as `impl<T> Trait for T`, specialization (ledger "polymorphism") |
| Built-in traits `Copy`, `Eq`, `Ord` (with contracts, built-in impls for integers·`bool`·`u8`) | Trait objects `dyn Trait` (static call target: ledger "call resolution") |
| Type arguments of a call `max::<i64>(a, b)` | Omitted type arguments are filled in by the tool (§3) |

### 2. Semantics
- Type parameters follow the ownership rule. If the bound has no `Copy`, a `T` value moves (ADR-0031 §2, ADR-0047). With a `Copy` bound it is a copy.
- A trait method call is written `Trait::method(x, ..)` or with receiver syntax (ADR-0051). The target is fixed statically by the bounds and the argument types. If two or more bound traits have the same name, the path form is required (the same as Rust).
- Coherence: there is one impl per (trait, type) pair. An impl is placed in the same unit as either the trait or the type (Rust's orphan rule with the unit as the boundary).

### 3. Surface and normal form
- The canonical form is the call with type arguments written, `f::<T>(..)`. If type arguments are omitted, a mechanical fix that comes with the diagnostic fills them in from the argument types (principle 6: explicitness is a property of the artifact, the labor is the tool's job). All argument types are explicit (ledger "types"), so unification is deterministic.
- Map L1 records type parameters and bounds, and L2 records call edges to generic definitions and the type arguments.

### 4. Proof encoding
- A generic definition is proved **once**. A type parameter is an SMT uninterpreted sort (`declare-sort`), and the methods of a bound trait are uninterpreted functions on that sort plus the trait contract (facts). Slice self-sufficiency (principle 5) is kept by "body + contracts depended on + contracts of the bound traits".
- A caller uses the callee contract with the type arguments substituted (a copy with the sort replaced by the concrete sort).
- `impl Trait for Type` carries the obligation of the trait contract: each method of the impl has a precondition weaker than the trait's `requires` and a result stronger than its `ensures` (the same as the trait refinement obligation of Verus and Dafny).
- The contract of the built-in `Ord` is the total order axioms (reflexive, antisymmetric, transitive, total). The integer impl is `<=` itself.
- A data type instance (`Pair<i64, bool>`) uses the ADR-0027 encoding per instance.

### 5. Code generation
- Monomorphization (the same as rustc). The instance name is made deterministically from the path and the canonical text of the type arguments. The incremental IR key is per instance.

### 6. Rust habits to reject, and diagnostics (numbers fixed in `diag.rs` at acceptance)
| Habit | Handling |
| --- | --- |
| `impl<T> Trait for T` (blanket) | Rejected, hint: an explicit impl per type |
| `dyn Trait`, `Box<dyn Trait>` | Rejected, hint: closed variants with an enum, or a generic bound |
| Omitted type arguments | Diagnostic + mechanical fix (fill-in) |
| `#[derive(PartialEq, Ord, ..)]` | The existing handling (ADR-0026) + structural `Eq`·`Ord` derivation is open question 4 |

"generic parameters are not supported in milestone 1" and "`impl Trait for T` is not supported" of E0207 go away with acceptance (the code E0207 stays for other unsupported constructs).

## Rationale (alternatives considered)
| Option | Verdict |
| --- | --- |
| **Generics + trait bounds, proved once per definition, monomorphized code (proposed for acceptance)** | The Rust surface as is (S1). Verus proves generics once in the same way. Containers and algorithms can be written as libraries |
| Build in only containers such as maps and sets | Blocks user-defined containers, and the rules for built-in and user-defined diverge (S3). The standard library is tied to the compiler |
| Duplicate monomorphic libraries | A workaround (ADR-0045 §3). Proof cost and map nodes grow with every copy |
| Prove per instance (templates) | Proof cost is proportional to the number of instances, and the contract of a generic definition is not verified by the definition itself |

## Consequences
- The standard library (4B) is written with generic containers (`BTreeMap<K, V>`, `BTreeSet<T>`) and algorithms (`sort<T: Ord>`) (review R12).
- Sort parameters and trait contracts enter the proof encoding. Stability (V7) and cost (V4) are measured at implementation.
- Fuzzer: generic containers and bounds are added to the data fuzzer.

## Open questions
1. Const generics (the `N` of `[T; N]`): may become necessary together with array return.
2. Associated types: there are no iterators (ADR-0049), so not needed right now. Decided separately when needed.
3. Default method bodies: proved from the trait contract alone, so they can be included. Whether to leave them out of the first implementation is to be decided.
4. Structural `Eq`·`Ord` derivation: the relation to the ban on `#[derive]` (ADR-0026). Decided together with `==` on data types in executable code (ADR-0026 open question 1).
