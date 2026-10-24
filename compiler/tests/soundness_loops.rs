//! Soundness fuzzing of loops, arrays and `&mut` (backlog 1B-21).
//!
//! Random functions `g(a: &mut [i64], k: usize, x: i64) -> i64` with one
//! `while` loop, random invariants and random contracts are proved, and the
//! claims are checked against an independent interpreter written here:
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
    let mut f = match rng.below(4) {
        // Sum of bounded elements.
        0 => Func {
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
        },
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
                }
            }
            _ => {
                f.post
                    .push(rng.pick(&[Clause::REqX, Clause::FirstKept, Clause::RIn(-1000, 1000)]))
            }
        }
    }
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
    Func {
        name: format!("g{n}"),
        pre,
        inv,
        post,
        start,
        cond,
        body,
        ret,
    }
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
        let _ = writeln!(
            t,
            "    spec fn {n}_post(a: &[i64], b: &[i64], k: usize, x: i64, r: i64) -> bool {{ {} }}\n",
            conj(&self.post)
        );
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_inv, {n}_post)]\n    fn {n}(a: &mut [i64], k: usize, x: i64) -> (r: i64)\n        requires {n}_pre(a, k, x),\n        ensures {n}_post(a, old(a), k, x, r),\n    {{"
        );
        let start = match self.start {
            Start::Zero => "0",
            Start::One => "1",
            Start::K => "k",
        };
        let (cond, measure) = match self.cond {
            Cond::ILtLen => ("i < a.len()", "a.len() - i"),
            Cond::IPlus1LtLen => ("i + 1 < a.len()", "a.len() - i"),
            Cond::ILtK => ("i < k", "k - i"),
        };
        let _ = writeln!(t, "        let mut i: usize = {start};");
        let _ = writeln!(t, "        let mut acc: i64 = 0;");
        let _ = writeln!(
            t,
            "        while {cond}\n            invariant {n}_inv(a, old(a), k, x, i, acc),\n            decreases {measure},\n        {{"
        );
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
            }
        }
        let _ = writeln!(t, "            i = i + 1;\n        }}");
        let ret = match self.ret {
            Ret::Acc => "acc",
            Ret::X => "x",
            Ret::Elem0 => "a[0]",
            Ret::ElemK => "a[k]",
        };
        let _ = writeln!(t, "        {ret}\n    }}\n");
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
    result: Result<i64, Fault>,
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

    fn step(&mut self) -> Result<(), Fault> {
        for s in &self.f.body {
            match s {
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
        Ok(())
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
    };
    let mut inv_ok = true;
    let mut entry = None;
    let mut fuel = 64;
    let result = (|| -> Result<i64, Fault> {
        loop {
            let ok = st.inv();
            entry.get_or_insert(ok);
            inv_ok &= ok;
            if !st.cond()? {
                break;
            }
            st.step()?;
            fuel -= 1;
            assert!(
                fuel > 0,
                "generated loops are bounded by the array length or k"
            );
        }
        Ok(match f.ret {
            Ret::Acc => st.acc,
            Ret::X => st.x,
            Ret::Elem0 => st.a[st.idx(Idx::Zero)?],
            Ret::ElemK => st.a[st.idx(Idx::K)?],
        })
    })();
    *a = st.a.clone();
    Run {
        result,
        inv_ok,
        entry_inv_ok: entry,
    }
}

fn post_holds(f: &Func, a: &[i64], old: &[i64], k: u64, x: i64, r: i64) -> bool {
    all(
        &f.post,
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
    /// Failure codes over all functions, to see what keeps proofs from passing.
    codes: std::collections::BTreeMap<String, usize>,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let funcs: Vec<Func> = (0..6).map(|n| gen_func(&mut rng, n)).collect();
    let mut src = String::from("#[summary(\"루프 건전성 퍼징\")]\nmod m {\n");
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
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
            let _ = writeln!(
                src,
                "        let mut a{n}: [i64; {}] = [{elems}];\n        print_i64(m::{}(&mut a{n}, {k}, {}));",
                a.len(),
                f.name,
                lit(x)
            );
            let _ = writeln!(expected, "{v}");
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
        for (a, k, x) in inputs(f, &mut rng) {
            stats.verified_runs += 1;
            let mut after = a.clone();
            let r = run(f, &mut after, k, x);
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
                            "verified {name} breaks its postcondition at {at} (r = {v}, a after = {after:?})"
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
    eprintln!("failures by kind: {:#?}", stats.codes);
    assert!(stats.verified > 0 && stats.cexes > 0);
}
