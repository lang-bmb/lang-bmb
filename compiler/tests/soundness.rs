//! Soundness fuzzing of the proof gate (backlog 1B-17, 1B-18).
//!
//! Random integer functions with random contracts are proved, and the claims
//! are checked against an independent reference interpreter written here:
//!
//! - a verified function, on any input that satisfies its precondition, never
//!   overflows, divides by zero, or breaks its postcondition;
//! - every definite counterexample really breaks the function (the input
//!   satisfies the precondition and execution fails or the postcondition is
//!   false);
//! - the compiled program, built twice (proved checks removed, and every check
//!   kept), prints what the interpreter computes.
//!
//! `BMB_SOUND_ITERS` raises the number of batches for longer runs.

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
}

const INTERESTING: &[i64] = &[
    i64::MIN,
    i64::MIN + 1,
    -(1 << 62),
    -(1 << 32),
    -(1 << 31),
    -1000,
    -7,
    -2,
    -1,
    0,
    1,
    2,
    3,
    7,
    10,
    1000,
    1 << 31,
    1 << 32,
    1 << 62,
    i64::MAX - 1,
    i64::MAX,
];

fn value(rng: &mut Rng) -> i64 {
    match rng.below(4) {
        0 | 1 => INTERESTING[rng.below(INTERESTING.len())],
        2 => rng.below(201) as i64 - 100,
        _ => rng.next() as i64,
    }
}

/// Negative literals are parenthesized: as in Rust, `-4.wrapping_mul(y)` is
/// `-(4.wrapping_mul(y))`.
fn lit(v: i64) -> String {
    if v == i64::MIN {
        "(-9223372036854775807 - 1)".to_string()
    } else if v < 0 {
        format!("({v})")
    } else {
        v.to_string()
    }
}

// ---------------------------------------------------------------------------
// Executable expressions over i64.

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    WrapAdd,
    WrapMul,
}

#[derive(Clone, Copy)]
enum Cmp {
    Lt,
    Le,
    Eq,
    Ne,
}

enum E {
    Var(String),
    Lit(i64),
    Bin(Op, Box<E>, Box<E>),
    Neg(Box<E>),
    If(Box<C>, Box<E>, Box<E>),
}

enum C {
    Cmp(Cmp, E, E),
    And(Box<C>, Box<C>),
    Not(Box<C>),
}

/// Why execution stopped: an implicit obligation that fails at run time.
#[derive(Debug)]
struct Fault;

fn gen_e(rng: &mut Rng, vars: &[String], depth: usize) -> E {
    if depth == 0 || rng.chance(25) {
        return if rng.chance(70) {
            E::Var(vars[rng.below(vars.len())].clone())
        } else {
            E::Lit(value(rng))
        };
    }
    match rng.below(10) {
        0..=5 => {
            let op = [
                Op::Add,
                Op::Sub,
                Op::Mul,
                Op::Div,
                Op::Rem,
                Op::WrapAdd,
                Op::WrapMul,
            ][rng.below(7)];
            E::Bin(
                op,
                Box::new(gen_e(rng, vars, depth - 1)),
                Box::new(gen_e(rng, vars, depth - 1)),
            )
        }
        6 => E::Neg(Box::new(gen_e(rng, vars, depth - 1))),
        _ => E::If(
            Box::new(gen_c(rng, vars, depth - 1)),
            Box::new(gen_e(rng, vars, depth - 1)),
            Box::new(gen_e(rng, vars, depth - 1)),
        ),
    }
}

fn gen_cmp(rng: &mut Rng) -> Cmp {
    [Cmp::Lt, Cmp::Le, Cmp::Eq, Cmp::Ne][rng.below(4)]
}

fn gen_c(rng: &mut Rng, vars: &[String], depth: usize) -> C {
    match rng.below(6) {
        0 if depth > 0 => C::And(
            Box::new(gen_c(rng, vars, depth - 1)),
            Box::new(gen_c(rng, vars, depth - 1)),
        ),
        1 if depth > 0 => C::Not(Box::new(gen_c(rng, vars, depth - 1))),
        _ => C::Cmp(
            gen_cmp(rng),
            gen_e(rng, vars, depth.min(1)),
            gen_e(rng, vars, depth.min(1)),
        ),
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

fn cmp_eval<T: Ord>(c: Cmp, a: T, b: T) -> bool {
    match c {
        Cmp::Lt => a < b,
        Cmp::Le => a <= b,
        Cmp::Eq => a == b,
        Cmp::Ne => a != b,
    }
}

fn e_text(e: &E) -> String {
    match e {
        E::Var(v) => v.clone(),
        E::Lit(v) => lit(*v),
        E::Bin(op, a, b) => {
            let (a, b) = (e_text(a), e_text(b));
            match op {
                Op::Add => format!("({a} + {b})"),
                Op::Sub => format!("({a} - {b})"),
                Op::Mul => format!("({a} * {b})"),
                Op::Div => format!("({a} / {b})"),
                Op::Rem => format!("({a} % {b})"),
                Op::WrapAdd => format!("{a}.wrapping_add({b})"),
                Op::WrapMul => format!("{a}.wrapping_mul({b})"),
            }
        }
        E::Neg(a) => format!("(-{})", e_text(a)),
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
        C::Cmp(k, a, b) => format!("({} {} {})", e_text(a), cmp_text(*k), e_text(b)),
        C::And(a, b) => format!("({} && {})", c_text(a), c_text(b)),
        C::Not(a) => format!("!{}", c_text(a)),
    }
}

type Env = Vec<(String, i64)>;

fn lookup(env: &Env, v: &str) -> i64 {
    env.iter().rev().find(|(n, _)| n == v).unwrap().1
}

fn e_eval(e: &E, env: &Env) -> Result<i64, Fault> {
    Ok(match e {
        E::Var(v) => lookup(env, v),
        E::Lit(v) => *v,
        E::Bin(op, a, b) => {
            let (a, b) = (e_eval(a, env)?, e_eval(b, env)?);
            match op {
                Op::Add => a.checked_add(b).ok_or(Fault)?,
                Op::Sub => a.checked_sub(b).ok_or(Fault)?,
                Op::Mul => a.checked_mul(b).ok_or(Fault)?,
                Op::Div => a.checked_div(b).ok_or(Fault)?,
                Op::Rem => a.checked_rem(b).ok_or(Fault)?,
                Op::WrapAdd => a.wrapping_add(b),
                Op::WrapMul => a.wrapping_mul(b),
            }
        }
        E::Neg(a) => e_eval(a, env)?.checked_neg().ok_or(Fault)?,
        E::If(c, a, b) => {
            if c_eval(c, env)? {
                e_eval(a, env)?
            } else {
                e_eval(b, env)?
            }
        }
    })
}

fn c_eval(c: &C, env: &Env) -> Result<bool, Fault> {
    Ok(match c {
        C::Cmp(k, a, b) => cmp_eval(*k, e_eval(a, env)?, e_eval(b, env)?),
        // `&&` short-circuits: the right side's obligations hold only when
        // the left side is true.
        C::And(a, b) => c_eval(a, env)? && c_eval(b, env)?,
        C::Not(a) => !c_eval(a, env)?,
    })
}

// ---------------------------------------------------------------------------
// Specification expressions: mathematical integers (ADR-0017).

enum S {
    Var(&'static str),
    Lit(i64),
    Add(Box<S>, Box<S>),
    Sub(Box<S>, Box<S>),
    Scale(i64, Box<S>),
}

enum P {
    Cmp(Cmp, S, S),
    And(Box<P>, Box<P>),
    Or(Box<P>, Box<P>),
    Not(Box<P>),
}

fn gen_s(rng: &mut Rng, vars: &[&'static str], depth: usize) -> S {
    if depth == 0 || rng.chance(40) {
        return if rng.chance(70) {
            S::Var(vars[rng.below(vars.len())])
        } else {
            S::Lit(value(rng))
        };
    }
    match rng.below(3) {
        0 => S::Add(
            Box::new(gen_s(rng, vars, depth - 1)),
            Box::new(gen_s(rng, vars, depth - 1)),
        ),
        1 => S::Sub(
            Box::new(gen_s(rng, vars, depth - 1)),
            Box::new(gen_s(rng, vars, depth - 1)),
        ),
        _ => S::Scale(
            rng.below(7) as i64 - 3,
            Box::new(gen_s(rng, vars, depth - 1)),
        ),
    }
}

fn gen_p(rng: &mut Rng, vars: &[&'static str], depth: usize) -> P {
    match rng.below(7) {
        0 if depth > 0 => P::And(
            Box::new(gen_p(rng, vars, depth - 1)),
            Box::new(gen_p(rng, vars, depth - 1)),
        ),
        1 if depth > 0 => P::Or(
            Box::new(gen_p(rng, vars, depth - 1)),
            Box::new(gen_p(rng, vars, depth - 1)),
        ),
        2 if depth > 0 => P::Not(Box::new(gen_p(rng, vars, depth - 1))),
        _ => P::Cmp(gen_cmp(rng), gen_s(rng, vars, 2), gen_s(rng, vars, 2)),
    }
}

fn s_text(s: &S) -> String {
    match s {
        S::Var(v) => v.to_string(),
        S::Lit(v) => lit(*v),
        S::Add(a, b) => format!("({} + {})", s_text(a), s_text(b)),
        S::Sub(a, b) => format!("({} - {})", s_text(a), s_text(b)),
        S::Scale(k, a) => format!("({} * {})", lit(*k), s_text(a)),
    }
}

fn p_text(p: &P) -> String {
    match p {
        P::Cmp(k, a, b) => format!("({} {} {})", s_text(a), cmp_text(*k), s_text(b)),
        P::And(a, b) => format!("({} && {})", p_text(a), p_text(b)),
        P::Or(a, b) => format!("({} || {})", p_text(a), p_text(b)),
        P::Not(a) => format!("!{}", p_text(a)),
    }
}

fn s_eval(s: &S, env: &[(&str, i128)]) -> i128 {
    match s {
        S::Var(v) => env.iter().find(|(n, _)| n == v).unwrap().1,
        S::Lit(v) => *v as i128,
        S::Add(a, b) => s_eval(a, env) + s_eval(b, env),
        S::Sub(a, b) => s_eval(a, env) - s_eval(b, env),
        S::Scale(k, a) => *k as i128 * s_eval(a, env),
    }
}

fn p_eval(p: &P, env: &[(&str, i128)]) -> bool {
    match p {
        P::Cmp(k, a, b) => cmp_eval(*k, s_eval(a, env), s_eval(b, env)),
        P::And(a, b) => p_eval(a, env) && p_eval(b, env),
        P::Or(a, b) => p_eval(a, env) || p_eval(b, env),
        P::Not(a) => !p_eval(a, env),
    }
}

// ---------------------------------------------------------------------------
// Functions.

struct Func {
    name: String,
    /// x in [lo.0, hi.0], y in [lo.1, hi.1], and `extra`.
    lo: (i64, i64),
    hi: (i64, i64),
    extra: Option<P>,
    lets: Vec<(String, E)>,
    body: E,
    post: P,
}

fn bounds(rng: &mut Rng) -> (i64, i64) {
    let (a, b) = (value(rng), value(rng));
    (a.min(b), a.max(b))
}

fn gen_func(rng: &mut Rng, k: usize) -> Func {
    let (lx, hx) = bounds(rng);
    let (ly, hy) = bounds(rng);
    let extra = rng.chance(30).then(|| gen_p(rng, &["x", "y"], 1));
    let mut vars = vec!["x".to_string(), "y".to_string()];
    let mut lets = Vec::new();
    for i in 0..rng.below(3) {
        let e = gen_e(rng, &vars, 2);
        let name = format!("v{i}");
        lets.push((name.clone(), e));
        vars.push(name);
    }
    let body = gen_e(rng, &vars, 3);
    let post = gen_p(rng, &["x", "y", "r"], 1);
    Func {
        name: format!("f{k}"),
        lo: (lx, ly),
        hi: (hx, hy),
        extra,
        lets,
        body,
        post,
    }
}

impl Func {
    fn pre_text(&self) -> String {
        let mut t = format!(
            "{} <= x && x <= {} && {} <= y && y <= {}",
            lit(self.lo.0),
            lit(self.hi.0),
            lit(self.lo.1),
            lit(self.hi.1)
        );
        if let Some(p) = &self.extra {
            let _ = write!(t, " && {}", p_text(p));
        }
        t
    }

    fn pre(&self, x: i64, y: i64) -> bool {
        let env = [("x", x as i128), ("y", y as i128)];
        self.lo.0 <= x
            && x <= self.hi.0
            && self.lo.1 <= y
            && y <= self.hi.1
            && self.extra.as_ref().is_none_or(|p| p_eval(p, &env))
    }

    fn run(&self, x: i64, y: i64) -> Result<i64, Fault> {
        let mut env: Env = vec![("x".into(), x), ("y".into(), y)];
        for (n, e) in &self.lets {
            let v = e_eval(e, &env)?;
            env.push((n.clone(), v));
        }
        e_eval(&self.body, &env)
    }

    fn post(&self, x: i64, y: i64, r: i64) -> bool {
        p_eval(
            &self.post,
            &[("x", x as i128), ("y", y as i128), ("r", r as i128)],
        )
    }

    fn text(&self) -> String {
        let n = &self.name;
        let mut t = format!(
            "    spec fn {n}_pre(x: i64, y: i64) -> bool {{ {} }}\n\n",
            self.pre_text()
        );
        let _ = write!(
            t,
            "    spec fn {n}_post(x: i64, y: i64, r: i64) -> bool {{ {} }}\n\n",
            p_text(&self.post)
        );
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_post)]\n    fn {n}(x: i64, y: i64) -> (r: i64)\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{"
        );
        for (v, e) in &self.lets {
            let _ = writeln!(t, "        let {v}: i64 = {};", e_text(e));
        }
        let _ = writeln!(t, "        {}\n    }}\n", e_text(&self.body));
        t
    }

    /// Inputs that satisfy the precondition: its corners and random values.
    fn inputs(&self, rng: &mut Rng) -> Vec<(i64, i64)> {
        let near = |lo: i64, hi: i64, rng: &mut Rng| -> i64 {
            match rng.below(5) {
                0 => lo,
                1 => hi,
                2 => lo.saturating_add(1).min(hi),
                3 => hi.saturating_sub(1).max(lo),
                _ => {
                    let span = (hi as i128 - lo as i128 + 1) as u128;
                    (lo as i128 + (rng.next() as u128 % span) as i128) as i64
                }
            }
        };
        let mut out = Vec::new();
        for _ in 0..40 {
            let (x, y) = (
                near(self.lo.0, self.hi.0, rng),
                near(self.lo.1, self.hi.1, rng),
            );
            if self.pre(x, y) && !out.contains(&(x, y)) {
                out.push((x, y));
            }
            if out.len() == 6 {
                break;
            }
        }
        out
    }
}

/// `name = value` pairs after "counterexample: ".
fn parse_cex(msg: &str) -> Option<Vec<(String, i64)>> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    rest.split(", ")
        .map(|kv| {
            let (k, v) = kv.split_once(" = ")?;
            Some((k.to_string(), v.trim().parse().ok()?))
        })
        .collect()
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
    compiled_runs: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let funcs: Vec<Func> = (0..8).map(|k| gen_func(&mut rng, k)).collect();
    let mut src = String::from("#[summary(\"건전성 퍼징\")]\nmod m {\n");
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    let mut expected = String::new();
    for f in &funcs {
        for (x, y) in f.inputs(&mut rng) {
            if let Ok(r) = f.run(x, y) {
                let _ = writeln!(
                    src,
                    "        print_i64(m::{}({}, {}));",
                    f.name,
                    lit(x),
                    lit(y)
                );
                let _ = writeln!(expected, "{r}");
            }
        }
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
        let Some(def) = field(line, "def") else {
            continue;
        };
        let Some(name) = def.strip_prefix("m::") else {
            continue;
        };
        let f = funcs.iter().find(|f| f.name == name).unwrap();
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        // A verified function is safe and meets its postcondition on every
        // input its precondition admits.
        for (x, y) in f.inputs(&mut rng) {
            stats.verified_runs += 1;
            match f.run(x, y) {
                Ok(r) => assert!(
                    f.post(x, y, r),
                    "{}",
                    ctx(&format!(
                        "verified {name} breaks its postcondition at x = {x}, y = {y}, r = {r}"
                    ))
                ),
                Err(Fault) => panic!(
                    "{}",
                    ctx(&format!("verified {name} faults at x = {x}, y = {y}"))
                ),
            }
        }
    }

    // Every definite counterexample breaks its function.
    for line in out.verify.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(cex) = parse_cex(msg) else {
            continue;
        };
        let Some(name) = funcs
            .iter()
            .map(|f| f.name.as_str())
            .find(|n| msg.contains(&format!("in `m::{n}`")))
        else {
            continue;
        };
        let f = funcs.iter().find(|f| f.name == name).unwrap();
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        let (Some(x), Some(y)) = (get("x"), get("y")) else {
            panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
        };
        stats.cexes += 1;
        assert!(
            f.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        let broken = match f.run(x, y) {
            Err(Fault) => true,
            Ok(r) => !f.post(x, y, r),
        };
        assert!(
            broken,
            "{}",
            ctx(&format!("counterexample does not break {name}: {msg}"))
        );
    }

    // Both builds print what the interpreter computes.
    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("s{seed}-{all_checks}"));
        bmb::toolchain::build_executable(&ir, exe.to_str().unwrap())
            .unwrap_or_else(|e| panic!("{}", ctx(&e)));
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
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            expected,
            "{}",
            ctx(&format!("build (all_checks = {all_checks}) output differs"))
        );
        stats.compiled_runs += n;
    }
}

#[test]
fn proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    // Random products are often nonlinear; a lower budget turns slow proofs
    // into "undecided" (never a false claim) and keeps a batch to seconds.
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let dir = std::env::temp_dir().join(format!("bmb-sound-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness: {} functions, {} verified ({} runs), {} counterexamples replayed, {} compiled runs",
        stats.funcs, stats.verified, stats.verified_runs, stats.cexes, stats.compiled_runs
    );
    // The generator must actually reach both outcomes.
    assert!(stats.verified > 0 && stats.cexes > 0);
}
