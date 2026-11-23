# BMB language reference (draft)

> Status: draft. What it describes is the accepted language pinned by the frozen fingerprint (`tests/surface.json`).
> Order of authority: the compiler's verdicts (pinned by the corpus) → this document. If the two differ, this document is wrong, and it is fixed.
> The reasons for the design are in `docs/DESIGN.md` and the decision records (`docs/adr/`). This document only states what is a program and what is not.

## Scope

- **Included:** grammar, static semantics (names, types, ownership, effects), the meaning of specifications, proof obligations. Each section gives the diagnostic codes reported when its rules are broken and the corpus files that show those rules.
- **Not included:** tutorials, tools (`bmb lint`, `weaken`, `uses`, `fix`, the map, MCP: `compiler/README.md`), the API of the standard library (the definition summaries of `lib/std/` and map L1), the details of code generation.
- A diagnostic that starts with `W` is a warning that only `bmb lint` and `bmb weaken` report, so it is not a verdict of the language. The list is in `docs/diagnostics.md`.

## Chapters

| Chapter | Contents |
| --- | --- |
| [01 Lexical structure and syntax](01-lexical-and-syntax.md) | Tokens, literals, comments and summaries, the skeleton of the syntax |
| [02 Units, items and names](02-units-items-names.md) | Units (`mod`), function, type, trait and constant items, attributes, name lookup |
| [03 Types and values](03-types-and-values.md) | Integers, `f64`, `bool`, `char`, arrays and slices, structs and enums, generics and traits, owning types |
| [04 Expressions, statements and control flow](04-expressions-and-control.md) | Expressions and statements, `if`, `match`, loops, the Rust habits that are not accepted and their replacements |
| [05 Ownership and references](05-ownership.md) | Moves, borrows, references that do not escape |
| [06 Contracts and specifications](06-contracts-and-specs.md) | `requires`, `ensures`, invariants, decreases expressions, `spec fn`, `proof fn`, quantifiers |
| [07 Proof gate](07-proof-gate.md) | Kinds of obligation, the proof context (`#[uses]`), verdicts and counterexamples |
| [08 Effects and escape hatches](08-effects-and-escape-hatches.md) | `#[effects]`, `#[external_body]`, `assume` and the rationale |

## How to read

- Examples point to corpus files. `tests/run/` holds programs fixed down to their execution output, `tests/verify/pass` and `fail` hold proof verdicts, and `tests/check/err` holds check errors and their diagnostics (`.jsonl`).
- Diagnostic codes do not change and are not reused. Retired codes are at the end of `docs/diagnostics.md`.
- What the gate (`scripts/reference.py check`) confirms: whether every diagnostic code of the language appears in some chapter, whether the corpus files the chapters point to exist, whether each section has at least one code or corpus file.

## Limits of the draft

The text of the chapters was carried over from the rule tables of `compiler/README.md`, the compiler source and the corpus, and the examples and doubtful rules were confirmed by running the compiler. Statements that had been written on the basis of documents and source alone were confirmed by execution (2026-11-11): operator precedence, short-circuit evaluation, evaluation order and the end value of `for` (`tests/run/order.bmb`), method lookup order (`tests/run/method_order.bmb`), the checks that `emit-llvm` leaves (chapter 07), the input of the rationale hash (chapter 08), the places of a type invariant (chapter 06), the types that have `Eq` and `Ord` (chapter 03). What remains is the full list of builtin methods (chapter 03 gives only examples) and section-by-section tests of deallocation points, and the conformance tests attach tests to each section.

## Diagnostic code index

<!-- reference:index begin (scripts/reference.py index --write; do not edit by hand) -->

| Code | Meaning | Chapter |
| --- | --- | --- |
| E0101 | unexpected character | [01](01-lexical-and-syntax.md) |
| E0102 | unterminated string literal | [01](01-lexical-and-syntax.md) |
| E0103 | syntax error: unexpected token | [01](01-lexical-and-syntax.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0104 | integer literal suffix not allowed; the type comes from context | [01](01-lexical-and-syntax.md), [03](03-types-and-values.md) |
| E0105 | integer literal too large | [01](01-lexical-and-syntax.md) |
| E0106 | block comments are not allowed; use `//` | [01](01-lexical-and-syntax.md) |
| E0107 | nesting too deep | [01](01-lexical-and-syntax.md), [04](04-expressions-and-control.md) |
| E0108 | malformed integer literal: a digit outside its base, or a base prefix without digits | [01](01-lexical-and-syntax.md) |
| E0109 | malformed float literal: digits on both sides of the point (`1.0`, `0.5`) and after an exponent `e` are required | [01](01-lexical-and-syntax.md) |
| E0201 | `let` requires an explicit type | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0202 | macros are not allowed | [04](04-expressions-and-control.md) |
| E0205 | `as` converts between integer types only (to `f64`: `f64::from(x)`) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0207 | construct not supported in milestone 1 | [02](02-units-items-names.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0208 | lifetime annotations are not allowed | [01](01-lexical-and-syntax.md), [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0209 | function return type is required (write `-> ()` for unit) | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0210 | `mut` parameters are not allowed | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0211 | comparison operators cannot be chained | [04](04-expressions-and-control.md) |
| E0212 | `T::from(x)` / `x.into()` / `x.try_into()`: convert with `x as T` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0213 | `T::try_from(x).expect(..)` is written `.unwrap()` | [04](04-expressions-and-control.md) |
| E0214 | `use` is not used: names are written in full | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0215 | `unsafe` is not used: a body that cannot be proved is `#[external_body]` | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0216 | Verus's `#[verifier::external_body]`: BMB writes `#[external_body]` | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0217 | Verus's `admit()`: BMB writes `assume(p(..))` with a rationale | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0218 | a `Vec` or `String` is passed where a slice or `&str` is expected: write `v.as_slice()` or `s.as_str()` | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0219 | `drop(v)` and `mem::take` are not used | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0220 | `#[derive(..)]` is not used | [04](04-expressions-and-control.md), [08](08-effects-and-escape-hatches.md) |
| E0221 | built-in variants are written bare (`Some`, not `Option::Some`) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0222 | field shorthand is not allowed (`x: x`) | [04](04-expressions-and-control.md) |
| E0223 | struct update syntax `..` is not allowed | [04](04-expressions-and-control.md) |
| E0224 | a wildcard arm is not allowed | [04](04-expressions-and-control.md) |
| E0225 | `if let` is not allowed; use `match` | [04](04-expressions-and-control.md) |
| E0226 | `while let`, `let .. else` and pattern `let` are not allowed | [04](04-expressions-and-control.md) |
| E0228 | the receiver needs its type (`self: &T`) | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0229 | `.unwrap()` / `.expect()` are not allowed; `match` both cases | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0230 | destructuring `let` is not allowed | [04](04-expressions-and-control.md) |
| E0231 | `(*p).x` is written `p.x` | [04](04-expressions-and-control.md) |
| E0232 | pattern form not allowed (guard, or, literal, nested, struct) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0233 | tuples, tuple structs and struct-like variants are not allowed | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0235 | a blanket impl `impl<T> Trait for T` is not used | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0236 | trait objects `dyn Trait` are not used | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0237 | a call of a generic function names its type arguments: `f::<T>(..)` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0238 | `Rc`, `RefCell`, `Cell` and `Arc` are not used: values have one owner | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0239 | an owned value is moved out from behind a `&mut`: use `std::mem::replace` | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0240 | a `where` clause is not used: bounds go next to the type parameter | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0241 | `?` whose `None`/`Err` the function cannot return (another error type, or not an `Option`/`Result` function) | [04](04-expressions-and-control.md) |
| E0242 | `crate::`, `super::` or `self::` path: a path names a unit from the root | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0250 | `break` or `continue` outside a loop | [04](04-expressions-and-control.md) |
| E0251 | `break` with a value outside a `loop` | [04](04-expressions-and-control.md) |
| E0252 | `for` over something other than an integer range | [04](04-expressions-and-control.md) |
| E0253 | `decreases` on a `for` loop (its measure is automatic) | [04](04-expressions-and-control.md) |
| E0254 | `#[invariant(p)]`: `p` is not a spec function `(x: &T) -> bool` over the type's parameters, or states nothing | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0255 | more than one `#[invariant(..)]` on a type: join the conditions with `&&` in one predicate | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0256 | a `proof fn` takes no effect attribute and no return type | [02](02-units-items-names.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0257 | ghost code (a `proof fn` body or a `proof { .. }` block) cannot run executable code: no executable call, assignment, `&mut` or loop | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0258 | a `proof fn` is called only from ghost code: a `proof { .. }` block or another `proof fn` | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0259 | a `const`'s value is not a constant expression: literals, other `const`s, operators, `if`, `match`, the wrapping and saturating methods, `as`, arrays, structs, variants, fields and elements (ADR-0052 §3) | [04](04-expressions-and-control.md) |
| E0260 | a `const`'s evaluation failed: overflow, division by zero, a shift of the width or more, a conversion out of range, an index past the end (ADR-0052 §4) | [04](04-expressions-and-control.md) |
| E0261 | a `static` item: an immutable one is a `const`, global mutable state is passed as an argument (ADR-0052 §7) | [04](04-expressions-and-control.md) |
| E0262 | a closure that captures a local is written `move`: captures move into it (ADR-0053 §2) | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0263 | a closure's parameters and result have their types written (ADR-0053 §2) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0264 | a function pointer type, a returned closure, or a closure that is not the argument of an `Fn`-bounded parameter (ADR-0053 §1, §3) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0265 | an argument-position `impl Fn(..)`: the form is a type parameter `F: Fn(..)` (ADR-0053 §1) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0301 | unknown name | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0302 | duplicate definition | [02](02-units-items-names.md) |
| E0303 | a local name hides a visible one (a parameter, an outer local, the result), or is bound elsewhere in the function with another type (ADR-0095 A3) | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0304 | unknown function | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0305 | unknown type or trait | [02](02-units-items-names.md) |
| E0306 | unknown field | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0307 | unknown variant | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0308 | `impl` block not in the unit of its type, or a second one with the same bounds (one block per list of bounds, ADR-0097) | [02](02-units-items-names.md) |
| E0309 | a built-in name cannot be defined | [02](02-units-items-names.md), [03](03-types-and-values.md) |
| E0310 | `self` names exactly the receiver | [02](02-units-items-names.md) |
| E0311 | two impls of a trait for types that can be the same | [02](02-units-items-names.md), [03](03-types-and-values.md) |
| E0312 | a trait impl in neither the unit of the trait nor the unit of the type | [02](02-units-items-names.md), [03](03-types-and-values.md) |
| E0313 | a method call names a method of more than one trait; call by path | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0314 | a method call names a function that has no receiver or type parameters of its own; call by path | [02](02-units-items-names.md), [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0315 | a trait impl misses a method of the trait, or has one the trait lacks | [02](02-units-items-names.md) |
| E0316 | a trait impl's function does not match the trait's signature | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0317 | a `break` or `continue` names a label no loop around it has | [02](02-units-items-names.md), [04](04-expressions-and-control.md) |
| E0318 | `const`s that refer to each other (ADR-0052 §3) | [02](02-units-items-names.md) |
| E0401 | type mismatch | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [05](05-ownership.md), [06](06-contracts-and-specs.md) |
| E0402 | unsupported type | [03](03-types-and-values.md), [05](05-ownership.md), [06](06-contracts-and-specs.md) |
| E0403 | references are only allowed in parameter position | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0404 | unknown method (no method lookup; only built-in methods exist) | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0405 | wrong number of arguments | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0406 | integer literal out of range for its type | [03](03-types-and-values.md) |
| E0407 | value is not indexable | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0408 | assignment to an immutable place | [04](04-expressions-and-control.md), [05](05-ownership.md) |
| E0409 | `&` / `&mut` may only appear directly as a call argument | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0410 | a `&mut` argument aliases another argument of the same call | [05](05-ownership.md) |
| E0411 | cannot negate an unsigned value | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0412 | `T::try_from(x)` takes an integer and is used as `.unwrap()` or `.ok()` | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0413 | use of a moved value, or a move out of a place | [05](05-ownership.md) |
| E0414 | a value that moves is moved on some paths and not on others | [05](05-ownership.md) |
| E0420 | malformed data type: wrong type arguments, or no field or variant | [03](03-types-and-values.md) |
| E0421 | a type contains itself | [03](03-types-and-values.md) |
| E0422 | struct literal fields: not a struct, a field given twice, or a field missing | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0423 | `match` needs an enum value | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0424 | the type of `None` / `Ok` / `Err` is not known here | [03](03-types-and-values.md) |
| E0425 | `==` on data types is not executable (compare the fields) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0426 | `match` arms: one per variant, in declaration order | [03](03-types-and-values.md), [04](04-expressions-and-control.md), [07](07-proof-gate.md) |
| E0427 | a pattern names the wrong number of payload values | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0428 | a type argument does not satisfy a bound of its parameter | [03](03-types-and-values.md), [05](05-ownership.md) |
| E0429 | wrong number of type arguments in a call | [03](03-types-and-values.md) |
| E0430 | a recursive generic call wraps a type parameter in a larger type | [03](03-types-and-values.md) |
| E0431 | `==` on a recursive type that holds a `Vec` (in a specification) | [03](03-types-and-values.md) |
| E0432 | a value of a recursive struct is built in a specification | [03](03-types-and-values.md) |
| E0433 | a value is used whole after one of its fields was moved | [05](05-ownership.md) |
| E0434 | struct literal fields must be given in declaration order (the order of evaluation) | [03](03-types-and-values.md), [04](04-expressions-and-control.md) |
| E0435 | a range `lo..hi` outside the source argument of `copy_within` and the bounds of a `&str` argument `&s[lo..hi]` | [03](03-types-and-values.md) |
| E0436 | a `std::process::Child` built or read by its fields (it comes from `std::process::spawn`) | [03](03-types-and-values.md) |
| E0437 | a type a `const` cannot have: a `const` holds a value type, an integer, `bool` or `char`, or an array, struct or enum of them (ADR-0052 §2) | [03](03-types-and-values.md) |
| E0438 | an inclusive range `lo..=hi` outside the range of a `for` loop | [03](03-types-and-values.md) |
| E0501 | a contract clause with a quantifier must be a call to a named predicate (`spec fn ... -> bool`); one without is an expression (ADR-0095 A2) | [06](06-contracts-and-specs.md) |
| E0502 | vacuous predicate used in a contract | [06](06-contracts-and-specs.md) |
| E0503 | specification-only construct used in executable code, or an executable statement in a specification | [04](04-expressions-and-control.md), [05](05-ownership.md), [06](06-contracts-and-specs.md) |
| E0504 | executable function called in a specification | [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0505 | spec function called in executable code | [06](06-contracts-and-specs.md) |
| E0506 | `while` requires `decreases` | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0507 | recursive function requires `decreases` | [06](06-contracts-and-specs.md) |
| E0508 | `old(..)` is only allowed in `ensures`, loop invariants and `assert` on a `&mut` parameter | [06](06-contracts-and-specs.md) |
| E0509 | `#[uses(..)]` entry is not a spec function | [06](06-contracts-and-specs.md), [07](07-proof-gate.md), [08](08-effects-and-escape-hatches.md) |
| E0510 | `#[trigger]` is only allowed inside `forall` or `exists` | [06](06-contracts-and-specs.md) |
| E0511 | `forall` and `exists` require at least one `#[trigger]` | [06](06-contracts-and-specs.md) |
| E0512 | the triggers of a `forall` or `exists` must together mention every bound variable | [06](06-contracts-and-specs.md) |
| E0513 | conversions are not used in specifications (integer types mix there) | [03](03-types-and-values.md), [06](06-contracts-and-specs.md) |
| E0514 | `==` is not defined for `Vec` | [03](03-types-and-values.md), [06](06-contracts-and-specs.md) |
| E0515 | `assert(..) by (bit_vector)` takes integer, `f64` and `bool` terms only (no quantifier, `old` or data value; an element, field or length that names a `let` of the assertion) | [06](06-contracts-and-specs.md) |
| E0516 | unknown prover in `assert(..) by (..)` (the one prover is `bit_vector`) | [04](04-expressions-and-control.md), [06](06-contracts-and-specs.md) |
| E0601 | function requires an `#[effects(..)]` attribute | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0602 | unknown effect | [08](08-effects-and-escape-hatches.md) |
| E0603 | `pure` function calls a function with effect `io` | [08](08-effects-and-escape-hatches.md) |
| E0604 | spec functions take no `#[effects(..)]` attribute | [02](02-units-items-names.md), [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0605 | unknown or misplaced attribute | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0606 | the body needs an effect the function does not declare | [08](08-effects-and-escape-hatches.md) |
| E0607 | `pure` listed with another effect, or an effect listed twice | [08](08-effects-and-escape-hatches.md) |
| E0608 | a function value whose effects the generic function that calls it does not declare (ADR-0053 §5) | [06](06-contracts-and-specs.md), [08](08-effects-and-escape-hatches.md) |
| E0701 | `main` must be `#[effects(io)] fn main() -> ()` | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0702 | more than one `main` | [02](02-units-items-names.md) |
| E0703 | items must be declared inside a `mod` | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md) |
| E0704 | `mod` requires a `#[summary("..")]` attribute | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E0801 | precondition of a call may not hold | [07](07-proof-gate.md) |
| E0802 | postcondition may not hold | [07](07-proof-gate.md) |
| E0803 | arithmetic may overflow | [07](07-proof-gate.md) |
| E0804 | divisor may be zero | [07](07-proof-gate.md) |
| E0805 | index may be out of bounds | [07](07-proof-gate.md) |
| E0806 | loop invariant may not hold on entry | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0807 | loop invariant may not be preserved | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0808 | loop `decreases` may not decrease or may be negative | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0809 | recursive call may not decrease the measure | [07](07-proof-gate.md) |
| E0810 | assertion may not hold | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0811 | solver could not decide (resource limit or incompleteness) | [07](07-proof-gate.md) |
| E0812 | proof gate cannot run (solver missing, wrong version or failed) | [07](07-proof-gate.md) |
| E0813 | recursive spec function may not terminate | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0815 | integer conversion may be out of range (to `f64`: a 64-bit value may be above 2^53 and inexact) | [07](07-proof-gate.md) |
| E0816 | the capacity of a `Vec` may exceed its maximum length | [07](07-proof-gate.md) |
| E0817 | a trait impl's function does not refine the trait's contract | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0818 | a `by (bit_vector)` sub-proof may not hold, or its assertion has no bit-vector translation | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0819 | an argument of a `proof fn` call may be outside its parameter's type (ghost arithmetic is mathematical, ADR-0062 §2) | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E0820 | a string position may not be a character boundary (`&s[a..b]` needs `s.is_char_boundary(a)`, `s.is_char_boundary(b)` and `a <= b`, ADR-0050 §2) | [07](07-proof-gate.md) |
| E0823 | a type invariant may not hold where a value leaves the function's hands: a call argument, a return, a `&mut` parameter at exit, a loop head (ADR-0060 §2) | [06](06-contracts-and-specs.md), [07](07-proof-gate.md) |
| E1001 | an escape (`assume`, `#[external_body]`) without a `#[rationale(..)]` before it | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E1002 | a malformed `#[rationale(..)]`: `kind` outside the list, no explanation, or not before an escape | [01](01-lexical-and-syntax.md), [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |
| E1003 | `assume` takes one named predicate call, in executable code | [08](08-effects-and-escape-hatches.md) |
| E1004 | `#[external_body]` on a spec fn, a proof fn or `main` | [02](02-units-items-names.md), [08](08-effects-and-escape-hatches.md) |

<!-- reference:index end -->
