# 03 Types and values

This chapter gives the types a value can have and the operations on those types: scalar types, arrays, slices and text, structs and enums, generics and traits, function values, owning types, conversions, the types of constants, and the rules by which the checker matches types. Whether a value moves or is copied and the rules of references are in chapter 05; the proof obligations of overflow, bounds and conversion ranges are in chapter 07.

## Scalar types

| Type | Values | Notes |
| --- | --- | --- |
| `i8` `i16` `i32` `i64` `isize` | signed integers | `isize` is 64 bits |
| `u8` `u16` `u32` `u64` `usize` | unsigned integers | `usize` is 64 bits |
| `f64` | IEEE 754 double precision | `f64` is the only floating-point type |
| `bool` | `true`, `false` | |
| `char` | a Unicode scalar value | `'a'`, `'\u{1F600}'` |
| `()` | one value | the return type of a function with no result |

- `i128`, `u128` and `f32` are not types (E0402, ADR-0048, ADR-0061).
- An integer literal has no suffix (E0104) and its type comes from the expected type. A literal that does not fit in that type is an E0406 error (`let z: u8 = 256;`). `-128` is one literal of `i8`.
- Unary `-` on an unsigned value is an E0411 error.
- Every integer type has the constants `MIN` and `MAX` (`i64::MAX`).
- Corpus: `tests/run/widths.bmb`, `tests/run/floats.bmb`, `tests/check/err/widths.bmb`.

## Where types are written

Types are written. The checker infers only the type of a literal and the type arguments of a construction expression.

| Position | Rule | If broken |
| --- | --- | --- |
| `let` | `let x: T = e;` | E0201 |
| function result | `-> T`, or `-> ()` if there is no result | E0209 |
| parameters and result of a closure | `\|x: T\| -> R { .. }` | E0263 |
| `const` | `const N: T = e;` | E0103 |
| type arguments of a generic call | `f::<i64>(x)`, `Cell::<u64>::new(v)` | E0237 |
| type arguments of a struct construction or variant value | not written. They are determined from the expected type or from the field and payload values | E0301 (if written), E0424 (if they cannot be determined) |

- Only a call that passes a function value writes `_` in a type argument position (`apply::<_>(inc, 10)`).
- Corpus: `tests/check/err/types.bmb`, `tests/check/err/data_values.bmb`.

## Operators and operands

If the expected type and the type of the expression differ, it is an E0401 error. There are no implicit conversions.

| Operation | Operands | Result |
| --- | --- | --- |
| `+` `-` `*` `/` | two of the same integer type, or two `f64` | that type |
| `%` | two of the same integer type | that type |
| unary `-` | signed integer, `f64` | that type |
| `&` `\|` `^` | two of the same integer type, or two `bool` | that type |
| `!` | `bool` (logical negation), integer (bitwise inversion) | that type |
| `<<` `>>` | the shifted value is an integer, the amount is any integer type | the type of the shifted value |
| `==` `!=` | the same integer type, `bool`, `char`, `f64`, text | `bool` |
| `<` `<=` `>` `>=` | the same integer type, `char`, `f64`, `bool` (`false < true`), text | `bool` |
| `&&` `\|\|` | two `bool` | `bool` |

- The same holds for mixed widths (`i32 + i64`), `+` on `bool`, `+` on `char`, `%` on `f64`, and an integer in a condition position.
- The type of an index is `usize` (E0401).
- `==` on structs, enums, arrays and `Box` exists only in specifications (E0425, chapter 06). `==` on `Vec` is an E0514 error.
- For a value of a type parameter `T`, executable code needs `T: Eq` to use `==` and `!=`, and `T: Ord` to use ordering comparisons (E0428).
- The methods of the integer types (`wrapping_add`, `saturating_sub`, `checked_mul`, `rotate_left`, `count_ones` and so on) and the methods of `f64` (`sqrt`, `abs`, `min`, `max`, `is_nan`, `is_finite`) are closed lists. A method that does not exist is E0404, and a wrong number of arguments to a method or function is an E0405 error.
- Overflow, division by zero and the range of the shift amount are proof obligations (chapter 07).
- `x.to_be_bytes()` and `x.to_le_bytes()` are the bytes of an integer as `[u8; w / 8]`, and `T::from_be_bytes(b)` and `T::from_le_bytes(b)` the integer of such an array (an array of another length is E0401). The proof decides the round trips in both directions, in the same order and width: `T::from_le_bytes(x.to_le_bytes()) == x` and `T::from_le_bytes(b).to_le_bytes() == b`, also through locals and fields, and in code through bytes copied, pushed or stored one at a time (ADR-0157). Corpus: `tests/verify/pass/byte_round_trips.bmb`, `tests/verify/fail/byte_round_trips.bmb`.
- Corpus: `tests/run/bits.bmb`, `tests/run/arith_methods.bmb`, `tests/check/err/bits.bmb`, `tests/check/err/floats.bmb`.

## Conversions

| Form | Meaning |
| --- | --- |
| `x as T` | from an integer to the integer type `T`. The value stays the same, and if `T` cannot hold the range of the source a range obligation arises (chapter 07) |
| `T::try_from(x).ok()` | from an integer to `Option<T>`. No obligation |
| `f64::from(x)` | from an integer to `f64` |
| `u32::from(c)` | from `char` to `u32` |
| `char::from_u32(x)` | from `u32` to `Option<char>` |

- If the target of `as` is not an integer type, it is an E0205 error (`u as char`, `x as f64`).
- If the source of `as` or `try_from` is not an integer (`bool`, `char`, `f64`), or `T::try_from(x)` is used without `.ok()` or `.unwrap()`, it is an E0412 error.
- `T::from(x)`, `x.into()` and `x.try_into()` are E0212 errors. A conversion inside a specification is an E0513 error (in specifications the integer types mix, chapter 06).
- Corpus (ADR-0028): `tests/run/conversions.bmb`, `tests/check/err/conversions.bmb`.

## Arrays and slices

- `[T; N]` is an array whose length is part of the type. Values are `[a, b, c]` and `[v; N]`, a read is `a[i]`, and the length is `a.len()` (`usize`). `N` is an integer literal (the normal form is decimal: `[a; 0b11]` is written as `[a; 3]`) or a `usize` constant, and a length that is too large is an E0406 error.
- Elements are scalars, structs and enums, `Box`, and fixed arrays (`[[T; M]; N]`). `Vec` and `String` are not elements of an array (E0402).
- `&[T]` and `&mut [T]` are slice references and are used only as the type of a parameter (chapter 05). `[T]` without a reference is not a type (E0402). An argument is `&arr` or `&mut arr`, `v.as_slice()` or `v.as_mut_slice()`, a slice parameter that was received, or a part `&v[lo..hi]` with integer elements.
- A value that cannot be indexed (`i64`, `&str`) is an E0407 error.
- A range expression `lo..hi` is not a value. It comes only as the range of a `for`, the first argument of `copy_within`, and the arguments `&v[lo..hi]` and `&s[lo..hi]`; any other position is an E0435 error.
- The end-inclusive `lo..=hi` comes only as the range of a `for`. Any other position is an E0438 error (in a position that takes a range, write `lo..hi + 1`), and the expression that holds the range is checked as it would be with `lo..hi` (`tests/check/err/inclusive_ranges.bmb`).
- Corpus: `tests/run/arrays.bmb`, `tests/check/err/data_arrays.bmb`, `tests/check/err/places.bmb`, `tests/check/err/subslices.bmb`.

## Text

- `String` is owned UTF-8 text, `&str` is borrowed text, and `char` is one character (ADR-0050).
- The type of the literal `"..."` is `&str`, and it comes only as a call argument and as an operand of a comparison. Any other position (`let t: String = "x";`) is an E0409 error. To own the text, use `String::from("..")`.
- An argument in a `&str` position is a literal, `s.as_str()`, `&s[a..b]`, or a `&str` parameter that was received. A `String` value or `&s` in that position is an E0218 error. `&mut str` is not a type (E0402).
- Operations: `String::new()`, `with_capacity(n)`, `from(t)`, `from_utf8(v)`; `push_str`, `push`, `clone`; `len`, `is_char_boundary`; `as_str`, `as_bytes`; `c.len_utf8()`. Comparison is lexicographic by bytes.
- Indexing text is E0407 (read a byte with `s.as_bytes()[i]`), a method outside the list such as `to_string` or `chars` is E0404, and arithmetic on `char` is an E0401 error.
- Corpus: `tests/run/strings.bmb`, `tests/check/err/strings.bmb`, `tests/check/err/bytes.bmb`.

## Structs and enums

```
struct Pt { x: i64, y: i64 }
enum Shape { Dot, Sq(i64) }
#[effects(pure)]
fn side(s: Shape, p: Pt) -> i64 {
    let q: Pt = Pt { x: p.y, y: 0 };
    match s {
        Shape::Dot => q.x,
        Shape::Sq(n) => n,
    }
}
```

- A struct has one or more named fields, and an enum has one or more variants (E0420). The payload of a variant is values written by position. There are no tuples, tuple structs, or variants with field names (E0233).
- Fields and payloads are value types. They cannot hold references and slices (E0403, chapter 05).
- A type that holds itself by value is an E0421 error. Under `Box` or `Vec` it can (`Cons(i64, Box<List>)`). A type with no finite value and a type that holds itself with different type arguments are also E0421 errors.
- A construction expression `T { f: e, .. }` writes every field once, in declaration order. If it is not a struct, or a field is missing or appears twice, it is E0422; a different order is E0434 (the written order is the evaluation order, ADR-0080); a field that does not exist is an E0306 error.
- The scrutinee of a `match` is an enum (E0423). The arms are written one per variant, in declaration order (E0426). A pattern is `T::V` or `T::V(a, b)`, and the number of names is the number of payload values (E0427). A variant that does not exist is E0307, and any other pattern form is an E0232 error.
- Type invariant: write `#[invariant(p)]` before a struct or enum. `p` is a `spec fn` that takes a `&T` of that type and returns `bool` (E0254). There is one per type (E0255). The meaning and the obligations are in chapters 06 and 07 (ADR-0060).
- `std::process::Child` cannot be made with a construction expression or read by field (E0436).
- `==` on a type that holds itself and has a `Vec` (E0431), and a construction expression of such a struct (E0432), cannot be used in a specification.
- Corpus: `tests/run/shapes.bmb`, `tests/run/points.bmb`, `tests/check/err/data_types.bmb`, `tests/check/err/match.bmb`, `tests/check/err/field_order.bmb`, `tests/check/err/rec_structs.bmb`, `tests/check/err/child.bmb`.

## `Option` and `Result`

- `Option<T>` (`None`, `Some(v)`) and `Result<T, E>` (`Ok(v)`, `Err(e)`) are built-in enums. These six names and `Box` cannot be redefined (E0309). Type arguments are value types, so `Option<&T>` and `Result<&str, E>` are E0403 errors (`tests/check/err/attr_places.bmb`). A wrong number of type arguments is an E0420 error.
- The variants are written without a path (`Option::Some` is E0221), and the order of the arms is `None`, `Some` and `Ok`, `Err` (E0426).
- There is no `.unwrap()` or `.expect()` (E0229). Write the two cases with `match`, or use `?` (chapter 04).
- Corpus: `tests/run/tries.bmb`, `tests/check/err/tries.bmb`, `tests/check/err/data_values.bmb`.

## Generics and traits

```
trait Area { #[effects(pure)] fn area(self: &Self) -> i64; }
struct Sq { s: i64 }
impl Area for Sq { #[effects(pure)] fn area(self: &Sq) -> i64 { self.s } }
#[effects(pure)]
fn larger<T: Copy + Ord>(a: T, b: T) -> T { if a < b { b } else { a } }
#[effects(pure)]
fn both<T: Area>(x: &T, q: Sq) -> i64 { larger::<i64>(x.area(), q.area()) }
```

- Functions, structs, enums and `impl`s have type parameters (`<T>`, `<T: Copy + Ord>`, ADR-0046). Bounds are written next to the parameter (`where` is E0240).
- The built-in bounds are `Copy`, `Eq` and `Ord`. A value of a parameter without `Copy` moves (chapter 05). The integer types, `bool`, `char` and `String` have `Eq` and `Ord`; `f64`, structs and enums, `Vec` and arrays do not (E0428). The order of `bool` is `false < true` (`tests/run/bool_order.bmb`).
- Type arguments are value types (a reference is E0403). With a wrong number, a call is E0429 and a type is an E0420 error. An argument that does not satisfy a bound is an E0428 error (`Vec<i64>` is not `Copy`). Putting a type that cannot be an element of a `Vec` into a `T` that the definition uses as `Vec<T>` is also an E0428 error.
- A recursive call that wraps its own type parameter in a larger type is an E0430 error (`f::<[T; 2]>` inside `f<T>`).
- A trait declares the signatures of methods and `impl Tr for T { .. }` implements them. `Self` is the implementing type. The agreement of declaration and implementation is in chapter 02.
- If two impls of one trait can be for the same type, it is an E0311 error. A type parameter is taken to overlap with anything. `impl<T> Tr for T` for all types is E0235, and `dyn Tr` is an E0236 error. An impl whose head was rejected (E0235, E0420, E0312, E0311) also stays as an impl of that type, so method calls do not become separate errors, and its functions are checked (`tests/check/err/rejected_impls.bmb`). The type parameters of an `impl` must appear in order in the implementing type (E0420): an inherent `impl` has exactly the form `impl<A, B> T<A, B>`, and a trait `impl` may mix in concrete types (`Two<T, i64>`), but the order in which the parameters first appear is the declaration order (`impl<A, B> Tr for Two<B, A>` is an error, `tests/conformance/03/trait_impl_parameters_in_order.bmb`).
- A method call `x.f(a)` refers to a function of the inherent `impl` of the type of `x` or, if there is none, to the one trait that has `f` (ADR-0051). A name that does not exist is E0404, two traits with the same name are E0313, and a function with no receiver is an E0314 error.
- Corpus: `tests/run/generics.bmb`, `tests/run/traits.bmb`, `tests/run/methods.bmb`, `tests/check/err/generics.bmb`, `tests/check/err/traits.bmb`, `tests/check/err/vec_params.bmb`.

## Function values

- A parameter that takes a function is a type parameter whose bound is `Fn(A, ..) -> R`, `FnMut(..) -> R` or `FnOnce(..) -> R`: `fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> i64` (ADR-0053). The body calls it as `f(x)`.
- The argument is the name of a function item or a closure `|x: T| -> R { .. }`, and the type argument is written as `_`.
- A closure comes only in that position. A closure bound with `let`, the function pointer type `fn(i64) -> i64` and a returned `impl Fn` are E0264, and `impl Fn(..)` in argument position is an E0265 error.
- The rules of capture are in chapter 05.
- Corpus: `tests/run/fn_values.bmb`, `tests/check/err/fn_values.bmb`.

## Owning types

| Type | Holds | Making | Reading and writing |
| --- | --- | --- | --- |
| `Vec<T>` | scalars, structs and enums without arrays, `Vec` and `Box` of those, `String` | `Vec::new()`, `Vec::with_capacity(n)` | `v[i]`, `len`, `push`, `pop` (`Option<T>`), `truncate`, `clone` |
| `String` | UTF-8 bytes | "Text" above | "Text" above |
| `Box<T>` | value types (scalars, structs and enums, arrays, `Box`, `Vec`) | `Box::new(v)` | `*b`, `b.f`, `*b = v` |

- A type that cannot be an element of a `Vec` (an array, a struct that has an array) and `Box<()>` are E0402 errors (ADR-0047).
- The element type of `Vec::new()` comes from the expected type (without one, E0401).
- A method not in the list (`is_empty`, `first`, `get`, `iter`) is E0404, and `clone` on a `Vec` whose element is a type parameter without a `Copy` bound is also an E0404 error (chapter 05, "Copy and move").
- A `spec fn` does not take or return a `Vec` or a `String` (E0402). It takes slices and `&str`.
- There is no `Rc`, `RefCell`, `Cell` or `Arc` (E0238).
- Corpus: `tests/run/vecs.bmb`, `tests/run/boxes.bmb`, `tests/check/err/vec.bmb`, `tests/check/err/boxes.bmb`, `tests/check/err/nested_vecs.bmb`.

## Types of constants

- The `T` of `const NAME: T = e;` is a value type: an integer, `bool`, `char`, and arrays, structs and enums of those (`Option` and `Result` included). Holding `Vec`, `String`, `Box` or a reference is an E0437 error (ADR-0052).
- The initializer expression is checked against the declared type (E0401, E0406, E0422).
- A `usize` constant is used in the `N` position of an array length `[T; N]` and of a repetition `[e; N]`.
- Corpus: `tests/run/consts.bmb`, `tests/run/consts_data.bmb`, `tests/check/err/consts.bmb`.
