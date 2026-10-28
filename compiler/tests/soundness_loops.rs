//! Soundness fuzzing of loops, arrays and `&mut` (backlog 1B-21).
//!
//! Random functions `g(a: &mut [i64], k: usize, x: i64) -> i64` with one
//! loop (`while`, or the range `for` when the loop counts), random
//! invariants and random contracts are proved, and the claims are checked
//! against an independent interpreter written here. The body may leave
//! early (backlog 2K-05): `break`, `continue`, `return e`, and `?` on a
//! callee's `Option` (the function then returns `Option<i64>`).
//!
//! - a verified function, on any input its precondition admits, never faults
//!   (index out of bounds, overflow), keeps its loop invariant at every loop
//!   head, and meets its postcondition;
//! - every counterexample satisfies the precondition, and one for an
//!   invariant that fails on entry really fails on entry (counterexamples for
//!   later obligations come from an arbitrary loop state, so only those before
//!   the loop are replayed);
//! - both builds (proved checks removed, every check kept) print what the
//!   interpreter computes, array contents included.
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len())]
    }
}

fn lit(v: i64) -> String {
    if v == i64::MIN {
        "(-9223372036854775807 - 1)".to_string()
    } else if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

#[derive(Debug)]
struct Fault;

// ---------------------------------------------------------------------------
// Executable parts.

#[derive(Clone, Copy, PartialEq)]
enum Idx {
    I,
    IPlus1,
    IMinus1,
    K,
    Zero,
    /// `a.len() - 1 - i`
    Mirror,
}

enum E {
    X,
    Acc,
    Lit(i64),
    Elem(Idx),
    Add(Box<E>, Box<E>),
    Sub(Box<E>, Box<E>),
    Mul(Box<E>, Box<E>),
    WrapAdd(Box<E>, Box<E>),
}

#[derive(Clone, Copy)]
enum Cmp {
    Lt,
    Le,
    Eq,
    Ne,
}

enum S {
    SetAcc(E),
    Write(Idx, E),
    IfWrite(Cmp, E, E, Idx, E),
    /// `if l c r { [pre] <exit> }`: what the branch writes first makes the
    /// exit's state differ from the fall-through's.
    Exit(Exit, Cmp, E, E, Option<Pre>),
    /// `acc = lim(e, c)?;`: `None` when `e > c`.
    Try(E, i64),
}

enum Exit {
    Break,
    /// `continue` (a `while` steps `i` first)
    Continue,
    Return(E),
    /// `acc = lim(e, c)?;`
    Try(E, i64),
}

/// A write before an exit.
enum Pre {
    Acc(E),
    Write(Idx, E),
}

#[derive(Clone, Copy, PartialEq)]
enum Form {
    While,
    For,
}

/// What the body did.
enum Flow {
    Next,
    Break,
    Return(Out),
}

/// The function's result: a value, or `None` from a `?`.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Out {
    Val(i64),
    None,
}

#[derive(Clone, Copy, PartialEq)]
enum Cond {
    ILtLen,
    IPlus1LtLen,
    ILtK,
}

#[derive(Clone, Copy)]
enum Start {
    Zero,
    One,
    K,
}

#[derive(Clone, Copy)]
enum Ret {
    Acc,
    X,
    Elem0,
    ElemK,
}

// ---------------------------------------------------------------------------
// Specification parts (mathematical integers).

#[derive(Clone, Copy)]
enum Clause {
    /// a.len() >= n
    LenGe(u64),
    /// a.len() <= n
    LenLe(u64),
    /// k <= a.len()
    KLeLen,
    /// k < a.len()
    KLtLen,
    /// lo <= a[j] <= hi for every j
    Elems(i64, i64),
    /// lo <= x <= hi
    XIn(i64, i64),
    /// i <= a.len() (invariants only)
    ILeLen,
    /// i <= k (invariants only)
    ILeK,
    /// -c * i <= acc <= c * i (invariants only)
    AccIn(i64),
    /// a.len() == old(a).len()
    SameLen,
    /// lo <= r <= hi (postconditions only)
    RIn(i64, i64),
    /// r == x (postconditions only)
    REqX,
    /// a.len() == 0 || a[0] == old(a)[0]
    FirstKept,
}

fn clause_text(c: Clause) -> String {
    match c {
        Clause::LenGe(n) => format!("a.len() >= {n}"),
        Clause::LenLe(n) => format!("a.len() <= {n}"),
        Clause::KLeLen => "k <= a.len()".into(),
        Clause::KLtLen => "k < a.len()".into(),
        Clause::Elems(lo, hi) => format!(
            "(forall|j: usize| j < a.len() ==> ({} <= #[trigger] a[j] && a[j] <= {}))",
            lit(lo),
            lit(hi)
        ),
        Clause::XIn(lo, hi) => format!("{} <= x && x <= {}", lit(lo), lit(hi)),
        Clause::ILeLen => "i <= a.len()".into(),
        Clause::ILeK => "i <= k".into(),
        Clause::AccIn(c) => format!("{} * i <= acc && acc <= {} * i", lit(-c), lit(c)),
        Clause::SameLen => "a.len() == b.len()".into(),
        Clause::RIn(lo, hi) => format!("{} <= r && r <= {}", lit(lo), lit(hi)),
        Clause::REqX => "r == x".into(),
        Clause::FirstKept => "(a.len() == 0 || a[0] == b[0])".into(),
    }
}

/// State a clause is evaluated in. `b` is `old(a)`.
struct View<'s> {
    a: &'s [i64],
    b: &'s [i64],
    k: u64,
    x: i64,
    i: u64,
    acc: i64,
    r: i64,
}

fn clause_eval(c: Clause, v: &View) -> bool {
    let len = v.a.len() as i128;
    match c {
        Clause::LenGe(n) => len >= n as i128,
        Clause::LenLe(n) => len <= n as i128,
        Clause::KLeLen => (v.k as i128) <= len,
        Clause::KLtLen => (v.k as i128) < len,
        Clause::Elems(lo, hi) => v.a.iter().all(|&e| lo <= e && e <= hi),
        Clause::XIn(lo, hi) => lo <= v.x && v.x <= hi,
        Clause::ILeLen => (v.i as i128) <= len,
        Clause::ILeK => v.i <= v.k,
        Clause::AccIn(c) => {
            let bound = c as i128 * v.i as i128;
            -bound <= v.acc as i128 && v.acc as i128 <= bound
        }
        Clause::SameLen => v.a.len() == v.b.len(),
        Clause::RIn(lo, hi) => lo <= v.r && v.r <= hi,
        Clause::REqX => v.r == v.x,
        Clause::FirstKept => v.a.is_empty() || v.a[0] == v.b[0],
    }
}

fn all(cs: &[Clause], v: &View) -> bool {
    cs.iter().all(|&c| clause_eval(c, v))
}

fn conj(cs: &[Clause]) -> String {
    if cs.is_empty() {
        // Never generated: an empty predicate body would be vacuous (E0502).
        unreachable!()
    }
    cs.iter()
        .map(|&c| clause_text(c))
        .collect::<Vec<_>>()
        .join(" && ")
}

// ---------------------------------------------------------------------------

struct Func {
    name: String,
    pre: Vec<Clause>,
    inv: Vec<Clause>,
    post: Vec<Clause>,
    start: Start,
    cond: Cond,
    body: Vec<S>,
    ret: Ret,
    form: Form,
    /// Returns `Option<i64>` (the body may use `?`).
    opt: bool,
}

fn gen_cmp(rng: &mut Rng) -> Cmp {
    rng.pick(&[Cmp::Lt, Cmp::Le, Cmp::Eq, Cmp::Ne])
}

/// An early exit, or a `?` when the function returns an `Option`.
fn gen_exit(rng: &mut Rng, opt: bool) -> S {
    let lim_c = |rng: &mut Rng| rng.pick(&[0, 10, 1000, -7]);
    if opt && rng.chance(25) {
        return S::Try(gen_e(rng, 1), lim_c(rng));
    }
    let exit = match rng.below(if opt { 4 } else { 3 }) {
        0 => Exit::Break,
        1 => Exit::Continue,
        2 => Exit::Return(gen_e(rng, 1)),
        _ => Exit::Try(gen_e(rng, 1), lim_c(rng)),
    };
    let pre = match rng.below(3) {
        0 => Some(Pre::Acc(gen_e(rng, 1))),
        1 => Some(Pre::Write(gen_idx(rng), gen_e(rng, 1))),
        _ => None,
    };
    S::Exit(exit, gen_cmp(rng), gen_e(rng, 1), gen_e(rng, 1), pre)
}

/// The loop form, `?`s and early exits added to a generated function.
fn add_exits(rng: &mut Rng, f: &mut Func) {
    if f.cond != Cond::IPlus1LtLen && rng.chance(50) {
        f.form = Form::For;
    }
    f.opt = f.opt || rng.chance(25);
    if f.opt || rng.chance(40) {
        let s = gen_exit(rng, f.opt);
        let at = rng.below(f.body.len() + 1);
        f.body.insert(at, s);
    }
}

fn gen_idx(rng: &mut Rng) -> Idx {
    rng.pick(&[
        Idx::I,
        Idx::I,
        Idx::I,
        Idx::IPlus1,
        Idx::IMinus1,
        Idx::K,
        Idx::Zero,
        Idx::Mirror,
    ])
}

fn small(rng: &mut Rng) -> i64 {
    rng.pick(&[
        0,
        1,
        2,
        3,
        7,
        10,
        100,
        1000,
        -1,
        -2,
        -7,
        -100,
        -1000,
        i64::MAX,
        i64::MIN,
    ])
}

fn gen_e(rng: &mut Rng, depth: usize) -> E {
    if depth == 0 || rng.chance(35) {
        return match rng.below(4) {
            0 => E::X,
            1 => E::Acc,
            2 => E::Lit(small(rng)),
            _ => E::Elem(gen_idx(rng)),
        };
    }
    let (a, b) = (
        Box::new(gen_e(rng, depth - 1)),
        Box::new(gen_e(rng, depth - 1)),
    );
    match rng.below(5) {
        0 | 1 => E::Add(a, b),
        2 => E::Sub(a, b),
        3 => E::Mul(a, b),
        _ => E::WrapAdd(a, b),
    }
}

fn bounds_pair(rng: &mut Rng) -> (i64, i64) {
    let (a, b) = (small(rng), small(rng));
    (a.min(b), a.max(b))
}

/// Correct-by-construction loops (most prove), sometimes with one mutation
/// that may or may not break them: near misses are where a verifier errs.
fn gen_template(rng: &mut Rng, n: usize) -> Func {
    let kind = rng.below(5);
    let mut f = match kind {
        // Sum of bounded elements (with a near miss when `kind` is 4).
        0 | 4 => sum_template(),
        // Fill with x; the first element afterwards is x (or the array is empty).
        1 => Func {
            name: String::new(),
            pre: vec![Clause::XIn(-1000, 1000)],
            inv: vec![Clause::ILeLen, Clause::SameLen],
            post: vec![Clause::SameLen, Clause::REqX],
            start: Start::Zero,
            cond: Cond::ILtLen,
            body: vec![S::Write(Idx::I, E::X)],
            ret: Ret::X,
            form: Form::While,
            opt: false,
        },
        // Shift left by one from position 1; the first element is kept only
        // when the loop starts at 1.
        2 => Func {
            name: String::new(),
            pre: vec![Clause::LenGe(1), Clause::XIn(-1000, 1000)],
            inv: vec![Clause::ILeLen, Clause::SameLen, Clause::FirstKept],
            post: vec![Clause::SameLen, Clause::FirstKept],
            start: Start::One,
            cond: Cond::IPlus1LtLen,
            body: vec![S::Write(Idx::I, E::Elem(Idx::IPlus1))],
            ret: Ret::Elem0,
            form: Form::While,
            opt: false,
        },
        // Clamp the first k elements to x when bigger; returns a[k].
        _ => Func {
            name: String::new(),
            pre: vec![Clause::KLtLen, Clause::XIn(-1000, 1000)],
            inv: vec![Clause::ILeK, Clause::KLeLen, Clause::SameLen],
            post: vec![Clause::SameLen],
            start: Start::Zero,
            cond: Cond::ILtK,
            body: vec![S::IfWrite(Cmp::Lt, E::X, E::Elem(Idx::I), Idx::I, E::X)],
            ret: Ret::ElemK,
            form: Form::While,
            opt: false,
        },
    };
    f.name = format!("g{n}");
    if rng.chance(40) {
        match rng.below(6) {
            0 => {
                let k = rng.below(f.inv.len());
                f.inv.remove(k);
                if f.inv.is_empty() {
                    f.inv.push(Clause::SameLen);
                }
            }
            1 => {
                let k = rng.below(f.pre.len());
                f.pre.remove(k);
                if f.pre.is_empty() {
                    f.pre.push(Clause::XIn(i64::MIN, i64::MAX));
                }
            }
            2 => f.start = rng.pick(&[Start::Zero, Start::One, Start::K]),
            3 => f.cond = rng.pick(&[Cond::ILtLen, Cond::IPlus1LtLen, Cond::ILtK]),
            4 => {
                let i = gen_idx(rng);
                f.body = match f.body.pop().unwrap() {
                    S::Write(_, e) => vec![S::Write(i, e)],
                    S::SetAcc(_) => vec![S::SetAcc(E::Add(Box::new(E::Acc), Box::new(E::Elem(i))))],
                    S::IfWrite(c, l, _, _, e) => vec![S::IfWrite(c, l, E::Elem(i), i, e)],
                    s => vec![s],
                }
            }
            _ => {
                f.post
                    .push(rng.pick(&[Clause::REqX, Clause::FirstKept, Clause::RIn(-1000, 1000)]))
            }
        }
    }
    if kind == 4 {
        let which = rng.below(4);
        kick(rng, &mut f, which);
    }
    add_exits(rng, &mut f);
    f
}

/// A near miss on the sum: on an element of a sign the inputs often hold,
/// push `acc` far past the invariant and the result bound, then leave.
/// Each exit's own obligation fails; a prover that skips it is caught when
/// the interpreter takes that branch.
fn kick(rng: &mut Rng, f: &mut Func, which: usize) {
    let hit = if rng.chance(50) {
        (Cmp::Lt, E::Elem(Idx::I), E::Lit(0))
    } else {
        (Cmp::Le, E::Lit(0), E::Elem(Idx::I))
    };
    let far = || E::Add(Box::new(E::Acc), Box::new(E::Lit(1_500_000_000)));
    let (exit, pre) = match which {
        0 => (Exit::Continue, Some(Pre::Acc(far()))),
        1 => (Exit::Break, Some(Pre::Acc(far()))),
        2 => (Exit::Return(far()), None),
        _ => {
            // `None` after changing `a[0]`: the `None` side keeps `a[0]`
            // (the loop keeps it otherwise, and says so).
            f.opt = true;
            f.post.push(Clause::FirstKept);
            f.inv.push(Clause::FirstKept);
            (
                Exit::Try(E::Acc, -1_000_000_000_000),
                Some(Pre::Write(Idx::Zero, E::Lit(99))),
            )
        }
    };
    f.post.push(Clause::RIn(-1_000_000_000, 1_000_000_000));
    let at = rng.below(f.body.len() + 1);
    f.body.insert(at, S::Exit(exit, hit.0, hit.1, hit.2, pre));
}

fn sum_template() -> Func {
    Func {
        name: String::new(),
        pre: vec![
            Clause::Elems(-1000, 1000),
            Clause::XIn(-1000, 1000),
            Clause::LenLe(1_000_000),
        ],
        inv: vec![
            Clause::LenLe(1_000_000),
            Clause::ILeLen,
            Clause::AccIn(1000),
            Clause::Elems(-1000, 1000),
            Clause::SameLen,
        ],
        post: vec![Clause::SameLen],
        start: Start::Zero,
        cond: Cond::ILtLen,
        body: vec![S::SetAcc(E::Add(
            Box::new(E::Acc),
            Box::new(E::Elem(Idx::I)),
        ))],
        ret: Ret::Acc,
        form: Form::While,
        opt: false,
    }
}

/// The near miss of exit kind `seed % 4` in every batch, in a `while` for
/// the first four seeds of each eight and a `for` for the others, so that a
/// few seeds already cover `continue`, `break`, `return` and `?`.
fn kicked_sum(rng: &mut Rng, seed: u64) -> Func {
    let mut f = sum_template();
    f.name = "g0".into();
    if (seed / 4) % 2 == 1 {
        f.form = Form::For;
    }
    kick(rng, &mut f, (seed % 4) as usize);
    f
}

fn gen_func(rng: &mut Rng, n: usize) -> Func {
    if rng.chance(60) {
        return gen_template(rng, n);
    }
    let cond = rng.pick(&[Cond::ILtLen, Cond::ILtLen, Cond::IPlus1LtLen, Cond::ILtK]);
    let start = rng.pick(&[Start::Zero, Start::Zero, Start::One, Start::K]);
    let mut pre = Vec::new();
    if rng.chance(50) {
        pre.push(Clause::LenGe(rng.below(3) as u64));
    }
    match rng.below(3) {
        0 => pre.push(Clause::KLeLen),
        1 => pre.push(Clause::KLtLen),
        _ => {}
    }
    if rng.chance(50) {
        let (lo, hi) = rng.pick(&[(-1000, 1000), (0, 100), (-7, 7), (0, 0)]);
        pre.push(Clause::Elems(lo, hi));
    }
    if rng.chance(40) || pre.is_empty() {
        let (lo, hi) = rng.pick(&[(-1000, 1000), (0, 10), (i64::MIN, i64::MAX)]);
        pre.push(Clause::XIn(lo, hi));
    }
    // Invariants: usually the bound the loop needs, plus random extras.
    let mut inv = Vec::new();
    if cond == Cond::ILtK {
        if rng.chance(85) {
            inv.push(Clause::ILeK);
        }
    } else if rng.chance(85) {
        inv.push(Clause::ILeLen);
    }
    for c in [
        Clause::SameLen,
        Clause::KLeLen,
        Clause::ILeLen,
        Clause::AccIn(1000),
        Clause::Elems(-1000, 1000),
        Clause::FirstKept,
    ] {
        if rng.chance(20) {
            inv.push(c);
        }
    }
    if inv.is_empty() {
        inv.push(Clause::SameLen);
    }
    let mut body = Vec::new();
    for _ in 0..1 + rng.below(2) {
        body.push(match rng.below(3) {
            0 => S::SetAcc(gen_e(rng, 2)),
            1 => S::Write(gen_idx(rng), gen_e(rng, 2)),
            _ => S::IfWrite(
                rng.pick(&[Cmp::Lt, Cmp::Le, Cmp::Eq, Cmp::Ne]),
                gen_e(rng, 1),
                gen_e(rng, 1),
                gen_idx(rng),
                gen_e(rng, 1),
            ),
        });
    }
    let ret = rng.pick(&[Ret::Acc, Ret::Acc, Ret::X, Ret::Elem0, Ret::ElemK]);
    let mut post = Vec::new();
    for c in [Clause::SameLen, Clause::REqX, Clause::FirstKept] {
        if rng.chance(25) {
            post.push(c);
        }
    }
    if rng.chance(50) || post.is_empty() {
        let (lo, hi) = bounds_pair(rng);
        post.push(Clause::RIn(lo, hi));
    }
    let mut f = Func {
        name: format!("g{n}"),
        pre,
        inv,
        post,
        start,
        cond,
        body,
        ret,
        form: Form::While,
        opt: false,
    };
    add_exits(rng, &mut f);
    f
}

fn idx_text(i: Idx) -> &'static str {
    match i {
        Idx::I => "i",
        Idx::IPlus1 => "i + 1",
        Idx::IMinus1 => "i - 1",
        Idx::K => "k",
        Idx::Zero => "0",
        Idx::Mirror => "a.len() - 1 - i",
    }
}

fn e_text(e: &E) -> String {
    match e {
        E::X => "x".into(),
        E::Acc => "acc".into(),
        E::Lit(v) => lit(*v),
        E::Elem(i) => format!("a[{}]", idx_text(*i)),
        E::Add(a, b) => format!("({} + {})", e_text(a), e_text(b)),
        E::Sub(a, b) => format!("({} - {})", e_text(a), e_text(b)),
        E::Mul(a, b) => format!("({} * {})", e_text(a), e_text(b)),
        E::WrapAdd(a, b) => format!("{}.wrapping_add({})", e_text(a), e_text(b)),
    }
}

fn cmp_text(c: Cmp) -> &'static str {
    match c {
        Cmp::Lt => "<",
        Cmp::Le => "<=",
        Cmp::Eq => "==",
        Cmp::Ne => "!=",
    }
}

impl Func {
    fn text(&self) -> String {
        let n = &self.name;
        let mut t = String::new();
        let _ = writeln!(
            t,
            "    spec fn {n}_pre(a: &[i64], k: usize, x: i64) -> bool {{ {} }}\n",
            conj(&self.pre)
        );
        let _ = writeln!(
            t,
            "    spec fn {n}_inv(a: &[i64], b: &[i64], k: usize, x: i64, i: usize, acc: i64) -> bool {{ {} }}\n",
            conj(&self.inv)
        );
        if self.opt {
            // `None` keeps the clauses that do not name the result.
            let rest: Vec<Clause> = self.post.iter().copied().filter(|c| !names_r(*c)).collect();
            let none = if rest.is_empty() {
                "true".to_string()
            } else {
                conj(&rest)
            };
            let _ = writeln!(
                t,
                "    spec fn {n}_post(a: &[i64], b: &[i64], k: usize, x: i64, o: Option<i64>) -> bool {{\n        match o {{\n            None => {none},\n            Some(r) => {},\n        }}\n    }}\n",
                conj(&self.post)
            );
        } else {
            let _ = writeln!(
                t,
                "    spec fn {n}_post(a: &[i64], b: &[i64], k: usize, x: i64, r: i64) -> bool {{ {} }}\n",
                conj(&self.post)
            );
        }
        let (rty, uses) = if self.opt {
            ("Option<i64>", ", lim_post")
        } else {
            ("i64", "")
        };
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_inv, {n}_post{uses})]\n    fn {n}(a: &mut [i64], k: usize, x: i64) -> (r: {rty})\n        requires {n}_pre(a, k, x),\n        ensures {n}_post(a, old(a), k, x, r),\n    {{"
        );
        let start = match self.start {
            Start::Zero => "0",
            Start::One => "1",
            Start::K => "k",
        };
        let (cond, measure, end) = match self.cond {
            Cond::ILtLen => ("i < a.len()", "a.len() - i", "a.len()"),
            Cond::IPlus1LtLen => ("i + 1 < a.len()", "a.len() - i", ""),
            Cond::ILtK => ("i < k", "k - i", "k"),
        };
        let _ = writeln!(t, "        let mut acc: i64 = 0;");
        if self.form == Form::For {
            let _ = writeln!(
                t,
                "        for i in {start}..{end}\n            invariant {n}_inv(a, old(a), k, x, i, acc),\n        {{"
            );
        } else {
            let _ = writeln!(t, "        let mut i: usize = {start};");
            let _ = writeln!(
                t,
                "        while {cond}\n            invariant {n}_inv(a, old(a), k, x, i, acc),\n            decreases {measure},\n        {{"
            );
        }
        let wrap = |v: String| if self.opt { format!("Some({v})") } else { v };
        for s in &self.body {
            match s {
                S::SetAcc(e) => {
                    let _ = writeln!(t, "            acc = {};", e_text(e));
                }
                S::Write(i, e) => {
                    let _ = writeln!(t, "            a[{}] = {};", idx_text(*i), e_text(e));
                }
                S::IfWrite(c, l, r, i, e) => {
                    let _ = writeln!(
                        t,
                        "            if {} {} {} {{\n                a[{}] = {};\n            }}",
                        e_text(l),
                        cmp_text(*c),
                        e_text(r),
                        idx_text(*i),
                        e_text(e)
                    );
                }
                S::Exit(x, c, l, r, pre) => {
                    let exit = match x {
                        Exit::Break => "break;".to_string(),
                        Exit::Continue if self.form == Form::While => {
                            "i = i + 1;\n                continue;".to_string()
                        }
                        Exit::Continue => "continue;".to_string(),
                        Exit::Return(e) => format!("return {};", wrap(e_text(e))),
                        Exit::Try(e, k) => format!("acc = lim({}, {})?;", e_text(e), lit(*k)),
                    };
                    let pre = match pre {
                        Some(Pre::Acc(e)) => format!("acc = {};\n                ", e_text(e)),
                        Some(Pre::Write(i, e)) => {
                            format!("a[{}] = {};\n                ", idx_text(*i), e_text(e))
                        }
                        None => String::new(),
                    };
                    let _ = writeln!(
                        t,
                        "            if {} {} {} {{\n                {pre}{exit}\n            }}",
                        e_text(l),
                        cmp_text(*c),
                        e_text(r)
                    );
                }
                S::Try(e, c) => {
                    let _ = writeln!(t, "            acc = lim({}, {})?;", e_text(e), lit(*c));
                }
            }
        }
        if self.form == Form::While {
            let _ = writeln!(t, "            i = i + 1;");
        }
        let _ = writeln!(t, "        }}");
        let ret = match self.ret {
            Ret::Acc => "acc",
            Ret::X => "x",
            Ret::Elem0 => "a[0]",
            Ret::ElemK => "a[k]",
        };
        let _ = writeln!(t, "        {}\n    }}\n", wrap(ret.to_string()));
        t
    }

    fn pre_holds(&self, a: &[i64], k: u64, x: i64) -> bool {
        all(
            &self.pre,
            &View {
                a,
                b: a,
                k,
                x,
                i: 0,
                acc: 0,
                r: 0,
            },
        )
    }
}

/// What a run did: the result, or a fault; and whether every invariant check
/// at a loop head passed (and whether the entry check did).
struct Run {
    result: Result<Out, Fault>,
    /// The body left the loop or the function early.
    early: bool,
    inv_ok: bool,
    entry_inv_ok: Option<bool>,
}

struct St<'f> {
    f: &'f Func,
    a: Vec<i64>,
    old: Vec<i64>,
    k: u64,
    x: i64,
    i: u64,
    acc: i64,
    /// The last iteration ended at a `continue`.
    skipped: bool,
}

impl St<'_> {
    fn idx(&self, i: Idx) -> Result<usize, Fault> {
        let len = self.a.len() as u64;
        let v = match i {
            Idx::I => self.i,
            Idx::IPlus1 => self.i.checked_add(1).ok_or(Fault)?,
            Idx::IMinus1 => self.i.checked_sub(1).ok_or(Fault)?,
            Idx::K => self.k,
            Idx::Zero => 0,
            Idx::Mirror => len
                .checked_sub(1)
                .ok_or(Fault)?
                .checked_sub(self.i)
                .ok_or(Fault)?,
        };
        if v < len { Ok(v as usize) } else { Err(Fault) }
    }

    fn e(&self, e: &E) -> Result<i64, Fault> {
        Ok(match e {
            E::X => self.x,
            E::Acc => self.acc,
            E::Lit(v) => *v,
            E::Elem(i) => self.a[self.idx(*i)?],
            E::Add(a, b) => self.e(a)?.checked_add(self.e(b)?).ok_or(Fault)?,
            E::Sub(a, b) => self.e(a)?.checked_sub(self.e(b)?).ok_or(Fault)?,
            E::Mul(a, b) => self.e(a)?.checked_mul(self.e(b)?).ok_or(Fault)?,
            E::WrapAdd(a, b) => self.e(a)?.wrapping_add(self.e(b)?),
        })
    }

    fn inv(&self) -> bool {
        all(
            &self.f.inv,
            &View {
                a: &self.a,
                b: &self.old,
                k: self.k,
                x: self.x,
                i: self.i,
                acc: self.acc,
                r: 0,
            },
        )
    }

    fn cond(&self) -> Result<bool, Fault> {
        let len = self.a.len() as u64;
        Ok(match self.f.cond {
            Cond::ILtLen => self.i < len,
            Cond::IPlus1LtLen => self.i.checked_add(1).ok_or(Fault)? < len,
            Cond::ILtK => self.i < self.k,
        })
    }

    fn test(&self, c: Cmp, l: &E, r: &E) -> Result<bool, Fault> {
        let (l, r) = (self.e(l)?, self.e(r)?);
        Ok(match c {
            Cmp::Lt => l < r,
            Cmp::Le => l <= r,
            Cmp::Eq => l == r,
            Cmp::Ne => l != r,
        })
    }

    fn step(&mut self) -> Result<Flow, Fault> {
        for s in &self.f.body {
            match s {
                S::Exit(x, c, l, r, pre) => {
                    if self.test(*c, l, r)? {
                        match pre {
                            Some(Pre::Acc(e)) => self.acc = self.e(e)?,
                            Some(Pre::Write(i, e)) => {
                                let v = self.e(e)?;
                                let j = self.idx(*i)?;
                                self.a[j] = v;
                            }
                            None => {}
                        }
                        return Ok(match x {
                            Exit::Break => Flow::Break,
                            Exit::Continue => {
                                self.i = self.i.checked_add(1).ok_or(Fault)?;
                                self.skipped = true;
                                Flow::Next
                            }
                            Exit::Return(e) => Flow::Return(Out::Val(self.e(e)?)),
                            Exit::Try(e, k) => {
                                let v = self.e(e)?;
                                if v > *k {
                                    Flow::Return(Out::None)
                                } else {
                                    self.acc = v;
                                    continue;
                                }
                            }
                        });
                    }
                }
                S::Try(e, c) => {
                    let v = self.e(e)?;
                    if v > *c {
                        return Ok(Flow::Return(Out::None));
                    }
                    self.acc = v;
                }
                S::SetAcc(e) => self.acc = self.e(e)?,
                S::Write(i, e) => {
                    // Rust order: the value first, then the place.
                    let v = self.e(e)?;
                    let j = self.idx(*i)?;
                    self.a[j] = v;
                }
                S::IfWrite(c, l, r, i, e) => {
                    let (l, r) = (self.e(l)?, self.e(r)?);
                    let take = match c {
                        Cmp::Lt => l < r,
                        Cmp::Le => l <= r,
                        Cmp::Eq => l == r,
                        Cmp::Ne => l != r,
                    };
                    if take {
                        let v = self.e(e)?;
                        let j = self.idx(*i)?;
                        self.a[j] = v;
                    }
                }
            }
        }
        self.i = self.i.checked_add(1).ok_or(Fault)?;
        Ok(Flow::Next)
    }
}

fn run(f: &Func, a: &mut Vec<i64>, k: u64, x: i64) -> Run {
    let mut st = St {
        f,
        a: a.clone(),
        old: a.clone(),
        k,
        x,
        i: match f.start {
            Start::Zero => 0,
            Start::One => 1,
            Start::K => k,
        },
        acc: 0,
        skipped: false,
    };
    let mut inv_ok = true;
    let mut entry = None;
    let mut early = false;
    let mut fuel = 64;
    let result = (|| -> Result<Out, Fault> {
        loop {
            let ok = st.inv();
            entry.get_or_insert(ok);
            inv_ok &= ok;
            if !st.cond()? {
                break;
            }
            match st.step()? {
                Flow::Next => early |= std::mem::take(&mut st.skipped),
                Flow::Break => {
                    early = true;
                    break;
                }
                Flow::Return(o) => {
                    early = true;
                    return Ok(o);
                }
            }
            fuel -= 1;
            assert!(
                fuel > 0,
                "generated loops are bounded by the array length or k"
            );
        }
        Ok(Out::Val(match f.ret {
            Ret::Acc => st.acc,
            Ret::X => st.x,
            Ret::Elem0 => st.a[st.idx(Idx::Zero)?],
            Ret::ElemK => st.a[st.idx(Idx::K)?],
        }))
    })();
    *a = st.a.clone();
    Run {
        result,
        early,
        inv_ok,
        entry_inv_ok: entry,
    }
}

/// Whether a postcondition clause names the result.
fn names_r(c: Clause) -> bool {
    matches!(c, Clause::RIn(..) | Clause::REqX)
}

fn post_holds(f: &Func, a: &[i64], old: &[i64], k: u64, x: i64, out: Out) -> bool {
    let (cs, r): (Vec<Clause>, i64) = match out {
        Out::Val(r) => (f.post.clone(), r),
        Out::None => (f.post.iter().copied().filter(|c| !names_r(*c)).collect(), 0),
    };
    all(
        &cs,
        &View {
            a,
            b: old,
            k,
            x,
            i: 0,
            acc: 0,
            r,
        },
    )
}

/// `lim`, the callee of every `?`, and `show`, which prints an `Option`.
const LIM: &str = "    spec fn lim_post(v: i64, c: i64, r: Option<i64>) -> bool {\n        match r {\n            None => c < v,\n            Some(w) => w == v && v <= c,\n        }\n    }\n\n    #[effects(pure)]\n    #[uses(lim_post)]\n    fn lim(v: i64, c: i64) -> (r: Option<i64>)\n        ensures lim_post(v, c, r),\n    {\n        if v <= c { Some(v) } else { None }\n    }\n\n";
const SHOW: &str = "    #[effects(io)]\n    fn show(o: Option<i64>) -> () {\n        match o {\n            None => print_bytes(b\"none\\n\"),\n            Some(v) => print_i64(v),\n        }\n    }\n\n";

/// Inputs that satisfy the precondition.
fn inputs(f: &Func, rng: &mut Rng) -> Vec<(Vec<i64>, u64, i64)> {
    let mut out = Vec::new();
    for _ in 0..200 {
        let len = rng.below(6);
        let a: Vec<i64> = (0..len)
            .map(|_| match rng.below(3) {
                0 => small(rng),
                1 => rng.below(21) as i64 - 10,
                _ => rng.below(201) as i64 - 100,
            })
            .collect();
        let k = rng.below(7) as u64;
        let x = match rng.below(3) {
            0 => small(rng),
            _ => rng.below(21) as i64 - 10,
        };
        if f.pre_holds(&a, k, x) {
            out.push((a, k, x));
        }
        if out.len() == 6 {
            break;
        }
    }
    out
}

/// Parses `a = [1, 2] (len 2), k = 0, x = 5`. `None` when an array is shown
/// only in part.
fn parse_cex(msg: &str) -> Option<(Vec<i64>, u64, i64)> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    let rest = rest.strip_prefix("a = [")?;
    let (elems, rest) = rest.split_once("] (len ")?;
    let (len, rest) = rest.split_once("), k = ")?;
    let (k, x) = rest.split_once(", x = ")?;
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
    Some((a, k.parse().ok()?, x.trim().parse().ok()?))
}

fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let start = line.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let end = start + line[start..].find('"')?;
    Some(&line[start..end])
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    verified_runs: usize,
    cexes: usize,
    entry_replays: usize,
    compiled_runs: usize,
    /// Verified functions with a range `for`, an early exit, a `?`.
    verified_for: usize,
    verified_exit: usize,
    verified_try: usize,
    /// Interpreted runs of verified functions that left early.
    early_runs: usize,
    /// Failure codes over all functions, to see what keeps proofs from passing.
    codes: std::collections::BTreeMap<String, usize>,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut funcs: Vec<Func> = vec![kicked_sum(&mut rng, seed)];
    funcs.extend((1..6).map(|n| gen_func(&mut rng, n)));
    let mut src = String::from("#[summary(\"루프 건전성 퍼징\")]\nmod m {\n");
    src.push_str(LIM);
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str("}\n\n#[summary(\"실행\")]\nmod app {\n");
    src.push_str(SHOW);
    src.push_str("    #[effects(io)]\n    fn main() -> () {\n");
    let mut expected = String::new();
    let mut n = 0;
    for f in &funcs {
        for (a, k, x) in inputs(f, &mut rng) {
            let mut after = a.clone();
            let r = run(f, &mut after, k, x);
            let Ok(v) = r.result else { continue };
            if a.is_empty() {
                continue;
            }
            n += 1;
            let elems = a.iter().map(|&e| lit(e)).collect::<Vec<_>>().join(", ");
            let print = if f.opt { "show" } else { "print_i64" };
            let _ = writeln!(
                src,
                "        let mut a{n}: [i64; {}] = [{elems}];\n        {print}(m::{}(&mut a{n}, {k}, {}));",
                a.len(),
                f.name,
                lit(x)
            );
            match v {
                Out::Val(v) => {
                    let _ = writeln!(expected, "{v}");
                }
                Out::None => expected.push_str("none\n"),
            }
            for (j, e) in after.iter().enumerate() {
                let _ = writeln!(src, "        print_i64(a{n}[{j}]);");
                let _ = writeln!(expected, "{e}");
            }
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("l.bmb".to_string(), src.clone())];
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
        let Some(f) = funcs.iter().find(|f| f.name == name) else {
            continue;
        };
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        stats.verified_for += usize::from(f.form == Form::For);
        stats.verified_exit += usize::from(f.body.iter().any(|s| matches!(s, S::Exit(..))));
        stats.verified_try += usize::from(
            f.body
                .iter()
                .any(|s| matches!(s, S::Try(..) | S::Exit(Exit::Try(..), ..))),
        );
        for (a, k, x) in inputs(f, &mut rng) {
            stats.verified_runs += 1;
            let mut after = a.clone();
            let r = run(f, &mut after, k, x);
            stats.early_runs += usize::from(r.early);
            let at = format!("a = {a:?}, k = {k}, x = {x}");
            match r.result {
                Err(Fault) => panic!("{}", ctx(&format!("verified {name} faults at {at}"))),
                Ok(v) => {
                    assert!(
                        r.inv_ok,
                        "{}",
                        ctx(&format!("verified {name} breaks its invariant at {at}"))
                    );
                    assert!(
                        post_holds(f, &after, &a, k, x, v),
                        "{}",
                        ctx(&format!(
                            "verified {name} breaks its postcondition at {at} (r = {v:?}, a after = {after:?})"
                        ))
                    );
                }
            }
        }
    }

    for line in out.verify.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}`", f.name)))
        else {
            continue;
        };
        if let Some(c) = field(line, "code") {
            let what = msg.split(" in `").next().unwrap_or("");
            *stats.codes.entry(format!("{c} {what}")).or_default() += 1;
        }
        let Some((a, k, x)) = parse_cex(msg) else {
            continue;
        };
        stats.cexes += 1;
        assert!(
            f.pre_holds(&a, k, x),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        if field(line, "code") == Some("E0806") {
            stats.entry_replays += 1;
            let mut after = a.clone();
            let r = run(f, &mut after, k, x);
            assert_eq!(
                r.entry_inv_ok,
                Some(false),
                "{}",
                ctx(&format!(
                    "entry counterexample does not break the invariant: {msg}"
                ))
            );
        }
    }

    let lines = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("l{seed}-{all_checks}"));
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
fn loop_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-loops-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (loops): {} functions, {} verified ({} runs), {} counterexamples ({} entry replays), {} compiled outputs",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.cexes,
        stats.entry_replays,
        stats.compiled_runs
    );
    eprintln!(
        "early exits (2K-05): verified {} with `for`, {} with an exit, {} with `?`; {} verified runs left early",
        stats.verified_for, stats.verified_exit, stats.verified_try, stats.early_runs
    );
    eprintln!("failures by kind: {:#?}", stats.codes);
    assert!(stats.verified > 0 && stats.cexes > 0);
}
