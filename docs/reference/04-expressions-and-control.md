# 04 Expressions, statements and control flow

This chapter describes what goes inside a function body: expressions (literals, paths, calls, operators, conversions, places, struct and enum values, blocks, closures), statements (`let`, assignment, expression statements, `assert`), control flow (`if`, `match`, `while`, range `for`, `loop`, `break` and `continue`, `return`, `?`), and the Rust habits that are not accepted together with the form to write in their place (all of E02xx). Ownership is in chapter 05, contracts in chapter 06, proof obligations in chapter 07.

## Expression grammar and precedence

```
expr    = "return" [expr] | "break" [label] [expr] | "continue" [label] | binary
binary  = unary { binop unary }
unary   = ("-" | "!" | "*" | "&" ["mut"]) unary | postfix
postfix = primary { "." name | "." name "(" args ")" | "[" expr "]" | "?" }
primary = literal | path | path ["::<" types ">"] "(" args ")" | path "{" name ":" expr {"," name ":" expr} "}"
        | "[" expr {"," expr} "]" | "[" expr ";" N "]" | "(" expr ")" | "()" | block
        | if | match | loop | closure
```

Binary operators bind, from loosest to tightest, in the order `||`, `&&`, comparison (`==` `!=` `<` `<=` `>` `>=`), `|`, `^`, `&`, `<<` `>>`, `+` `-`, `*` `/` `%`, `as` (the same as Rust, ADR-0048 §2: `a & b == c` is `(a & b) == c`). `return`, `break` and `continue` are the loosest. The specification-only `==>` is E0503 in executable code. Comparisons cannot be chained (E0211). Syntactic nesting goes up to 256 levels; beyond that it is E0107. Operators of the same level bind from the left (`7 - 4 - 2` is 1). Operands and call arguments are evaluated from the left. Corpus: `tests/run/order.bmb` (one value per level, evaluation order), `tests/run/basics.bmb`, `tests/check/err/rust_habits.bmb`.

## Literals, paths and calls

- The forms of literals are in chapter 01. `()` is the unit value.
- Local names are made by `let`, parameters, `match` bindings and `for` variables. An unknown name is E0301, and a new name that shadows a visible name is E0303.
- A function call is written with a path: `f(a)` for a function of the same unit, `unit::f(a)` from the root for a function of another unit, `T::f(a)` for a function of a type. There is no `use` (E0214), so a name from another unit is always a full path. `crate::`, `super::` and `self::` are E0242, an unknown function is E0304, the argument count is E0405, and an argument type is E0401.
- A call of a generic function writes its type arguments: `id::<i64>(x)`. Leaving them out is E0237 (ADR-0046 §3). A type position that has no name (a function value) is `_`.

Corpus: `tests/run/generics.bmb`, `tests/check/err/rust_paths.bmb`, `tests/check/err/generics.bmb`.

## Method calls

`x.f(a, ..)` is `T::f(adjust(x), a, ..)` (ADR-0051). `T` is the type of `x` (the pointed-to type if it is a reference), and `f` is first an inherent `impl` function of `T`, and if there is none, the one trait that has `f`. A built-in method comes before a trait method of the same name. The adjustment is one step and the declared receiver decides it: `&x` for `self: &T`, `&mut x` for `self: &mut T` (E0408 if it is not a writable place), `x` for `self: T`. If `x` is already a reference it is passed as it is.

If two traits have the same name it is E0313, calling as a method an associated function with no receiver or a function with a parameter of its own type is E0314 (both are written as path calls), and an unknown method is E0404. Corpus: `tests/run/methods.bmb`, `tests/run/method_order.bmb` (an inherent function before a trait, a built-in method before a trait), `tests/check/err/methods.bmb`.

## Operators and operands

| Operator | Operands | Value |
| --- | --- | --- |
| `+` `-` `*` `/` `%` | two of the same integer type, or two `f64` (except `%`) | that type |
| unary `-` | a signed integer, `f64`. An unsigned integer is E0411 | that type |
| `==` `!=` `<` `<=` `>` `>=` | two integers, `f64`, `char` or `bool` of the same type. For `bool`, `false < true` (`tests/run/bool_order.bmb`) | `bool` |
| `&&` `\|\|` | two `bool`. The right side is evaluated only when needed (`tests/run/order.bmb`) | `bool` |
| `!` | `bool` (logical negation) or an integer (bit inversion) | that type |
| `&` `\|` `^` | two of the same integer type, or two `bool` (both sides are evaluated) | that type |
| `<<` `>>` | the shifted value is an integer, the amount is any integer type | the type of the shifted value |

- Mixing integers of different width or signedness is E0401. There is no implicit widening; match them with `x as T` (ADR-0048 §1).
- `==` on structs and enums has a meaning only in specifications. In executable code it is E0425; compare the fields or `match`.
- Overflow, division by zero and a shift amount of the width or more are not type errors but proof obligations (chapter 07).

Compound assignment `t op= e;` (`+=` `-=` `*=` `/=` `%=` `&=` `|=` `^=` `<<=` `>>=`) is an input alias of `t = t op e;` (ADR-0095). The normal form is the expanded form. If the place `t` calls a function it is E0103. Corpus: `tests/run/bits.bmb`, `tests/check/err/bits.bmb`, `tests/check/err/widths.bmb`.

## Conversions

- `x as T` is a conversion between integer types (ADR-0028, ADR-0095 A4). It leaves the value as it is, and if the target cannot hold the range of the source a range obligation arises (chapter 07). `-x as u64` is `(-x) as u64`.
- `T::try_from(x).unwrap()` is an input alias of `x as T`. `T::try_from(x).ok()` is `Option<T>` and has no obligation. Any other use of `try_from` and a source that is not an integer (`bool`, `char`) are E0412.
- From an integer to `f64` it is `f64::from(x)` (ADR-0061). `x as f64` and `as` with a target that is not an integer are E0205. There is no conversion from `f64` to an integer.
- `T::from(x)`, `x.into()` and `x.try_into()` are E0212, and `T::try_from(x).expect(..)` is E0213.

Corpus: `tests/run/conversions.bmb`, `tests/check/err/conversions.bmb`, `tests/check/err/float_literals.bmb`.

## Indexing, fields and dereference

- `a[i]`: `a` is an array, a slice or a `Vec`, and `i` is `usize` (another integer type is E0401, a value that cannot be indexed is E0407). Being in range is a proof obligation (chapter 07).
- `p.x`: a field of a struct value or of a reference to one. A read through a reference is also written `p.x` (`(*p).x` is E0231). A field that does not exist is E0306.
- `*r` is the value the reference points to, and `&e` and `&mut e` are borrows in argument position (chapter 05).
- A place that can be assigned to is a path that chains variables, fields, elements and dereferences: `g[i][j]`, `b.rows[i].cells[j]`, `*r`.

Corpus: `tests/run/arrays.bmb`, `tests/run/fields.bmb`, `tests/check/err/places.bmb`.

## Struct and enum values

- A struct value is `T { a: e1, b: e2 }`. All fields are written in declaration order: a missing or repeated field is E0422, a different order is E0434 (the written order is the evaluation order, ADR-0080). The shorthand `T { x }` is E0222, and the update syntax `..p` is E0223.
- An enum value is `T::V` or `T::V(e, ..)`. A different number of payloads is E0405, and a variant that does not exist is E0307. Built-in variants are written by bare name: `Some(e)`, `None`, `Ok(e)`, `Err(e)` (`Option::Some` is E0221).
- Array values are `[a, b, c]` and `[v; N]` (chapter 03). Tuples have neither values nor types (E0233, `tests/check/err/reserved_names.bmb`).

Corpus: `tests/run/shapes.bmb`, `tests/run/points.bmb`, `tests/check/err/data_values.bmb`, `tests/check/err/field_order.bmb`.

## Blocks and tail expressions

```
block = "{" { stmt } [ expr ] "}"
```

A block is an expression. Its value is the value of the last expression without `;` (the tail expression), and if there is no tail expression it is `()`. A block without a tail that ends in a statement that does not finish (`return`, `break`, `continue`) has type `!` and fits any expected type (ADR-0049). A function body is a block and its value is the result of the function. Names inside a block are not visible outside (E0301). Corpus: `tests/run/scopes.bmb`, `tests/run/returns.bmb`.

## Closures

```
closure = ["move"] "|" [ name ":" type {"," name ":" type} ] "|" "->" ret [requires ..,] [ensures ..,] block
```

A closure comes only in the argument position of a type parameter that has an `Fn`, `FnMut` or `FnOnce` bound (ADR-0053). Binding it with `let` or returning it is E0264. The types of the parameters and of the result are written (E0263). If it captures an outer local it uses `move` (E0262): the captures move into the closure. A function item is passed in the same position too. A received function value is called as `f(a)`.

```
#[effects(pure)]
fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 { f(x) }

#[effects(pure)]
fn shifted(k: i64) -> i64 {
    apply::<_>(move |x: i64| -> i64 { x + k }, 1)
}
```

Corpus: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.

## Statements

```
stmt = "let" ["mut"] name ":" type "=" expr ";"
     | place "=" expr ";"  |  place op"=" expr ";"
     | expr ";"
     | "assert" "(" expr ")" ["by" "(" "bit_vector" ")"] ";"
     | while | for
```

- `let` requires a type and an initial value. Without a type it is E0201. Pattern `let` and `let .. else` are E0226, and the destructuring `let T { x, y } = p;` is E0230.
- Assignment is made only to `let mut` locals and to places reached through `&mut`. Assigning to a name bound with `let`, to a parameter or to a `for` variable is E0408 (a `mut` parameter is E0210: copy it into a `let mut` local). Assignment is not an expression (E0103).
- An expression statement `e;` discards the value. An `if`, `match` or `loop` in statement position ends at its closing `}` (`if c { return; } *x = 1;` is two statements).
- The `e` of `assert(e);` is `bool`. It is not a run-time check but a proof obligation (chapter 07). A macro such as `assert!(..)` is E0202. The only prover after `by` is `bit_vector` (E0516).

Corpus: `tests/check/err/rust_habits.bmb`, `tests/check/err/habits.bmb`, `tests/verify/pass/conditions.bmb`, `tests/verify/pass/bit_vector.bmb`.

## if and match

```
if    = "if" expr block [ "else" (if | block) ]
match = "match" expr "{" { pat "=>" expr "," } "}"
pat   = path | path "(" bind {"," bind} ")"        bind = name | "_"
```

- The condition of `if` is `bool` (E0401). The two branches have the same type and that is the type of the expression. Without `else` the type is `()`. `if let` is E0225.
- The subject of `match` is an enum value or a reference to one (anything else is E0423: branch with `if`). If a reference is `match`ed, the bindings are references too (ADR-0055).
- The arms are written exactly one per variant, in declaration order. A missing variant, a variant written twice and a different order are all E0426. The wildcard arm `_ =>` is E0224 (exhaustiveness is shown by writing the variants, ADR-0013).
- A pattern is one variant and, for each of its payload positions, a new name or `_`: `Shape::Dot`, `Shape::Rect(a, _)`, `Some(v)`, `None`. A different number of positions is E0427, and something that is not a variant path is E0307. Guards, or-patterns, literal, tuple and reference patterns, nested patterns and struct patterns are E0232.
- The types of the arms are all the same. An arm of type `!` (`None => return d,`) follows the type of the other arms.

```
#[effects(pure)]
fn pick(s: S, d: i64) -> i64 {
    match s {
        S::A => d,
        S::B(n) => n,
        S::C(_) => return 0,
    }
}
```

(`enum S { A, B(i64), C(P) }`) Corpus: `tests/run/shapes.bmb`, `tests/run/mut_match.bmb`, `tests/check/err/match.bmb`.

## Loops

```
while = [label ":"] "while" expr [invariant e {"," e} ","] "decreases" e "," block
for   = [label ":"] "for" name "in" expr (".." | "..=") expr [invariant e {"," e} ","] block
loop  = [label ":"] "loop" [invariant e {"," e} ","] "decreases" e "," block
```

- The grammar is that of the normal form: the comma at the end of a clause and the comma after an arm with a block body are accepted when missing from the input, and the normal form writes them.
- The condition of `while` is `bool`. `while` and `loop` require `decreases` (E0506). The meaning of invariants and of the decreasing expression is in chapters 06 and 07. `while let` is E0226.
- `for` runs over integer ranges only (ADR-0049 §2). `lo..hi` excludes `hi` and `lo..=hi` includes it. The type of the variable is the integer type of the two ends (`i64` if both are literals), and the two ends are evaluated once, from the left, on entry (`tests/run/order.bmb`). The variable is visible only inside the loop (E0301 outside) and cannot be assigned to (E0408). An iterator (`for x in v.iter()`) is E0252, and `decreases` is E0253 (the measure is automatic). An end that is not an integer is E0401.
- `while` and `for` are statements and have no value. `loop` is an expression: its value is the value of `break e`, and the values of all `break`s have one type. A `loop` without `break` has type `!`.

```
#[effects(pure)]
fn first(a: &[i64], t: i64) -> Option<usize> {
    'scan: for i in 0..a.len() {
        if a[i] == t { return Some(i); }
        if a[i] > t { break 'scan; }
    }
    None
}
```

Corpus: `tests/run/ranges.bmb`, `tests/run/loop_exits.bmb`, `tests/check/err/ranges.bmb`, `tests/verify/pass/loops.bmb`.

## Early exit: return, break, continue, ?

- `return e` and `return` leave the function. The type of `e` is the result type of the function. They also come in expression position, and the type is `!`.
- `break` and `continue` act on the innermost loop, and `break 'l` and `continue 'l` on the enclosing loop labelled `'l`. Outside a loop it is E0250, and a label that no enclosing loop has is E0317. `break e` with a value leaves only a `loop` (in `while` and `for` it is E0251).
- `x?`: if `x` is `Option<T>` and the function result is `Option<U>` it returns `None`, if `x` is `Result<T, E>` and the result is a `Result<U, E>` of the same `E` it returns `Err(e)`, and otherwise the value is `T`. A different error type, mixing `Option` and `Result`, and any other result type are E0241 (there is no implicit conversion). `.unwrap()` and `.expect()` are E0229.

Corpus: `tests/run/returns.bmb`, `tests/run/tries.bmb`, `tests/check/err/loop_exits.bmb`, `tests/check/err/tries.bmb`.

## Rust habits that are not accepted

The forms below are not programs (ADR-0013: make choices mandatory). The diagnostic carries the form to write instead in `fix`, and where it is determined mechanically it also carries the replacement text (`replacement`). Examples are in `tests/check/err/rust_habits.bmb`, `tests/check/err/habits.bmb` and the files under each table.

### Declarations and signatures

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0201 | `let y = e;` | `let y: T = e;` |
| E0209 | `fn f()` without a return type | `fn f() -> ()` |
| E0210 | `fn f(mut x: T)` | copy into a `let mut` local inside the body |
| E0208 | lifetime `'a` | delete it (references do not write lifetimes) |
| E0228 | `&self`, `&mut self`, `self` | `self: &T`, `self: &mut T`, `self: T` |
| E0220 | `#[derive(..)]` | delete it |
| E0261 | `static X`, `static mut X` | `const X`; global mutable state is passed as an argument |
| E0207 | `pub`, const generics, generic traits, supertraits, items a unit does not hold, contract clauses on a `spec fn`, `decreases` on a trait method | delete `pub`; lengths are slices and `len()`; type parameters go on the method; bounds are `T: A + B` |

Corpus: `tests/check/err/items.bmb`, `tests/check/err/unsupported.bmb`, `tests/check/err/consts.bmb`.

### Names, paths, macros and escape hatches

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0214 | `use a::f;` and the names shortened by it | the full path `a::f(..)` at the place of use |
| E0242 | `crate::`, `super::`, `self::` | the unit path from the root `unit::f` |
| E0221 | `Option::Some`, `Result::Ok` and the like | `Some`, `None`, `Ok`, `Err` |
| E0237 | a generic call without type arguments `id(x)` | `id::<i64>(x)` |
| E0202 | macro `name!(..)` | for `println!`, `print_i64(x)`; for `vec!`, `[a, b]` and `[v; N]`, or `Vec::with_capacity(n)` and `push`; for `assert!`, `assert(c);`; for the `panic!` family, prove that the case does not occur |
| E0215 | `unsafe` | a body that cannot be proved is an `#[external_body]` function with a rationale (chapter 08) |
| E0216 | `#[verifier::external_body]` | `#[external_body]` |
| E0217 | `admit()` | `assume(p(..));` with a rationale (chapter 08) |

Corpus: `tests/check/err/rust_paths.bmb`, `tests/check/err/escapes.bmb`.

### Conversions and operators

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0205 | `x as f64`, `as` with a target that is not an integer | `f64::from(x)`; between integers `x as T` |
| E0212 | `T::from(x)`, `x.into()`, `x.try_into()` | `x as T` |
| E0213 | `T::try_from(x).expect("..")` | `T::try_from(x).unwrap()`, the canonical form is `x as T` |
| E0218 | a `Vec` or `String` in the position of a slice or `&str` | `v.as_slice()`, `s.as_str()` |
| E0211 | `a < b < c` | `a < b && b < c` |

Corpus: `tests/check/err/conversions.bmb`, `tests/check/err/strings.bmb`, `tests/check/err/vec.bmb`.

### Data and patterns

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0222 | `Point { x, y }` | `Point { x: x, y: y }` |
| E0223 | `Point { x: 1, ..p }` | all fields: `Point { x: 1, y: p.y }` |
| E0224 | `_ => e` | one arm per variant (the fix carries the whole `match` in declaration order) |
| E0225 | `if let Some(v) = o { a } else { b }` | `match o { None => { b }, Some(v) => { a } }` |
| E0226 | `while let`, `let .. else`, pattern `let` | a `while` that runs on a condition with a `match` inside; `match` the value |
| E0229 | `o.unwrap()`, `o.expect(..)` | `match` the two cases: `match o { None => .., Some(v) => v }` |
| E0230 | `let Point { x, y } = p;` | `let x: i64 = p.x; let y: i64 = p.y;` |
| E0231 | `(*p).x` | `p.x` |
| E0232 | guards, or-patterns, literal, tuple, reference, nested and struct patterns | an `if` in the arm body; an arm per variant; bind a name and `match` in the body; fields are `p.x` |
| E0233 | tuples, tuple structs, struct-form variants | a struct with named fields; a variant is `V(Data)` |

### Control flow

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0241 | `?` on a `None` or `Err` that the function cannot return | `match` and return it converted with `return Err(..)` |
| E0250 | `break` or `continue` outside a loop | use it inside a loop; to leave the function, `return` |
| E0251 | `break e` in `while` or `for` | assign to a variable before the loop, then `break` |
| E0252 | `for x in v.iter()`, `for x in &v` | `for i in 0..v.len()` and `v[i]` |
| E0253 | `decreases` on `for` | delete it (the range bounds the loop) |

### Ownership, generics and function values

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0219 | `drop(v)`, `std::mem::take` | a value is freed at the end of its block; to change it, assign a new value |
| E0238 | `Rc`, `RefCell`, `Cell`, `Arc` | an ownership tree (`Box`, `Vec`) or indices into a `Vec` |
| E0239 | moving an owned value out from behind `&mut` | `std::mem::replace(&mut r.f, new value)`, `std::mem::swap` |
| E0235 | `impl<T> Trait for T` | an `impl` per type, or the bound `T: Trait` at the place of use |
| E0236 | `dyn Trait` | a type parameter with a trait bound, or an enum of closed variants |
| E0240 | `where T: Copy` | the bound next to the parameter `<T: Copy>` |
| E0262 | no `move` on a capturing closure | `move \|x: T\| -> R { .. }` |
| E0263 | untyped closure `\|x\| x + 1` | `\|x: T\| -> R { .. }` |
| E0264 | function pointer type `fn(A) -> R`, a returned closure, a closure in a position that is not an argument | pass it directly as the argument of a type parameter `F: Fn(A) -> R` |
| E0265 | argument position `f: impl Fn(A) -> R` | `<F: Fn(A) -> R>` and `f: F` |

Corpus: `tests/check/err/mem_moves.bmb`, `tests/check/err/boxes.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/fn_values.bmb`.

### Specifications, ghost code and constants

The rules are in chapters 06 and 02.

| Code | Form not accepted | Form to write instead |
| --- | --- | --- |
| E0254 | the `p` of `#[invariant(p)]` is not a `spec fn p(x: &T) -> bool` | declare a `spec fn` of that form |
| E0255 | two or more `#[invariant(..)]` on one type | join them with `&&` in one predicate |
| E0256 | an effect attribute or a return type on a `proof fn` | delete it (what it proves is the `ensures`) |
| E0257 | an executable call, assignment, `let mut`, `&mut` or loop inside ghost code | make the call outside the `proof` block; bind values with `let` |
| E0258 | a `proof fn` called outside a `proof` block | `proof { f(x); }` |
| E0259 | something that is not a constant expression, such as a function call, in a `const` value | literals, other `const`s, operators, `if`, `match`, `as`, arrays, structs and variants (ADR-0052 §3) |
| E0260 | overflow, division by zero, or an out-of-range conversion or index in a `const` computation | fix the value or give a type that holds that value |

Corpus: `tests/check/err/invariants.bmb`, `tests/check/err/proofs.bmb`, `tests/check/err/consts.bmb`.
