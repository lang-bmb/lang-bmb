//! Soundness fuzzing of every integer width and the bit operations (backlog
//! 2N-04, ADR-0048 §1, §2, §5): random functions of three parameters whose
//! types range over `i8`..`i64`, `isize`, `u8`..`u64` and `usize` (the
//! shift amount `k` of any of them, so widths mix where Rust lets them:
//! shift amounts, conversions, comparisons and `if` arms of a converted
//! value, and specifications), with `+ - * / %`, unary `-`, `& | ^ !`,
//! `<<` and `>>` by literal amounts (below, at and beyond the width) and by
//! variable ones (any type, negative or huge), `wrapping_*`, the bit methods
//! (`rotate_left/right`, `wrapping_shl/shr`, `count_ones/zeros`,
//! `leading/trailing_zeros`, `swap_bytes`), conversions through
//! `T::try_from(..).unwrap()` and `.ok()`, and `if`. Random contracts bound
//! the parameters and the result.
//!
//! An independent interpreter (this test: Rust's debug-build semantics for
//! each width, computed on `i128` values and two's complement bit patterns,
//! not the compiler's code) judges three things, which must agree:
//!
//! - the prover: a verified function meets its postcondition and never
//!   panics on any admitted input the interpreter tries; every
//!   counterexample is admitted and replays to a panic or a broken
//!   postcondition;
//! - the build with proved checks removed and the `--all-checks` build:
//!   both print what the interpreter computes, and a call that panics in
//!   Rust (overflow, a shift amount outside `0..w`, division by zero,
//!   `MIN / -1`, an out-of-range conversion) traps in both (`SIGILL`, a
//!   program of its own per call, as a trap ends the run and loses the
//!   buffered output). A trap in the first build is a check the prover did
//!   not erase, so a wrongly proved obligation shows as a missing trap.
//!
//! Registers hold 64-bit extended values (ADR-0048 §5), so the generator
//! puts inputs at each type's edges and bit patterns (`MIN`, `MAX`, `-1`,
//! `0x55..`, powers of two): narrow `<<` must truncate and extend again,
//! `>>` is `ashr` for signed and `lshr` for unsigned types, `!` of an
//! unsigned value flips its own width. Two functions per batch are one
//! operation on `x` (a shift, `!`, a bit method, or over the whole types
//! `+ - * / %` and a conversion from another width, called first at
//! `MIN / -1`, `MAX * MAX` and negative-to-unsigned), whose checks are where
//! widths differ (`gen_shaped`). Every width comes in turn as `x`'s type.
//!
//! Through the CLI, as a user runs it (HEALTH S5). `BMB_SOUND_ITERS` /
//! `BMB_SOUND_SEED` as in `soundness.rs`.

mod common;

use std::fmt::Write as _;
use std::fs;
use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::scalar::{Rng, field};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ty {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
}

const ALL: [Ty; 10] = [
    Ty::I8,
    Ty::I16,
    Ty::I32,
    Ty::I64,
    Ty::Isize,
    Ty::U8,
    Ty::U16,
    Ty::U32,
    Ty::U64,
    Ty::Usize,
];

impl Ty {
    fn name(self) -> &'static str {
        match self {
            Ty::I8 => "i8",
            Ty::I16 => "i16",
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::Isize => "isize",
            Ty::U8 => "u8",
            Ty::U16 => "u16",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
            Ty::Usize => "usize",
        }
    }
    fn bits(self) -> u32 {
        match self {
            Ty::I8 | Ty::U8 => 8,
            Ty::I16 | Ty::U16 => 16,
            Ty::I32 | Ty::U32 => 32,
            _ => 64,
        }
    }
    fn signed(self) -> bool {
        matches!(self, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64 | Ty::Isize)
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
    fn contains(self, v: i128) -> bool {
        self.min() <= v && v <= self.max()
    }
    /// The `w`-bit two's complement pattern of `v` (any integer).
    fn pattern(self, v: i128) -> u128 {
        v.rem_euclid(1i128 << self.bits()) as u128
    }
    /// The value of the `w`-bit pattern `u` (higher bits ignored).
    fn of_pattern(self, u: u128) -> i128 {
        let u = (u & ((1u128 << self.bits()) - 1)) as i128;
        if self.signed() && u > self.max() {
            u - (1i128 << self.bits())
        } else {
            u
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Bin {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, PartialEq)]
enum Meth {
    WrapAdd,
    WrapSub,
    WrapMul,
    Rotl,
    Rotr,
    Wshl,
    Wshr,
}

#[derive(Clone, Copy, PartialEq)]
enum Count {
    Ones,
    Zeros,
    Leading,
    Trailing,
}

#[derive(Clone, Copy)]
enum Cmp {
    Lt,
    Le,
    Eq,
    Ne,
}

enum E {
    /// A parameter: `x`, `y` or `k`.
    Var(&'static str, Ty),
    /// A literal (or `T::MIN`) of the type its context gives it.
    Lit(i128, Ty),
    Bin(Bin, Box<E>, Box<E>),
    /// `a << b` (`true`) or `a >> b`; the amount has any integer type, a
    /// bare literal amount is `u32`.
    Shift(bool, Box<E>, Box<E>),
    Not(Box<E>),
    Neg(Box<E>),
    /// `a.m(b)`: `b` has `a`'s type for `wrapping_add/sub/mul`, `u32` for
    /// the rest.
    Method(Meth, Box<E>, Box<E>),
    /// `a.count_ones()` etc.: `u32`.
    Count(Count, Box<E>),
    SwapBytes(Box<E>),
    /// `T::try_from(a).unwrap()`
    Conv(Ty, Box<E>),
    /// `match T::try_from(a).ok() { None => d, Some(v) => v }`
    ConvOk(Ty, Box<E>, Box<E>, String),
    /// `if a cmp b { t } else { e }`
    If(Cmp, Box<E>, Box<E>, Box<E>, Box<E>),
}

/// Binding names are unique in the program, so in each function (E0303).
static BINDINGS: AtomicUsize = AtomicUsize::new(0);

fn binding() -> String {
    format!("v{}", BINDINGS.fetch_add(1, Ordering::Relaxed))
}

impl E {
    fn ty(&self) -> Ty {
        match self {
            E::Var(_, t) | E::Lit(_, t) => *t,
            E::Bin(_, a, _)
            | E::Shift(_, a, _)
            | E::Not(a)
            | E::Neg(a)
            | E::Method(_, a, _)
            | E::SwapBytes(a) => a.ty(),
            E::Count(..) => Ty::U32,
            E::Conv(t, _) | E::ConvOk(t, ..) => *t,
            E::If(_, _, _, t, _) => t.ty(),
        }
    }

    /// Whether the term has a type of its own (a literal alone takes the
    /// one its context gives; as a receiver or a conversion source it
    /// would be `i64`).
    fn typed(&self) -> bool {
        match self {
            E::Var(..) => true,
            E::Lit(v, t) => t.signed() && *v == t.min(),
            E::Bin(_, a, b) => a.typed() || b.typed(),
            E::Shift(_, a, _) | E::Not(a) | E::Neg(a) => a.typed(),
            E::Method(..) | E::Count(..) | E::SwapBytes(_) | E::Conv(..) | E::ConvOk(..) => true,
            E::If(_, _, _, t, e) => t.typed() || e.typed(),
        }
    }

    /// Whether a variable occurs (a function of constants alone is decided
    /// by the checker's folding, not the prover).
    fn has_var(&self) -> bool {
        match self {
            E::Var(..) => true,
            E::Lit(..) => false,
            E::Bin(_, a, b) | E::Shift(_, a, b) | E::Method(_, a, b) => a.has_var() || b.has_var(),
            E::Not(a) | E::Neg(a) | E::Count(_, a) | E::SwapBytes(a) | E::Conv(_, a) => a.has_var(),
            E::ConvOk(_, a, d, _) => a.has_var() || d.has_var(),
            E::If(_, a, b, t, e) => a.has_var() || b.has_var() || t.has_var() || e.has_var(),
        }
    }

    /// Whether a shift by a variable amount occurs.
    fn var_shift(&self) -> bool {
        self.any(&|e| matches!(e, E::Shift(_, _, b) if b.has_var()))
    }

    fn bit_method(&self) -> bool {
        self.any(&|e| {
            matches!(
                e,
                E::Count(..)
                    | E::SwapBytes(_)
                    | E::Method(Meth::Rotl | Meth::Rotr | Meth::Wshl | Meth::Wshr, ..)
            )
        })
    }

    fn any(&self, p: &dyn Fn(&E) -> bool) -> bool {
        if p(self) {
            return true;
        }
        match self {
            E::Var(..) | E::Lit(..) => false,
            E::Bin(_, a, b) | E::Shift(_, a, b) | E::Method(_, a, b) => a.any(p) || b.any(p),
            E::Not(a) | E::Neg(a) | E::Count(_, a) | E::SwapBytes(a) | E::Conv(_, a) => a.any(p),
            E::ConvOk(_, a, d, _) => a.any(p) || d.any(p),
            E::If(_, a, b, t, e) => a.any(p) || b.any(p) || t.any(p) || e.any(p),
        }
    }
}

/// Edge values and bit patterns of `t`.
fn interesting(t: Ty) -> Vec<i128> {
    let w = t.bits() as i128;
    let mut v = vec![
        t.min(),
        t.min() + 1,
        t.max(),
        t.max() - 1,
        0,
        1,
        2,
        3,
        7,
        w - 1,
        w,
        w + 1,
        t.of_pattern(0x5555_5555_5555_5555),
        t.of_pattern(0xaaaa_aaaa_aaaa_aaaa),
        t.of_pattern(0x0f0f_0f0f_0f0f_0f0f),
        t.of_pattern(0x8000_0000_0000_0001),
        t.max() / 2 + 1,
        t.max() / 3,
    ];
    if t.signed() {
        v.extend([-1, -2, -7, -w, t.min() / 2]);
    }
    v.retain(|x| t.contains(*x));
    v
}

fn value(rng: &mut Rng, t: Ty) -> i128 {
    match rng.below(3) {
        0 => {
            let v = interesting(t);
            v[rng.below(v.len())]
        }
        1 => t.of_pattern(rng.below(130) as u128).clamp(t.min(), t.max()),
        _ => t.of_pattern(rng.next() as u128),
    }
}

/// The parameters' types: `x: T`, `y: T2`, `k: S`.
#[derive(Clone, Copy)]
struct Sig {
    x: Ty,
    y: Ty,
    k: Ty,
}

impl Sig {
    fn vars(self) -> [(&'static str, Ty); 3] {
        [("x", self.x), ("y", self.y), ("k", self.k)]
    }
}

/// A leaf of type `t`: a parameter of that type, a conversion of one, or a
/// literal.
fn leaf(rng: &mut Rng, s: Sig, t: Ty) -> E {
    let own: Vec<_> = s.vars().into_iter().filter(|(_, vt)| *vt == t).collect();
    if !own.is_empty() && rng.chance(65) {
        let (n, vt) = own[rng.below(own.len())];
        return E::Var(n, vt);
    }
    if rng.chance(25) {
        return conv_leaf(rng, s, t);
    }
    E::Lit(value(rng, t), t)
}

/// A term of type `t` with a type of its own.
fn typed_leaf(rng: &mut Rng, s: Sig, t: Ty) -> E {
    let own: Vec<_> = s.vars().into_iter().filter(|(_, vt)| *vt == t).collect();
    if !own.is_empty() && rng.chance(80) {
        let (n, vt) = own[rng.below(own.len())];
        return E::Var(n, vt);
    }
    conv_leaf(rng, s, t)
}

fn conv_leaf(rng: &mut Rng, s: Sig, t: Ty) -> E {
    let (n, vt) = s.vars()[rng.below(3)];
    let src = Box::new(E::Var(n, vt));
    if rng.chance(70) {
        E::Conv(t, src)
    } else {
        E::ConvOk(t, src, Box::new(leaf(rng, s, t)), binding())
    }
}

/// `e` if it has a type of its own, else `e` combined with one.
fn ensure_typed(rng: &mut Rng, s: Sig, e: E) -> E {
    if e.typed() {
        e
    } else {
        let t = e.ty();
        E::Bin(Bin::Xor, Box::new(typed_leaf(rng, s, t)), Box::new(e))
    }
}

/// A term of type `t` with a type of its own.
fn typed_gen(rng: &mut Rng, s: Sig, t: Ty, depth: usize) -> E {
    let e = gen_e(rng, s, t, depth);
    ensure_typed(rng, s, e)
}

/// A shift amount for a value of width `w`: a literal below the width
/// (no obligation), at or beyond it, or a term of any type.
fn amount(rng: &mut Rng, s: Sig, w: u32, depth: usize) -> E {
    match rng.below(10) {
        0..=3 => E::Lit(rng.below(w as usize) as i128, Ty::U32),
        4 => E::Lit(
            [w, w + 1, 2 * w, 63, 64, 100][rng.below(6)] as i128,
            Ty::U32,
        ),
        5..=7 => E::Var("k", s.k),
        _ => {
            let t = if rng.chance(60) {
                s.k
            } else {
                ALL[rng.below(ALL.len())]
            };
            typed_gen(rng, s, t, depth.min(1))
        }
    }
}

/// A `u32` method amount: a literal (any `u32`) or a `u32` term.
fn u32_amount(rng: &mut Rng, s: Sig, depth: usize) -> E {
    if rng.chance(40) {
        let v = [
            0,
            1,
            7,
            8,
            15,
            16,
            31,
            32,
            33,
            63,
            64,
            65,
            1000,
            u32::MAX as i128,
        ];
        E::Lit(v[rng.below(v.len())], Ty::U32)
    } else {
        typed_gen(rng, s, Ty::U32, depth.min(1))
    }
}

fn gen_e(rng: &mut Rng, s: Sig, t: Ty, depth: usize) -> E {
    if depth == 0 || rng.chance(20) {
        return leaf(rng, s, t);
    }
    let d = depth - 1;
    let b = |rng: &mut Rng| Box::new(gen_e(rng, s, t, d));
    let typed = |rng: &mut Rng| Box::new(typed_gen(rng, s, t, d));
    match rng.below(20) {
        0..=5 => {
            let op = [
                Bin::Add,
                Bin::Sub,
                Bin::Mul,
                Bin::Div,
                Bin::Rem,
                Bin::And,
                Bin::Or,
                Bin::Xor,
            ][rng.below(8)];
            let (l, r) = (b(rng), b(rng));
            let l = if l.typed() || r.typed() {
                l
            } else {
                Box::new(typed_leaf(rng, s, t))
            };
            E::Bin(op, l, r)
        }
        6..=8 => {
            let a = typed(rng);
            E::Shift(rng.chance(50), a, Box::new(amount(rng, s, t.bits(), d)))
        }
        9 => E::Not(typed(rng)),
        10 if t.signed() => E::Neg(typed(rng)),
        11 => {
            let m = [Meth::WrapAdd, Meth::WrapSub, Meth::WrapMul][rng.below(3)];
            E::Method(m, typed(rng), b(rng))
        }
        12 | 13 => {
            let m = [Meth::Rotl, Meth::Rotr, Meth::Wshl, Meth::Wshr][rng.below(4)];
            E::Method(m, typed(rng), Box::new(u32_amount(rng, s, d)))
        }
        14 if t == Ty::U32 => {
            let c = [Count::Ones, Count::Zeros, Count::Leading, Count::Trailing][rng.below(4)];
            let from = ALL[rng.below(ALL.len())];
            E::Count(c, Box::new(typed_gen(rng, s, from, d)))
        }
        15 => E::SwapBytes(typed(rng)),
        16 => {
            // From another width.
            let from = ALL[rng.below(ALL.len())];
            let a = Box::new(typed_gen(rng, s, from, d));
            if rng.chance(60) {
                E::Conv(t, a)
            } else {
                E::ConvOk(t, a, b(rng), binding())
            }
        }
        17 | 18 => {
            let ct = if rng.chance(50) {
                t
            } else {
                ALL[rng.below(ALL.len())]
            };
            let ca = Box::new(typed_gen(rng, s, ct, d.min(1)));
            let cb = Box::new(gen_e(rng, s, ct, d.min(1)));
            let c = [Cmp::Lt, Cmp::Le, Cmp::Eq, Cmp::Ne][rng.below(4)];
            let (th, el) = (b(rng), b(rng));
            let th = if th.typed() || el.typed() {
                th
            } else {
                Box::new(typed_leaf(rng, s, t))
            };
            E::If(c, ca, cb, th, el)
        }
        _ => gen_e(rng, s, t, d),
    }
}

/// A literal of type `t` in program text.
fn lit_text(v: i128, t: Ty) -> String {
    if t.signed() && v == t.min() {
        format!("{}::MIN", t.name())
    } else if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

fn text(e: &E) -> String {
    match e {
        E::Var(n, _) => n.to_string(),
        E::Lit(v, t) => lit_text(*v, *t),
        E::Bin(op, a, b) => {
            let o = match op {
                Bin::Add => "+",
                Bin::Sub => "-",
                Bin::Mul => "*",
                Bin::Div => "/",
                Bin::Rem => "%",
                Bin::And => "&",
                Bin::Or => "|",
                Bin::Xor => "^",
            };
            format!("({} {o} {})", text(a), text(b))
        }
        E::Shift(left, a, b) => {
            let o = if *left { "<<" } else { ">>" };
            format!("({} {o} {})", text(a), text(b))
        }
        // Parenthesized: a method call binds tighter than a prefix.
        E::Not(a) => format!("(!{})", text(a)),
        E::Neg(a) => format!("(-{})", text(a)),
        E::Method(m, a, b) => {
            let n = match m {
                Meth::WrapAdd => "wrapping_add",
                Meth::WrapSub => "wrapping_sub",
                Meth::WrapMul => "wrapping_mul",
                Meth::Rotl => "rotate_left",
                Meth::Rotr => "rotate_right",
                Meth::Wshl => "wrapping_shl",
                Meth::Wshr => "wrapping_shr",
            };
            format!("{}.{n}({})", text(a), text(b))
        }
        E::Count(c, a) => {
            let n = match c {
                Count::Ones => "count_ones",
                Count::Zeros => "count_zeros",
                Count::Leading => "leading_zeros",
                Count::Trailing => "trailing_zeros",
            };
            format!("{}.{n}()", text(a))
        }
        E::SwapBytes(a) => format!("{}.swap_bytes()", text(a)),
        E::Conv(t, a) => format!("{}::try_from({}).unwrap()", t.name(), text(a)),
        E::ConvOk(t, a, d, v) => format!(
            "(match {}::try_from({}).ok() {{ None => {}, Some({v}) => {v} }})",
            t.name(),
            text(a),
            text(d)
        ),
        E::If(c, a, b, t, e) => {
            let o = match c {
                Cmp::Lt => "<",
                Cmp::Le => "<=",
                Cmp::Eq => "==",
                Cmp::Ne => "!=",
            };
            format!(
                "(if {} {o} {} {{ {} }} else {{ {} }})",
                text(a),
                text(b),
                text(t),
                text(e)
            )
        }
    }
}

/// A Rust panic (debug build): overflow, a shift amount outside `0..w`,
/// division by zero, `MIN / -1` and `MIN % -1`, an out-of-range
/// `unwrap`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Panic {
    Overflow,
    Shift,
    DivZero,
    MinDiv,
    Conv,
}

#[derive(Clone, Copy)]
struct Env {
    x: i128,
    y: i128,
    k: i128,
}

/// `v` if it is in `t`, else a panic.
fn fit(t: Ty, v: Option<i128>) -> Result<i128, Panic> {
    v.filter(|v| t.contains(*v)).ok_or(Panic::Overflow)
}

fn eval(e: &E, env: Env) -> Result<i128, Panic> {
    Ok(match e {
        E::Var("x", _) => env.x,
        E::Var("y", _) => env.y,
        E::Var(..) => env.k,
        E::Lit(v, _) => *v,
        E::Bin(op, a, b) => {
            let t = a.ty();
            let (a, b) = (eval(a, env)?, eval(b, env)?);
            let (pa, pb) = (t.pattern(a), t.pattern(b));
            match op {
                Bin::Add => fit(t, a.checked_add(b))?,
                Bin::Sub => fit(t, a.checked_sub(b))?,
                Bin::Mul => fit(t, a.checked_mul(b))?,
                // `i128`'s `/` and `%` truncate as Rust's do at every width.
                Bin::Div | Bin::Rem => {
                    if b == 0 {
                        return Err(Panic::DivZero);
                    }
                    if t.signed() && a == t.min() && b == -1 {
                        return Err(Panic::MinDiv);
                    }
                    if *op == Bin::Div { a / b } else { a % b }
                }
                Bin::And => t.of_pattern(pa & pb),
                Bin::Or => t.of_pattern(pa | pb),
                Bin::Xor => t.of_pattern(pa ^ pb),
            }
        }
        E::Shift(left, a, b) => {
            let t = a.ty();
            let (a, n) = (eval(a, env)?, eval(b, env)?);
            if n < 0 || n >= t.bits() as i128 {
                return Err(Panic::Shift);
            }
            if *left {
                t.of_pattern(t.pattern(a) << n)
            } else {
                // Arithmetic for a signed value (floor), logical for an
                // unsigned one.
                a >> n
            }
        }
        E::Not(a) => {
            let t = a.ty();
            t.of_pattern(!t.pattern(eval(a, env)?))
        }
        E::Neg(a) => {
            let t = a.ty();
            fit(t, Some(-eval(a, env)?))?
        }
        E::Method(m, a, b) => {
            let t = a.ty();
            let w = t.bits();
            let (a, b) = (eval(a, env)?, eval(b, env)?);
            let (pa, pb) = (t.pattern(a), t.pattern(b));
            match m {
                Meth::WrapAdd => t.of_pattern(pa.wrapping_add(pb)),
                Meth::WrapSub => t.of_pattern(pa.wrapping_sub(pb)),
                Meth::WrapMul => t.of_pattern(pa.wrapping_mul(pb)),
                Meth::Rotl | Meth::Rotr => {
                    let n = (b as u32) % w;
                    let n = if *m == Meth::Rotr { (w - n) % w } else { n };
                    if n == 0 {
                        a
                    } else {
                        t.of_pattern((pa << n) | (pa >> (w - n)))
                    }
                }
                Meth::Wshl => t.of_pattern(pa << (b as u32 & (w - 1))),
                Meth::Wshr => a >> (b as u32 & (w - 1)),
            }
        }
        E::Count(c, a) => {
            let t = a.ty();
            let w = t.bits();
            let p = t.pattern(eval(a, env)?);
            let ones = p.count_ones();
            i128::from(match c {
                Count::Ones => ones,
                Count::Zeros => w - ones,
                Count::Leading => w - (128 - p.leading_zeros()),
                Count::Trailing => p.trailing_zeros().min(w),
            })
        }
        E::SwapBytes(a) => {
            let t = a.ty();
            let p = t.pattern(eval(a, env)?);
            let n = t.bits() / 8;
            let mut r = 0u128;
            for i in 0..n {
                r |= ((p >> (8 * i)) & 0xff) << (8 * (n - 1 - i));
            }
            t.of_pattern(r)
        }
        E::Conv(t, a) => fit(*t, Some(eval(a, env)?)).map_err(|_| Panic::Conv)?,
        E::ConvOk(t, a, d, _) => {
            let v = eval(a, env)?;
            if t.contains(v) { v } else { eval(d, env)? }
        }
        E::If(c, a, b, th, el) => {
            let (a, b) = (eval(a, env)?, eval(b, env)?);
            let go = match c {
                Cmp::Lt => a < b,
                Cmp::Le => a <= b,
                Cmp::Eq => a == b,
                Cmp::Ne => a != b,
            };
            if go { eval(th, env)? } else { eval(el, env)? }
        }
    })
}

/// A conjunct of the postcondition (a specification: widths mix).
#[derive(Clone, Copy)]
enum Post {
    In(i128, i128),
    Le(&'static str),
    Ge(&'static str),
    Ne(&'static str),
}

struct Func {
    name: String,
    sig: Sig,
    ret: Ty,
    /// Inclusive bounds of `x`, `y`, `k`.
    pre: [(i128, i128); 3],
    body: E,
    post: Vec<Post>,
    /// Inputs start at the types' edges (`gen_shaped`).
    edges: bool,
}

impl Func {
    fn admits(&self, env: Env) -> bool {
        let v = [env.x, env.y, env.k];
        (0..3).all(|i| self.pre[i].0 <= v[i] && v[i] <= self.pre[i].1)
    }

    fn post_holds(&self, env: Env, r: i128) -> bool {
        let var = |n: &str| match n {
            "x" => env.x,
            "y" => env.y,
            _ => env.k,
        };
        self.post.iter().all(|c| match c {
            Post::In(lo, hi) => *lo <= r && r <= *hi,
            Post::Le(n) => r <= var(n),
            Post::Ge(n) => r >= var(n),
            Post::Ne(n) => r != var(n),
        })
    }

    /// Whether running at `env` panics or breaks the postcondition.
    fn broken(&self, env: Env) -> bool {
        match eval(&self.body, env) {
            Err(_) => true,
            Ok(r) => !self.post_holds(env, r),
        }
    }

    fn text(&self) -> String {
        let n = &self.name;
        let s = self.sig;
        let params = format!("x: {}, y: {}, k: {}", s.x.name(), s.y.name(), s.k.name());
        let pre = s
            .vars()
            .iter()
            .zip(self.pre)
            .map(|((v, t), (lo, hi))| {
                format!("{} <= {v} && {v} <= {}", lit_text(lo, *t), lit_text(hi, *t))
            })
            .collect::<Vec<_>>()
            .join(" && ");
        let r = self.ret;
        let post = self
            .post
            .iter()
            .map(|c| match c {
                Post::In(lo, hi) => {
                    format!("{} <= r && r <= {}", lit_text(*lo, r), lit_text(*hi, r))
                }
                Post::Le(v) => format!("r <= {v}"),
                Post::Ge(v) => format!("r >= {v}"),
                Post::Ne(v) => format!("r != {v}"),
            })
            .collect::<Vec<_>>()
            .join(" && ");
        format!(
            "    spec fn {n}_pre({params}) -> bool {{ {pre} }}\n\n    spec fn {n}_post({params}, r: {}) -> bool {{ {post} }}\n\n    #[effects(pure)]\n    #[uses({n}_pre, {n}_post)]\n    fn {n}({params}) -> (r: {})\n        requires {n}_pre(x, y, k),\n        ensures {n}_post(x, y, k, r),\n    {{\n        {}\n    }}\n\n",
            r.name(),
            r.name(),
            text(&self.body)
        )
    }

    /// Admitted inputs: the bounds, edge values and patterns inside them,
    /// random values.
    fn inputs(&self, rng: &mut Rng, n: usize) -> Vec<Env> {
        let pick = |rng: &mut Rng, i: usize, t: Ty| -> i128 {
            let (lo, hi) = self.pre[i];
            match rng.below(5) {
                0 => lo,
                1 => hi,
                2 => {
                    let mut v = interesting(t);
                    v.retain(|x| lo <= *x && *x <= hi);
                    if v.is_empty() {
                        lo
                    } else {
                        v[rng.below(v.len())]
                    }
                }
                _ => lo + (rng.next() as i128).rem_euclid(hi - lo + 1),
            }
        };
        let mut out: Vec<Env> = Vec::new();
        if self.edges {
            // The edges where widths differ come first: `MIN / -1`, a full
            // product, a negative value into an unsigned type.
            let (tx, ty) = (self.sig.x, self.sig.y);
            for (a, b) in [
                (tx.min(), -1),
                (tx.max(), ty.max()),
                (tx.min(), ty.min()),
                (-1, ty.min()),
                (tx.max(), -1),
                (tx.min(), 1),
            ] {
                let e = Env {
                    x: a,
                    y: b,
                    k: pick(rng, 2, self.sig.k),
                };
                if self.admits(e) && out.len() < n {
                    out.push(e);
                }
            }
        }
        for _ in 0..n * 5 {
            let e = Env {
                x: pick(rng, 0, self.sig.x),
                y: pick(rng, 1, self.sig.y),
                k: pick(rng, 2, self.sig.k),
            };
            if !out.iter().any(|o| o.x == e.x && o.y == e.y && o.k == e.k) {
                out.push(e);
            }
            if out.len() == n {
                break;
            }
        }
        out
    }
}

fn range(rng: &mut Rng, t: Ty) -> (i128, i128) {
    let (a, b) = (value(rng, t), value(rng, t));
    (a.min(b), a.max(b))
}

fn gen_func(rng: &mut Rng, k: usize, x: Ty) -> Func {
    let pick = |rng: &mut Rng| ALL[rng.below(ALL.len())];
    let y = if rng.chance(60) { x } else { pick(rng) };
    let kt = if rng.chance(40) { Ty::U32 } else { pick(rng) };
    let sig = Sig { x, y, k: kt };
    let ret = if rng.chance(70) { x } else { pick(rng) };
    let w = x.bits() as i128;
    // The amount: often below `x`'s width (shifts can be proved), else at
    // or beyond it, or anywhere in its type.
    let kr = match rng.below(10) {
        0..=4 => {
            let (a, b) = (rng.below(w as usize) as i128, rng.below(w as usize) as i128);
            (a.min(b), a.max(b))
        }
        5 | 6 => (0, w + rng.below(3) as i128),
        _ => range(rng, kt),
    };
    let kr = (
        kr.0.clamp(kt.min(), kt.max()),
        kr.1.clamp(kt.min(), kt.max()),
    );
    let body = typed_gen(rng, sig, ret, 3);
    let body = if body.has_var() {
        body
    } else {
        let xv = Box::new(E::Var("x", x));
        let xv = if x == ret {
            xv
        } else {
            Box::new(E::Conv(ret, xv))
        };
        E::Bin(Bin::Xor, xv, Box::new(body))
    };
    let mut post = Vec::new();
    for c in [Post::Le("x"), Post::Ge("y"), Post::Ne("k")] {
        if rng.chance(15) {
            post.push(c);
        }
    }
    if post.is_empty() || rng.chance(40) {
        post.push(if rng.chance(30) {
            Post::In(ret.min(), ret.max())
        } else {
            let (a, b) = range(rng, ret);
            Post::In(a, b)
        });
    }
    Func {
        name: format!("f{k}"),
        sig,
        ret,
        pre: [range(rng, x), range(rng, y), kr],
        body,
        post,
        edges: false,
    }
}

/// The kinds of `gen_shaped` functions, taken in turn.
const SHAPES: usize = 8;

/// A function whose body is one operation on `x` of a kind whose code
/// differs by width (`shape`), with a postcondition that always holds: its
/// verdict rests on the obligations alone, its output and traps on the code
/// generator.
fn gen_shaped(rng: &mut Rng, k: usize, shape: usize) -> Func {
    let pick = |rng: &mut Rng, from: &[Ty]| from[rng.below(from.len())];
    let narrow = [Ty::I8, Ty::I16, Ty::I32, Ty::U8, Ty::U16, Ty::U32];
    let signed = [Ty::I8, Ty::I16, Ty::I32, Ty::I64, Ty::Isize];
    // `MIN / -1` is a signed type's, a conversion's width matters below 64
    // bits; the rest take any type.
    let x = match shape {
        4 => pick(rng, &signed),
        6 => pick(rng, &narrow),
        _ => pick(rng, &ALL),
    };
    let mut f = gen_func(rng, k, x);
    let s = f.sig;
    let xv = Box::new(E::Var("x", x));
    let kv = Box::new(E::Var("k", s.k));
    let w = x.bits();
    let lit = |rng: &mut Rng| Box::new(E::Lit(rng.below(w as usize) as i128, Ty::U32));
    let full = |t: Ty| (t.min(), t.max());
    f.ret = x;
    f.body = match shape {
        // A literal amount of exactly the width: no input is admitted.
        0 => E::Shift(rng.chance(50), xv, Box::new(E::Lit(w as i128, Ty::U32))),
        // A variable amount, in its whole type (negative, at or beyond the
        // width) or below the width.
        1 => {
            if rng.chance(50) {
                f.pre[2] = full(s.k);
            }
            E::Shift(rng.chance(50), xv, kv)
        }
        // A narrow `<<` truncates and extends again, then `ashr`/`lshr`.
        2 => E::Shift(false, Box::new(E::Shift(true, xv, lit(rng))), lit(rng)),
        3 => match rng.below(3) {
            0 => E::Not(xv),
            1 => E::SwapBytes(xv),
            _ => {
                let m = [Meth::Rotl, Meth::Rotr, Meth::Wshl, Meth::Wshr][rng.below(4)];
                E::Method(m, xv, Box::new(u32_amount(rng, s, 0)))
            }
        },
        // Arithmetic over the whole type: `/`, `%` (`MIN / -1`), and the
        // narrow `+ - *` computed in 64 bits and range-checked.
        4 | 5 => {
            f.sig.y = x;
            f.pre[0] = full(x);
            f.pre[1] = full(x);
            f.edges = true;
            let op = if shape == 4 {
                [Bin::Div, Bin::Rem][rng.below(2)]
            } else {
                [Bin::Mul, Bin::Add, Bin::Sub][rng.below(3)]
            };
            E::Bin(op, xv, Box::new(E::Var("y", x)))
        }
        // A conversion from another width, over its whole type, unwrapped
        // or through `.ok()`.
        6 => {
            f.pre[1] = full(s.y);
            f.edges = true;
            let yv = Box::new(E::Var("y", s.y));
            if rng.chance(70) {
                E::Conv(x, yv)
            } else {
                E::ConvOk(x, yv, Box::new(E::Lit(0, x)), binding())
            }
        }
        // A count of `x`'s bits (`u32`).
        _ => {
            f.ret = Ty::U32;
            let c = [Count::Ones, Count::Zeros, Count::Leading, Count::Trailing][rng.below(4)];
            E::Count(c, xv)
        }
    };
    f.post = vec![Post::In(f.ret.min(), f.ret.max())];
    f
}

/// The values after "counterexample: ".
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
    programs: usize,
    funcs: usize,
    verified: usize,
    verified_var_shift: usize,
    verified_bit_method: usize,
    verified_by_width: [usize; 10],
    interpreted: usize,
    cexes: usize,
    undecided: usize,
    compared: usize,
    traps: usize,
}

/// A call of `f` at `env` in `main`, printing its result.
fn call_text(f: &Func, env: Env) -> String {
    let s = f.sig;
    let args = format!(
        "{}, {}, {}",
        lit_text(env.x, s.x),
        lit_text(env.y, s.y),
        lit_text(env.k, s.k)
    );
    if f.ret.signed() {
        format!(
            "        print_i64(i64::try_from(m::{}({args})).unwrap());\n",
            f.name
        )
    } else {
        format!(
            "        print_u64(u64::try_from(m::{}({args})).unwrap());\n",
            f.name
        )
    }
}

struct Built {
    output: String,
    ir: Option<String>,
}

fn emit(file: &Path, cache: &Path, all_checks: bool) -> Built {
    let ll = file.with_extension(if all_checks { "all.ll" } else { "ll" });
    let _ = fs::remove_file(&ll);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
    cmd.arg("emit-llvm");
    if all_checks {
        cmd.arg("--all-checks");
    } else {
        cmd.arg("--cache").arg(cache);
    }
    let out = cmd.arg(file).arg("-o").arg(&ll).output().unwrap();
    let ir = fs::read_to_string(&ll).ok();
    let _ = fs::remove_file(&ll);
    Built {
        output: String::from_utf8_lossy(&out.stdout).into_owned(),
        ir,
    }
}

/// Builds `ir` and runs it: (stdout, the signal that ended it, if any,
/// and whether it exited with 0).
fn run(ir: &str, exe: &Path) -> Result<(String, Option<i32>, bool), String> {
    let ll = exe.with_extension("build.ll");
    fs::write(&ll, ir).unwrap();
    let cc = Command::new("clang")
        .args(["-O2", "-Wno-override-module", "-o"])
        .arg(exe)
        .arg(&ll)
        .output()
        .unwrap();
    let _ = fs::remove_file(&ll);
    if !cc.status.success() {
        return Err(String::from_utf8_lossy(&cc.stderr).into_owned());
    }
    let r = Command::new(exe).output().unwrap();
    let _ = fs::remove_file(exe);
    Ok((
        String::from_utf8_lossy(&r.stdout).into_owned(),
        r.status.signal(),
        r.status.success(),
    ))
}

/// Traps per batch run as programs of their own (one trap ends a run).
const TRAPS: usize = 3;

fn batch(seed: u64, stats: &mut Stats, dir: &Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    // Every width in turn, both signednesses in every batch.
    let funcs: Vec<Func> = (0..8)
        .map(|k| {
            if k < 3 {
                gen_shaped(&mut rng, k, (seed as usize * 3 + k) % SHAPES)
            } else {
                gen_func(&mut rng, k, ALL[(seed as usize * 8 + k) % ALL.len()])
            }
        })
        .collect();
    let mut module = String::from("#[summary(\"정수 폭과 비트 연산 건전성 퍼징\")]\nmod m {\n");
    for f in &funcs {
        module.push_str(&f.text());
    }
    module.push_str("}\n\n");
    let uses = funcs
        .iter()
        .map(|f| format!("m::{}_pre", f.name))
        .collect::<Vec<_>>()
        .join(", ");
    let main_head = format!(
        "#[summary(\"실행\")]\nmod app {{\n    #[effects(io)]\n    #[uses({uses})]\n    fn main() -> () {{\n"
    );
    // Printed calls (the interpreter's value) and trapping ones.
    let mut calls = String::new();
    let mut expected = String::new();
    let mut traps: Vec<(usize, Env, Panic)> = Vec::new();
    for (i, f) in funcs.iter().enumerate() {
        for env in f.inputs(&mut rng, 6) {
            match eval(&f.body, env) {
                Ok(r) => {
                    calls.push_str(&call_text(f, env));
                    let _ = writeln!(expected, "{r}");
                }
                Err(p) => traps.push((i, env, p)),
            }
        }
    }
    let src = format!("{module}{main_head}{calls}    }}\n}}\n");
    let file = dir.join(format!("w{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    let cache = dir.join(format!("cache{seed}"));
    let built = emit(&file, &cache, false);
    let output = built.output.clone();
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{output}");
    assert!(
        built.ir.is_some()
            && output
                .lines()
                .filter_map(|l| field(l, "code"))
                .all(|c| c.starts_with("E08") || c.starts_with('W')),
        "{}",
        ctx("the generated program must check (only proof failures)")
    );
    assert!(
        output
            .lines()
            .any(|l| field(l, "def") == Some("app::main") && field(l, "status") == Some("verified")),
        "{}",
        ctx("main must be proved (its calls are admitted)")
    );
    stats.programs += 1;

    // The prover's verdicts against the interpreter.
    for (i, f) in funcs.iter().enumerate() {
        let name = format!("m::{}", f.name);
        stats.funcs += 1;
        let status = output
            .lines()
            .find(|l| field(l, "def") == Some(name.as_str()))
            .and_then(|l| field(l, "status"))
            .unwrap_or("missing");
        if status == "verified" {
            stats.verified += 1;
            stats.verified_var_shift += usize::from(f.body.var_shift());
            stats.verified_bit_method += usize::from(f.body.bit_method());
            stats.verified_by_width[ALL.iter().position(|t| *t == f.sig.x).unwrap()] += 1;
            for env in f.inputs(&mut rng, 40) {
                stats.interpreted += 1;
                match eval(&f.body, env) {
                    Err(_) => panic!(
                        "{}",
                        ctx(&format!(
                            "verified {name} panics at x = {}, y = {}, k = {}",
                            env.x, env.y, env.k
                        ))
                    ),
                    Ok(r) => assert!(
                        f.post_holds(env, r),
                        "{}",
                        ctx(&format!(
                            "verified {name}({}, {}, {}) = {r} breaks its postcondition",
                            env.x, env.y, env.k
                        ))
                    ),
                }
            }
            assert!(
                !traps.iter().any(|t| t.0 == i),
                "{}",
                ctx(&format!("verified {name} panics at a printed input"))
            );
            continue;
        }
        for msg in output
            .lines()
            .filter_map(|l| field(l, "message"))
            .filter(|m| m.contains(&format!("in `{name}`")))
        {
            if msg.contains("could not decide") {
                stats.undecided += 1;
                continue;
            }
            let cex = parse_cex(msg)
                .unwrap_or_else(|| panic!("{}", ctx(&format!("{name}: no counterexample: {msg}"))));
            let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, x)| *x);
            // A parameter the model does not show may take any admitted
            // value: both bounds must break the function.
            let vals = |i: usize, v: &str| -> Vec<i128> {
                get(v).map_or(vec![f.pre[i].0, f.pre[i].1], |x| vec![x])
            };
            stats.cexes += 1;
            for &x in &vals(0, "x") {
                for &y in &vals(1, "y") {
                    for &k in &vals(2, "k") {
                        let env = Env { x, y, k };
                        assert!(
                            f.admits(env),
                            "{}",
                            ctx(&format!("{name}: counterexample not admitted: {msg}"))
                        );
                        assert!(
                            f.broken(env),
                            "{}",
                            ctx(&format!(
                                "{name}: the counterexample x = {x}, y = {y}, k = {k} breaks nothing: {msg}"
                            ))
                        );
                    }
                }
            }
        }
    }

    // Both builds against the interpreter's output.
    let n = expected.lines().count();
    let all = emit(&file, &cache, true);
    for (all_checks, ir) in [(false, built.ir.clone()), (true, all.ir)] {
        let ir = ir.unwrap_or_else(|| panic!("{}", ctx("no IR from --all-checks")));
        let exe = dir.join(format!("w{seed}-{all_checks}"));
        let (out, sig, ok) = run(&ir, &exe).unwrap_or_else(|e| panic!("{}", ctx(&e)));
        assert!(
            ok,
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) ended by signal {sig:?}"
            ))
        );
        assert_eq!(
            out,
            expected,
            "{}",
            ctx(&format!(
                "build (all_checks = {all_checks}) disagrees with the interpreter"
            ))
        );
        stats.compared += n;
    }

    // Calls that panic in Rust trap in both builds.
    // Each kind of panic once, then other functions.
    let mut chosen: Vec<(usize, Env, Panic)> = Vec::new();
    for t in &traps {
        if chosen.len() < TRAPS && !chosen.iter().any(|c| c.2 == t.2 || c.0 == t.0) {
            chosen.push(*t);
        }
    }
    for t in &traps {
        if chosen.len() < TRAPS && !chosen.iter().any(|c| c.0 == t.0) {
            chosen.push(*t);
        }
    }
    for (j, (i, env, _)) in chosen.into_iter().enumerate() {
        let f = &funcs[i];
        let src = format!(
            "{module}{main_head}{}        print_i64(2);\n    }}\n}}\n",
            call_text(f, env)
        );
        let file = dir.join(format!("w{seed}-trap{j}.bmb"));
        fs::write(&file, &src).unwrap();
        for all_checks in [false, true] {
            let b = emit(&file, &cache, all_checks);
            let ir = b
                .ir
                .unwrap_or_else(|| panic!("{}", ctx(&format!("no IR for the trap of {}", f.name))));
            let exe = dir.join(format!("w{seed}-trap{j}-{all_checks}"));
            let (out, sig, _) = run(&ir, &exe).unwrap_or_else(|e| panic!("{}", ctx(&e)));
            assert!(
                out.is_empty() && sig == Some(4),
                "{}",
                ctx(&format!(
                    "m::{}({}, {}, {}) panics in Rust, but the build (all_checks = {all_checks}) printed {out:?} and ended by signal {sig:?}",
                    f.name, env.x, env.y, env.k
                ))
            );
            stats.traps += 1;
        }
        let _ = fs::remove_file(&file);
    }
    let _ = fs::remove_file(&file);
}

#[test]
fn widths_and_bit_operations_agree_with_the_interpreter() {
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let dir = std::env::temp_dir().join(format!("bmb-sound-widths-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    let widths = ALL
        .iter()
        .zip(stats.verified_by_width)
        .map(|(t, n)| format!("{} {n}", t.name()))
        .collect::<Vec<_>>()
        .join(", ");
    eprintln!(
        "soundness (widths): {} programs, {} functions, {} verified ({widths}; {} with a variable shift, {} with a bit method; {} interpreted runs), {} counterexamples replayed, {} undecided, {} values compared over both builds, {} traps confirmed",
        stats.programs,
        stats.funcs,
        stats.verified,
        stats.verified_var_shift,
        stats.verified_bit_method,
        stats.interpreted,
        stats.cexes,
        stats.undecided,
        stats.compared,
        stats.traps
    );
    assert!(stats.verified > 0 && stats.cexes > 0 && stats.traps > 0);
}
