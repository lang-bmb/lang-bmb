//! The division and multiplication facts the verifier assumes are theorems of its own SMT
//! definitions (ADR-0017 §3). If this test fails, the gate is unsound.
//!
//! The terms of bit operations (ADR-0048 §4) compute Rust's results, and the
//! facts assumed with them hold (backlog 2N-02). Those are checked by
//! evaluating the terms rather than by the solver (which does not decide
//! them at 32 bits in minutes): every pair of 8-bit operands, and edge and
//! pseudo-random values of the wider types.

use std::io::Write;
use std::process::{Command, Stdio};

use bmb::ast::{BinOp, IntTy};
use bmb::smt::{self, Sexp};
use bmb::verify::{
    MUL_BOUND_LEMMA, MUL_LEMMAS, PRELUDE, SIGNED_DIV_LEMMAS, UNSIGNED_DIV_LEMMAS, bit_op, bv_value,
    instantiate, instantiate_mul, not_term,
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

/// The value of the SMT-LIB term `e`, its free symbols `a` and `b`.
fn eval(e: &Sexp, a: i128, b: i128) -> V {
    match e {
        Sexp::Atom(t) => match t.as_str() {
            "a" => V::I(a),
            "b" => V::I(b),
            "true" => V::B(true),
            "false" => V::B(false),
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
            // `((_ int2bv w) x)`: the value modulo `2^w`.
            if let Sexp::List(f) = &xs[0] {
                let w: u32 = smt::print(&f[2]).parse().unwrap();
                let x = int(eval(&xs[1], a, b));
                return V::Bv(x.rem_euclid(1i128 << w) as u128, w);
            }
            let Sexp::Atom(head) = &xs[0] else {
                unreachable!()
            };
            let args: Vec<V> = xs[1..].iter().map(|x| eval(x, a, b)).collect();
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
                "=" => V::B(args[0] == args[1]),
                "and" => V::B(args.iter().all(|v| boolean(*v))),
                "or" => V::B(args.iter().any(|v| boolean(*v))),
                "not" => V::B(!bl(0)),
                "=>" => V::B(!bl(0) || bl(1)),
                "bv2nat" => V::I(bv(args[0]).0 as i128),
                "bvnot" => {
                    let (x, w) = bv(args[0]);
                    V::Bv(!x & mask(w), w)
                }
                op => {
                    let ((x, w), (y, w2)) = (bv(args[0]), bv(args[1]));
                    assert_eq!(w, w2);
                    let r = match op {
                        "bvand" => x & y,
                        "bvor" => x | y,
                        "bvxor" => x ^ y,
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
