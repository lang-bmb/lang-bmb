//! Soundness fuzzing of the proof gate over function values (ADR-0053,
//! backlog 2V-20i).
//!
//! Each random function `hK(a, y)` (its contract a random scalar `Func`'s,
//! over `a` for `x`) computes an argument `x0 = ARG` from `a` and `y` and
//! passes a closure to a higher-order function: `apply` (`F: Fn`), `once`
//! (`F: FnMut`, the closure also counts its calls in a captured `n`) or
//! `consume` (`F: FnOnce`). The closure's contract and body are those of
//! another random `Func` over its parameter `x` and its capture `y`; half
//! the batches pass a named function `gK` with that contract instead.
//! As in `soundness.rs`, an independent interpreter checks every claim:
//!
//! - a verified closure (or `gK`) never faults on an input its `requires`
//!   admits and meets its `ensures`;
//! - a verified `hK`, given a verified closure, never faults computing the
//!   argument, the argument meets the closure's `requires`, and the result
//!   meets `hK`'s `ensures`;
//! - every definite counterexample of `hK` or the closure breaks it;
//! - the compiled program, built twice (proved checks removed, and every
//!   check kept), prints what the interpreter computes.
//!
//! It drives the `bmb` command line (HEALTH.md S5). `BMB_SOUND_ITERS`
//! raises the number of batches for longer runs.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::cli;
use common::scalar::*;

/// The higher-order function a caller passes its closure to.
#[derive(Clone, Copy)]
enum Bound {
    Fn,
    FnMut,
    FnOnce,
}

struct Case {
    /// `hK`'s contract (over `x`, written `a`) and the closure's (over `x`
    /// and the capture `y`), with its body.
    outer: Func,
    inner: Func,
    /// `x0 = ARG` over `a` and `y`.
    arg: E,
    bound: Bound,
    /// Passes the named function `gK` instead of a closure (`Fn` only).
    item: bool,
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    closures_verified: usize,
    callers_verified: usize,
    verified_runs: usize,
    cexes: usize,
    /// Caller counterexamples a replay cannot decide: the argument is
    /// admitted and the closure's own result meets `hK`'s `ensures`, but
    /// another result its contract allows may not.
    undecided_cexes: usize,
    compiled_runs: usize,
}

/// `e` with `x` as `a` (the caller's own parameter).
fn rename_e(e: &E) -> E {
    match e {
        E::Var(v) if v == "x" => E::Var("a".into()),
        E::Var(v) => E::Var(v.clone()),
        E::Lit(v) => E::Lit(*v),
        E::Bin(op, l, r) => E::Bin(*op, Box::new(rename_e(l)), Box::new(rename_e(r))),
        E::Neg(x) => E::Neg(Box::new(rename_e(x))),
        E::If(c, l, r) => E::If(
            Box::new(rename_c(c)),
            Box::new(rename_e(l)),
            Box::new(rename_e(r)),
        ),
    }
}

fn rename_c(c: &C) -> C {
    match c {
        C::Cmp(k, l, r) => C::Cmp(*k, rename_e(l), rename_e(r)),
        C::And(l, r) => C::And(Box::new(rename_c(l)), Box::new(rename_c(r))),
        C::Not(x) => C::Not(Box::new(rename_c(x))),
    }
}

/// The closure's body and contract: `inner`'s, with its `lets` and array
/// steps (`Func::text` without the function head).
fn closure_body(f: &Func) -> String {
    let mut t = String::new();
    for (v, e) in &f.lets {
        let _ = write!(t, "let {v}: i64 = {}; ", e_text(e));
    }
    if !f.arr.is_empty() {
        t.push_str("let mut t: [i64; 3] = [x, y, 0]; ");
    }
    for (cond, elems) in &f.arr {
        let lit = format!(
            "[{}]",
            elems.iter().map(e_text).collect::<Vec<_>>().join(", ")
        );
        let rhs = match cond {
            Some(c) => format!("if {} {{ {lit} }} else {{ t }}", c_text(c)),
            None => lit,
        };
        let _ = write!(t, "t = {rhs}; ");
    }
    t.push_str(&e_text(&f.body));
    t
}

impl Case {
    fn text(&self, k: usize) -> String {
        let (o, i) = (&self.outer, &self.inner);
        let mut t = String::new();
        let _ = write!(
            t,
            "    spec fn h{k}_pre(x: i64, y: i64) -> bool {{ {} }}\n\n    spec fn h{k}_post(x: i64, y: i64, r: i64) -> bool {{ {} }}\n\n",
            o.pre_text(),
            p_text(&o.post)
        );
        let _ = write!(
            t,
            "    spec fn g{k}_pre(x: i64, y: i64) -> bool {{ {} }}\n\n    spec fn g{k}_post(x: i64, y: i64, r: i64) -> bool {{ {} }}\n\n",
            i.pre_text(),
            p_text(&i.post)
        );
        let arg = e_text(&self.arg);
        let call = if self.item {
            // A function item captures nothing: `gK` takes `y` as its
            // second parameter (`apply2`).
            let _ = write!(
                t,
                "    #[effects(pure)]\n    #[uses(g{k}_pre, g{k}_post)]\n    fn g{k}(x: i64, y: i64) -> (r: i64)\n        requires g{k}_pre(x, y),\n        ensures g{k}_post(x, y, r),\n    {{\n        {}\n    }}\n\n",
                closure_body(i)
            );
            format!("apply2::<_>(g{k}, x0, y)")
        } else {
            // An `FnMut` closure also counts its calls in the captured `n`.
            let (f, req, count) = match self.bound {
                Bound::Fn => ("apply", "", ""),
                Bound::FnMut => ("once", "n < 100, ", "n = n + 1; "),
                Bound::FnOnce => ("consume", "", ""),
            };
            format!(
                // The contract inline (ADR-0095 A2): a closure's proof does
                // not see the spec fns its function reveals (backlog 2V-20k).
                "{f}::<_>(move |x: i64| -> (r: i64) requires {req}{}, ensures {}, {{ {count}{} }}, x0)",
                i.pre_text(),
                p_text(&i.post),
                closure_body(i)
            )
        };
        let counter = matches!(self.bound, Bound::FnMut) && !self.item;
        let _ = write!(
            t,
            "    #[effects(pure)]\n    #[uses(h{k}_pre, h{k}_post, g{k}_pre, g{k}_post)]\n    fn h{k}(a: i64, y: i64) -> (r: i64)\n        requires h{k}_pre(a, y),\n        ensures h{k}_post(a, y, r),\n    {{\n{}        let x0: i64 = {arg};\n        {call}\n    }}\n\n",
            if counter {
                "        let mut n: u64 = 0;\n"
            } else {
                ""
            }
        );
        t
    }

    /// What the interpreter says `hK(a, y)` does: the argument, then the
    /// closure on it (`None`: a fault on the way).
    fn run(&self, a: i64, y: i64) -> (Option<i64>, Option<Result<i64, Fault>>) {
        let env: Env = vec![("a".into(), a), ("y".into(), y)];
        let Ok(x0) = e_eval(&self.arg, &env) else {
            return (None, None);
        };
        (Some(x0), Some(self.inner.run(x0, y)))
    }
}

const HEAD: &str = "    #[effects(pure)]
    fn apply<F: Fn(i64) -> i64>(f: F, x: i64) -> (r: i64)
        requires f.requires(x),
        ensures f.ensures(x, r),
    {
        f(x)
    }

    #[effects(pure)]
    fn apply2<F: Fn(i64, i64) -> i64>(f: F, x: i64, y: i64) -> (r: i64)
        requires f.requires(x, y),
        ensures f.ensures(x, y, r),
    {
        f(x, y)
    }

    #[effects(pure)]
    fn consume<F: FnOnce(i64) -> i64>(f: F, x: i64) -> (r: i64)
        requires f.requires(x),
        ensures f.ensures(x, r),
    {
        f(x)
    }

    #[effects(pure)]
    fn once<F: FnMut(i64) -> i64>(f: F, x: i64) -> (r: i64)
        requires f.requires(x),
        ensures f.ensures(x, r),
    {
        f(x)
    }

";

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let cases: Vec<Case> = (0..6)
        .map(|k| {
            let outer = gen_func(&mut rng, k);
            let inner = gen_func(&mut rng, k);
            let arg = rename_e(&gen_e(&mut rng, &["x".to_string(), "y".to_string()], 2));
            let bound = [Bound::Fn, Bound::FnMut, Bound::FnOnce][rng.below(3)];
            let item = seed.is_multiple_of(2) && rng.chance(50);
            Case {
                outer,
                inner,
                arg,
                bound,
                item,
            }
        })
        .collect();
    let mut module = format!("#[summary(\"건전성 퍼징: 함수 값\")]\nmod m {{\n{HEAD}");
    for (k, c) in cases.iter().enumerate() {
        module.push_str(&c.text(k));
    }
    module.push_str("}\n\n");
    let mut src = format!(
        "{module}#[summary(\"실행\")]\nmod app {{\n    #[effects(io)]\n    fn main() -> () {{\n"
    );
    let mut expected = String::new();
    for (k, c) in cases.iter().enumerate() {
        for (a, y) in c.outer.inputs(&mut rng) {
            // Only calls the interpreter runs to the end, and whose argument
            // the closure admits (its own checks would trap otherwise).
            let (Some(x0), Some(Ok(r))) = c.run(a, y) else {
                continue;
            };
            if !c.inner.pre(x0, y) {
                continue;
            }
            let _ = writeln!(src, "        print_i64(m::h{k}({}, {}));", lit(a), lit(y));
            let _ = writeln!(expected, "{r}");
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("s.bmb".to_string(), src.clone())];
    let out = cli::emit_llvm(&files, &dir.join(format!("src{seed}")), None, false);
    assert!(
        out.ir.is_some(),
        "seed {seed}: generated program must check\n{src}\n{}",
        out.output
    );
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{}", out.output);

    // The closure of `hK` is the K-th one in source order (the functions
    // sort by name: `h0` .. `h5`), and `gK` is itself.
    let mut closures = Vec::new();
    for (k, c) in cases.iter().enumerate() {
        if !c.item {
            closures.push(k);
        }
    }
    let status = |def: &str| {
        out.output
            .lines()
            .find(|l| field(l, "def") == Some(def))
            .and_then(|l| field(l, "status"))
            == Some("verified")
    };
    for (k, c) in cases.iter().enumerate() {
        stats.funcs += 2;
        let inner_def = if c.item {
            format!("m::g{k}")
        } else {
            let n = closures.iter().position(|x| *x == k).unwrap();
            let head = format!("<m::__closure_{n} as ");
            out.output
                .lines()
                .filter_map(|l| field(l, "def"))
                .find(|d| d.starts_with(&head) && d.ends_with(">::call"))
                .unwrap_or_else(|| panic!("{}", ctx(&format!("no status line of closure {n}"))))
                .to_string()
        };
        let inner_ok = status(&inner_def);
        if inner_ok {
            stats.closures_verified += 1;
            for (x, y) in c.inner.inputs(&mut rng) {
                stats.verified_runs += 1;
                match c.inner.run(x, y) {
                    Ok(r) => assert!(
                        c.inner.post(x, y, r),
                        "{}",
                        ctx(&format!(
                            "verified {inner_def} breaks its ensures at x = {x}, y = {y}, r = {r}"
                        ))
                    ),
                    Err(Fault) => panic!(
                        "{}",
                        ctx(&format!("verified {inner_def} faults at x = {x}, y = {y}"))
                    ),
                }
            }
        }
        if !status(&format!("m::h{k}")) {
            continue;
        }
        stats.callers_verified += 1;
        if !inner_ok {
            continue;
        }
        for (a, y) in c.outer.inputs(&mut rng) {
            stats.verified_runs += 1;
            let (x0, r) = c.run(a, y);
            let Some(x0) = x0 else {
                panic!(
                    "{}",
                    ctx(&format!(
                        "verified m::h{k} faults on its argument at a = {a}, y = {y}"
                    ))
                );
            };
            assert!(
                c.inner.pre(x0, y),
                "{}",
                ctx(&format!(
                    "verified m::h{k} passes x = {x0}, which the closure does not admit (a = {a}, y = {y})"
                ))
            );
            match r {
                Some(Ok(r)) => assert!(
                    c.outer.post(a, y, r),
                    "{}",
                    ctx(&format!(
                        "verified m::h{k} breaks its ensures at a = {a}, y = {y}, r = {r}"
                    ))
                ),
                _ => panic!(
                    "{}",
                    ctx(&format!(
                        "verified closure of m::h{k} faults at x = {x0}, y = {y}"
                    ))
                ),
            }
        }
    }

    // Every definite counterexample breaks what it is reported in.
    for line in out.output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        let Some(cex) = parse_cex(msg) else {
            continue;
        };
        // Opaque spec fns: not a definite counterexample (as `soundness_specs`).
        if line.contains("only by name") {
            continue;
        }
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        for (k, c) in cases.iter().enumerate() {
            let broken = if msg.contains(&format!("in `m::h{k}`;")) {
                let (Some(a), Some(y)) = (get("a"), get("y")) else {
                    panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
                };
                if !c.outer.pre(a, y) {
                    panic!(
                        "{}",
                        ctx(&format!("counterexample breaks the requires: {msg}"))
                    );
                }
                // The caller sees the closure through its contract: it is
                // broken when the argument faults, the closure does not
                // admit it, or a result the contract allows breaks `hK`'s
                // `ensures`; the replay checks that last case for the
                // closure's own result only.
                match c.run(a, y) {
                    (None, _) => true,
                    (Some(x0), _) if !c.inner.pre(x0, y) => true,
                    (Some(_), Some(Ok(r))) if !c.outer.post(a, y, r) => true,
                    _ => {
                        stats.undecided_cexes += 1;
                        continue;
                    }
                }
            } else if msg.contains(&format!("in `m::h{k}::{{closure#0}}`;"))
                || msg.contains(&format!("in `m::g{k}`;"))
            {
                let (Some(x), Some(y)) = (get("x"), get("y")) else {
                    panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
                };
                if !c.inner.pre(x, y) {
                    panic!(
                        "{}",
                        ctx(&format!(
                            "counterexample breaks the closure's requires: {msg}"
                        ))
                    );
                }
                match c.inner.run(x, y) {
                    Err(Fault) => true,
                    Ok(r) => !c.inner.post(x, y, r),
                }
            } else {
                continue;
            };
            assert!(
                broken,
                "{}",
                ctx(&format!("counterexample does not break it: {msg}"))
            );
            stats.cexes += 1;
        }
    }

    // Both builds print what the interpreter computes.
    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            cli::emit_llvm(&files, &dir.join(format!("src{seed}")), None, true)
                .ir
                .unwrap()
        } else {
            out.ir.clone().unwrap()
        };
        let exe = dir.join(format!("s{seed}-{all_checks}"));
        cli::build(&ir, &exe).unwrap_or_else(|e| panic!("{}", ctx(&e)));
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
fn closure_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let dir = std::env::temp_dir().join(format!("bmb-sound-fn-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (function values): {} definitions, {} closures and {} callers verified ({} runs), {} counterexamples replayed ({} undecided), {} compiled runs",
        stats.funcs,
        stats.closures_verified,
        stats.callers_verified,
        stats.verified_runs,
        stats.cexes,
        stats.undecided_cexes,
        stats.compiled_runs
    );
    // The generator must reach every outcome.
    assert!(stats.closures_verified > 0 && stats.callers_verified > 0 && stats.cexes > 0);
}
