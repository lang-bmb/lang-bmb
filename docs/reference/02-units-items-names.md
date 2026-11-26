# 02 Units, items and names

This chapter gives the unit a program is built from (`mod`), the items that come in a unit (functions, types, traits, `impl`, constants), attributes and their positions, the entry point `main`, and the rules by which a name resolves to a definition. Tokens and the skeleton of the syntax are in chapter 01, the type rules of each item in chapter 03, contracts in chapter 06, and the meaning of effects and escape hatches in chapter 08. Breaking a rule of this chapter gives E03xx and E07xx.

## Units

```
unit = "#[summary(" string ")]" "mod" ident "{" item* "}"
```

- A program is a sequence of units. Items come only inside a unit. A top-level function is an E0703 error (`tests/check/err/items.bmb`).
- A unit must have a summary. Without one it is E0704, and it is also an E0704 error if the argument of `#[summary(..)]` is not one string (`tests/check/err/attrs.bmb`). Any other attribute on a unit is an E0605 error.
- A unit holds units. A nested unit must have a summary too, and its path is `outer::inner` (`outer::inner` in `tests/check/err/rust_paths.bmb`).
- The name of a top-level unit is unique in the program. A second unit with the same name is an E0302 error. One unit cannot be written as several blocks.
- The top-level unit `std` belongs to the standard library and cannot be defined (E0309, `tests/check/err/std_unit.bmb`).
- There is no `pub` (E0207). Every definition of a unit can be used from another unit by its full path.

## Items

```
const  = "const" ident ":" type "=" expr ";"
struct = "struct" ident [generics] "{" ident ":" type ("," ident ":" type)* [","] "}"
enum   = "enum" ident [generics] "{" variant ("," variant)* [","] "}"
variant = ident [ "(" type ("," type)* ")" ]
trait  = "trait" ident "{" ( fn-head (";" | block) )* "}"
impl   = "impl" [generics] ident [type-args] "{" fn* "}"
       | "impl" [generics] path "for" type "{" fn* "}"
fn     = ["spec" | "proof"] "fn" ident [generics] "(" params ")" ["->" ret] clauses block
```

- There are three kinds of function: the executable function `fn`, the specification function `spec fn` and the proof function `proof fn` (chapter 06). `fn` and `spec fn` write the return type (including `-> ()`; without it, E0209). `proof fn` does not write one (E0256). The accepted forms are in `tests/run/basics.bmb`, `tests/verify/pass/proofs.bmb`.
- A `const` writes its type. `const`s that refer to each other are an E0318 error (ADR-0052 §3, `tests/check/err/consts.bmb`). The expressions that can come as the value are in chapter 03.
- Inside a unit, the names of constants, types, traits, nested units and functions are one namespace. Two of the same name are an E0302 error (`tests/check/err/items.bmb`). The fields of one struct, the variants of one enum, the methods of one trait, the type parameters of one list and the functions of the `impl` blocks of one type also each have unique names (E0302, `tests/check/err/data_types.bmb`, `tests/check/err/impl_blocks.bmb`). The variants of an enum are not in the namespace of the unit.
- Types, functions, traits and constants cannot use the built-in names `Option`, `Result`, `Some`, `None`, `Ok`, `Err`, `Vec`, `Box` (ADR-0026), and types, traits and constants cannot use the names of primitive types (`i64`, `bool`, `String` and so on) either (`tests/check/err/reserved_names.bmb`). A trait cannot use the names of the built-in traits `Copy`, `Eq`, `Ord` (ADR-0046 §1). Both are E0309 errors (`tests/check/err/data_types.bmb`, `tests/check/err/traits.bmb`).

## `impl` and receivers

- An inherent `impl T { .. }` comes only in the unit that declares `T`. For a type of another unit or a type that does not exist it is an E0308 error. Each type has one block per list of bounds, and a second block with the same bounds is an E0308 error (ADR-0097, `tests/check/err/impl_blocks.bmb`, accepted forms `tests/run/impl_blocks.bmb`).
- In a function of an `impl`, if the type of the first parameter is `T`, `&T` or `&mut T`, it is the receiver and its name is `self`. Any other name is an E0310 error. `self` names only the receiver: as the name of a parameter from the second on, of a parameter of a function outside an `impl`, or of a parameter that does not have the receiver type, it is an E0310 error (`tests/check/err/data_types.bmb`). The type of the receiver is written (`self: &T`. `&self` is E0228).
- A trait impl `impl Tr for T { .. }` comes in the unit of `Tr` or in the unit of `T`. In neither it is an E0312 error. Two impls of one trait that can be for the same type are an E0311 error (`tests/check/err/traits.bmb`).
- A trait impl defines all the methods of the trait (a method that has a body can be left out: `tests/run/trait_defaults.bmb`). A missing method and a function that the trait does not have are E0315 errors. The signature of a function must be the same as that of the trait with `Self` replaced by the implementing type, and it cannot have an effect that the trait method does not declare. Both are E0316 errors (`tests/check/err/traits.bmb`, `tests/check/err/effects.bmb`).
- `Self` is a type only inside a trait declaration. In any other position it is an E0305 error.

## Attributes

An attribute comes right before the item it attaches to. The names are a closed list, and a name not in the table or a position not in the table is an E0605 error (`tests/check/err/attrs.bmb`, `tests/check/err/items.bmb`).

| Attribute | Position | Chapter with the meaning |
| --- | --- | --- |
| `#[summary("..")]` | unit (required), function, `struct`, `enum`, `trait`, `const` (optional) | this section |
| `#[effects(..)]` | `fn` (required; without it, E0601). E0604 on a `spec fn`, E0256 on a `proof fn` | chapter 08 |
| `#[uses(..)]` | function | chapter 07 |
| `#[external_body]` | a `fn` that is not `main` (E1004), with `#[rationale]` before it (E1001) | chapter 08 |
| `#[rationale(kind = K, "..")]` | function (the rationale of `#[external_body]`), before the statement `assume(..)` | chapter 08 |
| `#[invariant(p)]` | `struct`, `enum` (one per type, E0255) | chapter 06 |

- A summary takes one string. Otherwise it is an E0704 error, and two on one definition are an E0605 error (ADR-0117, `tests/check/err/summaries.bmb`). The accepted forms are in `tests/verify/pass/summaries.bmb`. `///` is an input form of this attribute (chapter 01). The summary is not part of the identity hash or of the proof.
- A function inside a trait or an `impl` also takes the attributes of a function. An `impl` block itself takes no attributes: an attribute or `///` before it is an E0605 error (`tests/check/err/attr_places.bmb`).
- The `#[summary]` of a unit is also written only once. A second one is an E0605 error (same file).
- The `#[rationale(..)]` of a function comes only together with `#[external_body]`. Alone it is an E1002 error (same file).
- Inside a function body, the only position where an attribute comes is `#[rationale(..)]` before an `assume(..)` statement. Anywhere else is an E1002 error.
- The attributes of one definition are the same program in whatever order they are written.

## Entry point

The executable function `main` is the entry point. Its form is fixed as `#[effects(io)] fn main() -> ()`: it has no parameters, its result is `()`, and its declared effects include `io` (other effects can be written with it). Otherwise it is an E0701 error. Whatever unit it is in (nested units included), a program has one `main`, and from the second on it is an E0702 error (`tests/check/err/effects.bmb`). A program with no `main` is also subject to checking and proof. `spec fn main` is not an entry point. The accepted form is `app::main` in `tests/run/basics.bmb`.

## Paths and name lookup

There is no `use` (E0214). A name resolves in the following order.

- A single name: a definition of the unit where the name is used. In a value position it is a local name or a `const` of that unit; in a type position, a type of that unit or a built-in type; in a call, a function of that unit and, if there is none, a built-in function (`print_i64` and so on).
- `T::f`, `Tr::f`, `T::V`: if `T` or `Tr` is a type or trait of the unit where it is used, that function, method or variant.
- Any other path is a full path that starts from a top-level unit: `unit::f`, `unit::T`, `unit::T::f`, `unit::N`, `outer::inner::f`. There are no relative paths. A nested unit using a name of the outer unit, and an outer unit using a name of a nested unit, also use the full path. A name of the unit itself may be written with its full path. `crate::`, `super::` and `self::` are E0242 errors (`tests/check/err/rust_paths.bmb`).

If it does not resolve, the code depends on the kind.

| Code | What did not resolve | Corpus |
| --- | --- | --- |
| E0301 | a name in a value position (local, `const`), a path that is not a value (`unit::f` with no call arguments) | `tests/check/err/types.bmb`, `tests/check/err/consts.bmb` |
| E0304 | the function of a call | `tests/check/err/types.bmb` |
| E0305 | a type, a trait, a constant of a built-in type (`i64::MID`) | `tests/check/err/data_types.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/types.bmb` |
| E0306 | a field of a struct (construction, read) | `tests/check/err/data_values.bmb` |
| E0307 | a variant of an enum (construction, pattern) | `tests/check/err/data_values.bmb`, `tests/check/err/match.bmb` |
| E0317 | the label of `break` or `continue` (only those of enclosing loops) | `tests/check/err/loop_exits.bmb` |

```
#[summary("Shapes")]
mod geo {
    struct P { x: i64 }

    #[effects(pure)]
    fn origin() -> P { P { x: 0 } }
}

#[summary("Entry point")]
mod app {
    #[effects(io)]
    fn main() -> () {
        let p: geo::P = geo::origin();
        print_i64(p.x);
    }
}
```

## Method call names

The `f` of `x.f(..)` is first a function of the inherent `impl` of the type of `x` and, if there is none, the one trait that has a method of that name (ADR-0051, `tests/run/methods.bmb`). If two or more traits have that name it is an E0313 error, and the call is written with a path (`Tr::f(..)`). A function with no receiver and a function with type parameters of its own cannot be called as a method and are called with a path (E0314). Both are in `tests/check/err/methods.bmb`. Receiver adjustment and the type rules are in chapters 03 and 05.

## Local names

- A local name (`let`, a binding of a pattern, a `for` variable) cannot shadow a name visible at that position: a parameter, a local of an outer block, the result name (the `r` of `-> (r: T)`), a `const` of the unit where it is used. Shadowing is an E0303 error (`tests/check/err/types.bmb`, `tests/check/err/consts.bmb`, `tests/check/err/match.bmb`). Two parameters with the same name are also an E0303 error.
- Positions whose scopes do not overlap (sibling blocks, two `for`s, different `match` arms) can use the same name again. But in one function one name has one type. A different type is an E0303 error (ADR-0095 A3, accepted forms `tests/run/scopes.bmb`).
- A local name may coincide with the name of a function, type or unit. They are told apart by position.
- A `for` variable is visible only inside the loop. Using it after the loop is an E0301 error (`tests/check/err/ranges.bmb`).

## Standard library

The units of the standard library are one file each in `lib/std/` (`lib/std/str.bmb` is `std::str`), and as units under the top-level unit `std` they are used by the full path `std::NAME::..` (ADR-0092). There is no declaration or import. Only the units that the program uses by path and the units those units use are added to the program, and they are checked and proved by the same rules as the program's units. A unit or a name that does not exist is an E0304 or E0305 error like any other path. The accepted forms are in `tests/run/std_str.bmb`. Each file of `lib/std/` is the normal form of its unit, like any other BMB file, and the explanation of a definition is its `#[summary]` (ADR-0158). The API is outside the scope of this document.
