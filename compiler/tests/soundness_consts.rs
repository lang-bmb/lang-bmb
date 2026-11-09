//! Constant evaluation against execution (backlog 2V-19b, ADR-0052 §4):
//! random `const` initializers over the integer widths, built from
//! literals, `T::MIN`/`T::MAX`, `+ - * / %`, unary `-` and `!`, `& | ^`,
//! `<<`/`>>` by literal amounts (below, at and beyond the width), the
//! wrapping and saturating methods, `x as T` from another width, `if` on a
//! comparison, and an element of an array literal (an index past its end
//! too).
//!
//! An independent interpreter (this test: Rust's rules for each width on
//! `i128`, not the compiler's code) judges each initializer, and three
//! readings must agree with it:
//!
//! - the checker's evaluation: `bmb check` reports E0260 for exactly the
//!   initializers Rust rejects (overflow, division by zero, `MIN / -1` and
//!   `MIN % -1`, a shift of the width or more, a conversion out of range,
//!   an index past the end), and nothing else;
//! - the build: a program prints every accepted `const` (its value is a
//!   literal where it is read) and the same expression computed by a
//!   function, which the prover must accept, and both lines are the
//!   interpreter's value;
//! - the prover: the function of a rejected initializer is not verified.
//!
//! A literal whose type no context gives (an `as` operand, a comparison,
//! a method's receiver, an element of an indexed array literal) is written
//! `(Z + v)` with `Z` a zero `const` of the wanted type, so the interpreter
//! and the compiler read the same types. Through the CLI (HEALTH S5). `BMB_SOUND_ITERS` /
//! `BMB_SOUND_SEED` as in `soundness.rs`.

mod common;

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

use common::scalar::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ty {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

const ALL: [Ty; 8] = [
    Ty::I8,
    Ty::I16,
    Ty::I32,
    Ty::I64,
    Ty::U8,
    Ty::U16,
    Ty::U32,
    Ty::U64,
];

impl Ty {
    fn name(self) -> &'static str {
        match self {
            Ty::I8 => "i8",
            Ty::I16 => "i16",
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::U8 => "u8",
            Ty::U16 => "u16",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
        }
    }

    fn bits(self) -> u32 {
        match self {
            Ty::I8 | Ty::U8 => 8,
            Ty::I16 | Ty::U16 => 16,
            Ty::I32 | Ty::U32 => 32,
            Ty::I64 | Ty::U64 => 64,
        }
    }

    fn signed(self) -> bool {
        matches!(self, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64)
    }

    fn min(self) -> i128 {
        if self.signed() {
            -(1i128 << (self.bits() - 1))
        } else {
            0
        }
    }

    fn max(self) -> i128 {
        if self.signed() {
            (1i128 << (self.bits() - 1)) - 1
        } else {
            (1i128 << self.bits()) - 1
        }
    }

    fn zero(self) -> String {
        format!("Z{}", self.name().to_uppercase())
    }

    fn wrap(self, v: i128) -> i128 {
        let m = 1i128 << self.bits();
        let r = v.rem_euclid(m);
        if self.signed() && r > self.max() {
            r - m
        } else {
            r
        }
    }
}

#[derive(Clone, Debug)]
enum X {
    /// A literal its context types.
    Lit(i128),
    /// A literal no context types: `(Z + v)`.
    Typed(i128),
    Bound(bool),
    Neg(Box<X>),
    Not(Box<X>),
    Bin(&'static str, Box<X>, Box<X>),
    Shift(&'static str, Box<X>, u32),
    Method(&'static str, Box<X>, Box<X>),
    /// `(src as T)`, `src` of the given type.
    As(Ty, Box<X>),
    /// `if a CMP b { then } else { els }`, `a` and `b` of the given type.
    If(Ty, &'static str, Box<X>, Box<X>, Box<X>, Box<X>),
    /// `[items..][k]`
    Index(Vec<X>, usize),
}

#[derive(Debug)]
struct Fault;

fn literal(rng: &mut Rng, t: Ty) -> i128 {
    match rng.below(8) {
        0 => 0,
        1 => 1,
        2 if t.signed() => -1,
        3 => t.max(),
        4 => t.min(),
        5 => t.max() / 2 + 1,
        6 => (rng.next() as i128 % 100) - if t.signed() { 50 } else { 0 },
        _ => t.min() + (rng.next() as i128).rem_euclid(t.max() - t.min() + 1),
    }
}

/// An expression of type `t`; `anchor`: its type must be readable without
/// a context (its leftmost leaf is typed).
fn make(rng: &mut Rng, t: Ty, depth: usize, anchor: bool) -> X {
    if depth == 0 || rng.chance(25) {
        return match rng.below(4) {
            0 => X::Bound(rng.chance(50)),
            _ if anchor => X::Typed(literal(rng, t)),
            _ => X::Lit(literal(rng, t)),
        };
    }
    let d = depth - 1;
    match rng.below(11) {
        0 if t.signed() => X::Neg(Box::new(make(rng, t, d, true))),
        1 => X::Not(Box::new(make(rng, t, d, anchor))),
        2..=4 => {
            let op = ["+", "-", "*", "/", "%", "&", "|", "^"][rng.below(8)];
            X::Bin(
                op,
                Box::new(make(rng, t, d, anchor)),
                Box::new(make(rng, t, d, false)),
            )
        }
        5 => {
            let op = ["<<", ">>"][rng.below(2)];
            let k = match rng.below(4) {
                0 => t.bits(),
                1 => t.bits() + 1,
                _ => rng.below(t.bits() as usize) as u32,
            };
            X::Shift(op, Box::new(make(rng, t, d, anchor)), k)
        }
        6 => {
            let m = [
                "wrapping_add",
                "wrapping_sub",
                "wrapping_mul",
                "saturating_add",
                "saturating_sub",
                "saturating_mul",
            ][rng.below(6)];
            X::Method(
                m,
                Box::new(make(rng, t, d, true)),
                Box::new(make(rng, t, d, false)),
            )
        }
        7 => {
            let s = ALL[rng.below(ALL.len())];
            X::As(s, Box::new(make(rng, s, d, true)))
        }
        8 => {
            let s = ALL[rng.below(ALL.len())];
            let cmp = ["==", "!=", "<", "<=", ">", ">="][rng.below(6)];
            X::If(
                s,
                cmp,
                Box::new(make(rng, s, d, true)),
                Box::new(make(rng, s, d, false)),
                Box::new(make(rng, t, d, anchor)),
                Box::new(make(rng, t, d, anchor)),
            )
        }
        9 => {
            let n = 1 + rng.below(3);
            // The index's result type is not the literal's context
            // (BMB, as the checker reads it): each element is typed.
            let items = (0..n).map(|_| make(rng, t, d, true)).collect();
            X::Index(items, rng.below(n + 1))
        }
        _ => X::Bin(
            "+",
            Box::new(make(rng, t, d, anchor)),
            Box::new(X::Lit(literal(rng, t))),
        ),
    }
}

fn text(x: &X, t: Ty) -> String {
    match x {
        X::Lit(v) => format!("{v}"),
        X::Typed(v) => format!("({} + {v})", t.zero()),
        X::Bound(max) => format!("{}::{}", t.name(), if *max { "MAX" } else { "MIN" }),
        X::Neg(a) => format!("-({})", text(a, t)),
        X::Not(a) => format!("!({})", text(a, t)),
        X::Bin(op, a, b) => format!("({} {op} {})", text(a, t), text(b, t)),
        X::Shift(op, a, k) => format!("({} {op} {k})", text(a, t)),
        X::Method(m, a, b) => format!("({}).{m}({})", text(a, t), text(b, t)),
        X::As(s, a) => format!("(({}) as {})", text(a, *s), t.name()),
        X::If(s, cmp, a, b, th, el) => format!(
            "(if {} {cmp} {} {{ {} }} else {{ {} }})",
            text(a, *s),
            text(b, *s),
            text(th, t),
            text(el, t)
        ),
        X::Index(items, k) => format!(
            "([{}][{k}])",
            items
                .iter()
                .map(|i| text(i, t))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Rust's value of `x` as a `t`, or the fault Rust reports.
fn eval(x: &X, t: Ty) -> Result<i128, Fault> {
    let fit = |v: i128| {
        if v < t.min() || v > t.max() {
            Err(Fault)
        } else {
            Ok(v)
        }
    };
    match x {
        X::Lit(v) | X::Typed(v) => Ok(*v),
        X::Bound(max) => Ok(if *max { t.max() } else { t.min() }),
        X::Neg(a) => fit(-eval(a, t)?),
        X::Not(a) => {
            let v = eval(a, t)?;
            Ok(if t.signed() { -v - 1 } else { t.max() ^ v })
        }
        X::Bin(op, a, b) => {
            let (p, q) = (eval(a, t)?, eval(b, t)?);
            match *op {
                "+" => fit(p + q),
                "-" => fit(p - q),
                "*" => fit(p.checked_mul(q).ok_or(Fault)?),
                "/" | "%" if q == 0 => Err(Fault),
                "/" | "%" if t.signed() && p == t.min() && q == -1 => Err(Fault),
                "/" => fit(p / q),
                "%" => fit(p % q),
                "&" => Ok(p & q),
                "|" => Ok(p | q),
                _ => Ok(p ^ q),
            }
        }
        X::Shift(op, a, k) => {
            let v = eval(a, t)?;
            if *k >= t.bits() {
                return Err(Fault);
            }
            Ok(if *op == "<<" { t.wrap(v << k) } else { v >> k })
        }
        X::Method(m, a, b) => {
            let (p, q) = (eval(a, t)?, eval(b, t)?);
            let exact = match &m[m.len() - 3..] {
                "add" => p + q,
                "sub" => p - q,
                _ => p.saturating_mul(q),
            };
            Ok(if m.starts_with("wrapping") {
                if m.ends_with("mul") {
                    t.wrap(p.wrapping_mul(q))
                } else {
                    t.wrap(exact)
                }
            } else {
                exact.clamp(t.min(), t.max())
            })
        }
        X::As(s, a) => fit(eval(a, *s)?),
        X::If(s, cmp, a, b, th, el) => {
            let (p, q) = (eval(a, *s)?, eval(b, *s)?);
            let c = match *cmp {
                "==" => p == q,
                "!=" => p != q,
                "<" => p < q,
                "<=" => p <= q,
                ">" => p > q,
                _ => p >= q,
            };
            if c { eval(th, t) } else { eval(el, t) }
        }
        X::Index(items, k) => {
            let vals = items
                .iter()
                .map(|i| eval(i, t))
                .collect::<Result<Vec<_>, _>>()?;
            vals.get(*k).copied().ok_or(Fault)
        }
    }
}

fn zeros() -> String {
    ALL.iter()
        .map(|t| format!("    const {}: {} = 0;\n", t.zero(), t.name()))
        .collect()
}

fn printed(t: Ty, e: &str) -> String {
    if t.signed() {
        format!("print_i64(({e}) as i64);")
    } else {
        format!("print_u64(({e}) as u64);")
    }
}

fn bmb(dir: &std::path::Path, args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn const_evaluation_agrees_with_rust_and_with_execution() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let seed: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0x2019_b0c5);
    let mut rng = Rng(seed);
    let (mut accepted, mut rejected) = (0, 0);
    for batch in 0..iters {
        let cases: Vec<(Ty, X)> = (0..16)
            .map(|k| {
                let t = ALL[k % ALL.len()];
                (t, make(&mut rng, t, 3, false))
            })
            .collect();
        let dir = common::cli::fresh_dir();
        // 1. The checker's evaluation: E0260 exactly where Rust rejects.
        let mut src = format!("#[summary(\"const fuzz {batch}\")]\nmod kf {{\n{}", zeros());
        let first = src.lines().count() + 1;
        for (k, (t, x)) in cases.iter().enumerate() {
            let _ = writeln!(src, "    const K{k}: {} = {};", t.name(), text(x, *t));
        }
        src.push_str("}\n");
        fs::write(dir.join("k.bmb"), &src).unwrap();
        let (_, out) = bmb(&dir, &["check", "k.bmb"]);
        let mut failed = vec![false; cases.len()];
        for line in out.lines() {
            let code = common::scalar::field(line, "code").unwrap_or_default();
            let at: usize = line
                .split("\"line\":")
                .nth(1)
                .map(|r| {
                    r.chars()
                        .take_while(char::is_ascii_digit)
                        .collect::<String>()
                })
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            assert!(
                code == "E0260" && at >= first && at < first + cases.len(),
                "unexpected diagnostic {line}\n{src}"
            );
            failed[at - first] = true;
        }
        for (k, (t, x)) in cases.iter().enumerate() {
            let rust = eval(x, *t);
            assert_eq!(
                rust.is_err(),
                failed[k],
                "K{k} = {}: Rust {rust:?}, the checker {}\n{src}",
                text(x, *t),
                if failed[k] {
                    "rejects it"
                } else {
                    "accepts it"
                }
            );
        }
        // 2. The build: each accepted `const` and its expression computed.
        let mut src = format!(
            "#[summary(\"const fuzz run {batch}\")]\nmod kr {{\n{}",
            zeros()
        );
        let mut main = String::new();
        let mut want = String::new();
        for (k, (t, x)) in cases.iter().enumerate().filter(|(k, _)| !failed[*k]) {
            let e = text(x, *t);
            let _ = writeln!(src, "    const K{k}: {} = {e};", t.name());
            let _ = writeln!(
                src,
                "    #[effects(pure)]\n    fn r{k}() -> {} {{ {e} }}",
                t.name()
            );
            let _ = writeln!(
                main,
                "        {}\n        {}",
                printed(*t, &format!("K{k}")),
                printed(*t, &format!("r{k}()"))
            );
            let v = eval(x, *t).unwrap();
            let _ = writeln!(want, "{v}\n{v}");
            accepted += 1;
        }
        let _ = write!(
            src,
            "    #[effects(io)]\n    fn main() -> () {{\n{main}    }}\n}}\n"
        );
        fs::write(dir.join("r.bmb"), &src).unwrap();
        let (ok, out) = bmb(&dir, &["build", "r.bmb", "-o", "r"]);
        assert!(
            ok,
            "the build of the accepted constants failed:\n{out}\n{src}"
        );
        let run = Command::new(dir.join("r")).output().unwrap();
        assert_eq!(String::from_utf8(run.stdout).unwrap(), want, "{src}");
        // 3. The prover: a rejected initializer's function is not verified.
        let bad: Vec<usize> = (0..cases.len()).filter(|k| failed[*k]).collect();
        if !bad.is_empty() {
            let mut src = format!(
                "#[summary(\"const fuzz fail {batch}\")]\nmod kv {{\n{}",
                zeros()
            );
            for &k in &bad {
                let (t, x) = &cases[k];
                let _ = writeln!(
                    src,
                    "    #[effects(pure)]\n    fn r{k}() -> {} {{ {} }}",
                    t.name(),
                    text(x, *t)
                );
                rejected += 1;
            }
            src.push_str("}\n");
            fs::write(dir.join("v.bmb"), &src).unwrap();
            let (_, out) = bmb(&dir, &["verify", "v.bmb"]);
            for &k in &bad {
                let verified = out.lines().any(|l| {
                    l.contains(&format!("\"def\":\"kv::r{k}\""))
                        && l.contains("\"status\":\"verified\"")
                });
                assert!(
                    !verified,
                    "r{k} is verified, but Rust rejects its constant:\n{out}\n{src}"
                );
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }
    assert!(
        accepted > 0 && rejected > 0,
        "accepted {accepted}, rejected {rejected}"
    );
}
