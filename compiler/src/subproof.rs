//! Sub-proofs in an exact theory (ADR-0048 §4, backlog 2N-03).
//!
//! `assert(e) by (bit_vector);` proves `e` in a query of its own and the
//! enclosing proof then assumes it. The sub-proof sees only `e`'s free
//! variables, each ranging over its whole type, and no fact of the path
//! (Verus's rule): a needed fact is written into `e` as `p ==> q`.
//!
//! The query is a translation of the term the encoder makes of `e` (its
//! mixed integer and bit-vector term over a fresh symbol per free variable),
//! not of `e`'s syntax. So the sub-proof proves exactly the fact the enclosing
//! query assumes, whatever linear or bit-vector form the encoder chose for
//! each operation (`verify::bit_op`, `verify::bit_method`). Every integer
//! subterm becomes a two's complement bit-vector wide enough to hold all of
//! its values (an interval is kept for each), so `+`, `*`, `div` and `mod` by
//! constants, comparisons and `bv2nat`/`int2bv` are exact: for every value of
//! the free variables in their types, the translation and the term agree
//! (checked by evaluation in `tests/lemmas.rs`).
//!
//! A spec fn application is unfolded through its body (`unfold`, which
//! translates the body as the encoder does). Its definition holds for
//! arguments in the parameters' types (the axiom's guard): an integer argument
//! not known to be in its type becomes a hypothesis of the sub-proof and an
//! antecedent of the assumed fact (`Query::guards`).
//!
//! Another theory plugs in at `Sort`: ADR-0061 §5 adds `f64` variables as
//! `(_ FloatingPoint 11 53)` and interprets the encoder's `f64.*` symbols as
//! `fp.*` with rounding to nearest even here only (`verify::F64_FUNS`, the
//! same definitions the counterexample search uses); `Query::logic` then
//! names a logic with both. An integer converted by `f64.from` is a signed
//! bit-vector, which `to_fp` rounds as LLVM's `sitofp` does.

use std::collections::BTreeMap;

use crate::ast::IntTy;
use crate::smt::Sexp;

/// The sort of a free variable of a sub-proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    Int(IntTy),
    Bool,
    /// `f64` (ADR-0061 §5).
    F64,
}

/// A spec fn's body over placeholder symbols for its parameters (the
/// encoder's translation of the body), and the parameters' sorts.
pub struct Unfolded {
    pub body: Sexp,
    pub params: Vec<Sort>,
}

/// A sub-proof query: `base()` then `(assert (not goal))`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub logic: &'static str,
    /// Declarations of the free variables and definitions of shared terms.
    pub decls: Vec<String>,
    /// Hypotheses: arguments of unfolded spec fns in their parameters' types.
    pub hyps: Vec<String>,
    pub goal: String,
    /// The free variables (symbol, sort), for counterexamples.
    pub vars: Vec<(String, Sort)>,
    /// The hypotheses as integer terms over the free variables' symbols:
    /// the enclosing query assumes the assertion under them.
    pub guards: Vec<String>,
}

impl Query {
    pub fn base(&self) -> String {
        let mut out = format!("(set-logic {})\n", self.logic);
        for d in &self.decls {
            out.push_str(d);
            out.push('\n');
        }
        for h in &self.hyps {
            out.push_str(&format!("(assert {h})\n"));
        }
        out
    }

    pub fn negated_goal(&self) -> String {
        format!("(assert (not {}))", self.goal)
    }
}

/// The widest bit-vector a translation may make.
pub const MAX_WIDTH: u32 = 1024;

/// Unfolding depth bound (a recursive spec fn is refused by `unfold`).
const MAX_UNFOLD: usize = 64;

/// Bounds of an integer term's values, when known.
type Interval = Option<(i128, i128)>;

/// An integer term's bit-vector, width and interval.
type IntBits = (String, u32, Interval);

/// A translated term.
#[derive(Clone, Debug)]
enum T {
    Bool(String),
    /// An integer: a two's complement bit-vector of `w` bits holding exactly
    /// the value, which lies in `iv` when that is known.
    Int {
        t: String,
        w: u32,
        iv: Interval,
    },
    Bv {
        t: String,
        w: u32,
    },
    /// An `f64`: a `(_ FloatingPoint 11 53)` term.
    Fp(String),
}

type Unfold<'u> = dyn FnMut(&str, &[String]) -> Result<Option<Unfolded>, String> + 'u;

struct Tr<'a, 'u> {
    syms: BTreeMap<String, T>,
    /// Each placeholder's argument as an integer term (for guards).
    args: BTreeMap<String, Sexp>,
    decls: Vec<String>,
    hyps: Vec<String>,
    guards: Vec<String>,
    memo: BTreeMap<String, T>,
    next: usize,
    depth: usize,
    unfold: &'a mut Unfold<'u>,
}

/// Translates the boolean term `term` over the free variables `vars`
/// (symbol, sort) into a bit-vector query (see the module comment).
pub fn bit_vector(
    term: &Sexp,
    vars: &[(String, Sort)],
    unfold: &mut Unfold<'_>,
) -> Result<Query, String> {
    let mut tr = Tr {
        syms: BTreeMap::new(),
        args: BTreeMap::new(),
        decls: Vec::new(),
        hyps: Vec::new(),
        guards: Vec::new(),
        memo: BTreeMap::new(),
        next: 0,
        depth: 0,
        unfold,
    };
    for (s, sort) in vars {
        let (decl, t) = var(s, *sort);
        tr.decls.push(format!("(declare-const {s} {decl})"));
        tr.syms.insert(s.clone(), t);
    }
    let goal = match tr.term(term)? {
        T::Bool(g) => g,
        _ => return Err("the assertion is not a `bool`".into()),
    };
    // The `f64` operations, defined (ADR-0061 §5), before the definitions
    // that apply them.
    let all = format!("{} {} {goal}", tr.decls.join(" "), tr.hyps.join(" "));
    let defs = crate::verify::f64_decls(&all, true);
    if !defs.is_empty() {
        let mut d: Vec<String> = defs.lines().map(String::from).collect();
        d.append(&mut tr.decls);
        tr.decls = d;
    }
    let fp = vars.iter().any(|(_, s)| *s == Sort::F64)
        || !defs.is_empty()
        || all.contains("(fp ")
        || all.contains("(_ NaN")
        || all.contains("to_fp");
    Ok(Query {
        logic: if fp { "QF_FPBV" } else { "QF_BV" },
        decls: tr.decls,
        hyps: tr.hyps,
        goal,
        vars: vars.to_vec(),
        guards: tr.guards,
    })
}

/// The translation of the integer term `term` (no spec fn applications):
/// a bit-vector of the returned width holding its value in two's
/// complement, and the declarations it needs. `tests/lemmas.rs` evaluates
/// it against `term`.
pub fn bit_vector_int(term: &Sexp, vars: &[(String, Sort)]) -> Result<(String, u32), String> {
    let mut none = |_: &str, _: &[String]| Ok::<Option<Unfolded>, String>(None);
    let mut tr = Tr {
        syms: BTreeMap::new(),
        args: BTreeMap::new(),
        decls: Vec::new(),
        hyps: Vec::new(),
        guards: Vec::new(),
        memo: BTreeMap::new(),
        next: 0,
        depth: 0,
        unfold: &mut none,
    };
    for (s, sort) in vars {
        tr.syms.insert(s.clone(), var(s, *sort).1);
    }
    match tr.term(term)? {
        T::Int { t, w, .. } => Ok((t, w)),
        _ => Err("not an integer term".into()),
    }
}

/// A free variable's declared sort and its translation.
fn var(s: &str, sort: Sort) -> (String, T) {
    match sort {
        Sort::Bool => ("Bool".to_string(), T::Bool(s.to_string())),
        Sort::F64 => ("Float64".to_string(), T::Fp(s.to_string())),
        Sort::Int(i) => {
            let w = i.bits();
            let iv = Some((i.min_value(), i.max_value()));
            // An unsigned value gets a zero sign bit.
            let t = if i.signed() {
                T::Int {
                    t: s.to_string(),
                    w,
                    iv,
                }
            } else {
                T::Int {
                    t: format!("((_ zero_extend 1) {s})"),
                    w: w + 1,
                    iv,
                }
            };
            (format!("(_ BitVec {w})"), t)
        }
    }
}

/// The bit-vector functions the encoder names for the bit methods
/// (`verify::bit_method`): `bmb_ctpop{w}`, `bmb_clz{w}`, `bmb_ctz{w}`,
/// `bmb_bswap{w}` of one argument and `bmb_rotl{w}`, `bmb_rotr{w}` of a
/// value and an amount (taken modulo `w`), for `w` in 8, 16, 32, 64. A
/// query that mentions one gets its definition (`helper_defs`); a named
/// function keeps the terms of code and specification comparable for the
/// congruence facts of the abstracted query (`verify::bit_congruences`).
pub fn helper_def(name: &str) -> Option<(String, u32)> {
    let (op, w) = name
        .strip_prefix("bmb_")?
        .split_at(name[4..].find(|c: char| c.is_ascii_digit())?);
    let w: u32 = w.parse().ok()?;
    if ![8, 16, 32, 64].contains(&w) {
        return None;
    }
    let bit = |k: u32| format!("(= ((_ extract {k} {k}) x) #b1)");
    let n = |v: u32| format!("(_ bv{v} {w})");
    let sort = format!("(_ BitVec {w})");
    let body = match op {
        "ctpop" => format!(
            "(bvadd {})",
            (0..w)
                .map(|k| format!("((_ zero_extend {}) ((_ extract {k} {k}) x))", w - 1))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        // The first set bit from the top (`clz`) or the bottom (`ctz`).
        "clz" => (0..w).fold(n(w), |t, k| {
            format!("(ite {} {} {t})", bit(k), n(w - 1 - k))
        }),
        "ctz" => (0..w)
            .rev()
            .fold(n(w), |t, k| format!("(ite {} {} {t})", bit(k), n(k))),
        "bswap" if w > 8 => format!(
            "(concat {})",
            (0..w / 8)
                .map(|b| format!("((_ extract {} {}) x)", 8 * b + 7, 8 * b))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        "rotl" | "rotr" => {
            let s = format!("(bvand s {})", n(w - 1));
            let (to, back) = if op == "rotl" {
                ("bvshl", "bvlshr")
            } else {
                ("bvlshr", "bvshl")
            };
            let def = format!(
                "(define-fun {name} ((x {sort}) (s {sort})) {sort} (bvor ({to} x {s}) ({back} x (bvsub {} {s}))))",
                n(w)
            );
            return Some((def, w));
        }
        _ => return None,
    };
    Some((format!("(define-fun {name} ((x {sort})) {sort} {body})"), w))
}

/// The definitions of the helpers (`helper_def`) that `text` applies, in
/// name order.
pub fn helper_defs(text: &str) -> String {
    if !text.contains("(bmb_") {
        return String::new();
    }
    let mut out = String::new();
    for op in ["bswap", "clz", "ctpop", "ctz", "rotl", "rotr"] {
        for w in [16, 32, 64, 8] {
            let name = format!("bmb_{op}{w}");
            if text.contains(&format!("({name} ")) {
                let (def, _) = helper_def(&name).expect("a helper");
                out.push_str(&def);
                out.push('\n');
            }
        }
    }
    out
}

/// The value of a bit-vector literal of a model (`#x0f`, `#b101`, `(_ bv5
/// 8)`) as an integer of `i`.
pub fn model_int(v: &str, i: IntTy) -> Option<i128> {
    let u = if let Some(h) = v.strip_prefix("#x") {
        u128::from_str_radix(h, 16).ok()?
    } else if let Some(b) = v.strip_prefix("#b") {
        u128::from_str_radix(b, 2).ok()?
    } else {
        let inner = v.strip_prefix("(_ bv")?.strip_suffix(')')?;
        inner.split_whitespace().next()?.parse().ok()?
    } as i128;
    let size = 1i128 << i.bits();
    Some(if i.signed() && u > i.max_value() {
        u - size
    } else {
        u
    })
}

fn bitlen(x: i128) -> u32 {
    128 - x.leading_zeros()
}

/// The fewest bits that hold `lo..=hi` in two's complement.
fn bits(lo: i128, hi: i128) -> u32 {
    let one = |v: i128| if v >= 0 { bitlen(v) } else { bitlen(-(v + 1)) } + 1;
    one(lo).max(one(hi))
}

/// `t`, a two's complement value of `from` bits, as `to` bits (`to` is
/// enough to hold it when smaller).
fn resize(t: &str, from: u32, to: u32) -> String {
    use std::cmp::Ordering::*;
    match to.cmp(&from) {
        Greater => format!("((_ sign_extend {}) {t})", to - from),
        Less => format!("((_ extract {} 0) {t})", to - 1),
        Equal => t.to_string(),
    }
}

/// The literal `v` as `w` bits (two's complement; `v` fits).
fn lit(v: i128, w: u32) -> String {
    if w <= 128 {
        let mask = if w == 128 {
            u128::MAX
        } else {
            (1u128 << w) - 1
        };
        return format!("(_ bv{} {w})", (v as u128) & mask);
    }
    if v >= 0 {
        return format!("(_ bv{v} {w})");
    }
    // A negative value: its 128-bit image, sign-extended.
    format!("((_ sign_extend {}) (_ bv{} 128))", w - 128, v as u128)
}

fn int_lit(v: i128) -> T {
    let w = bits(v, v);
    T::Int {
        t: lit(v, w),
        w,
        iv: Some((v, v)),
    }
}

/// An integer of `w` bits, narrowed to its interval's width.
fn int(t: String, w: u32, iv: Interval) -> Result<T, String> {
    let need = iv.map(|(lo, hi)| bits(lo, hi));
    let (t, w) = match need {
        Some(n) if n < w => (resize(&t, w, n), n),
        _ => (t, w),
    };
    if w > MAX_WIDTH {
        return Err(format!(
            "a term needs more than {MAX_WIDTH} bits as a bit-vector"
        ));
    }
    Ok(T::Int { t, w, iv })
}

fn numeral(e: &Sexp) -> Option<i128> {
    match e {
        Sexp::Atom(a) => a.parse().ok(),
        Sexp::List(xs) => match xs.as_slice() {
            [Sexp::Atom(m), Sexp::Atom(a)] if m == "-" => a.parse::<i128>().ok().map(|v| -v),
            _ => None,
        },
    }
}

fn atom(e: &Sexp) -> Option<&str> {
    match e {
        Sexp::Atom(a) => Some(a),
        _ => None,
    }
}

fn hull(a: Interval, b: Interval) -> Interval {
    let ((l1, h1), (l2, h2)) = (a?, b?);
    Some((l1.min(l2), h1.max(h2)))
}

fn print(e: &Sexp) -> String {
    crate::smt::print(e)
}

impl Tr<'_, '_> {
    fn term(&mut self, e: &Sexp) -> Result<T, String> {
        if let Some(v) = numeral(e) {
            return Ok(int_lit(v));
        }
        match e {
            Sexp::Atom(a) => match a.as_str() {
                "true" | "false" => Ok(T::Bool(a.clone())),
                _ if a.starts_with("#b") => Ok(T::Bv {
                    t: a.clone(),
                    w: a.len() as u32 - 2,
                }),
                _ if a.starts_with("#x") => Ok(T::Bv {
                    t: a.clone(),
                    w: 4 * (a.len() as u32 - 2),
                }),
                _ => self
                    .syms
                    .get(a)
                    .cloned()
                    .ok_or_else(|| format!("`{a}` has no bit-vector translation")),
            },
            Sexp::List(xs) if xs.is_empty() => Err("empty term".into()),
            Sexp::List(xs) => match &xs[0] {
                Sexp::List(f) => self.indexed(f, &xs[1..], e),
                Sexp::Atom(h) if h == "_" => match xs.as_slice() {
                    [_, Sexp::Atom(v), Sexp::Atom(w)] if v.starts_with("bv") => {
                        let w: u32 = w.parse().map_err(|_| "bad literal width")?;
                        Ok(T::Bv { t: print(e), w })
                    }
                    // `(_ NaN 11 53)`, `(_ +zero 11 53)`: `f64` constants.
                    [_, Sexp::Atom(v), Sexp::Atom(eb), Sexp::Atom(sb)]
                        if eb == "11"
                            && sb == "53"
                            && matches!(v.as_str(), "NaN" | "+zero" | "-zero" | "+oo" | "-oo") =>
                    {
                        Ok(T::Fp(print(e)))
                    }
                    _ => Err(format!("`{}` has no bit-vector translation", print(e))),
                },
                Sexp::Atom(h) => self.app(h, &xs[1..], e),
            },
        }
    }

    fn ints(&mut self, xs: &[Sexp]) -> Result<Vec<IntBits>, String> {
        xs.iter()
            .map(|x| match self.term(x)? {
                T::Int { t, w, iv } => Ok((t, w, iv)),
                _ => Err(format!("`{}` is not an integer", print(x))),
            })
            .collect()
    }

    fn bools(&mut self, xs: &[Sexp]) -> Result<Vec<String>, String> {
        xs.iter()
            .map(|x| match self.term(x)? {
                T::Bool(t) => Ok(t),
                _ => Err(format!("`{}` is not a `bool`", print(x))),
            })
            .collect()
    }

    fn fps(&mut self, xs: &[Sexp]) -> Result<Vec<String>, String> {
        xs.iter()
            .map(|x| match self.term(x)? {
                T::Fp(t) => Ok(t),
                _ => Err(format!("`{}` is not an `f64`", print(x))),
            })
            .collect()
    }

    /// An `f64` operation of the encoder (ADR-0061 §5). It stays an
    /// application of its symbol, which the query defines with its IEEE
    /// meaning (`verify::f64_decls`, interpreted: `fp.*` with rounding to
    /// nearest even), except `f64.from`: its integer operand is a signed
    /// bit-vector here, which `to_fp` rounds as `sitofp` does.
    fn float(&mut self, h: &str, args: &[Sexp]) -> Result<T, String> {
        if h == "f64.from" {
            let (t, _, _) = self.ints(args)?.remove(0);
            return Ok(T::Fp(format!("((_ to_fp 11 53) RNE {t})")));
        }
        let ret = match h {
            "f64.is_finite" => "Bool",
            _ => crate::verify::F64_FUNS
                .iter()
                .find(|(n, ..)| *n == h)
                .map(|(_, _, ret, _)| *ret)
                .ok_or_else(|| format!("`{h}` has no floating-point translation"))?,
        };
        let xs = self.fps(args)?;
        let t = format!("({h} {})", xs.join(" "));
        Ok(if ret == "Bool" { T::Bool(t) } else { T::Fp(t) })
    }

    fn bvs(&mut self, xs: &[Sexp]) -> Result<Vec<(String, u32)>, String> {
        xs.iter()
            .map(|x| match self.term(x)? {
                T::Bv { t, w } => Ok((t, w)),
                _ => Err(format!("`{}` is not a bit-vector", print(x))),
            })
            .collect()
    }

    /// `(op a b)` on integers, computed at a width that holds it exactly.
    fn arith(&mut self, op: &str, args: &[Sexp]) -> Result<T, String> {
        let xs = self.ints(args)?;
        let mut it = xs.into_iter();
        let (mut t, mut w, mut iv) = it.next().ok_or("an operation without operands")?;
        if op == "-" && args.len() == 1 {
            let iv = iv.and_then(|(lo, hi)| Some((hi.checked_neg()?, lo.checked_neg()?)));
            let ww = w + 1;
            return int(format!("(bvneg {})", resize(&t, w, ww)), ww, iv);
        }
        for (u, wu, ivu) in it {
            let (ww, f, niv) = match op {
                "+" => (
                    w.max(wu) + 1,
                    "bvadd",
                    iv.zip(ivu)
                        .and_then(|((a, b), (c, d))| Some((a.checked_add(c)?, b.checked_add(d)?))),
                ),
                "-" => (
                    w.max(wu) + 1,
                    "bvsub",
                    iv.zip(ivu)
                        .and_then(|((a, b), (c, d))| Some((a.checked_sub(d)?, b.checked_sub(c)?))),
                ),
                _ => (
                    w + wu,
                    "bvmul",
                    iv.zip(ivu).and_then(|((a, b), (c, d))| {
                        let ps = [
                            a.checked_mul(c)?,
                            a.checked_mul(d)?,
                            b.checked_mul(c)?,
                            b.checked_mul(d)?,
                        ];
                        Some((*ps.iter().min()?, *ps.iter().max()?))
                    }),
                ),
            };
            // Modulo `2^n` the result is exact when it and the operands fit
            // `n` bits: the interval's width, if known, is enough.
            let ww = match niv {
                Some((lo, hi)) => bits(lo, hi).max(w).max(wu),
                None => ww,
            };
            if ww > MAX_WIDTH {
                return Err(format!(
                    "a term needs more than {MAX_WIDTH} bits as a bit-vector"
                ));
            }
            let r = format!("({f} {} {})", resize(&t, w, ww), resize(&u, wu, ww));
            match int(r, ww, niv)? {
                T::Int {
                    t: t2,
                    w: w2,
                    iv: i2,
                } => {
                    t = t2;
                    w = w2;
                    iv = i2;
                }
                _ => unreachable!(),
            }
        }
        Ok(T::Int { t, w, iv })
    }

    /// `(div a c)` / `(mod a c)` for a positive constant `c` (floor
    /// division, as SMT-LIB's on integers).
    fn divmod(&mut self, op: &str, args: &[Sexp]) -> Result<T, String> {
        let c = match args.get(1).and_then(numeral) {
            Some(c) if c > 0 && args.len() == 2 => c,
            _ => {
                return Err(format!(
                    "`{op}` by a value that is not a positive constant has no exact bit-vector form (a division in the assertion)"
                ));
            }
        };
        let (t, w, iv) = self.ints(&args[..1])?.remove(0);
        let pow = (c & (c - 1)) == 0;
        let k = c.trailing_zeros();
        if op == "div" {
            let niv = iv.map(|(lo, hi)| (lo.div_euclid(c), hi.div_euclid(c)));
            if pow {
                if k >= w {
                    // Every value of `w` bits is above `-c` and below `c`.
                    let r = format!(
                        "(ite (bvslt {t} {}) {} {})",
                        lit(0, w),
                        lit(-1, w),
                        lit(0, w)
                    );
                    return int(r, w, niv);
                }
                return int(format!("(bvashr {t} {})", lit(k as i128, w)), w, niv);
            }
            let ww = w.max(bits(c, c)) + 1;
            let (a, cc) = (resize(&t, w, ww), lit(c, ww));
            let r = format!("(bvsdiv (bvsub {a} (bvsmod {a} {cc})) {cc})");
            return int(r, ww, niv);
        }
        let niv = Some((0, c - 1));
        if pow {
            if k == 0 {
                return Ok(int_lit(0));
            }
            let low = format!("((_ extract {} 0) {})", k - 1, resize(&t, w, w.max(k)));
            return int(format!("((_ zero_extend 1) {low})"), k + 1, niv);
        }
        let ww = w.max(bits(c, c));
        let r = format!("(bvsmod {} {})", resize(&t, w, ww), lit(c, ww));
        int(r, ww, niv)
    }

    fn compare(&mut self, op: &str, args: &[Sexp]) -> Result<T, String> {
        let xs = self.ints(args)?;
        let ww = xs.iter().map(|x| x.1).max().unwrap_or(1);
        let f = match op {
            "<" => "bvslt",
            "<=" => "bvsle",
            ">" => "bvsgt",
            _ => "bvsge",
        };
        let parts: Vec<String> = xs
            .windows(2)
            .map(|p| {
                format!(
                    "({f} {} {})",
                    resize(&p[0].0, p[0].1, ww),
                    resize(&p[1].0, p[1].1, ww)
                )
            })
            .collect();
        Ok(T::Bool(conj(parts)))
    }

    fn equal(&mut self, args: &[Sexp]) -> Result<T, String> {
        let xs: Vec<T> = args
            .iter()
            .map(|a| self.term(a))
            .collect::<Result<_, _>>()?;
        let ww = xs
            .iter()
            .map(|x| match x {
                T::Int { w, .. } => *w,
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        let text = |x: &T| match x {
            T::Int { t, w, .. } => resize(t, *w, ww),
            T::Bool(t) | T::Bv { t, .. } | T::Fp(t) => t.clone(),
        };
        let parts: Vec<String> = xs
            .windows(2)
            .map(|p| format!("(= {} {})", text(&p[0]), text(&p[1])))
            .collect();
        Ok(T::Bool(conj(parts)))
    }

    fn ite(&mut self, args: &[Sexp]) -> Result<T, String> {
        let [c, a, b] = args else {
            return Err("`ite` takes three operands".into());
        };
        let c = self.bools(std::slice::from_ref(c))?.remove(0);
        Ok(match (self.term(a)?, self.term(b)?) {
            (T::Bool(x), T::Bool(y)) => T::Bool(format!("(ite {c} {x} {y})")),
            (T::Bv { t: x, w }, T::Bv { t: y, .. }) => T::Bv {
                t: format!("(ite {c} {x} {y})"),
                w,
            },
            (T::Fp(x), T::Fp(y)) => T::Fp(format!("(ite {c} {x} {y})")),
            (
                T::Int {
                    t: x,
                    w: wx,
                    iv: ix,
                },
                T::Int {
                    t: y,
                    w: wy,
                    iv: iy,
                },
            ) => {
                let ww = wx.max(wy);
                let r = format!("(ite {c} {} {})", resize(&x, wx, ww), resize(&y, wy, ww));
                int(r, ww, hull(ix, iy))?
            }
            _ => return Err("`ite` branches of different sorts".into()),
        })
    }

    fn app(&mut self, h: &str, args: &[Sexp], e: &Sexp) -> Result<T, String> {
        match h {
            _ if h.starts_with("f64.") => self.float(h, args),
            // An `f64` literal: its sign, exponent and significand.
            "fp" => Ok(T::Fp(print(e))),
            "+" | "-" | "*" => self.arith(h, args),
            "div" | "mod" => self.divmod(h, args),
            "<" | "<=" | ">" | ">=" => self.compare(h, args),
            "=" => self.equal(args),
            "ite" => self.ite(args),
            "and" | "or" | "xor" | "=>" => {
                let xs = self.bools(args)?;
                Ok(T::Bool(format!("({h} {})", xs.join(" "))))
            }
            "not" => {
                let x = self.bools(args)?;
                Ok(T::Bool(format!("(not {})", x.join(" "))))
            }
            "bv2nat" => {
                let (t, w) = self.bvs(args)?.remove(0);
                let iv = (w <= 126).then(|| (0, (1i128 << w) - 1));
                int(format!("((_ zero_extend 1) {t})"), w + 1, iv)
            }
            "bvnot" | "bvneg" => {
                let (t, w) = self.bvs(args)?.remove(0);
                Ok(T::Bv {
                    t: format!("({h} {t})"),
                    w,
                })
            }
            "bvand" | "bvor" | "bvxor" | "bvadd" | "bvmul" | "bvsub" | "bvshl" | "bvlshr"
            | "bvashr" | "bvudiv" | "bvurem" | "bvsdiv" | "bvsrem" | "bvsmod" => {
                let xs = self.bvs(args)?;
                let w = xs.first().map(|x| x.1).ok_or("no operands")?;
                let ts: Vec<String> = xs.into_iter().map(|x| x.0).collect();
                Ok(T::Bv {
                    t: format!("({h} {})", ts.join(" ")),
                    w,
                })
            }
            "concat" => {
                let xs = self.bvs(args)?;
                let w = xs.iter().map(|x| x.1).sum();
                let ts: Vec<String> = xs.into_iter().map(|x| x.0).collect();
                Ok(T::Bv {
                    t: format!("(concat {})", ts.join(" ")),
                    w,
                })
            }
            "bvult" | "bvule" | "bvugt" | "bvuge" | "bvslt" | "bvsle" | "bvsgt" | "bvsge" => {
                let xs = self.bvs(args)?;
                let ts: Vec<String> = xs.into_iter().map(|x| x.0).collect();
                Ok(T::Bool(format!("({h} {})", ts.join(" "))))
            }
            _ if helper_def(h).is_some() => {
                let (def, w) = helper_def(h).expect("a helper");
                let xs = self.bvs(args)?;
                if !self.decls.contains(&def) {
                    self.decls.push(def);
                }
                let ts: Vec<String> = xs.into_iter().map(|x| x.0).collect();
                Ok(T::Bv {
                    t: format!("({h} {})", ts.join(" ")),
                    w,
                })
            }
            _ => self.call(h, args, e),
        }
    }

    /// `((_ f k ..) x)`: `int2bv`, `extract`, `zero_extend`, `sign_extend`.
    fn indexed(&mut self, f: &[Sexp], args: &[Sexp], e: &Sexp) -> Result<T, String> {
        let name = f.get(1).and_then(atom).unwrap_or_default();
        let ks: Vec<u32> = f[2..]
            .iter()
            .map(|k| atom(k).and_then(|a| a.parse().ok()))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("`{}` has no bit-vector translation", print(e)))?;
        if name == "int2bv" {
            let (t, w, _) = self.ints(args)?.remove(0);
            let to = ks[0];
            return Ok(T::Bv {
                t: resize(&t, w, to),
                w: to,
            });
        }
        let (t, w) = self.bvs(args)?.remove(0);
        let nw = match (name, ks.as_slice()) {
            ("extract", [i, j]) => i - j + 1,
            ("zero_extend" | "sign_extend", [k]) => w + k,
            ("rotate_left" | "rotate_right", [_]) => w,
            _ => return Err(format!("`{}` has no bit-vector translation", print(e))),
        };
        Ok(T::Bv {
            t: format!("({} {t})", print(&Sexp::List(f.to_vec()))),
            w: nw,
        })
    }

    /// A spec fn application, unfolded through its body. Each argument is
    /// named once (`define-fun`), so nested applications stay small.
    fn call(&mut self, f: &str, args: &[Sexp], e: &Sexp) -> Result<T, String> {
        let vals: Vec<T> = args
            .iter()
            .map(|a| self.term(a))
            .collect::<Result<_, _>>()?;
        let key = format!(
            "{f} {}",
            vals.iter()
                .map(|v| match v {
                    T::Bool(t) | T::Int { t, .. } | T::Bv { t, .. } | T::Fp(t) => t.as_str(),
                })
                .collect::<Vec<_>>()
                .join(" ")
        );
        if let Some(r) = self.memo.get(&key) {
            return Ok(r.clone());
        }
        if self.depth >= MAX_UNFOLD {
            return Err(format!("`{f}` unfolds more than {MAX_UNFOLD} levels deep"));
        }
        let mut names = Vec::new();
        for (a, v) in args.iter().zip(&vals) {
            let p = format!("|a!{}|", self.next);
            self.next += 1;
            let bound = match v {
                T::Bool(t) if t.contains(' ') => {
                    self.decls.push(format!("(define-fun {p} () Bool {t})"));
                    T::Bool(p.clone())
                }
                T::Int { t, w, iv } if t.contains(' ') => {
                    self.decls
                        .push(format!("(define-fun {p} () (_ BitVec {w}) {t})"));
                    T::Int {
                        t: p.clone(),
                        w: *w,
                        iv: *iv,
                    }
                }
                T::Fp(t) if t.contains(' ') => {
                    self.decls.push(format!("(define-fun {p} () Float64 {t})"));
                    T::Fp(p.clone())
                }
                other => other.clone(),
            };
            self.syms.insert(p.clone(), bound);
            self.args.insert(p.clone(), a.clone());
            names.push(p);
        }
        let Some(u) = (self.unfold)(f, &names)? else {
            return Err(format!("`{}` has no bit-vector translation", print(e)));
        };
        for ((p, v), sort) in names.iter().zip(&vals).zip(&u.params) {
            let (Sort::Int(ty), T::Int { w, iv, .. }) = (sort, v) else {
                continue;
            };
            let (min, max) = (ty.min_value(), ty.max_value());
            if iv.is_some_and(|(lo, hi)| min <= lo && hi <= max) {
                continue;
            }
            let ww = (*w).max(bits(min, max));
            let x = resize(p, *w, ww);
            self.hyps.push(format!(
                "(and (bvsle {} {x}) (bvsle {x} {}))",
                lit(min, ww),
                lit(max, ww)
            ));
            let arg = print(&self.outer(&Sexp::Atom(p.clone())));
            self.guards.push(format!(
                "(and (<= {} {arg}) (<= {arg} {}))",
                num(min),
                num(max)
            ));
        }
        self.depth += 1;
        let r = self.term(&u.body);
        self.depth -= 1;
        let r = r?;
        self.memo.insert(key, r.clone());
        Ok(r)
    }

    /// `e` with every placeholder replaced by its argument: a term over
    /// the free variables' symbols.
    fn outer(&self, e: &Sexp) -> Sexp {
        match e {
            Sexp::Atom(a) => match self.args.get(a) {
                Some(x) => self.outer(x),
                None => e.clone(),
            },
            Sexp::List(xs) => Sexp::List(xs.iter().map(|x| self.outer(x)).collect()),
        }
    }
}

fn num(v: i128) -> String {
    if v < 0 {
        format!("(- {})", -v)
    } else {
        v.to_string()
    }
}

fn conj(parts: Vec<String>) -> String {
    match parts.len() {
        0 => "true".into(),
        1 => parts.into_iter().next().unwrap_or_default(),
        _ => format!("(and {})", parts.join(" ")),
    }
}
