//! Soundness fuzzing of calls (backlog 1B-32): callers rely on their callees'
//! contracts. Random callees come from the straight-line generator; random
//! callers call them with computed arguments, sometimes under a condition.
//!
//! When a caller and every callee it uses are verified, then on every input
//! the caller's precondition admits, the interpreter (which runs the callee
//! bodies, not their contracts) must find: no fault, every call's arguments
//! satisfy the callee's precondition, and the caller's postcondition holds.
//! Counterexamples must satisfy the caller's precondition (they are not
//! replayed: past a call the solver picks any result the callee's
//! postcondition allows). Both builds must print what the interpreter
//! computes.
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;

/// `let aK: i64 = <call or value>;`
enum Bind {
    Call(usize, E, E),
    /// `if C { call } else { E }`
    CondCall(C, usize, E, E, E),
}

struct Caller {
    name: String,
    lo: (i64, i64),
    hi: (i64, i64),
    binds: Vec<Bind>,
    body: E,
    post: P,
}

#[derive(Debug)]
enum Outcome {
    Fault,
    /// A call whose arguments break the callee's precondition.
    BadCall(String),
    Value(i64),
}

impl Caller {
    fn pre(&self, x: i64, y: i64) -> bool {
        self.lo.0 <= x && x <= self.hi.0 && self.lo.1 <= y && y <= self.hi.1
    }

    fn uses(&self) -> Vec<usize> {
        let mut u: Vec<usize> = self
            .binds
            .iter()
            .map(|b| match b {
                Bind::Call(k, ..) | Bind::CondCall(_, k, ..) => *k,
            })
            .collect();
        u.sort();
        u.dedup();
        u
    }

    fn text(&self, callees: &[Func]) -> String {
        let n = &self.name;
        let mut uses = vec![format!("{n}_pre"), format!("{n}_post")];
        for k in self.uses() {
            uses.push(format!("{}_pre", callees[k].name));
            uses.push(format!("{}_post", callees[k].name));
        }
        let mut t = format!(
            "    spec fn {n}_pre(x: i64, y: i64) -> bool {{ {} <= x && x <= {} && {} <= y && y <= {} }}\n\n",
            lit(self.lo.0),
            lit(self.hi.0),
            lit(self.lo.1),
            lit(self.hi.1)
        );
        let _ = write!(
            t,
            "    spec fn {n}_post(x: i64, y: i64, r: i64) -> bool {{ {} }}\n\n",
            p_text(&self.post)
        );
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({})]\n    fn {n}(x: i64, y: i64) -> (r: i64)\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{",
            uses.join(", ")
        );
        for (i, b) in self.binds.iter().enumerate() {
            let rhs = match b {
                Bind::Call(k, a, b) => {
                    format!("{}({}, {})", callees[*k].name, e_text(a), e_text(b))
                }
                Bind::CondCall(c, k, a, b, other) => format!(
                    "if {} {{ {}({}, {}) }} else {{ {} }}",
                    c_text(c),
                    callees[*k].name,
                    e_text(a),
                    e_text(b),
                    e_text(other)
                ),
            };
            let _ = writeln!(t, "        let a{i}: i64 = {rhs};");
        }
        let _ = writeln!(t, "        {}\n    }}\n", e_text(&self.body));
        t
    }

    fn run(&self, callees: &[Func], x: i64, y: i64) -> Outcome {
        let mut env: Env = vec![("x".into(), x), ("y".into(), y)];
        let call = |k: usize, a: &E, b: &E, env: &Env| -> Result<i64, Outcome> {
            let (a, b) = match (e_eval(a, env), e_eval(b, env)) {
                (Ok(a), Ok(b)) => (a, b),
                _ => return Err(Outcome::Fault),
            };
            let f = &callees[k];
            if !f.pre(a, b) {
                return Err(Outcome::BadCall(format!("{}({a}, {b})", f.name)));
            }
            f.run(a, b).map_err(|_| Outcome::Fault)
        };
        for (i, bind) in self.binds.iter().enumerate() {
            let v = match bind {
                Bind::Call(k, a, b) => call(*k, a, b, &env),
                Bind::CondCall(c, k, a, b, other) => match c_eval(c, &env) {
                    Err(_) => Err(Outcome::Fault),
                    Ok(true) => call(*k, a, b, &env),
                    Ok(false) => e_eval(other, &env).map_err(|_| Outcome::Fault),
                },
            };
            match v {
                Ok(v) => env.push((format!("a{i}"), v)),
                Err(o) => return o,
            }
        }
        match e_eval(&self.body, &env) {
            Ok(v) => Outcome::Value(v),
            Err(_) => Outcome::Fault,
        }
    }

    fn post_holds(&self, x: i64, y: i64, r: i64) -> bool {
        p_eval(
            &self.post,
            &[("x", x as i128), ("y", y as i128), ("r", r as i128)],
        )
    }

    fn inputs(&self, rng: &mut Rng) -> Vec<(i64, i64)> {
        let near = |lo: i64, hi: i64, rng: &mut Rng| -> i64 {
            match rng.below(4) {
                0 => lo,
                1 => hi,
                _ => {
                    let span = (hi as i128 - lo as i128 + 1) as u128;
                    (lo as i128 + (rng.next() as u128 % span) as i128) as i64
                }
            }
        };
        let mut out = Vec::new();
        for _ in 0..30 {
            let p = (
                near(self.lo.0, self.hi.0, rng),
                near(self.lo.1, self.hi.1, rng),
            );
            if !out.contains(&p) {
                out.push(p);
            }
            if out.len() == 6 {
                break;
            }
        }
        out
    }
}

/// Small, mostly linear expressions keep callers provable often enough.
fn small_e(rng: &mut Rng, vars: &[String]) -> E {
    match rng.below(5) {
        0 => E::Lit(rng.below(21) as i64 - 10),
        1 | 2 => E::Var(vars[rng.below(vars.len())].clone()),
        _ => gen_e(rng, vars, 1),
    }
}

fn gen_caller(rng: &mut Rng, n: usize, callees: usize) -> Caller {
    let range = |rng: &mut Rng| {
        let (a, b) = (rng.below(201) as i64 - 100, rng.below(201) as i64 - 100);
        (a.min(b), a.max(b))
    };
    let (lx, hx) = range(rng);
    let (ly, hy) = range(rng);
    let mut vars = vec!["x".to_string(), "y".to_string()];
    let mut binds = Vec::new();
    for i in 0..1 + rng.below(2) {
        let k = rng.below(callees);
        let (a, b) = (small_e(rng, &vars), small_e(rng, &vars));
        binds.push(if rng.chance(30) {
            let c = gen_c(rng, &vars, 1);
            let other = small_e(rng, &vars);
            Bind::CondCall(c, k, a, b, other)
        } else {
            Bind::Call(k, a, b)
        });
        vars.push(format!("a{i}"));
    }
    let body = small_e(rng, &vars);
    let post = if rng.chance(50) {
        let b = [1_000, 1_000_000_000_000][rng.below(2)];
        P::And(
            Box::new(P::Cmp(Cmp::Le, S::Lit(-b), sv("r"))),
            Box::new(P::Cmp(Cmp::Le, sv("r"), S::Lit(b))),
        )
    } else {
        gen_p(rng, &["x", "y", "r"], 1)
    };
    Caller {
        name: format!("c{n}"),
        lo: (lx, ly),
        hi: (hx, hy),
        binds,
        body,
        post,
    }
}

fn v(n: &str) -> Box<E> {
    Box::new(E::Var(n.into()))
}

fn sv(n: &'static str) -> S {
    S::Var(n)
}

/// Callees with exact contracts (sum, difference, maximum), or random ones.
fn gen_callee(rng: &mut Rng, k: usize) -> Func {
    let mut f = gen_func(rng, k);
    if rng.chance(60) {
        let w = [10, 100, 1000, 1 << 40][rng.below(4)];
        f.lo = (-w, -w);
        f.hi = (w, w);
        f.extra = None;
        f.lets = Vec::new();
        let (body, post) = match rng.below(3) {
            0 => (
                E::Bin(Op::Add, v("x"), v("y")),
                P::Cmp(
                    Cmp::Eq,
                    sv("r"),
                    S::Add(Box::new(sv("x")), Box::new(sv("y"))),
                ),
            ),
            1 => (
                E::Bin(Op::Sub, v("x"), v("y")),
                P::Cmp(
                    Cmp::Eq,
                    sv("r"),
                    S::Sub(Box::new(sv("x")), Box::new(sv("y"))),
                ),
            ),
            _ => (
                E::If(
                    Box::new(C::Cmp(Cmp::Lt, E::Var("x".into()), E::Var("y".into()))),
                    v("y"),
                    v("x"),
                ),
                P::And(
                    Box::new(P::Cmp(Cmp::Le, sv("x"), sv("r"))),
                    Box::new(P::Cmp(Cmp::Le, sv("y"), sv("r"))),
                ),
            ),
        };
        f.body = body;
        f.post = post;
        return f;
    }
    if rng.chance(60) {
        let w = [10, 100, 1000][rng.below(3)];
        f.lo = (-w, -w);
        f.hi = (w, w);
        f.extra = None;
    }
    f
}

#[derive(Default)]
struct Stats {
    callers: usize,
    checked: usize,
    runs: usize,
    cexes: usize,
    compiled_runs: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let callees: Vec<Func> = (0..3).map(|k| gen_callee(&mut rng, k)).collect();
    let callers: Vec<Caller> = (0..5)
        .map(|n| gen_caller(&mut rng, n, callees.len()))
        .collect();
    let mut src = String::from("#[summary(\"호출 건전성 퍼징\")]\nmod m {\n");
    for f in &callees {
        src.push_str(&f.text());
    }
    for c in &callers {
        src.push_str(&c.text(&callees));
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    let mut expected = String::new();
    for c in &callers {
        for (x, y) in c.inputs(&mut rng) {
            if let Outcome::Value(v) = c.run(&callees, x, y) {
                let _ = writeln!(
                    src,
                    "        print_i64(m::{}({}, {}));",
                    c.name,
                    lit(x),
                    lit(y)
                );
                let _ = writeln!(expected, "{v}");
            }
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("c.bmb".to_string(), src.clone())];
    let out = bmb::emit_llvm_sources(&files);
    assert!(
        out.ir.is_some(),
        "seed {seed}: generated program must check\n{src}\n{}",
        out.verify.output
    );
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{}", out.verify.output);
    let verified = |name: &str| {
        out.verify.output.lines().any(|l| {
            field(l, "def") == Some(&format!("m::{name}")) && field(l, "status") == Some("verified")
        })
    };

    for c in &callers {
        stats.callers += 1;
        if !verified(&c.name) || !c.uses().iter().all(|&k| verified(&callees[k].name)) {
            continue;
        }
        stats.checked += 1;
        for (x, y) in c.inputs(&mut rng) {
            stats.runs += 1;
            let at = format!("{}({x}, {y})", c.name);
            match c.run(&callees, x, y) {
                Outcome::Fault => panic!("{}", ctx(&format!("verified {at} faults"))),
                Outcome::BadCall(call) => {
                    panic!(
                        "{}",
                        ctx(&format!("verified {at} breaks the precondition of {call}"))
                    )
                }
                Outcome::Value(v) => assert!(
                    c.post_holds(x, y, v),
                    "{}",
                    ctx(&format!("verified {at} = {v} breaks its postcondition"))
                ),
            }
        }
    }

    for line in out.verify.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(c) = callers
            .iter()
            .find(|c| msg.contains(&format!("in `m::{}`", c.name)))
        else {
            continue;
        };
        let Some(cex) = parse_cex(msg) else { continue };
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        let (Some(x), Some(y)) = (get("x"), get("y")) else {
            panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
        };
        stats.cexes += 1;
        assert!(
            c.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
    }

    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("c{seed}-{all_checks}"));
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
fn call_proofs_and_builds_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-calls-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        batch(seed, &mut stats, &dir);
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (calls): {} callers, {} checked with verified callees ({} runs), {} counterexamples, {} compiled runs",
        stats.callers, stats.checked, stats.runs, stats.cexes, stats.compiled_runs
    );
    assert!(stats.checked > 0 && stats.cexes > 0);
}
