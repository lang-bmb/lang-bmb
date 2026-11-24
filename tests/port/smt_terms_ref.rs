//! Differential test: the term builders of stage0's proof encoder that need
//! no encoder state (`compiler/src/verify.rs`, `verify/text.rs`, the reader
//! of s-expressions of `smt.rs`), each called over a fixed grid of
//! arguments written out here, one JSONL row per call. `smt_terms.bmb`
//! does the same job with the port's builders (`selfhost/encode_terms.bmb`)
//! over the same grid; the case `smt-terms` compares the two outputs. The
//! grid reaches every branch: every integer type and bit operation,
//! numerals up to the ends of 128 bits, both byte orders, every bit and
//! integer method, `f64` bit patterns, and values of every shape. No
//! solver runs.
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
            let bytes = bytes_val(i, le, "a");
            term_row(o, "bytes_val", &format!("{name} {le}"), &term_val(&bytes));
            term_row(
                o,
                "from_bytes_val",
                &format!("{name} {le}"),
                &term_val(&from_bytes_val(i, le, &bytes)),
            );
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
    out
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
