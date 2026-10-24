//! Soundness fuzzing of recursion and termination (backlog 1B-32).
//!
//! `fn r(n: u64, x: i64) -> (out: i64) decreases n` with a random base case,
//! a random step `n - c` (c may be 0, which never terminates), a random
//! guard and a random combination of the recursive result. When the verifier
//! accepts one, the interpreter must find, on every input the precondition
//! admits: termination, no fault (overflow, u64 underflow), and the
//! postcondition. Counterexamples must satisfy the precondition. Both builds
//! must print what the interpreter computes.
//!
//! `BMB_SOUND_ITERS` / `BMB_SOUND_SEED` as in `soundness.rs`.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;

struct Rec {
    name: String,
    /// n <= n_max
    n_max: u64,
    x_lo: i64,
    x_hi: i64,
    /// `if n < guard { base } else { step(r(n - c, arg)) }`
    guard: u64,
    c: u64,
    base: E,
    arg: E,
    /// over `x` and `acc` (the recursive result)
    step: E,
    post: P,
}

#[derive(Debug)]
enum Run {
    Fault,
    Diverges,
    Value(i64),
}

impl Rec {
    fn pre(&self, n: u64, x: i64) -> bool {
        n <= self.n_max && self.x_lo <= x && x <= self.x_hi
    }

    fn run(&self, n: u64, x: i64, depth: u32) -> Run {
        if depth > 200 {
            return Run::Diverges;
        }
        let env: Env = vec![("x".into(), x)];
        if n < self.guard {
            return match e_eval(&self.base, &env) {
                Ok(v) => Run::Value(v),
                Err(_) => Run::Fault,
            };
        }
        let Some(m) = n.checked_sub(self.c) else {
            return Run::Fault;
        };
        let Ok(a) = e_eval(&self.arg, &env) else {
            return Run::Fault;
        };
        match self.run(m, a, depth + 1) {
            Run::Value(acc) => {
                let env: Env = vec![("x".into(), x), ("acc".into(), acc)];
                match e_eval(&self.step, &env) {
                    Ok(v) => Run::Value(v),
                    Err(_) => Run::Fault,
                }
            }
            other => other,
        }
    }

    fn post_holds(&self, n: u64, x: i64, out: i64) -> bool {
        p_eval(
            &self.post,
            &[("n", n as i128), ("x", x as i128), ("r", out as i128)],
        )
    }

    fn text(&self) -> String {
        let n = &self.name;
        let mut t = format!(
            "    spec fn {n}_pre(n: u64, x: i64) -> bool {{ n <= {} && {} <= x && x <= {} }}\n\n",
            self.n_max,
            lit(self.x_lo),
            lit(self.x_hi)
        );
        let _ = write!(
            t,
            "    spec fn {n}_post(n: u64, x: i64, r: i64) -> bool {{ {} }}\n\n",
            p_text(&self.post)
        );
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_post)]\n    fn {n}(n: u64, x: i64) -> (r: i64)\n        requires {n}_pre(n, x),\n        ensures {n}_post(n, x, r),\n        decreases n,\n    {{"
        );
        let _ = writeln!(
            t,
            "        if n < {} {{\n            {}\n        }} else {{\n            let acc: i64 = {n}(n - {}, {});\n            {}\n        }}\n    }}\n",
            self.guard,
            e_text(&self.base),
            self.c,
            e_text(&self.arg),
            e_text(&self.step)
        );
        t
    }

    fn inputs(&self, rng: &mut Rng) -> Vec<(u64, i64)> {
        let mut out = Vec::new();
        for _ in 0..30 {
            let n = rng.below(self.n_max as usize + 1) as u64;
            let span = (self.x_hi as i128 - self.x_lo as i128 + 1) as u128;
            let x = match rng.below(3) {
                0 => self.x_lo,
                1 => self.x_hi,
                _ => (self.x_lo as i128 + (rng.next() as u128 % span) as i128) as i64,
            };
            if !out.contains(&(n, x)) {
                out.push((n, x));
            }
            if out.len() == 6 {
                break;
            }
        }
        out
    }
}

fn gen_rec(rng: &mut Rng, k: usize) -> Rec {
    let x = vec!["x".to_string()];
    let xa = vec!["x".to_string(), "acc".to_string()];
    let w = [10, 1000, 1 << 30][rng.below(3)];
    // Mostly the shape that terminates and fits; sometimes a near miss.
    let (c, guard) = match rng.below(10) {
        0 => (0, 1),
        1 => (2, 1),
        2 => (2, 2),
        3 => (1, 0),
        _ => (1, 1),
    };
    let base = if rng.chance(50) {
        E::Var("x".into())
    } else {
        gen_e(rng, &x, 1)
    };
    let arg = if rng.chance(60) {
        E::Var("x".into())
    } else {
        gen_e(rng, &x, 1)
    };
    let step = match rng.below(3) {
        0 => E::Bin(Op::Add, Box::new(E::Var("acc".into())), Box::new(E::Lit(1))),
        1 => E::Bin(
            Op::Add,
            Box::new(E::Var("acc".into())),
            Box::new(E::Var("x".into())),
        ),
        _ => gen_e(rng, &xa, 2),
    };
    let step_kind = match &step {
        E::Bin(Op::Add, _, b) if matches!(**b, E::Lit(1)) => 1,
        E::Bin(Op::Add, _, b) if matches!(**b, E::Var(_)) => 2,
        _ => 0,
    };
    let exact_base = matches!(base, E::Var(_)) && matches!(arg, E::Var(_));
    let post = match rng.below(4) {
        // r == x + n: exact for base x, arg x, step acc + 1.
        0 if step_kind == 1 && exact_base => P::Cmp(
            Cmp::Eq,
            S::Var("r"),
            S::Add(Box::new(S::Var("x")), Box::new(S::Var("n"))),
        ),
        // |r| <= w * (n + 1): inductive for base x, arg x, step acc + x.
        0 | 1 if exact_base => {
            let bound = S::Scale(
                w,
                Box::new(S::Add(Box::new(S::Var("n")), Box::new(S::Lit(1)))),
            );
            let neg = S::Scale(
                -w,
                Box::new(S::Add(Box::new(S::Var("n")), Box::new(S::Lit(1)))),
            );
            P::And(
                Box::new(P::Cmp(Cmp::Le, neg, S::Var("r"))),
                Box::new(P::Cmp(Cmp::Le, S::Var("r"), bound)),
            )
        }
        _ => gen_p(rng, &["n", "x", "r"], 1),
    };
    Rec {
        name: format!("r{k}"),
        n_max: [3, 10, 20][rng.below(3)],
        x_lo: -w,
        x_hi: w,
        guard,
        c,
        base,
        arg,
        step,
        post,
    }
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    runs: usize,
    cexes: usize,
    compiled_runs: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let recs: Vec<Rec> = (0..6).map(|k| gen_rec(&mut rng, k)).collect();
    let mut src = String::from("#[summary(\"재귀 건전성 퍼징\")]\nmod m {\n");
    for r in &recs {
        src.push_str(&r.text());
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    let mut expected = String::new();
    for r in &recs {
        for (n, x) in r.inputs(&mut rng) {
            if let Run::Value(v) = r.run(n, x, 0) {
                let _ = writeln!(src, "        print_i64(m::{}({n}, {}));", r.name, lit(x));
                let _ = writeln!(expected, "{v}");
            }
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("r.bmb".to_string(), src.clone())];
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
        let Some(r) = recs.iter().find(|r| r.name == name) else {
            continue;
        };
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        for (n, x) in r.inputs(&mut rng) {
            stats.runs += 1;
            match r.run(n, x, 0) {
                Run::Value(v) => assert!(
                    r.post_holds(n, x, v),
                    "{}",
                    ctx(&format!(
                        "verified {name}({n}, {x}) = {v} breaks its postcondition"
                    ))
                ),
                other => panic!("{}", ctx(&format!("verified {name}({n}, {x}): {other:?}"))),
            }
        }
    }

    for line in out.verify.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(r) = recs
            .iter()
            .find(|r| msg.contains(&format!("in `m::{}`", r.name)))
        else {
            continue;
        };
        let Some(cex) = parse_cex(msg) else { continue };
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        let (Some(n), Some(x)) = (get("n"), get("x")) else {
            continue;
        };
        stats.cexes += 1;
        assert!(
            n >= 0 && r.pre(n as u64, x),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
    }

    let lines = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            bmb::emit_llvm_sources_with(&files, true).ir.unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("r{seed}-{all_checks}"));
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
fn recursion_proofs_and_builds_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-rec-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        batch(seed, &mut stats, &dir);
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (recursion): {} functions, {} verified ({} runs), {} counterexamples, {} compiled runs",
        stats.funcs, stats.verified, stats.runs, stats.cexes, stats.compiled_runs
    );
    assert!(stats.verified > 0 && stats.cexes > 0);
}
