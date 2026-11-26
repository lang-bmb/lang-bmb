//! Differential test: the term builders of stage0's proof encoder that need
//! no encoder state (`compiler/src/verify.rs`, `verify/text.rs`, the reader
//! of s-expressions of `smt.rs`), each called over a fixed grid of
//! arguments written out here, one JSONL row per call. `smt_terms.bmb`
//! does the same job with the port's builders (`selfhost/encode_terms.bmb`)
//! over the same grid; the case `smt-terms` compares the two outputs. The
//! grid reaches every branch: every integer type and bit operation,
//! numerals up to the ends of 128 bits, both byte orders, every bit and
//! integer method, `f64` bit patterns, and values of every shape; then
//! the translations of sub-proofs (`compiler/src/subproof.rs`) over a grid
//! of terms and variables of every sort. No solver runs.
//!
//! An example of the `bmb` crate (`compiler/Cargo.toml`), so that it calls
//! the compiler's own functions; `compiler/tests/fmt.rs` reads `rows` too.

use bmb::ast::{BinOp, IntTy, Program, TypeKind};
use bmb::diag::Diags;
use bmb::smt;
use bmb::verify::*;
use std::collections::BTreeMap;

/// The program whose types `rows` shows the builders at: the
/// parameters of `g::sig`, in order. Every integer type, the other scalars,
/// a struct, a struct with an array, a `Vec` and an enum, a recursive struct
/// (a datatype with selector functions), enums, arrays and slices of
/// scalars, structs, arrays and `Vec`s, a type parameter, text and a
/// built-in enum.
const TERMS_SOURCE: &str = "#[summary(\"The types the term builders are shown at\")]
mod g {
    enum E { A, B(i64), C(P) }

    enum L { Nil, Cons(i32, Box<L>) }

    struct P { x: i32, on: bool }

    struct Q { p: P, a: [u8; 3], v: Vec<u16>, e: E }

    struct R { n: u64, kids: Vec<R> }

    struct W { ps: Vec<P>, grid: [[i8; 2]; 2], vv: Vec<Vec<u32>> }

    #[effects(pure)]
    fn sig<T: Ord + Copy>(a: i8, b: i16, c: i32, d: i64, e: isize, f: u8, g: u16, h: u32, i: u64, j: usize, k: bool, l: f64, m: (), n: P, o: &Q, p: &R, q: E, r: &L, s: [u8; 4], t: &[i64], u: [P; 2], v: &[P], w: &Vec<Vec<u8>>, x: [[u16; 2]; 3], y: &[W], z: T, aa: [T; 2], ab: &str, ac: Option<u32>, ad: &[R], ae: &[E], af: &[bool], ag: &[f64], ah: &[T], ai: &mut P, aj: &Vec<P>, al: &[[P; 2]]) -> () {}
}
";

/// Terms the numeral readers are shown at, one per line: numerals of both
/// signs, the ends of 64 and 128 bits and one past them, leading zeros and
/// a leading sign, and text that is no numeral.
const TERM_NUMS: &str = "x\n0\n1\n3\n5\n7\n8\n15\n16\n63\n64\n67\n127\n128\n255\n256\n65535\n007\n+5\n-3\n(- 0)\n(- 1)\n(- 3)\n(- 5)\n(- 64)\n(- x)\n(- )\n(-  5)\n(- 5) \n\n12a\n(+ a b)\n4294967295\n9223372036854775807\n9223372036854775808\n(- 9223372036854775808)\n18446744073709551615\n18446744073709551616\n170141183460469231731687303715884105727\n(- 170141183460469231731687303715884105727)\n170141183460469231731687303715884105728\n(- 170141183460469231731687303715884105728)\n340282366920938463463374607431768211455\n340282366920938463463374607431768211456\n99999999999999999999999999999999999999999";

/// Terms the text readers are shown at, one per line: applications of
/// `select` and of one-argument symbols, quoted symbols holding spaces,
/// parentheses and a character outside ASCII, implications, connectives
/// nested to each depth around `LIFT_DEPTH`, white space that is a tab or
/// outside ASCII, and unbalanced text.
const TERM_TEXTS: &str = "x\n\n|a b|\n(select a i)\n(select (store a 0 1) (+ i 1))\n(select |a b| |i (|)\n(select a)\n(select a i j)\n(select  a i)\n(select a i\n(f a)\n(f (g a))\n(f a b)\n(f)\n(|f g| a)\n(|f g| (h |) (|))\n(|f a)\n(|\u{e9}| |\u{e9} \u{e9}|)\n( a)\n(f a )\n(=> g x)\n(=> (and a b) (or c d))\n(=> g x y)\n(=> g)\n(=> |g h| (=> a (and b (or c d))))\n(=> g (and a (=> b (and c d))))\n(=> g x) \n(f\ta\u{a0}b\u{2003}c)\n(and a b)\n(and a (or b (=> c d)))\n(and (and (and (and a b) c) d) (or e f))\n(not (and a (or b c)))\n(f |(and (or| (and a b))\n(andy (ore a b))\n(and\n) (\n(a))\na b\n (a)\n|a";

/// The operand pairs the bit operations are shown at, `a;b`: terms, a zero and
/// constant amounts on both sides of each width, masks `2^k - 1` up to each
/// type's largest value on either side, constants that are no mask, and
/// negative constants.
const TERM_PAIRS: &str = "a;b\na;0\na;1\na;3\na;7\na;8\na;15\na;16\na;31\na;32\na;63\na;64\na;100\na;127\na;128\na;255\na;32767\na;65535\na;2147483647\na;4294967295\na;9223372036854775807\na;18446744073709551615\na;18446744073709551616\na;(- 1)\na;(- 8)\n255;b\n0;b\n6;b\n(+ a 1);(- b)\n(- 0);b\na;(- 0)";

/// The amounts the bit methods that take one are shown at.
const TERM_AMOUNTS: &str = "n\n0\n1\n3\n7\n8\n9\n16\n31\n32\n33\n63\n64\n67\n(- 1)\n(- 3)\n(- 64)\n18446744073709551616\n(+ n 1)";

/// The `f64` bit patterns `f64_lit` is shown at: both zeros, one and
/// minus one, the infinities, quiet, signalling and negative NaNs, the
/// smallest and largest subnormals, the smallest normal and the largest
/// finite value, and a value with every field mixed.
const TERM_F64: [u64; 14] = [
    0x0000_0000_0000_0000,
    0x8000_0000_0000_0000,
    0x3ff0_0000_0000_0000,
    0xbff0_0000_0000_0000,
    0x7ff0_0000_0000_0000,
    0xfff0_0000_0000_0000,
    0x7ff8_0000_0000_0000,
    0x7ff0_0000_0000_0001,
    0xfff8_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x000f_ffff_ffff_ffff,
    0x0010_0000_0000_0000,
    0x7fef_ffff_ffff_ffff,
    0x4009_21fb_5444_2d18,
];

/// The sub-proof translations (`subproof`), `variables;term` one per line.
/// A variable is `name:sort`, the sort an integer type, `bool`, `f64` or
/// `bvN` (a bit-vector constant of a satisfiability query, which a sub-proof
/// does not have: there the name is an unknown symbol). The terms reach
/// every operator class at every sort: numerals of both signs up to the
/// ends of `i128` and the arithmetic that leaves it, `+ - *` at each width
/// with and without a known interval and past the widest bit-vector, `div`
/// and `mod` by powers of two below and above the dividend's width, by
/// other constants, by zero, by negative constants and by terms (signed,
/// unsigned and mixed), the truncating forms, comparisons and equalities of
/// any arity, `ite` at each sort, the connectives, bit-vector literals,
/// operations, comparisons, extensions and the bit helpers, the `f64`
/// operations, literals and constants, applications that are no operator,
/// and the terms each of them refuses.
const TERM_SUB: &str = "a:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;0\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;1\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;5\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;127\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;128\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;255\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;256\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;+5\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;-5\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;007\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;18446744073709551615\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;18446744073709551616\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 5)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 128)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 129)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- +5)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- -5)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;170141183460469231731687303715884105727\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 170141183460469231731687303715884105727)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;170141183460469231731687303715884105728\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- 170141183460469231731687303715884105728)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (- 170141183460469231731687303715884105727) 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (- (- 170141183460469231731687303715884105727) 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ 170141183460469231731687303715884105727 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* (- (- 170141183460469231731687303715884105727) 1) (- 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (- 170141183460469231731687303715884105727) 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;true\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;false\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;o\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;zz\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;()\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;a\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;b\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;c\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;d\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;e\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;p\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;q\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;r\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;s\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;t\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ a 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ s 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ a b c d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- a 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* a a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* p 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* a (- 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* s s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* d d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* p p p p)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* a 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* s s s s s s s s s s s s s s s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* d d d d d d d d d d d d d d d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* d d d d d d d d d d d d d d d d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ (* s s) (* s s))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (* s s) 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (* s s))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ o 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(-)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(*)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(* a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ a zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ zz a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(- (- d))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ (- a) (* p q))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 8)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 128)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 256)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 100)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 1000)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div p 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div p 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div p 256)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div s 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div s 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d 18446744073709551616)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d 170141183460469231731687303715884105727)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d 85070591730234615865843651857942052864)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a p)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div p a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div o a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* s s) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* s s) 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* s s) s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (- a) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div 7 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (- 7) 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div 7 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (- 7) (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div 7 (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a (+ b 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* d d) 7)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (- (- 170141183460469231731687303715884105727) 1) 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (- (- 170141183460469231731687303715884105727) 1) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* d d d d d d d d d d d d d d d d) b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div (* s s s s s s s s s s s s s s s) s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div zz 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(div a zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 8)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 128)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 256)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 100)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 1000)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod p 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod p 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod p 256)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod s 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod s 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d 18446744073709551616)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d 170141183460469231731687303715884105727)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d 85070591730234615865843651857942052864)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod s s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a p)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod p a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod o a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* s s) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* s s) 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* s s) s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (- a) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod 7 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (- 7) 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod 7 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (- 7) (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod 7 (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a (+ b 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* d d) 7)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (- (- 170141183460469231731687303715884105727) 1) 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (- (- 170141183460469231731687303715884105727) 1) 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* d d d d d d d d d d d d d d d d) b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod (* s s s s s s s s s s s s s s s) s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod zz 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(mod a zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem a 3)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem a (- 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a (- 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv d d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv (* s s) d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem (* s s) (* s s))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv 7 (- 2))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem (- 7) 2)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem a p)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_trem p 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv (* d d d d d d d d d d d d d d d d) b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ (div a b) (mod a b))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(+ (bmb_tdiv a (+ b 1)) (bmb_trem a (+ b 1)) (div a b))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(< a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(<= a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(> p q)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(>= d s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(< a 0)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(< a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(<)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(< a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(<= 1 2 3 4)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(> (* s s) d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(>= a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(=)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= zz a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= p s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= (* s s) 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o p a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o o a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite a a a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a b c)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o (* s s) a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o 1 (- 1))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite (< a b) a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite zz a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o zz b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(ite o a zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(and o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(and o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(and)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(or o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(xor o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(=> o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(not o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(not a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(not)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(not o n)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(and o a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(=> (and o n) (or (not o) (< a b)))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(foo a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(|m::f| a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(foo)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(foo (+ a 1) o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(foo zz)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(bmb_tdiv)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;v\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;w\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;z\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;#b101\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;#xff\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;#b\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;#x\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5 8)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5 +8)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5 x)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5 8 9)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ foo 8)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(_ bv5 (8))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat z)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat (concat z z))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat (concat z ((_ extract 61 0) z)))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat ((_ zero_extend 119) v))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvnot v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvneg v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvnot a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvnot v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvand v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvor v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvxor v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvadd v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvmul v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsub v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvshl v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvlshr v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvashr v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvudiv v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvurem v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsdiv v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsrem v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsmod v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvand v v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvand)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvand v a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvadd w v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(concat v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(concat)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(concat v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(concat v a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvult v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvule v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvugt v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvuge v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvslt v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsle v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsgt v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvsge v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvult v a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bvult)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_ctpop8 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_clz8 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_ctz16 w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_bswap16 w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_bswap8 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_brev8 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_rotl8 v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_rotr64 z z)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(= (bmb_rotl8 v v) (bmb_rotl8 (bmb_ctz8 v) v))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_ctpop7 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_foo8 v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_ctpop8 a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bmb_ctpop32 ((_ int2bv 32) r))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8) a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8) d)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 64) a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8) s)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8) o)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv) a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8) a b)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ int2bv 8))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract 3 0) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract 7 7) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract 3) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract 3 0 1) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ zero_extend 4) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ sign_extend 4) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ rotate_left 3) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ rotate_right 3) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ foo 3) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract x 0) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract 3 0) a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ zero_extend +4) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((a b) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ extract (3) 0) v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;((_ zero_extend 4) v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(= (bv2nat (bvand ((_ int2bv 8) p) ((_ int2bv 8) 15))) (mod p 16))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(+ (bv2nat v) 1)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(bv2nat (bvadd ((_ int2bv 64) d) ((_ int2bv 64) 1)))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(= v #x0f)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(= v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(ite o v v)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(ite o v w)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(ite o v a)\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(= (bv2nat (bvlshr ((_ int2bv 64) s) ((_ int2bv 64) 3))) (div s 8))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool v:bv8 w:bv16 z:bv64;(< (bv2nat ((_ extract 3 0) ((_ int2bv 32) c))) 16)\nd:i64 s:u64 o:bool x:f64 y:f64;x\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.add x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.sub x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.mul x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.div x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.neg x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.sqrt x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.abs x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.min x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.max x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.lt x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.le x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.eq x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.is_nan x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.is_finite x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.from d)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.from s)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.from x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.from d s)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.foo x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.add x d)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.add x)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.lt d x)\nd:i64 s:u64 o:bool x:f64 y:f64;(fp #b0 #b01111111111 #x0000000000000)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ NaN 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ +zero 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ -zero 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ +oo 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ -oo 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ NaN 8 24)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ +zero 11)\nd:i64 s:u64 o:bool x:f64 y:f64;(_ zero 11 53)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.eq x (_ NaN 11 53))\nd:i64 s:u64 o:bool x:f64 y:f64;(= x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(ite o x y)\nd:i64 s:u64 o:bool x:f64 y:f64;(ite o x d)\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.lt (f64.from d) (fp #b0 #b01111111111 #x0000000000000))\nd:i64 s:u64 o:bool x:f64 y:f64;(f64.is_finite (f64.add (f64.from d) (f64.sqrt x)))\nd:i64 s:u64 o:bool x:f64 y:f64;(= (f64.is_nan x) (not (f64.eq x x)))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(f64.lt (f64.from d) (f64.from d))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= (_ NaN 11 53) (_ NaN 11 53))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(= (fp #b0 #b0 #x0) (fp #b0 #b0 #x0))\na:i8 b:i16 c:i32 d:i64 e:isize p:u8 q:u16 r:u32 s:u64 t:usize o:bool n:bool;(f64.is_finite (f64.from a))\n;true\n;(< 1 2)\n;a";

/// The names `helper_def` is shown at: every helper at every width, widths
/// that have none, and names that are no helper's.
const TERM_HELPERS: &str = "bmb_ctpop8\nbmb_ctpop16\nbmb_ctpop32\nbmb_ctpop64\nbmb_ctpop7\nbmb_ctpop128\nbmb_clz8\nbmb_clz16\nbmb_clz32\nbmb_clz64\nbmb_clz7\nbmb_clz128\nbmb_ctz8\nbmb_ctz16\nbmb_ctz32\nbmb_ctz64\nbmb_ctz7\nbmb_ctz128\nbmb_brev8\nbmb_brev16\nbmb_brev32\nbmb_brev64\nbmb_brev7\nbmb_brev128\nbmb_bswap8\nbmb_bswap16\nbmb_bswap32\nbmb_bswap64\nbmb_bswap7\nbmb_bswap128\nbmb_rotl8\nbmb_rotl16\nbmb_rotl32\nbmb_rotl64\nbmb_rotl7\nbmb_rotl128\nbmb_rotr8\nbmb_rotr16\nbmb_rotr32\nbmb_rotr64\nbmb_rotr7\nbmb_rotr128\nbmb_foo8\nbmb_foo16\nbmb_foo32\nbmb_foo64\nbmb_foo7\nbmb_foo128\nbmb_\nbmb_8\nctpop8\nbmb_ctpop\nbmb_ctpop08\nbmb_ctpop8x\nbmb_ctpop+8\n\nbmb\nbmb_c8t\nbmb_rotl4294967296";

/// The texts `helper_defs` is shown at.
const TERM_HELPER_TEXTS: &str = "\n(bmb_ctpop8 x)\n(bmb_rotr64 a b) (bmb_brev16 x) (bmb_clz8 y) (bmb_clz8 z)\nbmb_ctpop8\n(bmb_foo8 x)\n(bmb_ctpop8)\n(and (= (bmb_bswap64 x) (bmb_bswap32 y)) (bmb_ctz8 z) (bmb_rotl16 a b) (bmb_ctpop32 c))";

/// The model values `render_value` is shown at: numerals, a negation, a
/// value of a type parameter's sort, lists, and `f64` values in each form
/// (both zeros, the infinities, NaNs, a subnormal, values whose shortest
/// text has an exponent, fields of the wrong width or digits).
const TERM_VALUES: &str = "5\n(- 3)\ntrue\n|'T!val!1|\n|'T!val!|\n|'|\n|'T|\n|'a b!val!12|\nx\n(- (- 3))\n(- 3 4)\n(a b (c))\n()\n(-)\n(fp #b0 #b01111111111 #x0000000000000)\n(fp #b1 #b10000000000 #x921fb54442d18)\n(fp #b0 #b11111111111 #x0000000000000)\n(fp #b1 #b11111111111 #x0000000000000)\n(fp #b0 #b11111111111 #x8000000000000)\n(fp #b1 #b11111111111 #x0000000000001)\n(fp #b0 #b00000000000 #x0000000000001)\n(fp #b0 #b00000000000 #x0000000000000)\n(fp #b1 #b00000000000 #x0000000000000)\n(_ +zero 11 53)\n(_ -zero 11 53)\n(_ +oo 11 53)\n(_ -oo 11 53)\n(_ NaN 11 53)\n(_ NaN 8 24)\n(_ foo 11 53)\n(_ NaN 11)\n(fp #b0 #b0111111111 #x0000000000000)\n(fp #x0 #b01111111111 #x0000000000000)\n(fp #b0 #x3ff #x0000000000000)\n(fp #b0 #b01111111111 #b0000000000000000000000000000000000000000000000000001)\n(fp a b c)\n(fp #b2 #b01111111111 #x0000000000000)\n(fp #b0 #b01111111111 #x000000000000g)\n(fp #b0 #b01111111111 #x+00000000000A)\n(fp #b0 #b01111111111 #x0000000000000 x)\n(fp #b0 (b) #x0000000000000)\n(fp #b0 #b10000000000 #x0000000000000)\n(fp #b0 #b01111111011 #x999999999999a)\n(fp #b0 #b10001000100 #xb1ae4d6e2ef50)\n(fp #b0 #b01111100111 #xad7f29abcaf48)\n(fp #b0 #b11111111110 #xfffffffffffff)\n((x 5) (y (- 3)) (b true) (v |'T!val!1|) (f (fp #b0 #b01111111111 #x8000000000000)))";

/// The scripts `smt::canonical_script` is shown at, separated by `^`: a
/// query with symbols used twice, comment lines and a comment after a
/// command, a string literal, a `(reset)` among white space (of Unicode
/// too) and one with text after it, a symbol and a string left open at the
/// end of a line, text before the first `(reset)`, a last line without a
/// line feed, and the empty script.
const TERM_SCRIPTS: &str = "(reset)\n(declare-const |a| Int)\n(assert (> |a| |b c|))\n(assert (> |b c| |a|))\n(check-sat)\n^; comment |x| stays\n(reset)\n(echo \"|s|\") |x| ; tail |y|\n|y|\n  (reset)  \n|y| |x|\n^(reset)\n|unclosed\n|next|\n^no reset |a|\n   ; indented comment |a|\n\t(reset)\t\n|a|^(reset)\n\"open string |a|\n|a|\n(reset) x\n|b|\n(reset\n|c|\n^^|a|\n^\u{a0}(reset)\u{2003}\n|a| |\u{e9}|\n\u{b}(reset)\u{c}\n|\u{e9}|\n^(reset)\n(reset)\n|a|\n(reset)\n";

/// The text whose symbols `Canon::back` names answers after: datatype
/// symbols, a sort, a name with a space, one that starts with a digit, one
/// with a dot, the empty name, and a symbol left open.
const TERM_CANON: &str = "(declare-datatypes ((|Option<i64>| 0)) (((|Option<i64>::None|) (|Option<i64>::Some| (|v| Int)))))\n(declare-sort |T| 0) |x y| |9lives| |a.b| || |open";

/// The solver's answers `Canon::back` is shown at, one per line: bare and
/// quoted canonical symbols, values of an uninterpreted sort, symbols that
/// need quoting again, numbers past the symbols and past a machine word, a
/// `$` after a symbol character and after one outside ASCII, tokens that
/// are no canonical symbol, a bar left open, and text without a `$`.
const TERM_ANSWERS: &str = "(($2 7))\n((x $1))\n$4!val!0\n|$4|\n$x $99 a$1 |$q|\n$\n$5\n|$5|\n$6 $7 $8\n|$9|\n$9\n$10\n|$3| |$3\n\u{e9}$1 $1\u{e9}\n$18446744073709551616\n$00001\n(_ $2 $0)\n\"$1\"\n$3$4\n|$4!val!1| |$5!x|\n$4|$3|\n(no symbols)\n|\n$1|";

/// The model values `model_int` is shown at, at every integer type: the
/// three forms of a bit-vector literal, values on both sides of each
/// type's sign bit and of `i128`'s, and text that is no literal.
const TERM_MODELS: &str = "#x0f\n#xff\n#xFF\n#x80\n#x100\n#b101\n#b\n#x\n#x+f\n#xg\n#b102\n#x7fffffffffffffff\n#x8000000000000000\n#xffffffffffffffff\n#xffffffffffffffffffffffffffffffff\n#x80000000000000000000000000000000\n#x1ffffffffffffffffffffffffffffffff\n(_ bv5 8)\n(_ bv255 8)\n(_ bv128 8)\n(_ bv127 8)\n(_ bv5)\n(_ bv 5)\n(_ bv)\n(_ bv5 8\n5\n\n(_ bv340282366920938463463374607431768211455 128)\n(_ bv340282366920938463463374607431768211456 128)\n(_ bv170141183460469231731687303715884105728 128)\n(_ bv18446744073709551615 64)\n(_ bv9223372036854775808 64)\n(_ bv-5 8)\n(_ bv+5 8)\n(_ bv65535 16)\n(_ bv4294967295 32)\n(_ bv2147483648 32)\n(_ bv5\t8)";

/// The bit methods of an integer type: those taking an amount first.
const TERM_BIT_METHODS: [&str; 12] = [
    "wrapping_shl",
    "wrapping_shr",
    "rotate_left",
    "rotate_right",
    "swap_bytes",
    "count_ones",
    "count_zeros",
    "leading_zeros",
    "trailing_zeros",
    "leading_ones",
    "trailing_ones",
    "reverse_bits",
];

const TERM_OPS: [BinOp; 19] = [
    BinOp::Add,
    BinOp::Sub,
    BinOp::Mul,
    BinOp::Div,
    BinOp::Rem,
    BinOp::Eq,
    BinOp::Ne,
    BinOp::Lt,
    BinOp::Le,
    BinOp::Gt,
    BinOp::Ge,
    BinOp::And,
    BinOp::Or,
    BinOp::Implies,
    BinOp::BitAnd,
    BinOp::BitOr,
    BinOp::BitXor,
    BinOp::Shl,
    BinOp::Shr,
];

/// One row: the builder, its arguments and its result (JSON).
fn term_row(out: &mut String, f: &str, args: &str, value: &str) {
    let q = bmb::diag::json_str;
    out.push_str(&format!(
        "{{\"fn\":{},\"args\":{},\"out\":{value}}}\n",
        q(f),
        q(args)
    ));
}

fn term_list(xs: &[String]) -> String {
    let q = bmb::diag::json_str;
    format!(
        "[{}]",
        xs.iter().map(|x| q(x)).collect::<Vec<_>>().join(",")
    )
}

fn term_opt(x: Option<String>) -> String {
    x.map_or("null".to_string(), |s| bmb::diag::json_str(&s))
}

fn term_pair(x: Option<(&str, &str)>) -> String {
    x.map_or("null".to_string(), |(a, b)| {
        term_list(&[a.to_string(), b.to_string()])
    })
}

/// A value as JSON: its shape and the terms at its leaves.
fn term_val(v: &Val) -> String {
    let q = bmb::diag::json_str;
    match v {
        Val::S(t) => format!("{{\"s\":{}}}", q(t)),
        Val::A { elems, len } => format!("{{\"a\":{},\"len\":{}}}", q(elems), q(len)),
        Val::Unit => "\"unit\"".to_string(),
        Val::Struct(fs) => format!("{{\"struct\":{}}}", term_vals(fs)),
        Val::Arr { elem, len } => format!("{{\"arr\":{},\"len\":{}}}", term_val(elem), q(len)),
    }
}

fn term_vals(vs: &[Val]) -> String {
    format!(
        "[{}]",
        vs.iter().map(term_val).collect::<Vec<_>>().join(",")
    )
}

/// The value shapes the builders that need no type are shown at, their
/// terms named after `p`: a scalar, arrays of a fixed and of a variable
/// length, the unit value, a struct, slices of structs of a fixed and of a
/// variable length (the element's inner arrays likewise), an array of
/// arrays of structs, and a struct with a unit field. The unit value is no
/// element template: shapes 3 and 8 are left out where one is read.
fn term_shapes(p: &str) -> Vec<Val> {
    let s = |n: &str| Val::S(format!("{p}{n}"));
    let a = |n: &str, len: &str| Val::A {
        elems: format!("{p}{n}"),
        len: len.to_string(),
    };
    let grown = |n: &str, len: &str| Val::A {
        elems: format!("{p}{n}"),
        len: format!("{p}{len}"),
    };
    vec![
        s("s"),
        a("a", "3"),
        grown("b", "n"),
        Val::Unit,
        Val::Struct(vec![s("p"), a("q", "2"), grown("r", "m")]),
        Val::Arr {
            elem: Box::new(Val::Struct(vec![s("e0"), s("e1")])),
            len: "2".to_string(),
        },
        Val::Arr {
            elem: Box::new(Val::Struct(vec![s("f0"), a("f1", "4"), grown("f2", "fl")])),
            len: format!("{p}n"),
        },
        Val::Arr {
            elem: Box::new(Val::Arr {
                elem: Box::new(Val::Struct(vec![s("g")])),
                len: format!("{p}gl"),
            }),
            len: "2".to_string(),
        },
        Val::Struct(vec![s("u"), Val::Unit]),
    ]
}

fn term_steps(steps: &[Step]) -> String {
    steps
        .iter()
        .map(|s| match s {
            Step::Field(k) => format!("Field({k})"),
            Step::DtField(k, t) => format!("DtField({k},{})", bmb::check::ty_str(t)),
            Step::Index(i) => format!("Index({i})"),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The rows: each builder over its grid, as JSONL (the builder, its
/// arguments, its result). The numeral and text readers over `TERM_NUMS`
/// and `TERM_TEXTS`; the integer builders at every integer type; the bit
/// operations over `TERM_PAIRS` and the bit methods over `TERM_AMOUNTS`;
/// the `f64` builders; the lemma templates; the builders of text; the
/// value builders over `term_shapes`; and the builders that read the
/// tables at each parameter type of `TERMS_SOURCE`.
pub fn rows() -> String {
    let q = bmb::diag::json_str;
    let mut out = String::new();
    let o = &mut out;
    for t in TERM_NUMS.split('\n') {
        term_row(o, "is_numeral", t, &is_numeral(t).to_string());
        term_row(o, "numeral", t, &term_opt(numeral(t).map(num)));
        term_row(o, "fixed_len", t, &fixed_len(t).to_string());
        term_row(o, "select_len", t, &q(&select_len(t, "i")));
        term_row(o, "store_len", t, &q(&store_len(t, "i", "x")));
    }
    let texts: Vec<String> = TERM_TEXTS.split('\n').map(str::to_string).collect();
    for t in &texts {
        term_row(o, "bool_depth", t, &bool_depth(t).to_string());
        term_row(o, "implication", t, &term_pair(implication(t)));
        term_row(
            o,
            "first_term",
            t,
            &term_opt(first_term(t).map(str::to_string)),
        );
        term_row(o, "select_app", t, &term_pair(select_app(t)));
        term_row(o, "sel_app", t, &term_pair(sel_app(t)));
        term_row(
            o,
            "parse",
            t,
            &term_opt(smt::parse(t).map(|e| smt::print(&e))),
        );
        term_row(o, "text_decls", t, &q(text::text_decls(t)));
    }
    for n in [0, 1, 2, 5, texts.len()] {
        let facts = &texts[texts.len() - n..];
        term_row(o, "and", &n.to_string(), &q(&and(facts)));
        term_row(
            o,
            "guarded",
            &n.to_string(),
            &term_list(&guarded("c", facts)),
        );
    }
    term_row(o, "ite", "c a a", &q(&ite("c", "a", "a")));
    term_row(o, "ite", "c a b", &q(&ite("c", "a", "b")));
    term_row(o, "sym", "a b", &q(&sym("a b")));
    // Substitution of atoms, and the upper bounds a path condition states.
    let mut m = BTreeMap::new();
    for (a, x) in [("a", "(+ x 1)"), ("b", "a"), ("|c d|", "7"), ("f", "g")] {
        m.insert(a.to_string(), smt::parse(x).expect("a term"));
    }
    for t in &texts {
        if let Some(e) = smt::parse(t) {
            term_row(o, "subst_atoms", t, &q(&smt::print(&subst_atoms(&e, &m))));
        }
    }
    let pcs: [&[&str]; 5] = [
        &[],
        &["(< x 5)", "(<= x n)", "(> 9 x)"],
        &[
            "(and (< x a) (and (>= b x) (< y x)))",
            "(or (< x 1) (< x 2))",
        ],
        &["(< x 5)", "(and", "(< x 5)", "(<= (f x) |a b|)"],
        &[
            "(< x 1 2)",
            "(<= x)",
            "x",
            "(>= m x)",
            "(> (+ m 1) x)",
            "(< x 3)",
        ],
    ];
    for (k, pc) in pcs.iter().enumerate() {
        let pc: Vec<String> = pc.iter().map(|s| s.to_string()).collect();
        for term in ["x", "(f x)", "y"] {
            term_row(
                o,
                "upper_bounds",
                &format!("{k} {term}"),
                &term_list(&upper_bounds(&pc, term)),
            );
        }
    }
    // The integer builders at every integer type.
    for i in IntTy::ALL {
        let name = i.name();
        for x in ["x", "(- 5)"] {
            let args = format!("{name} {x}");
            term_row(o, "range_int", &args, &q(&range_int(i, x)));
            term_row(o, "wrap_int", &args, &q(&wrap_int(i, x)));
            term_row(o, "to_bv", &args, &q(&to_bv(i, x)));
            term_row(o, "from_bv", &args, &q(&from_bv(i, x)));
            term_row(o, "not_term", &args, &q(&not_term(i, x)));
            term_row(o, "bv_value", &args, &q(&bv_value(i, x)));
        }
        for op in [
            BinOp::BitAnd,
            BinOp::BitOr,
            BinOp::BitXor,
            BinOp::Shl,
            BinOp::Shr,
        ] {
            for pair in TERM_PAIRS.split('\n') {
                let (a, b) = pair.split_once(';').expect("two operands");
                let args = format!("{op:?} {name} {a} {b}");
                let (r, facts) = bit_op(op, i, a, b);
                term_row(
                    o,
                    "bit_op",
                    &args,
                    &format!("[{},{}]", q(&r), term_list(&facts)),
                );
            }
            term_row(
                o,
                "bv_term",
                &format!("{op:?} {name}"),
                &q(&bv_term(op, i, "xa", "xb")),
            );
            term_row(
                o,
                "bit_facts",
                &format!("{op:?} {name}"),
                &term_list(&bit_facts(op, i, "a", "b", "r")),
            );
        }
        for op in TERM_OPS {
            term_row(
                o,
                "linear_bit",
                &format!("{op:?} {name}"),
                &term_opt(linear_bit(op, i, "a", "3")),
            );
        }
        for m in TERM_BIT_METHODS {
            let amounts: Vec<Option<&str>> = if matches!(
                m,
                "wrapping_shl" | "wrapping_shr" | "rotate_left" | "rotate_right"
            ) {
                TERM_AMOUNTS.split('\n').map(Some).collect()
            } else {
                vec![None]
            };
            for n in amounts {
                let (r, facts) = bit_method(m, i, "a", n);
                term_row(
                    o,
                    "bit_method",
                    &format!("{m} {name} {}", n.unwrap_or("-")),
                    &format!("[{},{}]", q(&r), term_list(&facts)),
                );
            }
        }
        for le in [false, true] {
            let n = i.bits() / 8;
            for k in 0..n {
                term_row(
                    o,
                    "byte_term",
                    &format!("{name} {le} {k}"),
                    &q(&byte_term(i, le, "a", k)),
                );
            }
            let bs: Vec<String> = (0..n).map(|k| format!("b{k}")).collect();
            term_row(
                o,
                "from_bytes_term",
                &format!("{name} {le}"),
                &q(&from_bytes_term(i, le, &bs)),
            );
            let bytes = bytes_val(i, le, "a", false);
            term_row(o, "bytes_val", &format!("{name} {le}"), &term_val(&bytes));
            term_row(
                o,
                "from_bytes_val",
                &format!("{name} {le}"),
                &term_val(&from_bytes_val(i, le, &bytes)),
            );
            // Bytes of a value read from bytes (a specification), and bytes
            // that are an array of no value.
            let sel: Vec<String> = (0..n).map(|k| format!("(select e {k})")).collect();
            let read = from_bytes_term(i, le, &sel);
            term_row(
                o,
                "bytes_val_spec",
                &format!("{name} {le}"),
                &term_val(&bytes_val(i, le, &read, true)),
            );
            let e = Val::A {
                elems: "e".to_string(),
                len: n.to_string(),
            };
            term_row(
                o,
                "from_bytes_val_array",
                &format!("{name} {le}"),
                &term_val(&from_bytes_val(i, le, &e)),
            );
            term_row(
                o,
                "bytes_fact",
                &format!("{name} {le}"),
                &q(&bytes_fact(i, le, "a")),
            );
            let Val::A { elems, .. } = &bytes else {
                unreachable!()
            };
            for (what, arr) in [("e", "e"), ("bytes", elems.as_str())] {
                let fact = from_bytes_fact(i, le, arr).map_or("null".to_string(), |f| q(&f));
                term_row(o, "from_bytes_fact", &format!("{name} {le} {what}"), &fact);
            }
        }
        for kind in ["wrapping", "saturating", "checked"] {
            for base in ["add", "sub", "mul", "div", "rem"] {
                let op = format!("{kind}_{base}");
                let value = match int_method(&op, &TypeKind::Int(i), "a", "b") {
                    IntMethod::Value(v) => format!("{{\"value\":{}}}", q(&v)),
                    IntMethod::Checked { ok, v, lemmas } => format!(
                        "{{\"ok\":{},\"v\":{},\"lemmas\":{}}}",
                        q(&ok),
                        q(&v),
                        term_list(&lemmas)
                    ),
                };
                term_row(o, "int_method", &format!("{op} {name}"), &value);
            }
        }
    }
    // `f64`.
    let all: String = F64_FUNS
        .iter()
        .map(|(name, ..)| format!("({name} a)"))
        .collect::<Vec<_>>()
        .join(" ");
    let uses = [
        "(+ a b)".to_string(),
        "(f64.add a b)".to_string(),
        "(f64.lt (f64.neg a) (f64.from 1))".to_string(),
        "(f64.is_finite a)".to_string(),
        "(f64.is_nan a) (f64.is_finite a)".to_string(),
        "(f64.addx a) f64.sub (f64.)".to_string(),
        all,
    ];
    for u in &uses {
        for interpreted in [false, true] {
            term_row(
                o,
                "f64_decls",
                &format!("{interpreted} {u}"),
                &q(&f64_decls(u, interpreted)),
            );
        }
    }
    for bits in TERM_F64 {
        term_row(o, "f64_lit", &bits.to_string(), &q(&f64_lit(bits)));
    }
    for op in TERM_OPS {
        term_row(
            o,
            "f64_binary",
            &format!("{op:?}"),
            &term_opt(f64_binary(op, "a", "b")),
        );
    }
    term_row(o, "f64_method", "sqrt", &q(&f64_method("sqrt", "a", None)));
    term_row(
        o,
        "f64_method",
        "min",
        &q(&f64_method("min", "a", Some("b"))),
    );
    term_row(o, "F64_EXACT", "", &q(F64_EXACT));
    for op in [BinOp::BitAnd, BinOp::BitOr, BinOp::BitXor] {
        term_row(
            o,
            "bool_bit",
            &format!("{op:?}"),
            &q(&bool_bit(op, "a", "b")),
        );
    }
    // The lemma templates. The placeholders are replaced one after
    // another: a replacement that holds a later placeholder is read again.
    for (who, ls) in [
        ("signed", SIGNED_DIV_LEMMAS),
        ("unsigned", UNSIGNED_DIV_LEMMAS),
    ] {
        for (k, l) in ls.iter().enumerate() {
            term_row(
                o,
                "instantiate",
                &format!("{who} {k}"),
                &q(&instantiate(
                    l,
                    "x",
                    "(- y)",
                    "(bmb_tdiv x (- y))",
                    "{a}{q}",
                )),
            );
        }
    }
    term_row(
        o,
        "instantiate",
        "nested",
        &q(&instantiate(
            "{a} {b} {q} {r} {a}",
            "{b}",
            "{q}",
            "{r}",
            "{a}",
        )),
    );
    for (k, l) in MUL_LEMMAS.iter().chain([&MUL_BOUND_LEMMA]).enumerate() {
        term_row(
            o,
            "instantiate_mul",
            &k.to_string(),
            &q(&instantiate_mul(l, "x", "(- y)", "(* x (- y))", "7", "{a}")),
        );
    }
    term_row(o, "PRELUDE", "", &q(PRELUDE));
    for v in [&[][..], &[0], &[1, 2, 255]] {
        term_row(o, "byte_array", &v.len().to_string(), &q(&byte_array(v)));
    }
    // Text.
    term_row(o, "boundary", "a n i", &q(&text::boundary("a", "n", "i")));
    term_row(o, "scalar", "c", &q(&text::scalar("c")));
    term_row(o, "utf8_len", "c", &q(&text::utf8_len("c")));
    term_row(o, "utf8_at", "a n k", &q(&text::utf8_at("a", "n", "k")));
    term_row(
        o,
        "utf8_before",
        "a n i",
        &q(&text::utf8_before("a", "n", "i")),
    );
    // Values.
    let (xs, ys) = (term_shapes("x"), term_shapes("y"));
    for (k, v) in xs.iter().enumerate() {
        let template = k != 3 && k != 8;
        let mut flat = Vec::new();
        flatten(v, &mut flat);
        term_row(o, "flatten", &k.to_string(), &term_list(&flat));
        let mut flat = Vec::new();
        flatten_elem(v, &mut flat);
        term_row(o, "flatten_elem", &k.to_string(), &term_list(&flat));
        if matches!(v, Val::S(_) | Val::Unit) {
            term_row(o, "s", &k.to_string(), &q(&v.s()));
        }
        if let Val::A { .. } = v {
            let (elems, len) = text::parts(v);
            term_row(o, "parts", &k.to_string(), &term_list(&[elems, len]));
        }
        if template {
            term_row(
                o,
                "select_val",
                &k.to_string(),
                &term_val(&select_val(v, "i")),
            );
            term_row(
                o,
                "store_val",
                &k.to_string(),
                &term_val(&store_val(v, "i", &ys[k])),
            );
            let mut pairs = Vec::new();
            leaf_pairs(v, &ys[k], &mut pairs);
            let pairs: Vec<String> = pairs.iter().map(|(n, o)| format!("{n} {o}")).collect();
            term_row(o, "leaf_pairs", &k.to_string(), &term_list(&pairs));
        }
        for (j, w) in ys.iter().enumerate() {
            term_row(
                o,
                "ite_val",
                &format!("{k} {j}"),
                &term_val(&ite_val("c", v, w)),
            );
        }
        term_row(
            o,
            "ite_val",
            &format!("{k} same"),
            &term_val(&ite_val("c", v, v)),
        );
    }
    // The builders that read the tables, at each parameter type.
    let (p, t) = terms_program();
    let view = prepare(&p, &t);
    let prog = prog_of(&p, &view);
    let tb = &view;
    let tys = view.fns["g::sig"].params.clone();
    for t in &tys {
        let name = bmb::check::ty_str(t);
        term_row(o, "range", &name, &term_opt(range(t, "x")));
        term_row(o, "len_max", &name, &q(len_max(t)));
        term_row(o, "default_of", &name, &q(default_of(t)));
        for op in [BinOp::Lt, BinOp::Le, BinOp::Gt, BinOp::Ge] {
            term_row(
                o,
                "ord_term",
                &format!("{op:?} {name}"),
                &q(&ord_term(op, t, "a", "b")),
            );
        }
        if t.int().is_some() {
            term_row(o, "wrap", &name, &q(&wrap(t, "x")));
        }
        term_row(o, "default_val", &name, &term_val(&default_val(t, tb)));
        let mut ls = Vec::new();
        leaves(t, tb, "v", &mut ls);
        for (n, l) in &ls {
            term_row(o, "leaf_fact", &format!("{name} {n}"), &term_opt(l.fact(n)));
        }
        let built = build(t, tb, &mut ls.iter().map(|(n, _)| n.clone()));
        term_row(o, "build", &name, &term_val(&built));
        let mut flat = Vec::new();
        flatten(&built, &mut flat);
        term_row(o, "flatten", &name, &term_list(&flat));
        if is_array(t) {
            let e = elem_ty(t);
            let d = elem_default(&e, tb);
            term_row(o, "agg_elem", &name, &agg_elem(&e, tb).to_string());
            term_row(o, "elem_default", &name, &term_val(&d));
            term_row(o, "const_elems", &name, &term_val(&const_elems(&e, &d, tb)));
            let mut flat = Vec::new();
            flatten_elem_of(&d, &e, tb, &mut flat);
            term_row(o, "flatten_elem_of", &name, &term_list(&flat));
            let mut el = Vec::new();
            elem_leaves(&e, tb, "", &mut el);
            let template = elem_template(&e, tb, &mut el.iter().map(|(n, _)| format!("t{n}")));
            term_row(o, "elem_template", &name, &term_val(&template));
            let mut flat = Vec::new();
            flatten_elem(&template, &mut flat);
            term_row(o, "flatten_elem", &name, &term_list(&flat));
            let mut facts = Vec::new();
            inner_len_facts(&template, &e, tb, &mut facts);
            term_row(o, "inner_len_facts", &name, &term_list(&facts));
        }
        if dt_struct(t, tb) {
            term_row(o, "dt_fields", &name, &term_vals(&dt_fields(t, "x", tb)));
        }
    }
    // Assignment to a place. A step into a recursive struct makes a new
    // value of it: a constant and the facts of its fields.
    let s = |n: &str| Val::S(n.to_string());
    // The recursive struct `&R` (parameter `p`) and the slice of it (`ad`).
    let r = tys[15].clone();
    let rs = Val::A {
        elems: "rs".to_string(),
        len: "n".to_string(),
    };
    let i = || Step::Index("i".to_string());
    let j = || Step::Index("j".to_string());
    let places: Vec<(Val, Vec<Step>, Val)> = vec![
        (xs[0].clone(), vec![], s("new")),
        (xs[4].clone(), vec![Step::Field(0)], s("new")),
        (xs[4].clone(), vec![Step::Field(1), i()], s("new")),
        (xs[4].clone(), vec![Step::Field(2)], ys[2].clone()),
        (xs[2].clone(), vec![i()], s("new")),
        (
            xs[5].clone(),
            vec![i()],
            Val::Struct(vec![s("n0"), s("n1")]),
        ),
        (xs[5].clone(), vec![i(), Step::Field(1)], s("new")),
        (xs[6].clone(), vec![i(), Step::Field(2), j()], s("new")),
        (xs[6].clone(), vec![i(), Step::Field(1)], ys[1].clone()),
        (xs[7].clone(), vec![i(), j(), Step::Field(0)], s("new")),
        (xs[8].clone(), vec![Step::Field(1)], Val::Unit),
        (s("r"), vec![Step::DtField(0, r.clone())], s("new")),
        (s("r"), vec![Step::DtField(1, r.clone()), i()], s("kid")),
        (
            s("r"),
            vec![
                Step::DtField(1, r.clone()),
                i(),
                Step::DtField(0, r.clone()),
            ],
            s("new"),
        ),
        (rs, vec![i(), Step::DtField(1, r.clone())], ys[2].clone()),
    ];
    let mut enc = Enc::new(&prog, "g::sig", "g");
    for (k, (v, steps, new)) in places.into_iter().enumerate() {
        let from = enc.decls.len();
        let args = format!("{k} {}", term_steps(&steps));
        let nv = enc.set_at(v, &steps, new);
        term_row(
            o,
            "set_at",
            &args,
            &format!(
                "{{\"val\":{},\"decls\":{}}}",
                term_val(&nv),
                term_list(&enc.decls[from..])
            ),
        );
    }
    sub_rows(&mut out);
    out
}

/// The variables of a `TERM_SUB` line: (name, sort text).
fn term_sub_vars(spec: &str) -> Vec<(String, String)> {
    spec.split(' ')
        .filter_map(|v| v.split_once(':'))
        .map(|(n, s)| (n.to_string(), s.to_string()))
        .collect()
}

/// The rows of the sub-proof translations (`subproof`): `bit_vector`,
/// `bit_vector_int` and `bit_vector_sat` over `TERM_SUB`, the bit helpers'
/// definitions, and the readers of model values (`model_int`,
/// `smt::render_value`).
fn sub_rows(o: &mut String) {
    use bmb::subproof::{self, Const, Sort};
    let q = bmb::diag::json_str;
    let int_ty = |s: &str| IntTy::ALL.into_iter().find(|i| i.name() == s);
    let err = |e: &str| format!("{{\"err\":{}}}", q(e));
    for line in TERM_SUB.split('\n') {
        let (spec, term) = line.split_once(';').unwrap();
        let vars = term_sub_vars(spec);
        let sorts: Vec<(String, Sort)> = vars
            .iter()
            .filter_map(|(n, s)| {
                let sort = match s.as_str() {
                    "bool" => Sort::Bool,
                    "f64" => Sort::F64,
                    _ => Sort::Int(int_ty(s)?),
                };
                Some((n.clone(), sort))
            })
            .collect();
        let consts: Vec<(String, Const)> = vars
            .iter()
            .filter_map(|(n, s)| {
                let c = match s.as_str() {
                    "bool" => Const::Bool,
                    "f64" => return None,
                    _ => match s.strip_prefix("bv") {
                        Some(w) => Const::Bv(w.parse().ok()?),
                        None => Const::Int(int_ty(s)?),
                    },
                };
                Some((n.clone(), c))
            })
            .collect();
        let Some(e) = smt::parse(term) else { continue };
        let mut none = |_: &str, _: &[String]| Ok(None);
        let v = match subproof::bit_vector(&e, &sorts, &mut none) {
            Ok(b) => format!(
                "{{\"logic\":{},\"decls\":{},\"hyps\":{},\"goal\":{},\"guards\":{}}}",
                q(b.logic),
                term_list(&b.decls),
                term_list(&b.hyps),
                q(&b.goal),
                term_list(&b.guards)
            ),
            Err(why) => err(&why),
        };
        term_row(o, "bit_vector", line, &v);
        let v = match subproof::bit_vector_int(&e, &sorts) {
            Ok((t, w)) => format!("[{},{w}]", q(&t)),
            Err(why) => err(&why),
        };
        term_row(o, "bit_vector_int", line, &v);
        let v = match subproof::bit_vector_sat(std::slice::from_ref(&e), &consts) {
            Ok(b) => format!("{{\"script\":{},\"exact\":{}}}", q(&b.script), b.exact),
            Err(why) => err(&why),
        };
        term_row(o, "bit_vector_sat", line, &v);
    }
    for name in TERM_HELPERS.split('\n') {
        let v = subproof::helper_def(name);
        let v = v.map_or("null".to_string(), |(d, w)| format!("[{},{w}]", q(&d)));
        term_row(o, "helper_def", name, &v);
    }
    for t in TERM_HELPER_TEXTS.split('\n') {
        term_row(o, "helper_defs", t, &q(&subproof::helper_defs(t)));
    }
    for t in TERM_VALUES.split('\n') {
        let v = smt::parse(t).map(|e| smt::render_value(&e));
        term_row(o, "render_value", t, &term_opt(v));
    }
    for v in TERM_MODELS.split('\n') {
        for i in IntTy::ALL {
            let x = subproof::model_int(v, i).map(|x| x.to_string());
            term_row(o, "model_int", &format!("{v};{}", i.name()), &term_opt(x));
            // A counterexample of a sub-proof: bars at both ends of a
            // name go, and a variable without a value is left out.
            let vars = ["|x@0|", "||b||", "f", "|z|"].map(String::from);
            let sorts = [Sort::Int(i), Sort::Bool, Sort::F64, Sort::Int(i)];
            let vars: Vec<(String, Sort)> = vars.into_iter().zip(sorts).collect();
            let vals = vec![v.to_string(), "true".to_string(), v.to_string()];
            let shown = bmb::verify::sub_values(&vars, vals);
            term_row(o, "sub_values", &format!("{v};{}", i.name()), &q(&shown));
        }
    }
    for t in TERM_SCRIPTS.split('^') {
        let (text, legend) = smt::canonical_script(t);
        let v = format!("{{\"text\":{},\"legend\":{}}}", q(&text), q(&legend));
        term_row(o, "canonical_script", t, &v);
    }
    for t in TERM_SCRIPTS.split('^') {
        term_row(o, "canonical", t, &q(&smt::canonical(t)));
    }
    let mut canon = smt::Canon::default();
    canon.forward(TERM_CANON);
    for t in TERM_ANSWERS.split('\n') {
        term_row(o, "canon_back", t, &q(&canon.back(t)));
    }
    let none = smt::Canon::default();
    term_row(o, "canon_back", "none: $0 |$0|", &q(&none.back("$0 |$0|")));
}

/// The checked program of `TERMS_SOURCE` and its tables.
fn terms_program() -> (Program, Tables) {
    let mut sm = bmb::source::SourceMap::default();
    let mut diags = Diags::default();
    let id = sm.add("terms.bmb".to_string(), TERMS_SOURCE.to_string());
    let toks = bmb::lexer::lex(id, TERMS_SOURCE, &mut diags);
    let mut program = bmb::parser::Parser::new(toks, &mut diags, TERMS_SOURCE).parse_program();
    bmb::stdlib::add(&mut sm, &mut program, &mut diags);
    let (r, t) = bmb::check::check_resolved(program, &mut diags);
    assert!(!diags.has_errors(), "{}", diags.to_jsonl(&sm));
    (r, t)
}

#[allow(dead_code)]
fn main() {
    print!("{}", rows());
}
