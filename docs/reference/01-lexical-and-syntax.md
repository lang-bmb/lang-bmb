# 01 Lexical structure and syntax

This chapter gives the rules by which source text is split into tokens (identifiers, reserved words, literals, comments, `///`), the skeleton of the syntax (what comes at the top level and inside `mod`), and the nesting limit. The meaning of each item is in chapter 02; the grammar of types, expressions and contracts is in chapters 03 to 06. Breaking a rule of this chapter gives E01xx.

## Characters and whitespace

Source is UTF-8 text. Whitespace between tokens is ASCII whitespace characters (space, tab, newline and so on) and has no meaning. Non-ASCII characters come only in comments and inside string and character literals. A character that cannot start any token is an E0101 error (`` ` ``: `tests/check/err/lexing.bmb`). An empty file and a file with only comments are programs with no units.

## Identifiers and reserved words

```
ident = [A-Za-z_] [A-Za-z0-9_]*      (except reserved words)
```

An identifier uses only ASCII letters, digits and `_`. A non-ASCII letter is an E0101 error. There are no Rust raw identifiers (`r#fn`). A reserved word cannot be used in a name position (the name of a unit, item, parameter, local and so on); using one is an E0103 error.

```
fn spec proof mod let mut while if else true false requires ensures decreases
invariant forall exists old assert assume return for loop match break continue
as pub struct enum impl trait use const static type unsafe self Self crate
super where ref move dyn in async await
```

The reserved words that BMB's syntax does not use (`pub`, `use`, `static`, `where` and so on) are there to recognize Rust habits and reject them with E02xx (chapter 04). There are two exceptions: `self` is the name of the receiver parameter (chapter 02), and `invariant` is a name in the attribute name position (`#[invariant(p)]`, ADR-0060). Attribute names (`summary`, `effects` and so on) and the names of built-in types and functions are not reserved words. `'name` is a loop label only after `break` and `continue` and before `:` (`tests/run/loop_exits.bmb`); in any other position it is read as a lifetime annotation and is an E0208 error.

## Punctuation

```
==>  ..=
::  ->  =>  ==  !=  <=  >=  &&  ||  ..
( ) { } [ ] , ; : . | # ! & = < > + - * / % ? @ ^ ~ $
```

The longest one matches first. `<<` and `>>` are not tokens. The parser reads an adjacent `<` `<` or `>` `>` as a shift (ADR-0048, `tests/run/bits.bmb`). A token of this list in a position where the grammar does not allow it is an E0103 error.

## Integer literals

```
int = dec | "0x" hex | "0o" oct | "0b" bin      ("_" between digits and at the end is a separator)
```

- Decimal, and `0x`, `0o`, `0b` (the prefix is lower case). `_` is only layout and is not part of the value (ADR-0048 §3, `tests/run/bits.bmb`).
- A digit outside the base (`0b102`) and a prefix with no digits (`0x`) are E0108 errors (`tests/check/err/bits.bmb`).
- There are no suffixes. A letter or `_` right after the digits is an E0104 error (`5i64`, `0XFF`). The context decides the type (chapter 03).
- A value that does not fit in 128 bits is an E0105 error (`tests/check/err/lexing.bmb`). The range check of the type belongs to chapter 03.
- There are no negative literals. `-1` is unary `-` and a literal.
- The normal form (`bmb fmt`) keeps the base as written and writes the digits in lower case, without separators and leading zeros (`0xFF_00` → `0xff00`).

## Floating-point literals

```
float = digits "." digits [ ("e" | "E") ["+" | "-"] digits ]
```

There must be digits on both sides of the point, and the exponent must have digits (ADR-0061 §2). `1.`, `.5`, `1e5` and `1.0e` are E0109 errors (`tests/check/err/float_literals.bmb`). A suffix (`1.0f64`) is an E0104 error. A `.` after digits is not part of the literal if the next character starts an identifier or is `.`: `0..n` is read as a range and `1.f()` as a method call. The accepted forms are in `tests/run/floats.bmb`.

## Character, string and byte literals

```
char  = "'" (character | escape) "'"       type char
str   = '"' (character | escape)* '"'      type &str
byte  = "b'" (ASCII | bescape) "'"         type u8
bytes = 'b"' (ASCII | bescape)* '"'        type &[u8]
escape  = \n \r \t \\ \' \" \0 \xHH (up to 7f) \u{H..}
bescape = \n \r \t \\ \' \" \0 \xHH
```

- A `char` is one Unicode scalar value (ADR-0050 §1). Two or more characters (`'ab'`) and an escape outside the list are E0101 errors (`tests/check/err/strings.bmb`).
- A string holds non-ASCII characters and newlines as they are. With no closing `"` it is an E0102 error and the literal runs to the end of the file (`tests/check/err/lexing.bmb`).
- A byte literal holds only ASCII characters and escapes (ADR-0044 §2). A non-ASCII character, an escape outside the list and a `b'..'` that is not one byte are E0101 errors; an unclosed one is an E0102 error (`tests/check/err/bytes.bmb`).
- The expression positions where a string or byte string literal can come are in chapter 03. The accepted forms are in `tests/run/strings.bmb`, `tests/run/bytes.bmb`.

## Comments

A comment runs only from `//` to the end of the line and does not become a token. A line that starts with `////` and `//!` are comments too. A block comment `/* .. */` is an E0106 error (`tests/check/err/rust_habits.bmb`). Comments are not part of the meaning of the program.

## `///` and summaries

`///` (one not directly followed by `/`) is not a comment but an input form of `#[summary("..")]` (ADR-0117). Consecutive `///` lines are one group, and the text of each line with its leading and trailing whitespace removed is joined with one space into one string. An empty line in between ends the group. A group with no text produces no token. A group is read at that position as the attribute token `#[summary("..")]`, so it comes only where an attribute can come: before a statement it is E1002 (`tests/check/err/summaries.bmb`), and if no item follows it is an E0103 error. The rule of the attribute (E0704) is in chapter 02. The normal form writes the attribute form.

```
/// Conversion of length units.
mod len {
    /// The number of centimetres in one metre.
    const CM: i64 = 100;

    // A line comment is thrown away.
    #[effects(pure)]
    fn doubled() -> i64 {
        CM + 0x64
    }
}
```

## Syntax skeleton

```
program = unit*
unit    = attr* "mod" ident "{" item* "}"
item    = attr* ( unit | const | struct | enum | trait | impl | fn )
fn      = ["spec" | "proof"] "fn" ident [generics] "(" params ")" ["->" ret] clauses block
ret     = type | "(" ident ":" type ")"
attr    = "#" "[" name [ "(" arg ("," arg)* [","] ")" ] "]"
arg     = string | path | "<" type "as" path ">" "::" ident | ident "=" (ident | string)
```

- Only units come at the top level. A top-level `fn`, `spec fn` or `proof fn` is E0703; any other token is E0103 (`` expected `mod` ``) (`tests/check/err/items.bmb`).
- Only the seven kinds of item above come inside `mod`. Any other token is an E0103 error. Rust items (`use`, `pub`, `static`, `type`, `unsafe` and so on) are recognized and rejected with E02xx.
- Only `fn` and `spec fn` come in the body of an `impl` or a `trait` (E0103). A trait method can end with `;` instead of a body.
- The contract clauses of a function come in the order `requires`, `ensures`, `decreases`. A different order is an E0103 error (`tests/check/err/specs.bmb`). The grammar of clauses and bodies is in chapters 04 and 06.
- When exactly one token was expected, the `fix` of the E0103 diagnostic says to insert that token. An item with a syntax error is skipped and reading resumes from the next item.
- An example that is accepted as a whole is `tests/run/basics.bmb`.

## Nesting limit

Syntactic nesting goes up to 256 levels. Parenthesized and unary expressions, blocks, `if` and `else if`, loops, `==>` chains, chains of binary and postfix operations, types and units count toward one depth together. Beyond that it is an E0107 error (`tests/check/err/nesting.bmb`). If only units are nested, up to 256 levels are accepted.
