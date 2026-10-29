//! Soundness fuzzing of `f64` (backlog T2-03, ADR-0061 §7): random
//! expressions over two `f64` variables and an `i32` (through `f64::from`)
//! with `+ - * /`, unary `-`, `sqrt`, `abs`, `min`, `max`, comparisons,
//! `is_nan`, `is_finite` and `if`, on inputs drawn from finite values,
//! signed zeros, infinities, NaN and subnormals. An independent interpreter
//! (this test, with Rust's `f64`, which is IEEE 754 with rounding to
//! nearest even) checks three claims:
//!
//! - both builds of the program (proved checks removed, and `--all-checks`)
//!   print what the interpreter computes. Values are printed with 1074
//!   decimals, which tell every `f64` apart and show the sign of zero, so
//!   equal text is equal bits; every NaN prints `NaN` (compared by
//!   `is_nan`, as NaN payloads are not part of the language);
//! - a verified assertion (plain, or `by (bit_vector)`) holds at every
//!   sampled input;
//! - every counterexample breaks its assertion when replayed (G1: a model
//!   of the uninterpreted operations that IEEE arithmetic does not confirm
//!   must not be shown).
//!
//! `min` and `max` take a finite nonzero literal as their second operand:
//! Rust leaves the sign of `min`/`max` of two zeros open.
//!
//! Through the CLI, as a user runs it (HEALTH S5). `BMB_SOUND_ITERS` /
//! `BMB_SOUND_SEED` as in `soundness.rs`.

mod common;

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

use common::scalar::{Rng, field};

#[derive(Clone, Copy, PartialEq)]
enum Bin {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, PartialEq)]
enum Un {
    Neg,
    Sqrt,
    Abs,
}

#[derive(Clone, Copy, PartialEq)]
enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Clone)]
enum E {
    X,
    Y,
    /// `f64::from(n)`.
    N,
    Lit(f64),
    Bin(Bin, Box<E>, Box<E>),
    Un(Un, Box<E>),
    /// `a.min(c)` / `a.max(c)` with `c` a finite nonzero literal.
    MinMax(bool, Box<E>, f64),
    If(Box<C>, Box<E>, Box<E>),
}

#[derive(Clone)]
enum C {
    Cmp(Cmp, E, E),
    IsNan(E),
    IsFinite(E),
    Not(Box<C>),
    And(Box<C>, Box<C>),
    Or(Box<C>, Box<C>),
}

/// A value of each class: finite normal, signed zero, infinity, NaN,
/// subnormal, and the edges.
fn value(rng: &mut Rng) -> f64 {
    match rng.below(10) {
        0 => [0.0, -0.0][rng.below(2)],
        1 => [f64::INFINITY, f64::NEG_INFINITY][rng.below(2)],
        2 => f64::NAN,
        // A subnormal.
        3 => f64::from_bits(rng.next() & 0x800f_ffff_ffff_ffff | 1),
        4 => [
            f64::MAX,
            -f64::MAX,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            1.0,
            -1.0,
            0.5,
            2.0,
        ][rng.below(8)],
        // Any finite value.
        5 => {
            let v = f64::from_bits(rng.next());
            if v.is_finite() { v } else { 1.5 }
        }
        // Small values with short decimal forms.
        _ => (rng.below(4001) as f64 - 2000.0) / 16.0,
    }
}

fn nonzero_finite(rng: &mut Rng) -> f64 {
    loop {
        let v = value(rng);
        if v.is_finite() && v != 0.0 {
            return v;
        }
    }
}

/// A BMB literal (or constant) of `v`, parenthesized when negative.
fn lit(v: f64) -> String {
    if v.is_nan() {
        return "f64::NAN".into();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "f64::INFINITY"
        } else {
            "f64::NEG_INFINITY"
        }
        .into();
    }
    let s = format!("{:?}", v.abs());
    let s = match s.split_once('e') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0e{e}"),
        _ => s,
    };
    if v.is_sign_negative() {
        format!("(-{s})")
    } else {
        s
    }
}

fn gen_e(rng: &mut Rng, depth: usize) -> E {
    if depth == 0 || rng.chance(25) {
        return match rng.below(6) {
            0 | 1 => E::X,
            2 | 3 => E::Y,
            4 => E::N,
            _ => E::Lit(value(rng)),
        };
    }
    let d = depth - 1;
    match rng.below(10) {
        0..=4 => {
            let op = [Bin::Add, Bin::Sub, Bin::Mul, Bin::Div][rng.below(4)];
            E::Bin(op, Box::new(gen_e(rng, d)), Box::new(gen_e(rng, d)))
        }
        5 | 6 => {
            let op = [Un::Neg, Un::Sqrt, Un::Abs][rng.below(3)];
            E::Un(op, Box::new(gen_e(rng, d)))
        }
        7 => E::MinMax(rng.chance(50), Box::new(gen_e(rng, d)), nonzero_finite(rng)),
        _ => E::If(
            Box::new(gen_c(rng, d)),
            Box::new(gen_e(rng, d)),
            Box::new(gen_e(rng, d)),
        ),
    }
}

fn gen_c(rng: &mut Rng, depth: usize) -> C {
    let d = depth.saturating_sub(1);
    match rng.below(if depth == 0 { 3 } else { 8 }) {
        0 => C::IsNan(gen_e(rng, d)),
        1 => C::IsFinite(gen_e(rng, d)),
        2..=4 => {
            let op = [Cmp::Lt, Cmp::Le, Cmp::Gt, Cmp::Ge, Cmp::Eq, Cmp::Ne][rng.below(6)];
            C::Cmp(op, gen_e(rng, d), gen_e(rng, d))
        }
        5 => C::Not(Box::new(gen_c(rng, d))),
        6 => C::And(Box::new(gen_c(rng, d)), Box::new(gen_c(rng, d))),
        _ => C::Or(Box::new(gen_c(rng, d)), Box::new(gen_c(rng, d))),
    }
}

fn e_text(e: &E) -> String {
    match e {
        E::X => "x".into(),
        E::Y => "y".into(),
        E::N => "f64::from(n)".into(),
        E::Lit(v) => lit(*v),
        E::Bin(op, a, b) => {
            let o = match op {
                Bin::Add => "+",
                Bin::Sub => "-",
                Bin::Mul => "*",
                Bin::Div => "/",
            };
            format!("({} {o} {})", e_text(a), e_text(b))
        }
        E::Un(Un::Neg, a) => format!("(-{})", e_text(a)),
        E::Un(Un::Sqrt, a) => format!("({}).sqrt()", e_text(a)),
        E::Un(Un::Abs, a) => format!("({}).abs()", e_text(a)),
        E::MinMax(min, a, c) => {
            format!(
                "({}).{}({})",
                e_text(a),
                if *min { "min" } else { "max" },
                lit(*c)
            )
        }
        E::If(c, a, b) => format!(
            "(if {} {{ {} }} else {{ {} }})",
            c_text(c),
            e_text(a),
            e_text(b)
        ),
    }
}

fn c_text(c: &C) -> String {
    match c {
        C::Cmp(op, a, b) => {
            let o = match op {
                Cmp::Lt => "<",
                Cmp::Le => "<=",
                Cmp::Gt => ">",
                Cmp::Ge => ">=",
                Cmp::Eq => "==",
                Cmp::Ne => "!=",
            };
            format!("({} {o} {})", e_text(a), e_text(b))
        }
        C::IsNan(a) => format!("({}).is_nan()", e_text(a)),
        C::IsFinite(a) => format!("({}).is_finite()", e_text(a)),
        C::Not(a) => format!("!{}", c_text(a)),
        C::And(a, b) => format!("({} && {})", c_text(a), c_text(b)),
        C::Or(a, b) => format!("({} || {})", c_text(a), c_text(b)),
    }
}

fn e_vars(e: &E) -> bool {
    match e {
        E::X | E::Y | E::N => true,
        E::Lit(_) => false,
        E::Bin(_, a, b) => e_vars(a) || e_vars(b),
        E::Un(_, a) | E::MinMax(_, a, _) => e_vars(a),
        E::If(c, a, b) => c_vars(c) || e_vars(a) || e_vars(b),
    }
}

/// Whether `c` mentions a variable.
fn c_vars(c: &C) -> bool {
    match c {
        C::Cmp(_, a, b) => e_vars(a) || e_vars(b),
        C::IsNan(a) | C::IsFinite(a) => e_vars(a),
        C::Not(a) => c_vars(a),
        C::And(a, b) | C::Or(a, b) => c_vars(a) || c_vars(b),
    }
}

/// The interpreter: Rust's `f64`.
fn eval(e: &E, x: f64, y: f64, n: i32) -> f64 {
    match e {
        E::X => x,
        E::Y => y,
        E::N => f64::from(n),
        E::Lit(v) => *v,
        E::Bin(op, a, b) => {
            let (a, b) = (eval(a, x, y, n), eval(b, x, y, n));
            match op {
                Bin::Add => a + b,
                Bin::Sub => a - b,
                Bin::Mul => a * b,
                Bin::Div => a / b,
            }
        }
        E::Un(op, a) => {
            let a = eval(a, x, y, n);
            match op {
                Un::Neg => -a,
                Un::Sqrt => a.sqrt(),
                Un::Abs => a.abs(),
            }
        }
        E::MinMax(min, a, c) => {
            let a = eval(a, x, y, n);
            if *min { a.min(*c) } else { a.max(*c) }
        }
        E::If(c, a, b) => {
            if test(c, x, y, n) {
                eval(a, x, y, n)
            } else {
                eval(b, x, y, n)
            }
        }
    }
}

fn test(c: &C, x: f64, y: f64, n: i32) -> bool {
    match c {
        C::Cmp(op, a, b) => {
            let (a, b) = (eval(a, x, y, n), eval(b, x, y, n));
            match op {
                Cmp::Lt => a < b,
                Cmp::Le => a <= b,
                Cmp::Gt => a > b,
                Cmp::Ge => a >= b,
                Cmp::Eq => a == b,
                Cmp::Ne => a != b,
            }
        }
        C::IsNan(a) => eval(a, x, y, n).is_nan(),
        C::IsFinite(a) => eval(a, x, y, n).is_finite(),
        C::Not(a) => !test(a, x, y, n),
        C::And(a, b) => test(a, x, y, n) && test(b, x, y, n),
        C::Or(a, b) => test(a, x, y, n) || test(b, x, y, n),
    }
}

fn int(rng: &mut Rng) -> i32 {
    match rng.below(4) {
        0 => [0, 1, -1, i32::MAX, i32::MIN, 16777217][rng.below(6)],
        1 => rng.below(201) as i32 - 100,
        _ => rng.next() as i32,
    }
}

/// The sampled inputs: random ones and every pair of special values.
fn inputs(rng: &mut Rng) -> Vec<(f64, f64, i32)> {
    let mut v: Vec<(f64, f64, i32)> = (0..24)
        .map(|_| (value(rng), value(rng), int(rng)))
        .collect();
    let special = [
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
        5.0e-324,
        1.0,
    ];
    for &a in &special {
        for &b in &special {
            v.push((a, b, int(rng)));
        }
    }
    v
}

#[derive(Default)]
struct Stats {
    runs: usize,
    claims: usize,
    verified: usize,
    cexes: usize,
    undecided: usize,
}

/// `name = value` pairs of a counterexample message.
fn cex_values(msg: &str) -> Vec<(String, String)> {
    let Some(rest) = msg.split("; counterexample: ").nth(1) else {
        return Vec::new();
    };
    rest.split(", ")
        .filter_map(|kv| kv.split_once(" = "))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let bodies: Vec<E> = (0..6).map(|_| gen_e(&mut rng, 4)).collect();
    let claims: Vec<(C, bool)> = (0..6)
        .map(|k| {
            let c = gen_c(&mut rng, 2);
            // Some claims are made valid (a tautology over their atoms).
            let c = if rng.chance(30) {
                C::Or(Box::new(c.clone()), Box::new(C::Not(Box::new(c))))
            } else {
                c
            };
            (c, k % 2 == 1)
        })
        .collect();
    let mut src = String::from("#[summary(\"f64 퍼징\")]\nmod m {\n");
    for (k, b) in bodies.iter().enumerate() {
        let _ = write!(
            src,
            "    #[effects(pure)]\n    fn f{k}(x: f64, y: f64, n: i32) -> f64 {{\n        {}\n    }}\n\n",
            e_text(b)
        );
    }
    for (k, (c, bv)) in claims.iter().enumerate() {
        let by = if *bv { " by (bit_vector)" } else { "" };
        let _ = write!(
            src,
            "    #[effects(pure)]\n    fn c{k}(x: f64, y: f64, n: i32) -> () {{\n        assert({}){by};\n    }}\n\n",
            c_text(c)
        );
    }
    src.push_str("    #[effects(io)]\n    fn main() -> () {\n");
    let mut expected = String::new();
    let samples = inputs(&mut rng);
    for (k, b) in bodies.iter().enumerate() {
        for &(x, y, n) in samples.iter().step_by(3) {
            let _ = writeln!(
                src,
                "        print_f64(f{k}({}, {}, {n}), 1074);",
                lit(x),
                lit(y)
            );
            let _ = writeln!(expected, "{:.1074}", eval(b, x, y, n));
        }
    }
    src.push_str("    }\n}\n");
    let file = dir.join(format!("f{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    let emit = |all_checks: bool| -> (String, Option<String>) {
        let ll = dir.join(format!("f{seed}-{all_checks}.ll"));
        let _ = fs::remove_file(&ll);
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("emit-llvm");
        if all_checks {
            cmd.arg("--all-checks");
        }
        let out = cmd.arg(&file).arg("-o").arg(&ll).output().unwrap();
        let ir = fs::read_to_string(&ll).ok();
        let _ = fs::remove_file(&ll);
        (String::from_utf8_lossy(&out.stdout).into_owned(), ir)
    };
    let (output, ir) = emit(false);
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{output}");
    assert!(ir.is_some(), "{}", ctx("the generated program must check"));

    for (k, (c, _)) in claims.iter().enumerate() {
        let name = format!("m::c{k}");
        stats.claims += 1;
        let status = output
            .lines()
            .find(|l| field(l, "def") == Some(name.as_str()))
            .and_then(|l| field(l, "status"))
            .unwrap_or("missing");
        if status == "verified" {
            stats.verified += 1;
            for &(x, y, n) in &samples {
                assert!(
                    test(c, x, y, n),
                    "{}",
                    ctx(&format!(
                        "{name} verified, but its assertion fails at x = {x:?}, y = {y:?}, n = {n}"
                    ))
                );
            }
            continue;
        }
        let msg = output
            .lines()
            .filter_map(|l| field(l, "message"))
            .find(|m| m.contains(&format!("in `{name}`")))
            .unwrap_or_else(|| panic!("{}", ctx(&format!("{name} failed without a message"))));
        let vals = cex_values(msg);
        if vals.is_empty() {
            // A sub-proof without free variables has no values to show: its
            // assertion is false. One with variables shows none when IEEE
            // arithmetic does not confirm the model (`min`/`max` of zeros).
            let closed = !c_vars(c);
            assert!(
                msg.contains("could not decide")
                    || (msg.contains("bit-vector assertion may not hold")
                        && (!closed || !test(c, 0.0, 0.0, 0))),
                "{}",
                ctx(&format!(
                    "{name}: neither proved, undecided nor a counterexample"
                ))
            );
            if closed && !msg.contains("could not decide") {
                stats.cexes += 1;
            } else {
                stats.undecided += 1;
            }
            continue;
        }
        stats.cexes += 1;
        let get = |v: &str| vals.iter().find(|(k, _)| k == v).map(|(_, x)| x.clone());
        let f = |v: &str| -> Vec<f64> {
            match get(v) {
                Some(t) => vec![
                    t.parse()
                        .unwrap_or_else(|_| panic!("{}", ctx(&format!("`{t}` is not an f64")))),
                ],
                // Not shown (a sub-proof shows the assertion's variables):
                // any value does.
                None => vec![0.0, -1.5, f64::NAN, f64::INFINITY],
            }
        };
        let ns: Vec<i32> = match get("n") {
            Some(t) => vec![
                t.parse()
                    .unwrap_or_else(|_| panic!("{}", ctx(&format!("`{t}` is not an i32")))),
            ],
            None => vec![0, -7, i32::MAX],
        };
        for &x in &f("x") {
            for &y in &f("y") {
                for &n in &ns {
                    assert!(
                        !test(c, x, y, n),
                        "{}",
                        ctx(&format!(
                            "{name}: the counterexample x = {x:?}, y = {y:?}, n = {n} does not break the assertion: {msg}"
                        ))
                    );
                }
            }
        }
    }

    for all_checks in [false, true] {
        let ir = if all_checks {
            emit(true).1.unwrap()
        } else {
            ir.clone().unwrap()
        };
        let exe = dir.join(format!("f{seed}-{all_checks}"));
        let ll = dir.join(format!("f{seed}-{all_checks}.build.ll"));
        fs::write(&ll, &ir).unwrap();
        let cc = Command::new("clang")
            .args(["-O2", "-Wno-override-module", "-o"])
            .arg(&exe)
            .arg(&ll)
            .output()
            .unwrap();
        let _ = fs::remove_file(&ll);
        assert!(
            cc.status.success(),
            "{}",
            ctx(&String::from_utf8_lossy(&cc.stderr))
        );
        let run = Command::new(&exe).output().unwrap();
        let _ = fs::remove_file(&exe);
        assert!(
            run.status.success(),
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) exited with {}",
                run.status
            ))
        );
        let got = String::from_utf8_lossy(&run.stdout);
        for (k, (g, w)) in got.lines().zip(expected.lines()).enumerate() {
            assert_eq!(
                g,
                w,
                "{}",
                ctx(&format!(
                    "build (all_checks = {all_checks}): line {k} differs"
                ))
            );
        }
        assert_eq!(
            got.lines().count(),
            expected.lines().count(),
            "{}",
            ctx("line count")
        );
        stats.runs += expected.lines().count();
    }
    let _ = fs::remove_file(&file);
}

#[test]
fn f64_builds_proofs_and_counterexamples_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let dir = std::env::temp_dir().join(format!("bmb-sound-f64-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        batch(seed, &mut stats, &dir);
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (f64): {} values compared over both builds, {} assertions, {} verified, {} counterexamples replayed, {} undecided",
        stats.runs, stats.claims, stats.verified, stats.cexes, stats.undecided
    );
    assert!(stats.runs > 0 && stats.verified > 0 && stats.cexes > 0);
}
