# BMB Nullable Type Syntax (T?)

## Current Status (v0.60.260)

### Supported ✅

The parser supports `T?` as a **type annotation** (v0.37):

```bmb
// Return type
fn find(x: i64) -> i64? = None;

// Parameter type
fn process(opt: i64?) -> i64 = match opt {
    Option::Some(x) => x,
    Option::None => 0,
};

// Variable type
let x: i64? = Some(42);
```

### Not Supported ❌

The parser does **NOT** support `T?` as an **enum declaration name**:

```bmb
// This does NOT work
pub enum T? {
    Some(T),
    None,
}

// Parser error: Unrecognized token `?` found
```

## How T? Works

When the parser sees `i64?`, it:
1. Parses `i64` as a base type
2. Sees the `?` suffix
3. Creates `Type::Nullable(Box<Type::Named("i64")>)`

This is **syntactic sugar** that the type checker expands to `Option<i64>`.

## Option Enum Definition

The `Option` enum must be defined using standard enum syntax:

```bmb
// Correct way to define Option
pub enum Option<T> {
    Some(T),
    None,
}

// Then T? is syntactic sugar for Option<T>
fn example() -> i64? = Option::Some(42);
```

## packages/bmb-option

The file `packages/bmb-option/src/lib.bmb` incorrectly uses `enum T?` syntax.
It should be rewritten to use `enum Option<T>` syntax.

## Generic Struct Bug (FIXED in v0.60.261)

~~Note: There is a separate bug with generic struct field access (ISSUE-20260204).
This affects `Option<T>` if it's implemented as a struct with a tag and value field.~~

**Fixed in v0.60.261:** Generic struct field access now works correctly. See `claudedocs/issues/ISSUE-20260204-generic-struct-field-access.md` for details.

## Recommendation

Generic structs like `Pair<A, B>` and generic enums like `Option<T>` now work correctly:

```bmb
// Generic Pair - works in v0.60.261+
struct Pair<A, B> {
    fst: A,
    snd: B,
}

fn main() -> i64 = {
    let p = new Pair { fst: 1, snd: 2 };
    println(p.fst);  // Prints 1 (correct)
    println(p.snd);  // Prints 2 (correct)
    0
};
```

## Related Files

- `bmb/src/parser/grammar.lalrpop` - Parser grammar
- `bmb/src/ast/mod.rs` - `Type::Nullable` definition
- `bmb/src/types/mod.rs` - Type expansion
- `packages/bmb-option/src/lib.bmb` - Needs rewrite
