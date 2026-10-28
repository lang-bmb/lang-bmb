//! Soundness fuzzing of specifications (backlog 1B-36): named spec fns with
//! `forall` (explicit `#[trigger]`, one or two bound variables), `==>`,
//! recursive spec fns with `decreases` (sum and count over a prefix), and
//! `#[uses]` lists that sometimes leave out a needed spec fn, so opaque
//! reasoning (ADR-0017 §4) and the ground-instance retry (ADR-0023) run.
//!
//! Functions `gN(a: &[i64] | &mut [i64], i: usize, x: i64) -> (r: i64)` come
//! from templates that are correct by construction, often with one near-miss
//! mutation (a dropped pre or invariant clause, a changed index or literal, a
//! flipped comparison in a contract or in a spec fn body, an extra post
//! clause). The interpreter here evaluates the spec fns directly on concrete
//! arrays: `forall` over the finite index range (every generated quantifier
//! bounds its variables by an array length), recursive spec fns by recursion,
//! a call with an argument outside the parameter's type as undetermined (its
//! value is unspecified, ADR-0022), an out-of-range read as undetermined, and
//! `&&`, `||`, `==>` with three-valued logic. Claims:
//!
//! - a verified function, on every input whose precondition the interpreter
//!   finds true, does not fault, keeps its loop invariant at every loop head,
//!   and meets its postcondition;
//! - every definite counterexample (no "only by name" hint, so every spec fn
//!   the obligation reaches was unfolded) satisfies the precondition; for a
//!   loop-free function, running it on the counterexample faults or breaks the
//!   postcondition; for a loop, one for an invariant on entry breaks it on
//!   entry (later loop obligations start from an arbitrary loop state and are
//!   not replayed, as in `soundness_loops.rs`). Arrays shown in part are
//!   skipped;
//! - both builds (proved checks removed, every check kept) print what the
//!   interpreter computes, array contents included;
//! - lemmas (ADR-0062, backlog T1-18): two more functions per batch rest on a
//!   `proof fn` from templates (an induction on `i` over a recursive spec fn,
//!   or a plain proof) called in a `proof` block, with near-miss mutations of
//!   the lemma's or the caller's precondition, the conclusion, the recursive
//!   call (not decreasing, unguarded, missing) or a spec fn. A verified lemma's
//!   conclusion holds on every input its precondition accepts, a definite
//!   lemma counterexample meets the precondition and breaks the conclusion,
//!   and a verified caller (of a verified lemma) meets its postcondition.
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::{Rng, field, lit};

fn pick<T: Copy>(rng: &mut Rng, xs: &[T]) -> T {
    xs[rng.below(xs.len())]
}

// ---------------------------------------------------------------------------
// Specification expressions (mathematical integers).

#[derive(Clone, Copy, PartialEq, Debug)]
enum B {
    Add,
    Sub,
    Mul,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    Imp,
}

impl B {
    fn text(self) -> &'static str {
        match self {
            B::Add => "+",
            B::Sub => "-",
            B::Mul => "*",
            B::Lt => "<",
            B::Le => "<=",
            B::Gt => ">",
            B::Ge => ">=",
            B::Eq => "==",
            B::Ne => "!=",
            B::And => "&&",
            B::Or => "||",
            B::Imp => "==>",
        }
    }

    /// Near-miss partner of a comparison. Never turns a bound (`<`, `<=`)
    /// into `!=`, so quantifier ranges stay within one past an array length.
    fn flip(self) -> Option<B> {
        Some(match self {
            B::Lt => B::Le,
            B::Le => B::Lt,
            B::Gt => B::Ge,
            B::Ge => B::Gt,
            B::Eq => B::Ne,
            B::Ne => B::Eq,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
enum X {
    Int(i128),
    Var(String),
    Len(String),
    Idx(String, Box<X>),
    Trig(Box<X>),
    Bin(B, Box<X>, Box<X>),
    If(Box<X>, Box<X>, Box<X>),
    Call(String, Vec<X>),
    Forall(Vec<String>, Box<X>),
}

fn int_text(v: i128) -> String {
    if v == i64::MIN as i128 {
        "(-9223372036854775807 - 1)".into()
    } else if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

impl X {
    fn text(&self) -> String {
        match self {
            X::Int(v) => int_text(*v),
            X::Var(v) => v.clone(),
            X::Len(a) => format!("{a}.len()"),
            X::Idx(a, e) => format!("{a}[{}]", e.text()),
            X::Trig(e) => format!("(#[trigger] {})", e.text()),
            X::Bin(op, l, r) => format!("({} {} {})", l.text(), op.text(), r.text()),
            X::If(c, t, e) => format!(
                "(if {} {{ {} }} else {{ {} }})",
                c.text(),
                t.text(),
                e.text()
            ),
            X::Call(f, args) => format!(
                "{f}({})",
                args.iter().map(X::text).collect::<Vec<_>>().join(", ")
            ),
            X::Forall(vs, body) => format!(
                "(forall|{}| {})",
                vs.iter()
                    .map(|v| format!("{v}: usize"))
                    .collect::<Vec<_>>()
                    .join(", "),
                body.text()
            ),
        }
    }

    fn calls(&self, out: &mut Vec<String>) {
        match self {
            X::Int(_) | X::Var(_) | X::Len(_) => {}
            X::Idx(_, e) | X::Trig(e) | X::Forall(_, e) => e.calls(out),
            X::Bin(_, l, r) => {
                l.calls(out);
                r.calls(out);
            }
            X::If(c, t, e) => {
                c.calls(out);
                t.calls(out);
                e.calls(out);
            }
            X::Call(f, args) => {
                out.push(f.clone());
                args.iter().for_each(|a| a.calls(out));
            }
        }
    }

    /// Mutation sites: integer literals (bumped by one) and comparisons
    /// (flipped).
    fn sites(&self) -> usize {
        match self {
            X::Int(_) => 1,
            X::Var(_) | X::Len(_) => 0,
            X::Idx(_, e) | X::Trig(e) | X::Forall(_, e) => e.sites(),
            X::Bin(op, l, r) => op.flip().is_some() as usize + l.sites() + r.sites(),
            X::If(c, t, e) => c.sites() + t.sites() + e.sites(),
            X::Call(_, args) => args.iter().map(X::sites).sum(),
        }
    }

    fn mutate(&mut self, n: &mut usize) -> bool {
        match self {
            X::Int(v) => {
                if *n == 0 {
                    *v += 1;
                    return true;
                }
                *n -= 1;
                false
            }
            X::Var(_) | X::Len(_) => false,
            X::Idx(_, e) | X::Trig(e) | X::Forall(_, e) => e.mutate(n),
            X::Bin(op, l, r) => {
                if let Some(f) = op.flip() {
                    if *n == 0 {
                        *op = f;
                        return true;
                    }
                    *n -= 1;
                }
                l.mutate(n) || r.mutate(n)
            }
            X::If(c, t, e) => c.mutate(n) || t.mutate(n) || e.mutate(n),
            X::Call(_, args) => args.iter_mut().any(|a| a.mutate(n)),
        }
    }
}

fn mutate_some(xs: &mut [X], rng: &mut Rng) -> bool {
    let total: usize = xs.iter().map(X::sites).sum();
    if total == 0 {
        return false;
    }
    let mut n = rng.below(total);
    xs.iter_mut().any(|x| x.mutate(&mut n))
}

// A small parser, so templates read like the language.

#[derive(Clone, PartialEq, Debug)]
enum Tok {
    Int(i128),
    Id(String),
    Sym(&'static str),
}

fn lex(s: &str, prefix: &str) -> Vec<Tok> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            out.push(Tok::Int(s[st..i].parse().unwrap()));
        } else if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let st = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let w = &s[st..i];
            out.push(Tok::Id(match w.strip_prefix('$') {
                Some(lib) => format!("{prefix}_{lib}"),
                None => w.to_string(),
            }));
        } else {
            let sym = SYM_NAMES
                .iter()
                .find(|p| s[i..].starts_with(**p))
                .unwrap_or_else(|| panic!("unexpected `{c}` in {s}"));
            out.push(Tok::Sym(sym));
            i += sym.len();
        }
    }
    out
}

struct Parser {
    toks: Vec<Tok>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }
    fn eat(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(t)) if *t == s) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) {
        assert!(self.eat(s), "expected `{s}` at {:?}", self.peek());
    }
    fn id(&mut self) -> String {
        match self.toks.get(self.at).cloned() {
            Some(Tok::Id(s)) => {
                self.at += 1;
                s
            }
            t => panic!("expected an identifier, found {t:?}"),
        }
    }
    fn is_id(&self, w: &str) -> bool {
        matches!(self.peek(), Some(Tok::Id(s)) if s == w)
    }

    fn imp(&mut self) -> X {
        let l = self.or();
        if self.eat("==>") {
            return X::Bin(B::Imp, Box::new(l), Box::new(self.imp()));
        }
        l
    }
    fn or(&mut self) -> X {
        let mut l = self.and();
        while self.eat("||") {
            l = X::Bin(B::Or, Box::new(l), Box::new(self.and()));
        }
        l
    }
    fn and(&mut self) -> X {
        let mut l = self.cmp();
        while self.eat("&&") {
            l = X::Bin(B::And, Box::new(l), Box::new(self.cmp()));
        }
        l
    }
    fn cmp(&mut self) -> X {
        let l = self.add();
        for (s, op) in [
            ("==", B::Eq),
            ("!=", B::Ne),
            ("<=", B::Le),
            (">=", B::Ge),
            ("<", B::Lt),
            (">", B::Gt),
        ] {
            if self.eat(s) {
                return X::Bin(op, Box::new(l), Box::new(self.add()));
            }
        }
        l
    }
    fn add(&mut self) -> X {
        let mut l = self.mul();
        loop {
            if self.eat("+") {
                l = X::Bin(B::Add, Box::new(l), Box::new(self.mul()));
            } else if self.eat("-") {
                l = X::Bin(B::Sub, Box::new(l), Box::new(self.mul()));
            } else {
                return l;
            }
        }
    }
    fn mul(&mut self) -> X {
        let mut l = self.unary();
        while self.eat("*") {
            l = X::Bin(B::Mul, Box::new(l), Box::new(self.unary()));
        }
        l
    }
    fn unary(&mut self) -> X {
        if self.eat("-") {
            match self.toks.get(self.at).cloned() {
                Some(Tok::Int(v)) => {
                    self.at += 1;
                    return X::Int(-v);
                }
                t => panic!("negation of {t:?}"),
            }
        }
        if self.eat("#[trigger]") {
            return X::Trig(Box::new(self.unary()));
        }
        let mut e = self.primary();
        loop {
            if self.eat(".") {
                let w = self.id();
                assert_eq!(w, "len");
                self.expect("(");
                self.expect(")");
                let X::Var(a) = e else {
                    panic!("len of non-variable")
                };
                e = X::Len(a);
            } else if self.eat("[") {
                let i = self.imp();
                self.expect("]");
                let X::Var(a) = e else {
                    panic!("index of non-variable")
                };
                e = X::Idx(a, Box::new(i));
            } else {
                return e;
            }
        }
    }
    fn primary(&mut self) -> X {
        if let Some(Tok::Int(v)) = self.peek().cloned() {
            self.at += 1;
            return X::Int(v);
        }
        if self.eat("(") {
            let e = self.imp();
            self.expect(")");
            return e;
        }
        if self.is_id("if") {
            self.at += 1;
            let c = self.imp();
            self.expect("{");
            let t = self.imp();
            self.expect("}");
            assert!(self.is_id("else"));
            self.at += 1;
            self.expect("{");
            let e = self.imp();
            self.expect("}");
            return X::If(Box::new(c), Box::new(t), Box::new(e));
        }
        if self.is_id("forall") {
            self.at += 1;
            self.expect("|");
            let mut vs = Vec::new();
            loop {
                vs.push(self.id());
                self.expect(":");
                assert_eq!(self.id(), "usize");
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("|");
            return X::Forall(vs, Box::new(self.imp()));
        }
        let name = self.id();
        if self.eat("(") {
            let mut args = Vec::new();
            if !self.eat(")") {
                loop {
                    args.push(self.imp());
                    if self.eat(")") {
                        break;
                    }
                    self.expect(",");
                }
            }
            return X::Call(name, args);
        }
        X::Var(name)
    }
}

const SYM_NAMES: &[&str] = &[
    "#[trigger]",
    "==>",
    "&&",
    "||",
    "==",
    "!=",
    "<=",
    ">=",
    "<",
    ">",
    "+",
    "-",
    "*",
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ",",
    "|",
    ":",
    ".",
];

fn parse(s: &str, prefix: &str) -> X {
    let mut p = Parser {
        toks: lex(s, prefix),
        at: 0,
    };
    let e = p.imp();
    assert_eq!(p.at, p.toks.len(), "trailing tokens in {s}");
    e
}

// ---------------------------------------------------------------------------
// Spec fns and their evaluation.

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    Arr,
    Usize,
    I64,
    Bool,
}

impl Ty {
    fn text(self) -> &'static str {
        match self {
            Ty::Arr => "&[i64]",
            Ty::Usize => "usize",
            Ty::I64 => "i64",
            Ty::Bool => "bool",
        }
    }
    fn of(s: &str) -> Ty {
        match s {
            "&[i64]" => Ty::Arr,
            "usize" => Ty::Usize,
            "i64" => Ty::I64,
            "bool" => Ty::Bool,
            _ => panic!("type {s}"),
        }
    }
}

#[derive(Clone)]
struct SpecFn {
    name: String,
    params: Vec<(String, Ty)>,
    ret: Ty,
    decreases: Option<String>,
    body: X,
}

impl SpecFn {
    fn text(&self) -> String {
        let ps = self
            .params
            .iter()
            .map(|(n, t)| format!("{n}: {}", t.text()))
            .collect::<Vec<_>>()
            .join(", ");
        match &self.decreases {
            Some(d) => format!(
                "    spec fn {}({ps}) -> {}\n        decreases {d},\n    {{\n        {}\n    }}\n\n",
                self.name,
                self.ret.text(),
                self.body.text()
            ),
            None => format!(
                "    spec fn {}({ps}) -> {} {{ {} }}\n\n",
                self.name,
                self.ret.text(),
                self.body.text()
            ),
        }
    }
}

/// The library each function draws from, under its own prefix.
const LIB: &[(&str, &str, &str, Option<&str>, &str)] = &[
    (
        "inr",
        "a: &[i64], lo: i64, hi: i64",
        "bool",
        None,
        "forall|j: usize| j < a.len() ==> (lo <= #[trigger] a[j] && a[j] <= hi)",
    ),
    (
        "sorted",
        "a: &[i64]",
        "bool",
        None,
        "forall|j: usize, k: usize| j < k && k < a.len() ==> #[trigger] a[j] <= #[trigger] a[k]",
    ),
    (
        "sum",
        "a: &[i64], n: usize",
        "i64",
        Some("n"),
        "if n == 0 || n > a.len() { 0 } else { $sum(a, n - 1) + a[n - 1] }",
    ),
    (
        "cnt",
        "a: &[i64], n: usize, v: i64",
        "i64",
        Some("n"),
        "if n == 0 || n > a.len() { 0 } else { $cnt(a, n - 1, v) + (if a[n - 1] == v { 1 } else { 0 }) }",
    ),
    (
        "same",
        "a: &[i64], b: &[i64], i: usize",
        "bool",
        None,
        "a.len() == b.len() && (forall|j: usize| j < a.len() && j != i ==> #[trigger] a[j] == b[j])",
    ),
    (
        "filled",
        "a: &[i64], v: i64, n: usize",
        "bool",
        None,
        "forall|j: usize| j < a.len() && j < n ==> #[trigger] a[j] == v",
    ),
    // Quantifier-free and not recursive: ground instances are complete, so
    // the ADR-0023 retry can confirm counterexamples.
    (
        "btw",
        "v: i64, lo: i64, hi: i64",
        "bool",
        None,
        "lo <= v && v <= hi",
    ),
    (
        "at",
        "a: &[i64], n: usize",
        "i64",
        None,
        "if n < a.len() { a[n] } else { 0 }",
    ),
    (
        "max2",
        "p: i64, q: i64",
        "i64",
        None,
        "if p >= q { p } else { q }",
    ),
];

fn lib_fn(prefix: &str, k: usize) -> SpecFn {
    let (name, params, ret, dec, body) = LIB[k];
    SpecFn {
        name: format!("{prefix}_{name}"),
        params: params
            .split(", ")
            .map(|p| {
                let (n, t) = p.split_once(": ").unwrap();
                (n.to_string(), Ty::of(t))
            })
            .collect(),
        ret: Ty::of(ret),
        decreases: dec.map(str::to_string),
        body: parse(body, prefix),
    }
}

#[derive(Clone, Debug, PartialEq)]
enum V {
    I(i128),
    B(bool),
    A(Vec<i64>),
}

/// The value is not determined: an out-of-range read, a spec fn argument
/// outside its parameter's type, or recursion that does not end.
#[derive(Debug, PartialEq)]
struct Undef;

type Env = Vec<(String, V)>;

fn kleene_and(l: Result<bool, Undef>, r: Result<bool, Undef>) -> Result<bool, Undef> {
    match (l, r) {
        (Ok(false), _) | (_, Ok(false)) => Ok(false),
        (Ok(true), Ok(true)) => Ok(true),
        _ => Err(Undef),
    }
}

fn kleene_not(v: Result<bool, Undef>) -> Result<bool, Undef> {
    v.map(|b| !b)
}

struct Ev<'s> {
    lib: &'s [SpecFn],
}

impl Ev<'_> {
    fn int(&self, x: &X, env: &mut Env, depth: usize) -> Result<i128, Undef> {
        match self.ev(x, env, depth)? {
            V::I(v) => Ok(v),
            v => panic!("expected an integer from {}, got {v:?}", x.text()),
        }
    }

    fn bool(&self, x: &X, env: &mut Env, depth: usize) -> Result<bool, Undef> {
        match self.ev(x, env, depth)? {
            V::B(v) => Ok(v),
            v => panic!("expected a bool from {}, got {v:?}", x.text()),
        }
    }

    fn arr<'e>(&self, a: &str, env: &'e Env) -> &'e [i64] {
        match env.iter().rev().find(|(n, _)| n == a) {
            Some((_, V::A(v))) => v,
            _ => panic!("no array {a}"),
        }
    }

    fn ev(&self, x: &X, env: &mut Env, depth: usize) -> Result<V, Undef> {
        Ok(match x {
            X::Int(v) => V::I(*v),
            X::Var(n) => env
                .iter()
                .rev()
                .find(|(m, _)| m == n)
                .unwrap_or_else(|| panic!("unbound {n}"))
                .1
                .clone(),
            X::Len(a) => V::I(self.arr(a, env).len() as i128),
            X::Idx(a, e) => {
                let i = self.int(e, env, depth)?;
                let arr = self.arr(a, env);
                if i < 0 || i >= arr.len() as i128 {
                    return Err(Undef);
                }
                V::I(arr[i as usize] as i128)
            }
            X::Trig(e) => self.ev(e, env, depth)?,
            X::Bin(op, l, r) => match op {
                B::And => V::B(kleene_and(
                    self.bool(l, env, depth),
                    self.bool(r, env, depth),
                )?),
                B::Or => V::B(kleene_not(kleene_and(
                    kleene_not(self.bool(l, env, depth)),
                    kleene_not(self.bool(r, env, depth)),
                ))?),
                B::Imp => V::B(kleene_not(kleene_and(
                    self.bool(l, env, depth),
                    kleene_not(self.bool(r, env, depth)),
                ))?),
                _ => {
                    let (a, b) = (self.int(l, env, depth)?, self.int(r, env, depth)?);
                    match op {
                        B::Add => V::I(a.checked_add(b).ok_or(Undef)?),
                        B::Sub => V::I(a.checked_sub(b).ok_or(Undef)?),
                        B::Mul => V::I(a.checked_mul(b).ok_or(Undef)?),
                        B::Lt => V::B(a < b),
                        B::Le => V::B(a <= b),
                        B::Gt => V::B(a > b),
                        B::Ge => V::B(a >= b),
                        B::Eq => V::B(a == b),
                        B::Ne => V::B(a != b),
                        _ => unreachable!(),
                    }
                }
            },
            X::If(c, t, e) => match self.bool(c, env, depth) {
                Ok(true) => self.ev(t, env, depth)?,
                Ok(false) => self.ev(e, env, depth)?,
                Err(Undef) => {
                    let (t, e) = (self.ev(t, env, depth)?, self.ev(e, env, depth)?);
                    if t != e {
                        return Err(Undef);
                    }
                    t
                }
            },
            X::Call(f, args) => {
                let sf = self
                    .lib
                    .iter()
                    .find(|s| &s.name == f)
                    .unwrap_or_else(|| panic!("no spec fn {f}"));
                if depth > 64 {
                    return Err(Undef);
                }
                let mut inner = Env::new();
                for ((p, ty), a) in sf.params.iter().zip(args) {
                    let v = self.ev(a, env, depth)?;
                    let ok = match (&v, ty) {
                        (V::I(n), Ty::Usize) => 0 <= *n && *n <= u64::MAX as i128,
                        (V::I(n), Ty::I64) => i64::MIN as i128 <= *n && *n <= i64::MAX as i128,
                        _ => true,
                    };
                    if !ok {
                        return Err(Undef);
                    }
                    inner.push((p.clone(), v));
                }
                self.ev(&sf.body, &mut inner, depth + 1)?
            }
            X::Forall(vs, body) => {
                // Every generated guard bounds each variable by an array
                // length; mutations move a bound by at most one.
                let n = env
                    .iter()
                    .filter_map(|(_, v)| match v {
                        V::A(a) => Some(a.len()),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0)
                    + 2;
                let mut acc = Ok(true);
                let mut idx = vec![0usize; vs.len()];
                'all: loop {
                    for (v, &j) in vs.iter().zip(&idx) {
                        env.push((v.clone(), V::I(j as i128)));
                    }
                    let r = self.bool(body, env, depth);
                    env.truncate(env.len() - vs.len());
                    acc = kleene_and(acc, r);
                    if acc == Ok(false) {
                        break;
                    }
                    for p in (0..idx.len()).rev() {
                        idx[p] += 1;
                        if idx[p] < n {
                            continue 'all;
                        }
                        idx[p] = 0;
                    }
                    break;
                }
                V::B(acc?)
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Executable parts.

#[derive(Clone, Copy, PartialEq)]
enum Base {
    I,
    K,
    /// `a.len() - 1`
    Last,
}

#[derive(Clone, Copy)]
struct Ix {
    base: Base,
    off: i64,
}

fn ix(base: Base, off: i64) -> Ix {
    Ix { base, off }
}

#[derive(Clone)]
enum Ve {
    X,
    S,
    T,
    Lit(i64),
    El(Ix),
    Add(Box<Ve>, Box<Ve>),
    Sub(Box<Ve>, Box<Ve>),
}

#[derive(Clone, Copy)]
enum Cmp {
    Eq,
    Lt,
}

#[derive(Clone)]
enum St {
    SetS(Ve),
    SetT(Ve),
    Write(Ix, Ve),
    /// `if l OP r { s = v; }`
    IfS(Cmp, Ve, Ve, Ve),
}

fn add(a: Ve, b: Ve) -> Ve {
    Ve::Add(Box::new(a), Box::new(b))
}

fn sub(a: Ve, b: Ve) -> Ve {
    Ve::Sub(Box::new(a), Box::new(b))
}

fn ix_text(i: Ix) -> String {
    let b = match i.base {
        Base::I => "i",
        Base::K => "k",
        Base::Last => "(a.len() - 1)",
    };
    match i.off {
        0 => b.to_string(),
        o if o > 0 => format!("({b} + {o})"),
        o => format!("({b} - {})", -o),
    }
}

fn ve_text(e: &Ve) -> String {
    match e {
        Ve::X => "x".into(),
        Ve::S => "s".into(),
        Ve::T => "t".into(),
        Ve::Lit(v) => lit(*v),
        Ve::El(i) => format!("a[{}]", ix_text(*i)),
        Ve::Add(a, b) => format!("({} + {})", ve_text(a), ve_text(b)),
        Ve::Sub(a, b) => format!("({} - {})", ve_text(a), ve_text(b)),
    }
}

fn st_text(s: &St, indent: &str) -> String {
    match s {
        St::SetS(e) => format!("{indent}s = {};\n", ve_text(e)),
        St::SetT(e) => format!("{indent}t = {};\n", ve_text(e)),
        St::Write(i, e) => format!("{indent}a[{}] = {};\n", ix_text(*i), ve_text(e)),
        St::IfS(c, l, r, v) => format!(
            "{indent}if {} {} {} {{\n{indent}    s = {};\n{indent}}}\n",
            ve_text(l),
            match c {
                Cmp::Eq => "==",
                Cmp::Lt => "<",
            },
            ve_text(r),
            ve_text(v)
        ),
    }
}

/// Mutation sites of executable code: index offsets, `+`/`-`, literals.
fn ve_mutate(e: &mut Ve, n: &mut usize) -> bool {
    let here = match e {
        Ve::El(i) => {
            if *n == 0 {
                i.off += if i.off > 0 { -1 } else { 1 };
                return true;
            }
            *n -= 1;
            return false;
        }
        Ve::Lit(v) => {
            if *n == 0 {
                *v = v.saturating_add(1);
                return true;
            }
            *n -= 1;
            return false;
        }
        Ve::X | Ve::S | Ve::T => return false,
        Ve::Add(..) | Ve::Sub(..) => *n == 0,
    };
    if here {
        let old = std::mem::replace(e, Ve::X);
        *e = match old {
            Ve::Add(a, b) => Ve::Sub(a, b),
            Ve::Sub(a, b) => Ve::Add(a, b),
            _ => unreachable!(),
        };
        return true;
    }
    *n -= 1;
    match e {
        Ve::Add(a, b) | Ve::Sub(a, b) => ve_mutate(a, n) || ve_mutate(b, n),
        _ => unreachable!(),
    }
}

fn st_mutate(s: &mut St, n: &mut usize) -> bool {
    match s {
        St::SetS(e) | St::SetT(e) => ve_mutate(e, n),
        St::Write(i, e) => {
            if *n == 0 {
                i.off += if i.off > 0 { -1 } else { 1 };
                return true;
            }
            *n -= 1;
            ve_mutate(e, n)
        }
        St::IfS(_, l, r, v) => ve_mutate(l, n) || ve_mutate(r, n) || ve_mutate(v, n),
    }
}

#[derive(Debug)]
struct Fault;

// ---------------------------------------------------------------------------

struct Func {
    name: String,
    template: &'static str,
    mutable: bool,
    lib: Vec<SpecFn>,
    pre: Vec<X>,
    post: Vec<X>,
    inv: Vec<X>,
    before: Vec<St>,
    body: Option<Vec<St>>,
    ret: Ve,
    /// Spec fns left out of `#[uses]`.
    omitted: Option<String>,
    /// A lemma the function calls in a `proof` block (ADR-0062): then
    /// `pre` is the caller's precondition and `post` the lemma's
    /// conclusion, and the function returns 0.
    lemma: Option<Lemma>,
}

/// `proof fn gN_lemma(a: &[i64], i: usize, x: i64) requires gN_lpre(..),
/// ensures gN_post(a, i, x, 0)`, an induction on `i` or a plain proof.
struct Lemma {
    pre: Vec<X>,
    /// The argument of the recursive call (`i - 1`), or no call.
    step: Option<&'static str>,
    /// The call is under `if i > 0`.
    guarded: bool,
    /// Whether the lemma is an induction (its correct form recurses).
    recursive: bool,
}

/// Lemma templates (name, pre, post, induction on `i`), correct by
/// construction.
const LEMMAS: &[(&str, &[&str], &[&str], bool)] = &[
    ("cnt_le", &["i <= a.len()"], &["$cnt(a, i, x) <= i"], true),
    (
        "cnt_nonneg",
        &["a.len() <= 1000000"],
        &["$cnt(a, i, x) >= 0"],
        true,
    ),
    (
        "sum_bound",
        &["i <= a.len()", "$inr(a, -1000, 1000)"],
        &["-1000 * i <= $sum(a, i) && $sum(a, i) <= 1000 * i"],
        true,
    ),
    (
        "filled_cnt",
        &["i <= a.len()", "$filled(a, x, a.len())"],
        &["$cnt(a, i, x) == i"],
        true,
    ),
    (
        "sum_nonneg",
        &["i <= a.len()", "$inr(a, 0, 1000)"],
        &["$sum(a, i) >= 0"],
        true,
    ),
    (
        "sorted_first",
        &["i < a.len()", "$sorted(a)"],
        &["a[0] <= a[i]"],
        false,
    ),
];

/// Extra lemma conclusions a mutation may add: some hold, most do not.
const LEMMA_EXTRA: &[&str] = &[
    "$cnt(a, i, x) < i",
    "$sum(a, i) >= 0",
    "$cnt(a, i, x) == i",
    "i <= a.len()",
    "$sum(a, i) <= 1000 * i",
];

struct Tpl {
    name: &'static str,
    mutable: bool,
    pre: &'static [&'static str],
    post: &'static [&'static str],
    inv: &'static [&'static str],
}

const TEMPLATES: &[Tpl] = &[
    Tpl {
        name: "read_bound",
        mutable: false,
        pre: &[
            "i < a.len()",
            "$inr(a, -1000, 1000)",
            "-1000 <= x && x <= 1000",
        ],
        post: &["-2000 <= r && r <= 2000"],
        inv: &[],
    },
    Tpl {
        name: "sorted_gap",
        mutable: false,
        pre: &["i + 1 < a.len()", "$sorted(a)", "$inr(a, -1000, 1000)"],
        post: &["r >= 0"],
        inv: &[],
    },
    Tpl {
        name: "write",
        mutable: true,
        pre: &["i < a.len()"],
        post: &["a[i] == x", "$same(a, b, i)", "r == x"],
        inv: &[],
    },
    Tpl {
        name: "sum_step",
        mutable: false,
        pre: &["i < a.len()"],
        post: &["r == $sum(a, i + 1) - $sum(a, i)"],
        inv: &[],
    },
    Tpl {
        name: "cnt_step",
        mutable: false,
        pre: &["i < a.len()"],
        post: &["r == $cnt(a, i + 1, x) - $cnt(a, i, x)"],
        inv: &[],
    },
    Tpl {
        name: "implication",
        mutable: false,
        pre: &["i < a.len()", "$inr(a, 0, 1000)", "-1000 <= x && x <= 1000"],
        post: &["x > 0 ==> r > 0", "x <= 0 ==> r <= a[i]"],
        inv: &[],
    },
    Tpl {
        name: "swap",
        mutable: true,
        pre: &["i + 1 < a.len()"],
        post: &["a[i] == b[i + 1] && a[i + 1] == b[i]", "a.len() == b.len()"],
        inv: &[],
    },
    Tpl {
        name: "write_in_range",
        mutable: true,
        pre: &[
            "i < a.len()",
            "$inr(a, -1000, 1000)",
            "-1000 <= x && x <= 1000",
        ],
        post: &["$inr(a, -1000, 1000)", "-1000 <= r && r <= 1000"],
        inv: &[],
    },
    Tpl {
        name: "sorted_last",
        mutable: false,
        pre: &["a.len() > 0", "$sorted(a)"],
        post: &["forall|j: usize| j < a.len() ==> #[trigger] a[j] <= r"],
        inv: &[],
    },
    Tpl {
        name: "clamp",
        mutable: false,
        pre: &["i < a.len()"],
        post: &["r == $max2($at(a, i), x)", "r >= x"],
        inv: &[],
    },
    Tpl {
        name: "write_succ",
        mutable: true,
        pre: &["i < a.len()", "$btw(x, -1000, 1000)"],
        post: &["$at(a, i) == x + 1", "$btw(r, -999, 1001)"],
        inv: &[],
    },
    Tpl {
        name: "sum_loop",
        mutable: false,
        pre: &["$inr(a, -1000, 1000)", "a.len() <= 1000000"],
        post: &["r == $sum(a, a.len())"],
        inv: &[
            "k <= a.len()",
            "s == $sum(a, k)",
            "-1000 * k <= s && s <= 1000 * k",
        ],
    },
    Tpl {
        name: "cnt_loop",
        mutable: false,
        pre: &["-1000 <= x && x <= 1000"],
        post: &["r == $cnt(a, a.len(), x)", "0 <= r && r <= a.len()"],
        inv: &["k <= a.len()", "s == $cnt(a, k, x)", "0 <= s && s <= k"],
    },
    Tpl {
        name: "fill_loop",
        mutable: true,
        pre: &["-1000 <= x && x <= 1000"],
        post: &["$filled(a, x, a.len())", "r == x"],
        inv: &["k <= a.len()", "$filled(a, x, k)"],
    },
];

fn template_code(name: &str) -> (Vec<St>, Option<Vec<St>>, Ve) {
    let el = |b, o| Ve::El(ix(b, o));
    match name {
        "read_bound" | "implication" => (vec![], None, add(el(Base::I, 0), Ve::X)),
        "sorted_gap" => (vec![], None, sub(el(Base::I, 1), el(Base::I, 0))),
        "write" => (vec![St::Write(ix(Base::I, 0), Ve::X)], None, Ve::X),
        "sum_step" => (vec![], None, el(Base::I, 0)),
        "cnt_step" => (
            vec![St::IfS(Cmp::Eq, el(Base::I, 0), Ve::X, Ve::Lit(1))],
            None,
            Ve::S,
        ),
        "swap" => (
            vec![
                St::SetT(el(Base::I, 0)),
                St::Write(ix(Base::I, 0), el(Base::I, 1)),
                St::Write(ix(Base::I, 1), Ve::T),
            ],
            None,
            Ve::Lit(0),
        ),
        "write_in_range" => (vec![St::Write(ix(Base::I, 0), Ve::X)], None, el(Base::I, 0)),
        "sorted_last" => (vec![], None, el(Base::Last, 0)),
        "clamp" => (
            vec![
                St::SetS(el(Base::I, 0)),
                St::IfS(Cmp::Lt, Ve::S, Ve::X, Ve::X),
            ],
            None,
            Ve::S,
        ),
        "write_succ" => (
            vec![St::Write(ix(Base::I, 0), add(Ve::X, Ve::Lit(1)))],
            None,
            el(Base::I, 0),
        ),
        "sum_loop" => (
            vec![],
            Some(vec![St::SetS(add(Ve::S, el(Base::K, 0)))]),
            Ve::S,
        ),
        "cnt_loop" => (
            vec![],
            Some(vec![St::IfS(
                Cmp::Eq,
                el(Base::K, 0),
                Ve::X,
                add(Ve::S, Ve::Lit(1)),
            )]),
            Ve::S,
        ),
        "fill_loop" => (vec![], Some(vec![St::Write(ix(Base::K, 0), Ve::X)]), Ve::X),
        _ => unreachable!(),
    }
}

/// Extra post clauses a mutation may add: some hold, most do not.
const EXTRA_POST: &[&str] = &[
    "r >= 0",
    "$sorted(a)",
    "r == $sum(a, a.len())",
    "$inr(a, -1000, 1000)",
    "x > 0 ==> r > x",
    "r == x",
    "$btw(r, -1000, 1000)",
    "r == $at(a, 0)",
    "forall|j: usize, k: usize| j < a.len() && k < a.len() ==> #[trigger] a[j] + r >= #[trigger] a[k]",
];

/// A function whose postcondition is a lemma's conclusion, proved by a
/// `proof` block that calls the lemma; the lemma and the call may carry a
/// near-miss mutation (a dropped pre clause of either, a changed
/// conclusion, a recursive call that does not decrease, is not guarded or
/// is missing, a changed spec fn).
fn gen_lemma_func(rng: &mut Rng, n: usize) -> Func {
    let name = format!("g{n}");
    let (tname, tpre, tpost, recursive) = LEMMAS[rng.below(LEMMAS.len())];
    let p = |s: &str| parse(s, &name);
    let mut lpre: Vec<X> = tpre.iter().map(|s| p(s)).collect();
    let mut pre = lpre.clone();
    let mut post: Vec<X> = tpost.iter().map(|s| p(s)).collect();
    let mut lemma = Lemma {
        pre: Vec::new(),
        step: recursive.then_some("i - 1"),
        guarded: true,
        recursive,
    };
    if rng.chance(50) {
        let short = p("a.len() <= 4");
        pre.push(short.clone());
        lpre.push(short);
    }
    let mut lib: Vec<SpecFn> = (0..LIB.len()).map(|k| lib_fn(&name, k)).collect();
    if rng.chance(55) {
        match rng.below(6) {
            0 if lpre.len() > 1 => {
                lpre.remove(rng.below(lpre.len()));
            }
            0 | 1 if pre.len() > 1 => {
                pre.remove(rng.below(pre.len()));
            }
            2 => {
                mutate_some(&mut post, rng);
            }
            3 if recursive => match rng.below(3) {
                0 => lemma.step = Some("i"),
                1 => lemma.guarded = false,
                _ => lemma.step = None,
            },
            4 => {
                let used = reached(&lpre, &post, &[], &lib);
                let pool: Vec<usize> = (0..lib.len())
                    .filter(|&k| used.contains(&lib[k].name))
                    .collect();
                if !pool.is_empty() {
                    let k = pool[rng.below(pool.len())];
                    mutate_some(std::slice::from_mut(&mut lib[k].body), rng);
                }
            }
            _ => post.push(p(LEMMA_EXTRA[rng.below(LEMMA_EXTRA.len())])),
        }
    }
    let used = reached(&pre, &post, &lpre, &lib);
    lib.retain(|s| used.contains(&s.name));
    lemma.pre = lpre;
    let mut f = Func {
        name,
        template: tname,
        mutable: false,
        lib,
        pre,
        post,
        inv: Vec::new(),
        before: Vec::new(),
        body: None,
        ret: Ve::Lit(0),
        omitted: None,
        lemma: Some(lemma),
    };
    if rng.chance(25) {
        let names = f.uses_all();
        f.omitted = Some(names[rng.below(names.len())].clone());
    }
    f
}

fn gen_func(rng: &mut Rng, n: usize) -> Func {
    let name = format!("g{n}");
    let tpl = &TEMPLATES[rng.below(TEMPLATES.len())];
    let p = |s: &str| parse(s, &name);
    let mut pre: Vec<X> = tpl.pre.iter().map(|s| p(s)).collect();
    let mut post: Vec<X> = tpl.post.iter().map(|s| p(s)).collect();
    let mut inv: Vec<X> = tpl.inv.iter().map(|s| p(s)).collect();
    let (mut before, mut body, mut ret) = template_code(tpl.name);
    // Short arrays make counterexamples readable in full.
    if rng.chance(50) {
        pre.push(p("a.len() <= 4"));
    }
    let mut lib: Vec<SpecFn> = (0..LIB.len()).map(|k| lib_fn(&name, k)).collect();
    if rng.chance(55) {
        match rng.below(7) {
            0 => {
                pre.remove(rng.below(pre.len()));
                if pre.is_empty() {
                    pre.push(p("a.len() <= 1000000"));
                }
            }
            1 if !inv.is_empty() => {
                inv.remove(rng.below(inv.len()));
                if inv.is_empty() {
                    inv.push(p("k <= a.len()"));
                }
            }
            1 | 2 => {
                let mut stmts: Vec<&mut St> = before.iter_mut().collect();
                if let Some(b) = body.as_mut() {
                    stmts.extend(b.iter_mut());
                }
                let mut total = 0;
                for s in &stmts {
                    total += count_st(s);
                }
                total += count_ve(&ret);
                let mut k = rng.below(total.max(1));
                let done = stmts.into_iter().any(|s| st_mutate(s, &mut k));
                if !done {
                    ve_mutate(&mut ret, &mut k);
                }
            }
            3 => {
                mutate_some(&mut post, rng);
            }
            4 => {
                // A spec fn the contracts reach.
                let used = reached(&pre, &post, &inv, &lib);
                let pool: Vec<usize> = (0..lib.len())
                    .filter(|&k| used.contains(&lib[k].name))
                    .collect();
                if pool.is_empty() {
                    mutate_some(&mut pre, rng);
                } else {
                    let k = pool[rng.below(pool.len())];
                    mutate_some(std::slice::from_mut(&mut lib[k].body), rng);
                }
            }
            5 => {
                mutate_some(&mut pre, rng);
            }
            _ => {
                let extra = EXTRA_POST[rng.below(EXTRA_POST.len())];
                post.push(p(extra));
            }
        }
    }
    let used = reached(&pre, &post, &inv, &lib);
    lib.retain(|s| used.contains(&s.name));
    let mut f = Func {
        name,
        template: tpl.name,
        mutable: tpl.mutable,
        lib,
        pre,
        post,
        inv,
        before,
        body,
        ret,
        omitted: None,
        lemma: None,
    };
    if rng.chance(25) {
        let names = f.uses_all();
        f.omitted = Some(names[rng.below(names.len())].clone());
    }
    f
}

fn count_ve(e: &Ve) -> usize {
    match e {
        Ve::El(_) | Ve::Lit(_) => 1,
        Ve::X | Ve::S | Ve::T => 0,
        Ve::Add(a, b) | Ve::Sub(a, b) => 1 + count_ve(a) + count_ve(b),
    }
}

fn count_st(s: &St) -> usize {
    match s {
        St::SetS(e) | St::SetT(e) => count_ve(e),
        St::Write(_, e) => 1 + count_ve(e),
        St::IfS(_, l, r, v) => count_ve(l) + count_ve(r) + count_ve(v),
    }
}

/// Library spec fns the clauses reach, transitively.
fn reached(pre: &[X], post: &[X], inv: &[X], lib: &[SpecFn]) -> Vec<String> {
    let mut out = Vec::new();
    for x in pre.iter().chain(post).chain(inv) {
        x.calls(&mut out);
    }
    let mut k = 0;
    while k < out.len() {
        if let Some(s) = lib.iter().find(|s| s.name == out[k]) {
            let mut more = Vec::new();
            s.body.calls(&mut more);
            for m in more {
                if !out.contains(&m) {
                    out.push(m);
                }
            }
        }
        k += 1;
    }
    out.sort();
    out.dedup();
    out
}

fn conj(cs: &[X]) -> String {
    cs.iter().map(X::text).collect::<Vec<_>>().join(" && ")
}

/// Values of the scalar inputs and the state a clause sees.
struct View<'s> {
    a: &'s [i64],
    b: &'s [i64],
    i: u64,
    x: i64,
    r: i64,
    k: u64,
    s: i64,
}

impl Func {
    fn uses_all(&self) -> Vec<String> {
        let n = &self.name;
        let mut u = vec![format!("{n}_pre"), format!("{n}_post")];
        if self.body.is_some() {
            u.push(format!("{n}_inv"));
        }
        if self.lemma.is_some() {
            u.push(format!("{n}_lpre"));
        }
        u.extend(self.lib.iter().map(|s| s.name.clone()));
        u
    }

    fn text(&self) -> String {
        let n = &self.name;
        let b = if self.mutable { "b: &[i64], " } else { "" };
        let ob = if self.mutable { "old(a), " } else { "" };
        let mut t = String::new();
        let _ = writeln!(
            t,
            "    spec fn {n}_pre(a: &[i64], i: usize, x: i64) -> bool {{ {} }}\n",
            conj(&self.pre)
        );
        let _ = writeln!(
            t,
            "    spec fn {n}_post(a: &[i64], {b}i: usize, x: i64, r: i64) -> bool {{ {} }}\n",
            conj(&self.post)
        );
        if self.body.is_some() {
            let _ = writeln!(
                t,
                "    spec fn {n}_inv(a: &[i64], {b}i: usize, x: i64, k: usize, s: i64) -> bool {{ {} }}\n",
                conj(&self.inv)
            );
        }
        for s in &self.lib {
            t.push_str(&s.text());
        }
        let uses: Vec<String> = self
            .uses_all()
            .into_iter()
            .filter(|u| Some(u) != self.omitted.as_ref())
            .collect();
        if let Some(l) = &self.lemma {
            let _ = writeln!(
                t,
                "    spec fn {n}_lpre(a: &[i64], i: usize, x: i64) -> bool {{ {} }}\n",
                conj(&l.pre)
            );
            let dec = if l.recursive {
                "        decreases i,\n"
            } else {
                ""
            };
            let call = l.step.map(|s| format!("{n}_lemma(a, {s}, x);"));
            let body = match (call, l.guarded) {
                (None, _) => String::new(),
                (Some(c), true) => format!("        if i > 0 {{\n            {c}\n        }}\n"),
                (Some(c), false) => format!("        {c}\n"),
            };
            let _ = writeln!(
                t,
                "    #[uses({})]\n    proof fn {n}_lemma(a: &[i64], i: usize, x: i64)\n        requires {n}_lpre(a, i, x),\n        ensures {n}_post(a, i, x, 0),\n{dec}    {{\n{body}    }}\n",
                uses.join(", ")
            );
            let _ = writeln!(
                t,
                "    #[effects(pure)]\n    #[uses({})]\n    fn {n}(a: &[i64], i: usize, x: i64) -> (r: i64)\n        requires {n}_pre(a, i, x),\n        ensures {n}_post(a, i, x, r),\n    {{\n        proof {{\n            {n}_lemma(a, i, x);\n        }}\n        0\n    }}\n",
                uses.join(", ")
            );
            return t;
        }
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({})]\n    fn {n}(a: {}, i: usize, x: i64) -> (r: i64)\n        requires {n}_pre(a, i, x),\n        ensures {n}_post(a, {ob}i, x, r),\n    {{",
            uses.join(", "),
            if self.mutable { "&mut [i64]" } else { "&[i64]" }
        );
        t.push_str("        let mut s: i64 = 0;\n        let mut t: i64 = 0;\n        let mut k: usize = 0;\n");
        for s in &self.before {
            t.push_str(&st_text(s, "        "));
        }
        if let Some(body) = &self.body {
            let _ = writeln!(
                t,
                "        while k < a.len()\n            invariant {n}_inv(a, {ob}i, x, k, s),\n            decreases a.len() - k,\n        {{"
            );
            for s in body {
                t.push_str(&st_text(s, "            "));
            }
            t.push_str("            k = k + 1;\n        }\n");
        }
        let _ = writeln!(t, "        {}\n    }}\n", ve_text(&self.ret));
        t
    }

    /// Three-valued: `Err` when the clauses' value is not determined.
    fn holds(&self, cs: &[X], v: &View) -> Result<bool, Undef> {
        let mut env: Env = vec![
            ("a".into(), V::A(v.a.to_vec())),
            ("b".into(), V::A(v.b.to_vec())),
            ("i".into(), V::I(v.i as i128)),
            ("x".into(), V::I(v.x as i128)),
            ("r".into(), V::I(v.r as i128)),
            ("k".into(), V::I(v.k as i128)),
            ("s".into(), V::I(v.s as i128)),
        ];
        let ev = Ev { lib: &self.lib };
        // Not `try_fold`: a later false clause decides even after an
        // undetermined one.
        let mut acc = Ok(true);
        for c in cs {
            acc = kleene_and(acc, ev.bool(c, &mut env, 0));
        }
        acc
    }

    /// The lemma's own precondition.
    fn lemma_pre_holds(&self, a: &[i64], i: u64, x: i64) -> Result<bool, Undef> {
        let l = self.lemma.as_ref().expect("a lemma function");
        self.holds(
            &l.pre,
            &View {
                a,
                b: a,
                i,
                x,
                r: 0,
                k: 0,
                s: 0,
            },
        )
    }

    fn pre_holds(&self, a: &[i64], i: u64, x: i64) -> Result<bool, Undef> {
        self.holds(
            &self.pre,
            &View {
                a,
                b: a,
                i,
                x,
                r: 0,
                k: 0,
                s: 0,
            },
        )
    }

    fn post_holds(&self, a: &[i64], old: &[i64], i: u64, x: i64, r: i64) -> Result<bool, Undef> {
        self.holds(
            &self.post,
            &View {
                a,
                b: old,
                i,
                x,
                r,
                k: 0,
                s: 0,
            },
        )
    }
}

/// What a run did: the result (array after it included) or a fault, whether
/// the invariant was found false at some loop head, and its value on entry.
struct Run {
    result: Result<(i64, Vec<i64>), Fault>,
    inv_broken: bool,
    entry_inv: Option<Result<bool, Undef>>,
}

struct Mach<'f> {
    f: &'f Func,
    a: Vec<i64>,
    i: u64,
    x: i64,
    s: i64,
    t: i64,
    k: u64,
}

impl Mach<'_> {
    fn idx(&self, i: Ix) -> Result<usize, Fault> {
        let len = self.a.len() as u64;
        let base = match i.base {
            Base::I => self.i,
            Base::K => self.k,
            Base::Last => len.checked_sub(1).ok_or(Fault)?,
        };
        let v = if i.off >= 0 {
            base.checked_add(i.off as u64).ok_or(Fault)?
        } else {
            base.checked_sub(i.off.unsigned_abs()).ok_or(Fault)?
        };
        if v < len { Ok(v as usize) } else { Err(Fault) }
    }

    fn ve(&self, e: &Ve) -> Result<i64, Fault> {
        Ok(match e {
            Ve::X => self.x,
            Ve::S => self.s,
            Ve::T => self.t,
            Ve::Lit(v) => *v,
            Ve::El(i) => self.a[self.idx(*i)?],
            Ve::Add(a, b) => self.ve(a)?.checked_add(self.ve(b)?).ok_or(Fault)?,
            Ve::Sub(a, b) => self.ve(a)?.checked_sub(self.ve(b)?).ok_or(Fault)?,
        })
    }

    fn st(&mut self, s: &St) -> Result<(), Fault> {
        match s {
            St::SetS(e) => self.s = self.ve(e)?,
            St::SetT(e) => self.t = self.ve(e)?,
            St::Write(i, e) => {
                let v = self.ve(e)?;
                let j = self.idx(*i)?;
                self.a[j] = v;
            }
            St::IfS(c, l, r, v) => {
                let (l, r) = (self.ve(l)?, self.ve(r)?);
                let take = match c {
                    Cmp::Eq => l == r,
                    Cmp::Lt => l < r,
                };
                if take {
                    self.s = self.ve(v)?;
                }
            }
        }
        Ok(())
    }
}

fn run(f: &Func, a: &[i64], i: u64, x: i64) -> Run {
    let mut m = Mach {
        f,
        a: a.to_vec(),
        i,
        x,
        s: 0,
        t: 0,
        k: 0,
    };
    let mut inv_broken = false;
    let mut entry_inv = None;
    let result = (|| -> Result<i64, Fault> {
        for s in &m.f.before {
            m.st(s)?;
        }
        if let Some(body) = &m.f.body {
            loop {
                let v = m.f.holds(
                    &m.f.inv,
                    &View {
                        a: &m.a,
                        b: a,
                        i: m.i,
                        x: m.x,
                        r: 0,
                        k: m.k,
                        s: m.s,
                    },
                );
                inv_broken |= v == Ok(false);
                entry_inv.get_or_insert(v);
                if m.k >= m.a.len() as u64 {
                    break;
                }
                for s in body {
                    m.st(s)?;
                }
                m.k += 1;
            }
        }
        m.ve(&m.f.ret)
    })();
    Run {
        result: result.map(|r| (r, m.a.clone())),
        inv_broken,
        entry_inv,
    }
}

fn small(rng: &mut Rng) -> i64 {
    pick(
        rng,
        &[
            0,
            1,
            2,
            3,
            7,
            -1,
            -2,
            -7,
            999,
            1000,
            1001,
            -1000,
            -1001,
            i64::MAX,
            i64::MIN,
        ],
    )
}

/// Inputs whose precondition the interpreter finds true.
fn inputs(f: &Func, rng: &mut Rng) -> Vec<(Vec<i64>, u64, i64)> {
    inputs_where(rng, |a, i, x| f.pre_holds(a, i, x) == Ok(true))
}

/// Inputs `ok` accepts.
fn inputs_where(rng: &mut Rng, ok: impl Fn(&[i64], u64, i64) -> bool) -> Vec<(Vec<i64>, u64, i64)> {
    let mut out = Vec::new();
    for _ in 0..300 {
        let len = rng.below(6);
        let mut a: Vec<i64> = (0..len)
            .map(|_| match rng.below(4) {
                0 => small(rng),
                1 => rng.below(7) as i64 - 3,
                _ => rng.below(2001) as i64 - 1000,
            })
            .collect();
        if rng.chance(40) {
            a.sort();
        }
        let i = rng.below(6) as u64;
        let x = if !a.is_empty() && rng.chance(40) {
            a[rng.below(a.len())]
        } else if rng.chance(50) {
            small(rng)
        } else {
            rng.below(21) as i64 - 10
        };
        if ok(&a, i, x) {
            out.push((a, i, x));
        }
        if out.len() == 6 {
            break;
        }
    }
    out
}

/// Parses `a = [1, 2] (len 2), i = 0, x = 5`. `None` when an array is shown
/// only in part.
fn parse_cex(msg: &str) -> Option<(Vec<i64>, u64, i64)> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    let rest = rest.strip_prefix("a = [")?;
    let (elems, rest) = rest.split_once("] (len ")?;
    let (len, rest) = rest.split_once("), i = ")?;
    let (i, x) = rest.split_once(", x = ")?;
    if elems.contains("..") {
        return None;
    }
    let a: Vec<i64> = if elems.is_empty() {
        Vec::new()
    } else {
        elems
            .split(", ")
            .map(|v| v.parse().ok())
            .collect::<Option<_>>()?
    };
    if a.len() != len.parse::<usize>().ok()? {
        return None;
    }
    Some((a, i.parse().ok()?, x.trim().parse().ok()?))
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    verified_runs: usize,
    /// Verified runs whose postcondition the interpreter could not decide.
    undetermined: usize,
    /// Counterexamples whose replay the interpreter could not decide.
    cex_undetermined: usize,
    cexes: usize,
    replayed: usize,
    entry_replays: usize,
    /// Counterexamples with an "only by name" hint (opaque spec fns).
    opaque_cexes: usize,
    /// Lemmas generated and verified, the runs that checked a verified
    /// lemma's conclusion, and the lemma counterexamples replayed.
    lemmas: usize,
    lemmas_verified: usize,
    lemma_runs: usize,
    lemma_replayed: usize,
    partial_cexes: usize,
    compiled_runs: usize,
    by_template: std::collections::BTreeMap<&'static str, (usize, usize)>,
    codes: std::collections::BTreeMap<String, usize>,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    // Six functions as before, then two that rest on a lemma.
    let mut funcs: Vec<Func> = (0..6).map(|n| gen_func(&mut rng, n)).collect();
    funcs.extend((6..8).map(|n| gen_lemma_func(&mut rng, n)));
    let mut src = String::from("#[summary(\"명세 건전성 퍼징\")]\nmod m {\n");
    for f in &funcs {
        src.push_str(&f.text());
    }
    // One runner per function keeps each runner's proof context small.
    src.push_str("}\n\n#[summary(\"실행\")]\nmod app {\n");
    let mut expected = String::new();
    let mut n = 0;
    for f in &funcs {
        let _ = writeln!(src, "    #[effects(io)]\n    fn run_{}() -> () {{", f.name);
        for (a, i, x) in inputs(f, &mut rng) {
            let Ok((v, after)) = run(f, &a, i, x).result else {
                continue;
            };
            if a.is_empty() {
                continue;
            }
            n += 1;
            let elems = a.iter().map(|&e| lit(e)).collect::<Vec<_>>().join(", ");
            let (m, r) = if f.mutable {
                ("let mut", "&mut ")
            } else {
                ("let", "&")
            };
            let _ = writeln!(
                src,
                "        {m} a{n}: [i64; {}] = [{elems}];\n        print_i64(m::{}({r}a{n}, {i}, {}));",
                a.len(),
                f.name,
                lit(x)
            );
            let _ = writeln!(expected, "{v}");
            if f.mutable {
                for (j, e) in after.iter().enumerate() {
                    let _ = writeln!(src, "        print_i64(a{n}[{j}]);");
                    let _ = writeln!(expected, "{e}");
                }
            }
        }
        src.push_str("    }\n\n");
    }
    src.push_str("    #[effects(io)]\n    fn main() -> () {\n");
    for f in &funcs {
        let _ = writeln!(src, "        run_{}();", f.name);
    }
    src.push_str("    }\n}\n");
    let files = vec![("s.bmb".to_string(), src.clone())];
    let out = bmb::emit_llvm_sources(&files);
    assert!(
        out.ir.is_some(),
        "seed {seed}: generated program must check\n{src}\n{}",
        out.verify.output
    );
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{}", out.verify.output);

    for line in out.verify.output.lines() {
        let Some(name) = field(line, "def").and_then(|d| d.strip_prefix("m::")) else {
            continue;
        };
        // A verified lemma's conclusion holds wherever its precondition
        // does (ADR-0062 §2).
        if let Some(f) = funcs
            .iter()
            .find(|f| name.strip_suffix("_lemma") == Some(f.name.as_str()))
        {
            stats.lemmas += 1;
            if field(line, "status") != Some("verified") {
                continue;
            }
            stats.lemmas_verified += 1;
            for (a, i, x) in
                inputs_where(&mut rng, |a, i, x| f.lemma_pre_holds(a, i, x) == Ok(true))
            {
                stats.lemma_runs += 1;
                match f.post_holds(&a, &a, i, x, 0) {
                    Ok(true) => {}
                    Ok(false) => panic!(
                        "{}",
                        ctx(&format!(
                            "verified lemma {name} has a false conclusion at a = {a:?}, i = {i}, x = {x}"
                        ))
                    ),
                    Err(Undef) => stats.undetermined += 1,
                }
            }
            continue;
        }
        let Some(f) = funcs.iter().find(|f| f.name == name) else {
            continue;
        };
        // A caller assumes the lemma's conclusion (ADR-0062 §2): its proof
        // is sound when the lemma's is too, as for any callee.
        let lemma_failed = f.lemma.is_some()
            && !out.verify.output.lines().any(|l| {
                field(l, "def") == Some(&format!("m::{name}_lemma"))
                    && field(l, "status") == Some("verified")
            });
        if lemma_failed {
            continue;
        }
        stats.funcs += 1;
        let entry = stats.by_template.entry(f.template).or_default();
        entry.0 += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        entry.1 += 1;
        stats.verified += 1;
        for (a, i, x) in inputs(f, &mut rng) {
            stats.verified_runs += 1;
            let r = run(f, &a, i, x);
            let at = format!("a = {a:?}, i = {i}, x = {x}");
            let Ok((v, after)) = r.result else {
                panic!("{}", ctx(&format!("verified {name} faults at {at}")));
            };
            assert!(
                !r.inv_broken,
                "{}",
                ctx(&format!("verified {name} breaks its invariant at {at}"))
            );
            match f.post_holds(&after, &a, i, x, v) {
                Ok(true) => {}
                Ok(false) => panic!(
                    "{}",
                    ctx(&format!(
                        "verified {name} breaks its postcondition at {at} (r = {v}, a after = {after:?})"
                    ))
                ),
                Err(Undef) => stats.undetermined += 1,
            }
        }
    }

    for line in out.verify.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        // A lemma's counterexample: its precondition holds there and its
        // conclusion does not.
        if let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}_lemma`", f.name)))
        {
            let code = field(line, "code").unwrap_or("");
            let cex = if !msg.contains("; counterexample: ") {
                ""
            } else if line.contains("only by name") {
                " (opaque counterexample)"
            } else {
                " (counterexample)"
            };
            *stats.codes.entry(format!("lemma {code}{cex}")).or_default() += 1;
            if code != "E0802" || cex != " (counterexample)" {
                continue;
            }
            let Some((a, i, x)) = parse_cex(msg) else {
                continue;
            };
            assert_ne!(
                f.lemma_pre_holds(&a, i, x),
                Ok(false),
                "{}",
                ctx(&format!(
                    "lemma counterexample breaks its precondition: {msg}"
                ))
            );
            assert_ne!(
                f.post_holds(&a, &a, i, x, 0),
                Ok(true),
                "{}",
                ctx(&format!("lemma counterexample meets the conclusion: {msg}"))
            );
            stats.lemma_replayed += 1;
            continue;
        }
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}`", f.name)))
        else {
            continue;
        };
        let code = field(line, "code").unwrap_or("");
        let what = msg.split(" in `").next().unwrap_or("");
        let what = what.split(" `").next().unwrap_or(what);
        let how = if msg.contains("instantiated definitions only") {
            " (ground retry: instances only)"
        } else if msg.contains("resource limit") {
            " (resource limit)"
        } else {
            ""
        };
        *stats
            .codes
            .entry(format!("{code} {what}{how}"))
            .or_default() += 1;
        if !msg.contains("; counterexample: ") {
            continue;
        }
        stats.cexes += 1;
        if line.contains("only by name") {
            // Opaque spec fns: the model may interpret them in any way.
            stats.opaque_cexes += 1;
            continue;
        }
        let Some((a, i, x)) = parse_cex(msg) else {
            stats.partial_cexes += 1;
            continue;
        };
        assert_ne!(
            f.pre_holds(&a, i, x),
            Ok(false),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        if f.pre_holds(&a, i, x).is_err() {
            stats.cex_undetermined += 1;
            continue;
        }
        // The caller of a lemma fails where the lemma's precondition or
        // its conclusion does.
        if f.lemma.is_some() {
            let broken = f.lemma_pre_holds(&a, i, x) == Ok(false)
                || f.post_holds(&a, &a, i, x, 0) == Ok(false);
            if !broken {
                let undetermined =
                    f.lemma_pre_holds(&a, i, x).is_err() || f.post_holds(&a, &a, i, x, 0).is_err();
                assert!(
                    undetermined,
                    "{}",
                    ctx(&format!(
                        "counterexample meets the lemma's precondition and conclusion: {msg}"
                    ))
                );
                stats.cex_undetermined += 1;
                continue;
            }
            stats.replayed += 1;
            continue;
        }
        let r = run(f, &a, i, x);
        if f.body.is_none() {
            if let Ok((v, after)) = &r.result {
                let post = f.post_holds(after, &a, i, x, *v);
                assert_ne!(
                    post,
                    Ok(true),
                    "{}",
                    ctx(&format!(
                        "counterexample runs without fault and meets the postcondition: {msg} (r = {v}, a after = {after:?})"
                    ))
                );
                if post.is_err() {
                    stats.cex_undetermined += 1;
                    continue;
                }
            }
            stats.replayed += 1;
        } else if code == "E0806" {
            match r.entry_inv {
                Some(Ok(true)) => panic!(
                    "{}",
                    ctx(&format!(
                        "entry counterexample does not break the invariant: {msg}"
                    ))
                ),
                Some(Err(Undef)) => stats.cex_undetermined += 1,
                _ => stats.entry_replays += 1,
            }
        }
    }

    let lines = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("s{seed}-{all_checks}"));
        bmb::toolchain::build_executable(&ir, exe.to_str().unwrap())
            .unwrap_or_else(|e| panic!("{}", ctx(&e)));
        let res = Command::new(&exe).output().unwrap();
        let _ = fs::remove_file(&exe);
        assert!(
            res.status.success(),
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) exited with {}",
                res.status
            ))
        );
        assert_eq!(
            String::from_utf8_lossy(&res.stdout),
            expected,
            "{}",
            ctx(&format!("build (all_checks = {all_checks}) output differs"))
        );
        stats.compiled_runs += lines;
    }
}

#[test]
fn spec_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let dir = std::env::temp_dir().join(format!("bmb-sound-specs-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (specs): {} functions, {} verified ({} runs), {} counterexamples ({} replayed, {} entry replays, {} opaque, {} partial, {} undetermined), {} undetermined verified runs, {} compiled outputs",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.cexes,
        stats.replayed,
        stats.entry_replays,
        stats.opaque_cexes,
        stats.partial_cexes,
        stats.cex_undetermined,
        stats.undetermined,
        stats.compiled_runs
    );
    eprintln!(
        "verified by template (functions, verified): {:#?}",
        stats.by_template
    );
    eprintln!(
        "lemmas: {} generated, {} verified ({} runs), {} counterexamples replayed",
        stats.lemmas, stats.lemmas_verified, stats.lemma_runs, stats.lemma_replayed
    );
    eprintln!("failures by kind: {:#?}", stats.codes);
    assert!(stats.verified > 0 && stats.cexes > 0);
}
