# ADR-0026: Data type surface: structs, enums, `match`, `impl`

- Status: Accepted (approved adoption plan: at the start of 2B, 2026-10-24. 2R-04d reflected, 2026-10-25). The exclusion of user generics and traits in §1 is superseded by ADR-0046, the exclusion of recursive types is superseded by ADR-0047, the exclusion of method call syntax is superseded by ADR-0051, and the ordering of constructor expression fields (§1 "the normal form puts them in order", §6) is superseded by ADR-0080 (the checker requires declaration order and fixes it)
- Date: 2026-10-24 (proposed), 2026-10-25 (accepted)
- Related: DESIGN.md principles 3·6·8, §4.1·§5, ADR-0013·0015·0016·0019·0020·0027·0036, `dev-docs/measurements/syntax-eval-1.md` I3·I7, `syntax-eval-2.md` §2R-04d, backlog 2B-01·2B-11

## Context
M1 (ADR-0016) left out structs, enums and `impl`, and expected the absence of `Option` to bite first. 2B adds them (ROADMAP 2B). The Rust surface is inherited (ADR-0013), but the rules fixed in M1 (one normal form, every type explicit, no shadowing, contracts are named predicates, the call target is fixed by syntax) and second-class references (ADR-0015) must apply to data types unchanged. This ADR decides only the surface and the checker semantics. The proof encoding is decided by ADR-0027.

## Decision

### 1. Scope
| In | Out (reason) |
| --- | --- |
| `struct` with named fields | Tuple structs, unit structs, tuple types (one form: principle 8) |
| Unit variants `A` and tuple variants `B(T, U)` of `enum` | Struct-like variants `C { x: T }` (one form. If field names are needed, wrap a struct: `C(CData)`) |
| Built-in `Option<T>`, `Result<T, E>`. The variants `Some`·`None`·`Ok`·`Err` are built-in names (reserved like `print_i64`, defining them is forbidden) | User generics, `trait`, `impl Trait for T` (deferred to the generics phase. Without generics a trait does nothing beyond a namespace) |
| Inherent `impl T { fn .. }` (a namespace), receivers with explicit types `self: &T`·`self: &mut T`·`self: T` | Method call syntax (§4.1 no call resolution), `&self`·`&mut self`·`self` with the type omitted |
| `match` (one arm per variant) | `if let`, `while let`, `let else`, `matches!`, guards, nested·or (`\|`)·literal·range patterns |
| Field types: integers, `bool`, fixed arrays `[T; N]`, other structs and enums | Reference fields (ADR-0015, existing E0403), slice fields, recursive types (the heap is 2G) |

### 2. Value semantics
- The data types of 2B are **plain values** that hold no heap. Assignment, argument passing and return are copies. This holds without `#[derive(Clone, Copy)]`. Types for which a move has meaning are decided in 2G together with the heap (backlog 2B-04 "value semantics without moves").
- References appear only in argument position (ADR-0015): parameters `&Point`, `&mut Point`, `&[Point]`, `&mut [Point]`. Returning a struct is a return by value, so it is allowed.
- `#[derive(..)]` is not used. Structural equality `==` is built in **only in spec context** (SMT equality). `==` on data types in executable code does not exist in 2B (in Rust it does not work without `derive(PartialEq)` either. If needed, write a function that compares the fields). This is reviewed again if measurement shows large friction.

### 3. Syntax (one form)
| Job | BMB form | Rule |
| --- | --- | --- |
| Declaration | `struct Point { x: i64, y: i64 }`, `enum Shape { Dot, Circle(i64), Rect(Point, Point) }` | Inside `mod`. Within a unit, types, functions and `spec fn` share one namespace (one definition per name) |
| Construction | `Point { x: 1, y: e }`, `Shape::Circle(r)`, `Some(v)`, `None`, `Ok(v)`, `Err(e)` | All fields in declaration order (the normal form puts them in order). A variant of a user enum is always `Type::Variant`. The four built-in variants are bare names (the same as the Rust prelude). From another unit: `geo::Point { .. }` |
| Field read | `p.x`, `p.q.x`, `a[i].x` | Parameters `p: &Point`·`&mut Point` also use `p.x` (field projection is fixed by syntax, so the ban on call resolution does not apply). `(*p).x` becomes `p.x` |
| Field write | `p.x = e;`, `a[i].x = e;` | A place of a `let mut` local or a `&mut` parameter. Whole replacement is `*p = Point { .. };` |
| Branching | `match s { Shape::Dot => e0, Shape::Circle(r) => e1, Shape::Rect(a, b) => e2 }` | Exactly one arm per variant, in declaration order, no wildcard arm. A payload position is a new name or `_`. Binding names also follow the no-shadowing rule |
| Associated function | `impl Point { fn norm1(self: &Point) -> i64 { .. } }`, called as `Point::norm1(&p)` | One `impl` block per type, in the same unit as the type. The path is `unit::Point::norm1`. If the type of the first parameter is `T`·`&T`·`&mut T`, its name is `self` (one form). `self` is not used in any other position |

The exhaustiveness of `match` is checked by the checker, and the obligations of each arm (under the path condition that the value is that variant) by the prover (ROADMAP 2B criterion 2). Because wildcards are forbidden, adding a variant exposes every `match` on that enum as an exhaustiveness error. This is the same scope as the impact computed by `impact` (backlog 2D-02).

### 4. Specs
- In spec context, field reads, `match` (an expression that yields a value) and spec `==` can be used. `old(p)` also applies to `&mut` struct parameters (`old(p).x`).
- A contract clause is still only a call of a named predicate (principle 3). Variant discrimination is written as a `match` in a named `spec fn` instead of `matches!`.
  ```rust
  spec fn is_some(o: Option<i64>) -> bool { match o { Some(_) => true, None => false } }
  ```
  Whether to build in discriminating predicates such as `is_some`·`is_ok` is an open question (§Open questions 2).
- Counterexamples are returned in Rust notation: `p = Point { x: 1, y: 2 }`, `o = Some(992)` (ADR-0027. Built-in variants are written as bare names, as in the canonical form. This supersedes the `Option::Some(992)` notation of ADR-0027 §6).

### 5. Rust habits to reject, and diagnostic codes
Free ranges in `compiler/src/diag.rs` are used. The numbers are fixed in 2B-04, and the proposed ranges are as follows.

| Range | Category |
| --- | --- |
| E0212~E0239 | Rust habits (E02xx continued). The table below |
| E0305~E0319 | Names: unknown type·field·variant, duplicate field·variant, `impl` in a unit other than the type's, a second `impl` block |
| E0412~E0439 | Types: missing·extra constructor fields, payload count, field access on a value that is not a struct, `match` on a value that is not an enum, exhaustiveness (missing variant·duplicate arm), `==` on data types in executable code, recursive types |
| E0814 reused | A construct the proof gate does not handle yet (existing code) |

| Habit | Proposed form | Mechanical fix |
| --- | --- | --- |
| `#[derive(..)]` | Delete it (copying and spec equality are built in) | Yes |
| `Option::Some(x)`, `Option::None`, `Result::Ok`, `Result::Err` | Bare names such as `Some(x)` (canonical) | Yes |
| `Point { x, y }` (field shorthand) | `Point { x: x, y: y }` | Yes |
| `Point { x: 1, ..p }` (update syntax) | List every field (`y: p.y`) | Yes (the checker knows the fields) |
| `_ =>` arm | One arm per missing variant (`_` payload, same body) | Yes |
| `if let P = e { A } else { B }` | `match` (remaining variants → `B`) | Yes |
| `while let`, `let .. else` | `match` and `while` | No |
| `p.f(..)` (method call) | `T::f(&p, ..)` | Yes (the receiver type is fixed by the explicit type of the local·parameter·field·signature, and `&`/`&mut` by the reference kind in the signature) |
| `&self`, `&mut self`, `self` | `self: &T`, `self: &mut T`, `self: T` | Yes |
| An `impl` function whose first parameter is `p: &T` | `self: &T` (`p` in the body → `self`) | Yes |
| `.unwrap()`, `.expect(..)` | Make the failure path explicit with `match` (total functions without panic, §4.1 failure) | No |
| `let Point { x, y } = p;` | `let x: i64 = p.x;` … | Yes |
| `(*p).x` | `p.x` | Yes |
| Guards, nested·or (`\|`)·literal patterns | `if` in the arm body, nested `match` | No |
| Tuple structs, tuples, struct-like variants | Structs with named fields | No (names are needed) |

`matches!`·`vec!` and the like are the existing E0202 (macros), and only the fix message is made to point at the forms of this table. `?` is the existing E0206. `use E::*`·`trait`·`impl Trait for T` are outside the language scope (the existing E0207 family), not habit errors. These habits are counted in the Rust habit error rate (DESIGN.md §4.1, R4).

### 6. Normal form, hashes, map (ADR-0019·0020)
- Normal form: within a unit, sub-units → types (by name, each type's `impl` block right after it) → functions (by name). Struct fields and variants **keep declaration order** (it is part of the definition). The fields of a constructor expression are sorted into declaration order. Functions inside `impl` are by name.
- Hashes: for a type declaration, body hash = contract hash (the declaration itself is the surface. The same treatment as `spec fn`). The qualified normal form of a definition that mentions a type writes the type with its full path.
- Map (backward-compatible additions only): L0 `def_kind` gains `"struct"`·`"enum"`, and an `impl` function is a `"fn"` with the path `unit::T::f`. The `signature` of an L1 type row is the normal form of the declaration. L2 gains a new `kind` `"types"` (a definition mentions a type in its signature, body or contract), and `impact` propagates a type change with `reason: "types"`.
- The type capability view (DESIGN.md §7.2) flattens trivially because there is one `impl` block per type.

## Rationale
- **Surface succession (ADR-0013):** every allowed form is valid Rust as is (`Point::norm1(&p)` is a Rust associated function call, `Some` is a Rust prelude name, `self: &Point` is Rust's explicit receiver). Examples of "making the optional mandatory": qualifying variant paths, stating every field, listing every arm.
- **No call resolution (§4.1):** method call syntax relies on the receiver type and auto-ref to fix the target. With path calls only, the L2 `calls` edges come straight from the syntax.
- **One normal form (principle 8):** one `match` replaces `if let`·`matches!`·`let else`. With one form, isomorphic function detection (`duplicates`) and the normal form hash work as they are.
- **No wildcard:** adding a variant exposes every use site before the prover runs. This agrees with change impact (§7.2).
- **Value semantics:** without a heap, move and copy are observationally the same. Deferring the move check to 2G reduces the scope of the 2B checker and prover. The ban on reference fields follows from ADR-0015.
- **Only spec equality built in:** SMT equality is free and is often needed in contracts. Executable `==` is tangled with code generation and `derive`, so it is added after measurement confirms the need.
- **Explicitness is the tool's job (principle 6):** most of the rejection table has a mechanical fix. Even if an agent writes by Rust habit, `bmb fix` moves it to the normal form.
- **2R-04d measurement (`syntax-eval-2.md`, 6 draft corpus programs):** bare built-in variant names give −4.3% lexical tokens (−2.5~−9.4% in the 4 programs that use `Option`·`Result`), and nowhere in the repository's BMB code or benchmark solutions are these four names defined. It matches the Rust prelude habit, so habit errors are predicted to fall (C1). `self: &T` has the same token count, is valid Rust, and is the explicit form closest to the `&self` habit (the proposal's `this: &T` was a convention that does not exist in Rust). Accepting method calls as an input alias would cut another 0.6%, but accepting a habit form without an error is a change of the write surface policy, so it is decided in 2R-04e·2R-06 (approval). Until then it is rejection plus a mechanical fix.

Alternatives considered: (1) Include `trait`·`impl Trait for T` in 2B: useless without generics, and the no-overlap check must be designed together with generics. (2) Allow method call syntax as canonical and let the tool resolve it: the resolved target is not in the text, which breaks slice self-sufficiency (principle 5). (3) Allow struct-like variants: there would be two forms and two sets of construction and pattern grammar. (4) Full paths for built-in variants too (`Option::Some`, the original proposal): more tokens and at odds with the Rust habit. Reserving the four names causes no conflicting definition, so the full path gives no resolution benefit either. (5) Named receiver `this: &T` (the original proposal): a convention that does not exist in Rust.

## Consequences
- Implementation tasks 2B-03 (syntax, normal form) through 2B-07 (map) follow this surface.
- Defining `Some`·`None`·`Ok`·`Err` is a name error (E03xx, the number is fixed in 2B-04).
- Duplicated arm bodies (no wildcard) and listing every field make code longer. This shows up in the slice size measurement (R4).
- The absence of executable `==`, discriminating predicates, struct-like variants and method syntax can raise the Rust habit error rate. This is checked by the 2B-10 measurement, and reverted if the loss of analogy is large (§4.1 response 3).

## Open questions
1. Whether to build in structural `==` in executable code (expanded into a field-by-field comparison in code generation).
2. Whether to keep discriminating predicates such as `Option::is_some` as spec built-ins or have users write them as `spec fn` (a standard predicate unit `std::option` is also possible).
3. Type invariants (Verus `type_invariant`): whether to attach an obligation to every construction and field write. Since there is no field visibility (`pub`), the default for 2B is not to add them.
4. The types allowed for `T` in `Option<T>`·`Result<T, E>`: the 2B default is integers, `bool`, structs and enums (including nested `Option<Option<i64>>`).
5. Since `match` in executable code is an expression that yields a value, how the normal form writes the statement form (with or without the `;` of `match ..;`) when all arms are `()`.
