//! The division and multiplication facts the verifier assumes are theorems of its own SMT
//! definitions (ADR-0017 §3). If this test fails, the gate is unsound.
//!
//! The terms of bit operations (ADR-0048 §4) compute Rust's results, and the
//! facts assumed with them hold (backlog 2N-02). Those are checked by
//! evaluating the terms rather than by the solver (which does not decide
//! them at 32 bits in minutes): every pair of 8-bit operands, and edge and
//! pseudo-random values of the wider types.
//!
//! The same holds for the bit methods (backlog 2N-03), and the bit-vector
//! translation of a `by (bit_vector)` sub-proof (`subproof::bit_vector`) is
//! exact: at every sampled point its goal is true exactly when the term it
//! translates has the value the term has there, also for operands outside
//! their types (specification arithmetic).

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use bmb::ast::{BinOp, IntTy};
use bmb::smt::{self, Sexp};
use bmb::subproof::{self, Sort};
use bmb::verify::{
    MUL_BOUND_LEMMA, MUL_LEMMAS, PRELUDE, SIGNED_DIV_LEMMAS, UNSIGNED_DIV_LEMMAS, bit_method,
    bit_op, bv_value, instantiate, instantiate_mul, linear_bit, not_term,
};

fn prove_all(header: &str, lemmas: &[&str], q: &str, r: &str) {
    let goals: Vec<String> = lemmas
        .iter()
        .map(|l| instantiate(l, "a", "b", q, r))
        .collect();
    prove(header, &goals);
}

fn prove(header: &str, goals: &[String]) {
    for goal in goals {
        let script = format!(
            "{PRELUDE}\n(declare-const a Int)\n(declare-const b Int)\n(declare-const x Int)\n(declare-const y Int)\n{header}\n(assert (not {goal}))\n(check-sat)\n"
        );
        let mut child = Command::new("z3")
            .args(["-in", "-T:120"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("z3 must be installed (scripts/setup.sh)");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let answer = String::from_utf8_lossy(&out.stdout);
        assert_eq!(answer.trim(), "unsat", "lemma not proved: {goal}");
    }
}

#[test]
fn signed_division_lemmas_hold() {
    prove_all("", SIGNED_DIV_LEMMAS, "(bmb_tdiv a b)", "(bmb_trem a b)");
}

#[test]
fn unsigned_division_lemmas_hold() {
    prove_all(
        "(assert (>= a 0))",
        UNSIGNED_DIV_LEMMAS,
        "(div a b)",
        "(mod a b)",
    );
}

#[test]
fn multiplication_lemmas_hold() {
    let mut goals: Vec<String> = MUL_LEMMAS
        .iter()
        .map(|l| instantiate_mul(l, "a", "b", "(* a b)", "", ""))
        .collect();
    goals.push(instantiate_mul(
        MUL_BOUND_LEMMA,
        "a",
        "b",
        "(* a b)",
        "x",
        "y",
    ));
    prove("", &goals);
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum V {
    I(i128),
    B(bool),
    /// A bitvector: value and width.
    Bv(u128, u32),
}

fn int(v: V) -> i128 {
    match v {
        V::I(x) => x,
        other => panic!("not an integer: {other:?}"),
    }
}

fn boolean(v: V) -> bool {
    match v {
        V::B(x) => x,
        other => panic!("not a boolean: {other:?}"),
    }
}

fn bv(v: V) -> (u128, u32) {
    match v {
        V::Bv(x, w) => (x, w),
        other => panic!("not a bitvector: {other:?}"),
    }
}

fn mask(w: u32) -> u128 {
    if w == 128 {
        u128::MAX
    } else {
        (1u128 << w) - 1
    }
}

/// The two's complement value of the bit-vector `x` of `w` bits.
fn signed(x: u128, w: u32) -> i128 {
    if w == 128 || x >> (w - 1) & 1 == 0 {
        x as i128
    } else {
        // The sign bit repeated above.
        (x | !mask(w)) as i128
    }
}

/// `v` modulo `2^w` (`w <= 128`).
fn image(v: i128, w: u32) -> u128 {
    (v as u128) & mask(w)
}

type Env = BTreeMap<String, V>;

/// The value of the SMT-LIB term `e`, its free symbols `a` and `b`.
fn eval(e: &Sexp, a: i128, b: i128) -> V {
    let env: Env = [("a".to_string(), V::I(a)), ("b".to_string(), V::I(b))].into();
    eval_in(e, &env)
}

/// The value of the SMT-LIB term `e` under `env` (bit-vectors of up to 128
/// bits; the encoder's bit method functions by their definitions).
fn eval_in(e: &Sexp, env: &Env) -> V {
    match e {
        Sexp::Atom(t) => match t.as_str() {
            "true" => V::B(true),
            "false" => V::B(false),
            _ if env.contains_key(t) => env[t],
            _ if t.starts_with("#b") => V::Bv(
                u128::from_str_radix(&t[2..], 2).unwrap(),
                t.len() as u32 - 2,
            ),
            _ if t.starts_with("#x") => V::Bv(
                u128::from_str_radix(&t[2..], 16).unwrap(),
                4 * (t.len() as u32 - 2),
            ),
            n => V::I(n.parse().unwrap_or_else(|_| panic!("unknown atom {n}"))),
        },
        Sexp::List(xs) => {
            // `(_ bvN w)`
            if let [Sexp::Atom(u), Sexp::Atom(n), Sexp::Atom(w)] = xs.as_slice()
                && u == "_"
            {
                let v: u128 = n.strip_prefix("bv").unwrap().parse().unwrap();
                return V::Bv(v, w.parse().unwrap());
            }
            // `((_ f k ..) x)`
            if let Sexp::List(f) = &xs[0] {
                let ks: Vec<u32> = f[2..]
                    .iter()
                    .map(|k| smt::print(k).parse().unwrap())
                    .collect();
                let x = eval_in(&xs[1], env);
                return match smt::print(&f[1]).as_str() {
                    // The value modulo `2^w`.
                    "int2bv" => V::Bv(image(int(x), ks[0]), ks[0]),
                    "extract" => {
                        let (x, _) = bv(x);
                        V::Bv((x >> ks[1]) & mask(ks[0] - ks[1] + 1), ks[0] - ks[1] + 1)
                    }
                    "zero_extend" => {
                        let (x, w) = bv(x);
                        V::Bv(x, w + ks[0])
                    }
                    "sign_extend" => {
                        let (x, w) = bv(x);
                        V::Bv(image(signed(x, w), w + ks[0]), w + ks[0])
                    }
                    other => panic!("unknown indexed operation {other}"),
                };
            }
            let Sexp::Atom(head) = &xs[0] else {
                unreachable!()
            };
            let args: Vec<V> = xs[1..].iter().map(|x| eval_in(x, env)).collect();
            // A bit method's function: its body with the parameters bound.
            if let Some((def, _)) = subproof::helper_def(head) {
                let Some(Sexp::List(d)) = smt::parse(&def) else {
                    unreachable!()
                };
                let Sexp::List(params) = &d[2] else {
                    unreachable!()
                };
                let mut inner = Env::new();
                for (p, v) in params.iter().zip(&args) {
                    let Sexp::List(p) = p else { unreachable!() };
                    inner.insert(smt::print(&p[0]), *v);
                }
                return eval_in(&d[4], &inner);
            }
            let i = |k: usize| int(args[k]);
            let bl = |k: usize| boolean(args[k]);
            match head.as_str() {
                "-" if args.len() == 1 => V::I(-i(0)),
                "-" => V::I(i(0) - i(1)),
                "+" => V::I(args.iter().map(|v| int(*v)).sum()),
                "*" => V::I(i(0) * i(1)),
                "div" => {
                    assert!(i(1) > 0);
                    V::I(i(0).div_euclid(i(1)))
                }
                "mod" => {
                    assert!(i(1) > 0);
                    V::I(i(0).rem_euclid(i(1)))
                }
                "ite" => {
                    if bl(0) {
                        args[1]
                    } else {
                        args[2]
                    }
                }
                "<" => V::B(i(0) < i(1)),
                "<=" => V::B(i(0) <= i(1)),
                ">" => V::B(i(0) > i(1)),
                ">=" => V::B(i(0) >= i(1)),
                "=" => V::B(args[0] == args[1]),
                "and" => V::B(args.iter().all(|v| boolean(*v))),
                "or" => V::B(args.iter().any(|v| boolean(*v))),
                "xor" => V::B(bl(0) != bl(1)),
                "not" => V::B(!bl(0)),
                "=>" => V::B(!bl(0) || bl(1)),
                "bv2nat" => V::I(bv(args[0]).0 as i128),
                "bvnot" => {
                    let (x, w) = bv(args[0]);
                    V::Bv(!x & mask(w), w)
                }
                "bvneg" => {
                    let (x, w) = bv(args[0]);
                    V::Bv(x.wrapping_neg() & mask(w), w)
                }
                "concat" => {
                    let (mut r, mut w) = (0u128, 0u32);
                    for v in &args {
                        let (x, wx) = bv(*v);
                        r = if wx == 128 { x } else { (r << wx) | x };
                        w += wx;
                    }
                    assert!(w <= 128);
                    V::Bv(r, w)
                }
                "bvadd" | "bvmul" | "bvand" | "bvor" | "bvxor" if args.len() > 2 => {
                    let w = bv(args[0]).1;
                    let mut acc = args[0];
                    for v in &args[1..] {
                        let (x, y) = (bv(acc).0, bv(*v).0);
                        let r = match head.as_str() {
                            "bvadd" => x.wrapping_add(y),
                            "bvmul" => x.wrapping_mul(y),
                            "bvand" => x & y,
                            "bvor" => x | y,
                            _ => x ^ y,
                        };
                        acc = V::Bv(r & mask(w), w);
                    }
                    acc
                }
                op => {
                    let ((x, w), (y, w2)) = (bv(args[0]), bv(args[1]));
                    assert_eq!(w, w2, "{op}");
                    let (sx, sy) = (signed(x, w), signed(y, w));
                    let cmp = |c: bool| V::B(c);
                    let r = match op {
                        "bvult" => return cmp(x < y),
                        "bvule" => return cmp(x <= y),
                        "bvugt" => return cmp(x > y),
                        "bvuge" => return cmp(x >= y),
                        "bvslt" => return cmp(sx < sy),
                        "bvsle" => return cmp(sx <= sy),
                        "bvsgt" => return cmp(sx > sy),
                        "bvsge" => return cmp(sx >= sy),
                        "bvand" => x & y,
                        "bvor" => x | y,
                        "bvxor" => x ^ y,
                        "bvadd" => x.wrapping_add(y),
                        "bvsub" => x.wrapping_sub(y),
                        "bvmul" => x.wrapping_mul(y),
                        "bvshl" if y >= w as u128 => 0,
                        "bvshl" => x << y,
                        "bvlshr" if y >= w as u128 => 0,
                        "bvlshr" => x >> y,
                        "bvashr" => {
                            let neg = x >> (w - 1) & 1 == 1;
                            let s = y.min(w as u128 - 1) as u32;
                            let shifted = x >> s;
                            if neg {
                                shifted | (mask(w) & !(mask(w) >> s))
                            } else {
                                shifted
                            }
                        }
                        "bvsdiv" => {
                            assert!(sy != 0);
                            image(sx / sy, w)
                        }
                        "bvsmod" => {
                            assert!(sy > 0);
                            image(sx.rem_euclid(sy), w)
                        }
                        other => panic!("unknown operation {other}"),
                    };
                    V::Bv(r & mask(w), w)
                }
            }
        }
    }
}

fn value(term: &str, a: i128, b: i128) -> V {
    eval(&smt::parse(term).expect("an SMT term"), a, b)
}

/// Rust's result of `a op b` (or `!a`, `op == None`) on type `i`.
fn rust(op: Option<BinOp>, i: IntTy, a: i128, b: i128) -> i128 {
    macro_rules! on {
        ($t:ty) => {{
            let (x, y) = (a as $t, b as $t);
            let k = b as u32;
            (match op {
                None => !x,
                Some(BinOp::BitAnd) => x & y,
                Some(BinOp::BitOr) => x | y,
                Some(BinOp::BitXor) => x ^ y,
                Some(BinOp::Shl) => x << k,
                Some(BinOp::Shr) => x >> k,
                _ => unreachable!(),
            }) as i128
        }};
    }
    match i {
        IntTy::I8 => on!(i8),
        IntTy::I16 => on!(i16),
        IntTy::I32 => on!(i32),
        IntTy::I64 | IntTy::Isize => on!(i64),
        IntTy::U8 => on!(u8),
        IntTy::U16 => on!(u16),
        IntTy::U32 => on!(u32),
        IntTy::U64 | IntTy::Usize => on!(u64),
    }
}

/// The values of `i` the tests take: all of them for 8 bits; otherwise the
/// bounds, small values, powers of two and their neighbours, alternating
/// bits, and pseudo-random values.
fn values(i: IntTy) -> Vec<i128> {
    let (lo, hi) = (i.min_value(), i.max_value());
    if i.bits() == 8 {
        return (lo..=hi).collect();
    }
    let mut vs = vec![lo, lo + 1, -2, -1, 0, 1, 2, 3, hi - 1, hi];
    for k in 0..i.bits() {
        let p = 1i128 << k;
        vs.extend([p - 1, p, p + 1, -p, -p - 1]);
    }
    let bits = mask(i.bits()) as i128;
    vs.extend([bits / 3, 2 * (bits / 3)]);
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    for _ in 0..60 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        vs.push(seed as i128);
    }
    // Into the type's range by its two's complement image.
    let size = 1i128 << i.bits();
    let mut out: Vec<i128> = vs
        .into_iter()
        .map(|v| {
            let u = v.rem_euclid(size);
            if u > hi { u - size } else { u }
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

const BIT_OPS: [BinOp; 5] = [
    BinOp::BitAnd,
    BinOp::BitOr,
    BinOp::BitXor,
    BinOp::Shl,
    BinOp::Shr,
];

#[test]
fn bit_terms_are_rusts_operations() {
    for i in IntTy::ALL {
        let vs = values(i);
        let w = i.bits() as i128;
        for op in BIT_OPS {
            let (t, _) = bit_op(op, i, "a", "b");
            let term = smt::parse(&t).unwrap();
            for &a in &vs {
                if op.is_shift() {
                    // Amounts below the width (E0803 obliges it).
                    for k in 0..w {
                        let got = int(eval(&term, a, k));
                        assert_eq!(
                            got,
                            rust(Some(op), i, a, k),
                            "{a} {op:?} {k} on {}",
                            i.name()
                        );
                    }
                    continue;
                }
                for &b in &vs {
                    let got = int(eval(&term, a, b));
                    assert_eq!(
                        got,
                        rust(Some(op), i, a, b),
                        "{a} {op:?} {b} on {}",
                        i.name()
                    );
                }
            }
        }
        // Constant amounts: a multiplication or a floor division.
        for k in 0..w {
            for op in [BinOp::Shl, BinOp::Shr] {
                let (t, _) = bit_op(op, i, "a", &k.to_string());
                for &a in &vs {
                    let got = int(value(&t, a, 0));
                    assert_eq!(
                        got,
                        rust(Some(op), i, a, k),
                        "{a} {op:?} {k} on {}",
                        i.name()
                    );
                }
            }
        }
        // Low-bit masks `2^k - 1`, on either side.
        for k in 0..=i.bits() {
            let m = (1i128 << k) - 1;
            if m > i.max_value() {
                continue;
            }
            for (l, r) in [
                ("a".to_string(), m.to_string()),
                (m.to_string(), "a".to_string()),
            ] {
                let (t, facts) = bit_op(BinOp::BitAnd, i, &l, &r);
                assert!(facts.is_empty(), "a mask is linear: {t}");
                for &a in &vs {
                    let got = int(value(&t, a, 0));
                    assert_eq!(
                        got,
                        rust(Some(BinOp::BitAnd), i, a, m),
                        "{a} & {m} on {}",
                        i.name()
                    );
                }
            }
        }
        let t = not_term(i, "a");
        for &a in &vs {
            assert_eq!(
                int(value(&t, a, 0)),
                rust(None, i, a, 0),
                "!{a} on {}",
                i.name()
            );
        }
    }
}

/// Code names each operand's bitvector by its value (`bv_value`): the same
/// bitvector `int2bv` makes of a value in range, so code's terms are the
/// terms above.
#[test]
fn named_operands_are_their_bitvectors() {
    for i in IntTy::ALL {
        let t = smt::parse(&bv_value(i, "a")).unwrap();
        let size = 1i128 << i.bits();
        for a in values(i) {
            assert_eq!(
                int(eval(&t, a, 0)),
                a.rem_euclid(size),
                "{a} on {}",
                i.name()
            );
        }
    }
}

#[test]
fn bit_facts_hold() {
    for i in IntTy::ALL {
        let vs = values(i);
        for op in BIT_OPS {
            let (_, texts) = bit_op(op, i, "a", "b");
            assert!(!texts.is_empty(), "{op:?}: a bitvector term has facts");
            let facts: Vec<Sexp> = texts.iter().map(|f| smt::parse(f).unwrap()).collect();
            for &a in &vs {
                // A shift's facts hold for every amount the term takes.
                let bs: Vec<i128> = if op.is_shift() {
                    (0..i.bits() as i128).collect()
                } else {
                    vs.clone()
                };
                for &b in &bs {
                    for (f, text) in facts.iter().zip(&texts) {
                        assert!(boolean(eval(f, a, b)), "a = {a}, b = {b}: {text}");
                    }
                }
            }
        }
    }
}

const METHODS: [&str; 9] = [
    "rotate_left",
    "rotate_right",
    "wrapping_shl",
    "wrapping_shr",
    "count_ones",
    "count_zeros",
    "leading_zeros",
    "trailing_zeros",
    "swap_bytes",
];

/// Rust's result of bit method `m` on `a` of type `i` (amount `n`).
fn rust_method(m: &str, i: IntTy, a: i128, n: u32) -> i128 {
    macro_rules! on {
        ($t:ty) => {{
            let x = a as $t;
            match m {
                "rotate_left" => x.rotate_left(n) as i128,
                "rotate_right" => x.rotate_right(n) as i128,
                "wrapping_shl" => x.wrapping_shl(n) as i128,
                "wrapping_shr" => x.wrapping_shr(n) as i128,
                "count_ones" => x.count_ones() as i128,
                "count_zeros" => x.count_zeros() as i128,
                "leading_zeros" => x.leading_zeros() as i128,
                "trailing_zeros" => x.trailing_zeros() as i128,
                "swap_bytes" => x.swap_bytes() as i128,
                _ => unreachable!(),
            }
        }};
    }
    match i {
        IntTy::I8 => on!(i8),
        IntTy::I16 => on!(i16),
        IntTy::I32 => on!(i32),
        IntTy::I64 | IntTy::Isize => on!(i64),
        IntTy::U8 => on!(u8),
        IntTy::U16 => on!(u16),
        IntTy::U32 => on!(u32),
        IntTy::U64 | IntTy::Usize => on!(u64),
    }
}

const AMOUNTS: [u32; 18] = [
    0,
    1,
    2,
    3,
    7,
    8,
    9,
    15,
    16,
    31,
    32,
    33,
    63,
    64,
    65,
    100,
    1000,
    u32::MAX,
];

/// The bit methods' terms (backlog 2N-03) compute Rust's results for
/// every amount (a variable one through the named bit-vector functions, a
/// constant one in linear form), and their facts hold.
#[test]
fn bit_method_terms_are_rusts_methods() {
    for i in IntTy::ALL {
        let vs = values(i);
        for m in METHODS {
            let takes = bmb::check::bit_method_arity(m) == Some(1);
            let (t, texts) = bit_method(m, i, "a", takes.then_some("b"));
            let term = smt::parse(&t).unwrap();
            let facts: Vec<Sexp> = texts.iter().map(|f| smt::parse(f).unwrap()).collect();
            let ns: Vec<u32> = if takes { AMOUNTS.to_vec() } else { vec![0] };
            for &a in &vs {
                for &n in &ns {
                    let want = rust_method(m, i, a, n);
                    let got = int(eval(&term, a, n as i128));
                    assert_eq!(got, want, "{a}.{m}({n}) on {}", i.name());
                    for (f, text) in facts.iter().zip(&texts) {
                        assert!(boolean(eval(f, a, n as i128)), "a = {a}, n = {n}: {text}");
                    }
                    if takes {
                        let (lt, lf) = bit_method(m, i, "a", Some(&n.to_string()));
                        assert!(lf.is_empty() && !lt.contains("bv"), "linear: {lt}");
                        assert_eq!(int(value(&lt, a, 0)), want, "{a}.{m}({n}) on {}", i.name());
                    }
                }
            }
        }
    }
}

/// Terms over `a` and `b` of type `i` the encoder makes: bit operations
/// and methods, their linear forms, and arithmetic around them, with
/// operands that leave `i` (specification arithmetic).
fn encoder_terms(i: IntTy) -> Vec<String> {
    let w = i.bits();
    let (sum, diff) = ("(+ a b)", "(- a (* 3 b))");
    let mut ts = Vec::new();
    for op in BIT_OPS {
        ts.push(bit_op(op, i, "a", "b").0);
        ts.push(bit_op(op, i, sum, "b").0);
        ts.push(bit_op(op, i, diff, sum).0);
    }
    // (A 64-bit operand outside the type times `2^63` leaves `i128`, which
    // evaluates the terms here.)
    let wide = if w <= 32 { diff } else { "a" };
    for k in [1, 3, w - 1] {
        for op in [BinOp::Shl, BinOp::Shr] {
            ts.push(linear_bit(op, i, "a", &k.to_string()).unwrap());
            ts.push(linear_bit(op, i, wide, &k.to_string()).unwrap());
        }
    }
    ts.push(linear_bit(BinOp::BitAnd, i, diff, "15").unwrap());
    ts.push(not_term(i, "a"));
    ts.push(not_term(i, sum));
    for m in METHODS {
        ts.push(bit_method(m, i, "a", Some("b")).0);
        ts.push(bit_method(m, i, diff, Some(sum)).0);
        ts.push(bit_method(m, i, diff, Some("5")).0);
    }
    if w <= 32 {
        ts.push("(* a b)".to_string());
        ts.push("(* (- a b) (+ a 7))".to_string());
    }
    ts.push("(div a 3)".to_string());
    ts.push("(div (- a b) 8)".to_string());
    ts.push("(mod (- a b) 6)".to_string());
    ts.push("(mod (+ a b) 256)".to_string());
    ts.push("(ite (< a b) (+ a 1) (- b))".to_string());
    ts
}

/// The bit-vector translation of a sub-proof is exact (backlog 2N-03): for
/// every term the encoder makes, the translation's two's complement value
/// is the term's value at every sampled point of the variables' types.
#[test]
fn bit_vector_translations_are_exact() {
    for i in IntTy::ALL {
        let w = i.bits();
        let vars = vec![
            ("a".to_string(), Sort::Int(i)),
            ("b".to_string(), Sort::Int(i)),
        ];
        let all = values(i);
        // A spread of the values (all edges are near the ends).
        let step = (all.len() / 14).max(1);
        let vs: Vec<i128> = all
            .iter()
            .step_by(step)
            .chain(all.iter().rev().take(4))
            .copied()
            .collect();
        for t in encoder_terms(i) {
            let term = smt::parse(&t).unwrap();
            let (bt, bw) = subproof::bit_vector_int(&term, &vars)
                .unwrap_or_else(|e| panic!("{t} on {}: {e}", i.name()));
            assert!(bw <= 128, "{t}: {bw} bits");
            let bterm = smt::parse(&bt).unwrap();
            for &a in &vs {
                for &b in &vs {
                    let want = int(eval(&term, a, b));
                    let env: Env = [
                        ("a".to_string(), V::Bv(image(a, w), w)),
                        ("b".to_string(), V::Bv(image(b, w), w)),
                    ]
                    .into();
                    let (x, xw) = bv(eval_in(&bterm, &env));
                    assert_eq!(xw, bw);
                    assert_eq!(
                        signed(x, xw),
                        want,
                        "{t} at a = {a}, b = {b} on {}",
                        i.name()
                    );
                }
            }
        }
    }
}

/// A sub-proof's goal: true exactly when the assertion holds, and a spec fn
/// argument outside its parameter's type becomes a hypothesis and a guard.
#[test]
fn bit_vector_goals_and_guards() {
    let i = IntTy::U8;
    let vars = vec![
        ("a".to_string(), Sort::Int(i)),
        ("p".to_string(), Sort::Bool),
    ];
    let goal = smt::parse("(=> p (= (mod a 2) (|f| (+ a 1))))").unwrap();
    // `f(x) = (x + 1) mod 2` for `x: u8`.
    let mut unfold = |f: &str, ps: &[String]| {
        assert_eq!(f, "|f|");
        Ok(Some(subproof::Unfolded {
            body: smt::parse(&format!("(mod (+ {} 1) 2)", ps[0])).unwrap(),
            params: vec![Sort::Int(IntTy::U8)],
        }))
    };
    let q = subproof::bit_vector(&goal, &vars, &mut unfold).unwrap();
    assert_eq!(q.logic, "QF_BV");
    assert_eq!(q.hyps.len(), 1, "{q:?}");
    assert_eq!(
        q.guards,
        vec!["(and (<= 0 (+ a 1)) (<= (+ a 1) 255))".to_string()]
    );
    let hyp = smt::parse(&q.hyps[0]).unwrap();
    let g = smt::parse(&q.goal).unwrap();
    // The placeholders' definitions, in order.
    for a in 0..=255i128 {
        for p in [false, true] {
            let mut env: Env = [
                ("a".to_string(), V::Bv(a as u128, 8)),
                ("p".to_string(), V::B(p)),
            ]
            .into();
            for d in &q.decls {
                let Some(Sexp::List(xs)) = smt::parse(d) else {
                    unreachable!()
                };
                if smt::print(&xs[0]) == "define-fun" {
                    let v = eval_in(&xs[4], &env);
                    env.insert(smt::print(&xs[1]), v);
                }
            }
            assert_eq!(boolean(eval_in(&hyp, &env)), a < 255, "a = {a}");
            assert!(boolean(eval_in(&g, &env)), "a = {a}, p = {p}");
        }
    }
}
