//! Soundness fuzzing of `assert(e) by (bit_vector)` sub-proofs (backlog
//! 2N-03, ADR-0048 §4): random assertions over two variables of `u8` or
//! `i8` with arithmetic (unbounded in specifications), bit operations,
//! shifts, `!`, the bit methods, `if`, and a spec fn whose argument may
//! leave its type. An independent interpreter evaluates the assertion at
//! every one of the 65,536 points: a verified assertion holds wherever the
//! spec fn's arguments are in its parameter's type (the guard the enclosing
//! proof assumes it under), and a counterexample is such a point where it
//! does not hold.
//!
//! The interpreter computes what each operation means in a specification:
//! a bit operation on the two's complement images of its operands at the
//! type's width, `<<` by a constant as that, `>>` by a constant as floor
//! division and `!` as `-a - 1` (`MAX - a` unsigned) for any integer, as
//! the encoder defines them (`verify::bit_op`, `verify::bit_method`).
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

mod common;

use common::scalar::{Rng, field};

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    U8,
    I8,
}

impl Ty {
    fn name(self) -> &'static str {
        match self {
            Ty::U8 => "u8",
            Ty::I8 => "i8",
        }
    }
    fn range(self) -> (i128, i128) {
        match self {
            Ty::U8 => (0, 255),
            Ty::I8 => (-128, 127),
        }
    }
    /// The value of the 8-bit pattern `u` in this type.
    fn of_bits(self, u: i128) -> i128 {
        let u = u.rem_euclid(256);
        if self == Ty::I8 && u > 127 {
            u - 256
        } else {
            u
        }
    }
}

#[derive(Clone, Copy)]
enum Bin {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy)]
enum Meth {
    Rotl,
    Rotr,
    Wshl,
    Wshr,
}

#[derive(Clone)]
enum E {
    Var(&'static str),
    Lit(i128),
    Bin(Bin, Box<E>, Box<E>),
    Shl(Box<E>, u32),
    Shr(Box<E>, u32),
    Not(Box<E>),
    Method(Meth, Box<E>, u32),
    If(Box<C>, Box<E>, Box<E>),
    /// The spec fn `h`.
    Call(Box<E>),
}

#[derive(Clone, Copy)]
enum Count {
    Ones,
    Zeros,
    Leading,
    Trailing,
}

#[derive(Clone)]
enum C {
    Cmp(&'static str, E, E),
    Count(Count, E, &'static str, u32),
    And(Box<C>, Box<C>),
    Or(Box<C>, Box<C>),
    Implies(Box<C>, Box<C>),
    Not(Box<C>),
}

fn has_var(e: &E) -> bool {
    match e {
        E::Var(_) => true,
        E::Lit(_) => false,
        E::Bin(_, a, b) => has_var(a) || has_var(b),
        E::Shl(a, _) | E::Shr(a, _) | E::Not(a) | E::Method(_, a, _) | E::Call(a) => has_var(a),
        E::If(_, a, b) => has_var(a) || has_var(b),
    }
}

const CMPS: [&str; 6] = ["==", "!=", "<", "<=", ">", ">="];

fn gen_lit(rng: &mut Rng, t: Ty) -> i128 {
    let (lo, hi) = t.range();
    match rng.below(4) {
        0 => [lo, hi, 0, 1, (-1i128).max(lo), 0x0f, 0x55][rng.below(7)],
        _ => lo + rng.below((hi - lo + 1) as usize) as i128,
    }
}

/// An integer term with a variable in it (a literal alone would be `i64`).
fn gen_e(rng: &mut Rng, t: Ty, depth: usize, call: bool) -> E {
    let e = gen_any(rng, t, depth, call);
    if has_var(&e) {
        e
    } else {
        E::Bin(Bin::Xor, Box::new(E::Var("x")), Box::new(e))
    }
}

fn gen_any(rng: &mut Rng, t: Ty, depth: usize, call: bool) -> E {
    if depth == 0 || rng.chance(25) {
        return match rng.below(5) {
            0 | 1 => E::Var("x"),
            2 | 3 => E::Var("y"),
            _ => E::Lit(gen_lit(rng, t)),
        };
    }
    let d = depth - 1;
    match rng.below(11) {
        0..=3 => {
            let op = [Bin::Add, Bin::Sub, Bin::Mul, Bin::And, Bin::Or, Bin::Xor][rng.below(6)];
            let a = gen_any(rng, t, d, call);
            let b = gen_any(rng, t, d, call);
            // One operand with a variable fixes the literal's type.
            let a = if has_var(&a) || has_var(&b) {
                a
            } else {
                E::Var("y")
            };
            E::Bin(op, Box::new(a), Box::new(b))
        }
        4 => E::Shl(Box::new(gen_e(rng, t, d, call)), rng.below(8) as u32),
        5 => E::Shr(Box::new(gen_e(rng, t, d, call)), rng.below(8) as u32),
        6 => E::Not(Box::new(gen_e(rng, t, d, call))),
        7 => {
            let m = [Meth::Rotl, Meth::Rotr, Meth::Wshl, Meth::Wshr][rng.below(4)];
            E::Method(m, Box::new(gen_e(rng, t, d, call)), rng.below(20) as u32)
        }
        8 => E::If(
            Box::new(gen_c(rng, t, d, call)),
            Box::new(gen_e(rng, t, d, call)),
            Box::new(gen_e(rng, t, d, call)),
        ),
        9 | 10 if call => E::Call(Box::new(gen_e(rng, t, d, false))),
        _ => gen_e(rng, t, d, call),
    }
}

fn gen_c(rng: &mut Rng, t: Ty, depth: usize, call: bool) -> C {
    if depth == 0 || rng.chance(40) {
        if rng.chance(25) {
            let c = [Count::Ones, Count::Zeros, Count::Leading, Count::Trailing][rng.below(4)];
            return C::Count(
                c,
                gen_e(rng, t, depth.min(2), call),
                CMPS[rng.below(6)],
                rng.below(9) as u32,
            );
        }
        return C::Cmp(
            CMPS[rng.below(6)],
            gen_e(rng, t, depth.min(2), call),
            gen_any(rng, t, depth.min(2), call),
        );
    }
    let d = depth - 1;
    match rng.below(4) {
        0 => C::And(
            Box::new(gen_c(rng, t, d, call)),
            Box::new(gen_c(rng, t, d, call)),
        ),
        1 => C::Or(
            Box::new(gen_c(rng, t, d, call)),
            Box::new(gen_c(rng, t, d, call)),
        ),
        2 => C::Implies(
            Box::new(gen_c(rng, t, d, call)),
            Box::new(gen_c(rng, t, d, call)),
        ),
        _ => C::Not(Box::new(gen_c(rng, t, d, call))),
    }
}

fn lit_text(v: i128) -> String {
    v.to_string()
}

fn e_text(e: &E) -> String {
    match e {
        E::Var(v) => v.to_string(),
        E::Lit(v) if *v < 0 => format!("({})", lit_text(*v)),
        E::Lit(v) => lit_text(*v),
        E::Bin(op, a, b) => {
            let s = match op {
                Bin::Add => "+",
                Bin::Sub => "-",
                Bin::Mul => "*",
                Bin::And => "&",
                Bin::Or => "|",
                Bin::Xor => "^",
            };
            format!("({} {s} {})", e_text(a), e_text(b))
        }
        E::Shl(a, k) => format!("({} << {k})", e_text(a)),
        E::Shr(a, k) => format!("({} >> {k})", e_text(a)),
        // Parenthesized: a method call binds tighter than `!`.
        E::Not(a) => format!("(!{})", e_text(a)),
        E::Method(m, a, k) => {
            let n = match m {
                Meth::Rotl => "rotate_left",
                Meth::Rotr => "rotate_right",
                Meth::Wshl => "wrapping_shl",
                Meth::Wshr => "wrapping_shr",
            };
            format!("{}.{n}({k})", e_text(a))
        }
        E::If(c, a, b) => format!(
            "(if {} {{ {} }} else {{ {} }})",
            c_text(c),
            e_text(a),
            e_text(b)
        ),
        E::Call(a) => format!("h({})", e_text(a)),
    }
}

fn c_text(c: &C) -> String {
    match c {
        C::Cmp(op, a, b) => format!("({} {op} {})", e_text(a), e_text(b)),
        C::Count(k, a, op, n) => {
            let m = match k {
                Count::Ones => "count_ones",
                Count::Zeros => "count_zeros",
                Count::Leading => "leading_zeros",
                Count::Trailing => "trailing_zeros",
            };
            format!("({}.{m}() {op} {n})", e_text(a))
        }
        C::And(a, b) => format!("({} && {})", c_text(a), c_text(b)),
        C::Or(a, b) => format!("({} || {})", c_text(a), c_text(b)),
        C::Implies(a, b) => format!("({} ==> {})", c_text(a), c_text(b)),
        C::Not(a) => format!("!{}", c_text(a)),
    }
}

/// `e` with every variable `v` (`h`'s parameter).
fn rename_e(e: &E) -> E {
    let b = |x: &E| Box::new(rename_e(x));
    match e {
        E::Var(_) => E::Var("v"),
        E::Lit(v) => E::Lit(*v),
        E::Bin(op, x, y) => E::Bin(*op, b(x), b(y)),
        E::Shl(x, k) => E::Shl(b(x), *k),
        E::Shr(x, k) => E::Shr(b(x), *k),
        E::Not(x) => E::Not(b(x)),
        E::Method(m, x, k) => E::Method(*m, b(x), *k),
        E::If(c, x, y) => E::If(Box::new(rename_c(c)), b(x), b(y)),
        E::Call(x) => E::Call(b(x)),
    }
}

fn rename_c(c: &C) -> C {
    let b = |x: &C| Box::new(rename_c(x));
    match c {
        C::Cmp(op, x, y) => C::Cmp(op, rename_e(x), rename_e(y)),
        C::Count(k, x, op, n) => C::Count(*k, rename_e(x), op, *n),
        C::And(x, y) => C::And(b(x), b(y)),
        C::Or(x, y) => C::Or(b(x), b(y)),
        C::Implies(x, y) => C::Implies(b(x), b(y)),
        C::Not(x) => C::Not(b(x)),
    }
}

/// The spec fn `h(v)`: a fixed term of its parameter.
struct H {
    body: E,
}

struct Point {
    x: i128,
    y: i128,
    /// Every argument of `h` was in its parameter's type.
    guard: bool,
}

fn floor_div(a: i128, p: i128) -> i128 {
    a.div_euclid(p)
}

fn eval_e(e: &E, t: Ty, h: &H, p: &mut Point) -> i128 {
    match e {
        E::Var("x") => p.x,
        E::Var(_) => p.y,
        E::Lit(v) => *v,
        E::Bin(op, a, b) => {
            let (a, b) = (eval_e(a, t, h, p), eval_e(b, t, h, p));
            match op {
                Bin::Add => a + b,
                Bin::Sub => a - b,
                Bin::Mul => a * b,
                Bin::And => t.of_bits(a.rem_euclid(256) & b.rem_euclid(256)),
                Bin::Or => t.of_bits(a.rem_euclid(256) | b.rem_euclid(256)),
                Bin::Xor => t.of_bits(a.rem_euclid(256) ^ b.rem_euclid(256)),
            }
        }
        E::Shl(a, k) => shl(eval_e(a, t, h, p), *k, t),
        E::Shr(a, k) => shr(eval_e(a, t, h, p), *k),
        E::Not(a) => {
            let a = eval_e(a, t, h, p);
            match t {
                Ty::I8 => -a - 1,
                Ty::U8 => 255 - a,
            }
        }
        E::Method(m, a, k) => {
            let a = eval_e(a, t, h, p);
            let k = *k % 8;
            match m {
                Meth::Wshl => shl(a, k, t),
                Meth::Wshr => shr(a, k),
                Meth::Rotl | Meth::Rotr => {
                    let left = if matches!(m, Meth::Rotr) {
                        (8 - k) % 8
                    } else {
                        k
                    };
                    if left == 0 {
                        return a;
                    }
                    // The low `8 - left` bits move up, the high ones down.
                    let lo = 1i128 << (8 - left);
                    let u = if t == Ty::I8 { a.rem_euclid(256) } else { a };
                    let r = a.rem_euclid(lo) * (1 << left) + floor_div(u, lo);
                    if t == Ty::I8 { t.of_bits(r) } else { r }
                }
            }
        }
        E::If(c, a, b) => {
            if eval_c(c, t, h, p) {
                eval_e(a, t, h, p)
            } else {
                eval_e(b, t, h, p)
            }
        }
        E::Call(a) => {
            let v = eval_e(a, t, h, p);
            let (lo, hi) = t.range();
            p.guard &= lo <= v && v <= hi;
            let mut inner = Point {
                x: v,
                y: v,
                guard: true,
            };
            eval_e(&h.body, t, h, &mut inner)
        }
    }
}

fn shl(a: i128, k: u32, t: Ty) -> i128 {
    if k == 0 { a } else { t.of_bits(a * (1 << k)) }
}

fn shr(a: i128, k: u32) -> i128 {
    if k == 0 { a } else { floor_div(a, 1 << k) }
}

fn eval_c(c: &C, t: Ty, h: &H, p: &mut Point) -> bool {
    match c {
        C::Cmp(op, a, b) => {
            let (a, b) = (eval_e(a, t, h, p), eval_e(b, t, h, p));
            cmp(op, a, b)
        }
        C::Count(k, a, op, n) => {
            let u = eval_e(a, t, h, p).rem_euclid(256) as u8;
            let v = match k {
                Count::Ones => u.count_ones(),
                Count::Zeros => u.count_zeros(),
                Count::Leading => u.leading_zeros(),
                Count::Trailing => u.trailing_zeros(),
            };
            cmp(op, v as i128, *n as i128)
        }
        // Both sides are evaluated (their guards count either way: the
        // sub-proof assumes every unfolded argument in range).
        C::And(a, b) => {
            let (a, b) = (eval_c(a, t, h, p), eval_c(b, t, h, p));
            a && b
        }
        C::Or(a, b) => {
            let (a, b) = (eval_c(a, t, h, p), eval_c(b, t, h, p));
            a || b
        }
        C::Implies(a, b) => {
            let (a, b) = (eval_c(a, t, h, p), eval_c(b, t, h, p));
            !a || b
        }
        C::Not(a) => !eval_c(a, t, h, p),
    }
}

fn cmp(op: &str, a: i128, b: i128) -> bool {
    match op {
        "==" => a == b,
        "!=" => a != b,
        "<" => a < b,
        "<=" => a <= b,
        ">" => a > b,
        _ => a >= b,
    }
}

/// The points where the assertion fails under the guard.
fn failures(c: &C, t: Ty, h: &H) -> Vec<(i128, i128)> {
    let (lo, hi) = t.range();
    let mut out = Vec::new();
    for x in lo..=hi {
        for y in lo..=hi {
            let mut p = Point { x, y, guard: true };
            let v = eval_c(c, t, h, &mut p);
            if p.guard && !v {
                out.push((x, y));
            }
        }
    }
    out
}

fn parse_cex(msg: &str) -> Option<Vec<(String, i128)>> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    rest.split(", ")
        .map(|kv| {
            let (k, v) = kv.split_once(" = ")?;
            Some((k.to_string(), v.trim().parse().ok()?))
        })
        .collect()
}

#[derive(Default)]
struct Stats {
    asserts: usize,
    verified: usize,
    cexes: usize,
    guarded: usize,
    undecided: usize,
}

fn batch(seed: u64, stats: &mut Stats) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let t = if rng.chance(50) { Ty::U8 } else { Ty::I8 };
    let tn = t.name();
    // `h`'s body is over its parameter `v` alone.
    let hb = rename_e(&gen_e(&mut rng, t, 2, false));
    let h = H { body: hb };
    let h_text = e_text(&h.body);
    let mut src = format!(
        "#[summary(\"비트 벡터 하위 증명 건전성 퍼징\")]\nmod m {{\n    spec fn h(v: {tn}) -> {tn} {{ {h_text} }}\n\n"
    );
    let mut cases = Vec::new();
    for k in 0..8 {
        // Small assertions are often valid; some are made so.
        let c = gen_c(&mut rng, t, 3, true);
        let c = if rng.chance(30) {
            C::Or(Box::new(c.clone()), Box::new(C::Not(Box::new(c))))
        } else {
            c
        };
        src.push_str(&format!(
            "    #[effects(pure)]\n    #[uses(h)]\n    fn f{k}(x: {tn}, y: {tn}) -> () {{\n        assert({}) by (bit_vector);\n    }}\n\n",
            c_text(&c)
        ));
        cases.push(c);
    }
    src.push_str("}\n");
    // Through the CLI, as a user runs it (HEALTH S5).
    let path = std::env::temp_dir().join(format!("bmb-bv-{}-{seed}.bmb", std::process::id()));
    std::fs::write(&path, &src).unwrap();
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("verify")
        .arg(&path)
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&path);
    let output = String::from_utf8_lossy(&run.stdout).into_owned();
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{output}");
    assert!(
        !output.contains("\"code\":\"E0") || output.contains("E08"),
        "{}",
        ctx("the program must check")
    );
    assert!(
        !output.contains("no bit-vector translation"),
        "{}",
        ctx("every generated assertion has a translation")
    );
    for (k, c) in cases.iter().enumerate() {
        let name = format!("m::f{k}");
        stats.asserts += 1;
        let bad = failures(c, t, &h);
        let guarded = {
            let (lo, hi) = t.range();
            (lo..=hi).any(|x| {
                (lo..=hi).any(|y| {
                    let mut p = Point { x, y, guard: true };
                    eval_c(c, t, &h, &mut p);
                    !p.guard
                })
            })
        };
        stats.guarded += usize::from(guarded);
        let status = output
            .lines()
            .find(|l| field(l, "def") == Some(name.as_str()))
            .and_then(|l| field(l, "status"))
            .unwrap_or("missing");
        if status == "verified" {
            stats.verified += 1;
            assert!(
                bad.is_empty(),
                "{}",
                ctx(&format!(
                    "{name} verified, but its assertion fails at x = {}, y = {}",
                    bad[0].0, bad[0].1
                ))
            );
            continue;
        }
        let msg = output
            .lines()
            .filter_map(|l| field(l, "message"))
            .find(|m| m.contains(&format!("in `{name}`")))
            .unwrap_or_else(|| panic!("{}", ctx(&format!("{name} failed without a message"))));
        if msg.contains("could not decide") {
            stats.undecided += 1;
            continue;
        }
        let cex = parse_cex(msg)
            .unwrap_or_else(|| panic!("{}", ctx(&format!("{name}: no counterexample: {msg}"))));
        stats.cexes += 1;
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, x)| *x);
        let (lo, hi) = t.range();
        // A variable the assertion does not mention is not shown: any value
        // does.
        let xs: Vec<i128> = get("x").map_or((lo..=hi).collect(), |v| vec![v]);
        let ys: Vec<i128> = get("y").map_or((lo..=hi).collect(), |v| vec![v]);
        for &x in &xs {
            assert!(
                lo <= x && x <= hi,
                "{}",
                ctx(&format!("x out of range: {msg}"))
            );
            for &y in &ys {
                assert!(
                    bad.contains(&(x, y)),
                    "{}",
                    ctx(&format!(
                        "{name}: the counterexample x = {x}, y = {y} does not break the assertion: {msg}"
                    ))
                );
            }
        }
    }
}

#[test]
fn bit_vector_sub_proofs_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let mut stats = Stats::default();
    for seed in start..start + iters {
        batch(seed, &mut stats);
    }
    eprintln!(
        "soundness (bit_vector): {} assertions, {} verified, {} counterexamples replayed, {} with a guarded spec fn argument, {} undecided",
        stats.asserts, stats.verified, stats.cexes, stats.guarded, stats.undecided
    );
    assert!(stats.verified > 0 && stats.cexes > 0);
}
