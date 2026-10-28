//! Soundness fuzzing of the proof gate over data types (backlog 2B-09).
//!
//! Random functions build a struct (fields written in a random order), may
//! update a field, may route it through a nested array (backlog 2V-04b: a
//! 2x2 grid of the values, one cell written, one cell read back), pick an
//! enum variant under a random condition, `match` it and return an integer
//! or a struct. Half the batches give `P` a random type invariant
//! (ADR-0060, backlog T1-11): a function may then take a `q: P` (assumed
//! valid) and returns a `P` it must prove valid. As in `soundness.rs`, an
//! independent interpreter checks every claim:
//!
//! - a verified function, on any input that satisfies its precondition (and
//!   a `q` that satisfies the invariant), never overflows or divides by zero,
//!   meets its postcondition and returns a valid `P`;
//! - every definite counterexample really breaks the function (an E0823 one
//!   returns a `P` that breaks the invariant);
//! - the compiled program, built twice (proved checks removed, and every check
//!   kept), prints what the interpreter computes.
//!
//! It drives the `bmb` command line, as tests that outlive the Rust
//! implementation must (HEALTH.md S5). `BMB_SOUND_ITERS` raises the number
//! of batches for longer runs.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;

/// A variant of `enum E { A, B(i64), C(P) }`.
enum Ctor {
    A,
    B(E),
    C,
}

struct DFunc {
    /// Name, precondition bounds and `extra`; its `body`, `lets`, `arr` and
    /// `post` are unused (the data body and post are below).
    base: Func,
    /// Takes `q: P` (only when `P` has an invariant): its fields may be read
    /// wherever `p`'s are.
    q: bool,
    /// `#[uses(p_inv)]`: the invariant's definition is visible.
    uses_inv: bool,
    /// `P { a, b }` from `x`, `y`; written `b` first when `b_first`.
    pa: E,
    pb: E,
    b_first: bool,
    /// `p.a = ..` after the literal, over `x`, `y`, `p.a`, `p.b`.
    upd: Option<E>,
    /// `let mut g: [[i64; 2]; 2] = [[x, y], [p.a, p.b]]; g[w] = val;
    /// p.b = g[r];` with `w`, `r` cells (row, column).
    grid: Option<Grid>,
    /// `let e: E = if cond { first } else { second };` (no `if` without cond).
    cond: Option<C>,
    first: Ctor,
    second: Ctor,
    /// Arm bodies: `E::A`, `E::B(v)`, `E::C(q)`.
    arm_a: E,
    arm_b: E,
    arm_c: E,
    /// Returns `P { a: <match>, b: p.b }` (else the match value).
    returns_p: bool,
    post: P,
}

struct Grid {
    w: (usize, usize),
    val: E,
    r: (usize, usize),
}

fn gen_ctor(rng: &mut Rng, vars: &[String]) -> Ctor {
    match rng.below(3) {
        0 => Ctor::A,
        1 => Ctor::B(gen_e(rng, vars, 1)),
        _ => Ctor::C,
    }
}

fn gen_dfunc(rng: &mut Rng, k: usize, inv: bool) -> DFunc {
    let base = gen_func(rng, k);
    let q = inv && rng.chance(50);
    let uses_inv = inv && rng.chance(60);
    let mut xy: Vec<String> = vec!["x".into(), "y".into()];
    let mut pv: Vec<String> = ["x", "y", "p.a", "p.b"].map(String::from).to_vec();
    if q {
        xy.extend(["q.a", "q.b"].map(String::from));
        pv.extend(["q.a", "q.b"].map(String::from));
    }
    let returns_p = rng.chance(50);
    let post_vars: &[&'static str] = if returns_p {
        &["x", "y", "r.a", "r.b"]
    } else {
        &["x", "y", "r"]
    };
    DFunc {
        q,
        uses_inv,
        pa: gen_e(rng, &xy, 2),
        pb: gen_e(rng, &xy, 2),
        b_first: rng.chance(50),
        upd: rng.chance(50).then(|| gen_e(rng, &pv, 2)),
        grid: rng.chance(50).then(|| Grid {
            w: (rng.below(2), rng.below(2)),
            val: gen_e(rng, &pv, 2),
            r: (rng.below(2), rng.below(2)),
        }),
        cond: rng.chance(60).then(|| gen_c(rng, &pv, 1)),
        first: gen_ctor(rng, &pv),
        second: gen_ctor(rng, &pv),
        arm_a: gen_e(rng, &xy, 2),
        arm_b: gen_e(rng, &["x", "y", "v"].map(String::from), 2),
        arm_c: gen_e(rng, &["x", "c.a", "c.b"].map(String::from), 2),
        returns_p,
        post: gen_p(rng, post_vars, 1),
        base,
    }
}

impl DFunc {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn ctor_text(c: &Ctor) -> String {
        match c {
            Ctor::A => "E::A".into(),
            Ctor::B(e) => format!("E::B({})", e_text(e)),
            Ctor::C => "E::C(p)".into(),
        }
    }

    fn text(&self) -> String {
        let n = self.name();
        let mut t = format!(
            "    spec fn {n}_pre(x: i64, y: i64) -> bool {{ {} }}\n\n",
            self.base.pre_text()
        );
        let (ret, post_params) = if self.returns_p {
            ("P", "x: i64, y: i64, r: P")
        } else {
            ("i64", "x: i64, y: i64, r: i64")
        };
        let _ = write!(
            t,
            "    spec fn {n}_post({post_params}) -> bool {{ {} }}\n\n",
            p_text(&self.post)
        );
        let inv = if self.uses_inv { ", p_inv" } else { "" };
        let q = if self.q { ", q: P" } else { "" };
        let _ = writeln!(
            t,
            "    #[effects(pure)]\n    #[uses({n}_pre, {n}_post{inv})]\n    fn {n}(x: i64, y: i64{q}) -> (r: {ret})\n        requires {n}_pre(x, y),\n        ensures {n}_post(x, y, r),\n    {{"
        );
        let (a, b) = (e_text(&self.pa), e_text(&self.pb));
        let lit = if self.b_first {
            format!("P {{ b: {b}, a: {a} }}")
        } else {
            format!("P {{ a: {a}, b: {b} }}")
        };
        let _ = writeln!(t, "        let mut p: P = {lit};");
        if let Some(u) = &self.upd {
            let _ = writeln!(t, "        p.a = {};", e_text(u));
        }
        if let Some(g) = &self.grid {
            let _ = writeln!(
                t,
                "        let mut g: [[i64; 2]; 2] = [[x, y], [p.a, p.b]];\n        g[{}][{}] = {};\n        p.b = g[{}][{}];",
                g.w.0,
                g.w.1,
                e_text(&g.val),
                g.r.0,
                g.r.1
            );
        }
        let pick = match &self.cond {
            Some(c) => format!(
                "if {} {{ {} }} else {{ {} }}",
                c_text(c),
                Self::ctor_text(&self.first),
                Self::ctor_text(&self.second)
            ),
            None => Self::ctor_text(&self.first),
        };
        let _ = writeln!(t, "        let e: E = {pick};");
        let m = format!(
            "match e {{ E::A => {}, E::B(v) => {}, E::C(c) => {} }}",
            e_text(&self.arm_a),
            e_text(&self.arm_b),
            e_text(&self.arm_c)
        );
        if self.returns_p {
            let _ = writeln!(t, "        let m: i64 = {m};");
            let _ = writeln!(t, "        P {{ a: m, b: p.b }}\n    }}\n");
        } else {
            let _ = writeln!(t, "        {m}\n    }}\n");
        }
        t
    }

    /// The result: the match value, and `p.b` when a struct is returned.
    fn run(&self, x: i64, y: i64, q: (i64, i64)) -> Result<(i64, i64), Fault> {
        let mut env: Env = vec![("x".into(), x), ("y".into(), y)];
        if self.q {
            env.push(("q.a".into(), q.0));
            env.push(("q.b".into(), q.1));
        }
        // Both fields are evaluated; a fault in either stops the run.
        let (pa, pb) = if self.b_first {
            let pb = e_eval(&self.pb, &env)?;
            (e_eval(&self.pa, &env)?, pb)
        } else {
            let pa = e_eval(&self.pa, &env)?;
            (pa, e_eval(&self.pb, &env)?)
        };
        env.push(("p.a".into(), pa));
        env.push(("p.b".into(), pb));
        if let Some(u) = &self.upd {
            let v = e_eval(u, &env)?;
            env.push(("p.a".into(), v));
        }
        if let Some(g) = &self.grid {
            let mut cells = [[x, y], [lookup(&env, "p.a"), lookup(&env, "p.b")]];
            cells[g.w.0][g.w.1] = e_eval(&g.val, &env)?;
            env.push(("p.b".into(), cells[g.r.0][g.r.1]));
        }
        let chosen = match &self.cond {
            Some(c) if !c_eval(c, &env)? => &self.second,
            _ => &self.first,
        };
        let m = match chosen {
            Ctor::A => e_eval(&self.arm_a, &env)?,
            Ctor::B(e) => {
                let v = e_eval(e, &env)?;
                let arm: Env = vec![("x".into(), x), ("y".into(), y), ("v".into(), v)];
                e_eval(&self.arm_b, &arm)?
            }
            Ctor::C => {
                let arm: Env = vec![
                    ("x".into(), x),
                    ("c.a".into(), lookup(&env, "p.a")),
                    ("c.b".into(), lookup(&env, "p.b")),
                ];
                e_eval(&self.arm_c, &arm)?
            }
        };
        Ok((m, lookup(&env, "p.b")))
    }

    fn post(&self, x: i64, y: i64, (m, b): (i64, i64)) -> bool {
        let env: Vec<(&str, i128)> = if self.returns_p {
            vec![
                ("x", x as i128),
                ("y", y as i128),
                ("r.a", m as i128),
                ("r.b", b as i128),
            ]
        } else {
            vec![("x", x as i128), ("y", y as i128), ("r", m as i128)]
        };
        p_eval(&self.post, &env)
    }

    /// The printed lines of a run: the match value, and `p.b`.
    fn printed(&self, r: (i64, i64)) -> String {
        if self.returns_p {
            format!("{}\n{}\n", r.0, r.1)
        } else {
            format!("{}\n", r.0)
        }
    }
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    verified_runs: usize,
    cexes: usize,
    compiled_runs: usize,
    /// Under a random invariant: verified functions that take a `q` or
    /// return a `P`, and E0823 counterexamples replayed.
    inv_verified: usize,
    inv_cexes: usize,
}

/// `x` and `y` of a counterexample.
fn cex_xy(msg: &str) -> Option<(i64, i64)> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    let get = |k: &str| {
        rest.split(", ")
            .find_map(|kv| kv.strip_prefix(&format!("{k} = ")))
            .and_then(|v| v.trim().parse().ok())
    };
    Some((get("x")?, get("y")?))
}

/// `q = P { a: .., b: .. }` of a counterexample.
fn cex_q(msg: &str) -> Option<(i64, i64)> {
    let rest = msg.split("; counterexample: ").nth(1)?;
    let at = rest.find("q = P { a: ")? + "q = P { a: ".len();
    let (a, tail) = rest[at..].split_once(", b: ")?;
    let (b, _) = tail.split_once(" }")?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    // A random invariant of `P` in half the batches (ADR-0060).
    let inv: Option<P> = rng.chance(50).then(|| gen_p(&mut rng, &["p.a", "p.b"], 1));
    let valid = |a: i64, b: i64| {
        inv.as_ref()
            .is_none_or(|p| p_eval(p, &[("p.a", a as i128), ("p.b", b as i128)]))
    };
    let funcs: Vec<DFunc> = (0..6)
        .map(|k| gen_dfunc(&mut rng, k, inv.is_some()))
        .collect();
    let mut src = String::from("#[summary(\"데이터 타입 건전성 퍼징\")]\nmod m {\n");
    match &inv {
        Some(p) => {
            let _ = write!(
                src,
                "    #[invariant(p_inv)]\n    struct P {{ a: i64, b: i64 }}\n\n    spec fn p_inv(p: &P) -> bool {{ {} }}\n\n",
                p_text(p)
            );
        }
        None => src.push_str("    struct P { a: i64, b: i64 }\n\n"),
    }
    src.push_str("    enum E { A, B(i64), C(P) }\n\n");
    for f in &funcs {
        src.push_str(&f.text());
    }
    src.push_str(
        "}\n\n#[summary(\"실행\")]\nmod app {\n    #[effects(io)]\n    fn main() -> () {\n",
    );
    // Values of `P` that keep the invariant: a function's `q`.
    let mut qs: Vec<(i64, i64)> = Vec::new();
    for _ in 0..80 {
        let q = (value(&mut rng), value(&mut rng));
        if valid(q.0, q.1) && !qs.contains(&q) {
            qs.push(q);
        }
        if qs.len() == 3 {
            break;
        }
    }
    let q_inputs = |f: &DFunc| -> Vec<(i64, i64)> { if f.q { qs.clone() } else { vec![(0, 0)] } };
    let mut expected = String::new();
    for f in &funcs {
        for (x, y) in f.base.inputs(&mut rng) {
            for q in q_inputs(f) {
                if let Ok(r) = f.run(x, y, q) {
                    let qa = if f.q {
                        format!(", m::P {{ a: {}, b: {} }}", lit(q.0), lit(q.1))
                    } else {
                        String::new()
                    };
                    let call = format!("m::{}({}, {}{qa})", f.name(), lit(x), lit(y));
                    if f.returns_p {
                        let _ = writeln!(src, "        print_i64({call}.a);");
                        let _ = writeln!(src, "        print_i64({call}.b);");
                    } else {
                        let _ = writeln!(src, "        print_i64({call});");
                    }
                    expected.push_str(&f.printed(r));
                }
            }
        }
    }
    src.push_str("    }\n}\n");
    let file = dir.join(format!("d{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    // `emit-llvm` writes the IR whenever the program checks (with runtime
    // checks where proofs are missing) and prints the verification rows.
    let emit = |all_checks: bool| -> (String, Option<String>) {
        let ll = dir.join(format!("d{seed}-{all_checks}.ll"));
        let _ = fs::remove_file(&ll);
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("emit-llvm");
        if all_checks {
            cmd.arg("--all-checks");
        }
        let out = cmd.arg(&file).arg("-o").arg(&ll).output().unwrap();
        let ir = fs::read_to_string(&ll).ok();
        let _ = fs::remove_file(&ll);
        (String::from_utf8_lossy(&out.stdout).into_owned(), ir)
    };
    let (output, ir) = emit(false);
    assert!(
        ir.is_some(),
        "seed {seed}: generated program must check\n{src}\n{output}"
    );
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{output}");

    for line in output.lines() {
        let Some(name) = field(line, "def").and_then(|d| d.strip_prefix("m::")) else {
            continue;
        };
        let f = funcs.iter().find(|f| f.name() == name).unwrap();
        stats.funcs += 1;
        if field(line, "status") != Some("verified") {
            continue;
        }
        stats.verified += 1;
        if inv.is_some() && (f.q || f.returns_p) {
            stats.inv_verified += 1;
        }
        for (x, y) in f.base.inputs(&mut rng) {
            for q in q_inputs(f) {
                stats.verified_runs += 1;
                match f.run(x, y, q) {
                    Ok(r) => {
                        assert!(
                            f.post(x, y, r),
                            "{}",
                            ctx(&format!(
                                "verified {name} breaks its postcondition at x = {x}, y = {y}, q = {q:?}, r = {r:?}"
                            ))
                        );
                        // The `P` it returns keeps the invariant.
                        assert!(
                            !f.returns_p || valid(r.0, r.1),
                            "{}",
                            ctx(&format!(
                                "verified {name} returns a `P` that breaks the invariant at x = {x}, y = {y}, q = {q:?}, r = {r:?}"
                            ))
                        );
                    }
                    Err(Fault) => panic!(
                        "{}",
                        ctx(&format!(
                            "verified {name} faults at x = {x}, y = {y}, q = {q:?}"
                        ))
                    ),
                }
            }
        }
    }

    for line in output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        if !msg.contains("; counterexample: ") {
            continue;
        }
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::{}`", f.name())))
        else {
            continue;
        };
        let Some((x, y)) = cex_xy(msg) else {
            panic!("{}", ctx(&format!("counterexample without inputs: {msg}")));
        };
        let q = if f.q {
            cex_q(msg)
                .unwrap_or_else(|| panic!("{}", ctx(&format!("counterexample without `q`: {msg}"))))
        } else {
            (0, 0)
        };
        // E0823 (ADR-0060): the result breaks the invariant. With the
        // predicate opaque (no `#[uses(p_inv)]`) the model may give it any
        // truth value, so only a revealed one is replayed.
        let about_inv = msg.contains("type invariant of");
        if (about_inv || f.q) && !f.uses_inv {
            continue;
        }
        stats.cexes += 1;
        stats.inv_cexes += usize::from(about_inv);
        assert!(
            f.base.pre(x, y),
            "{}",
            ctx(&format!("counterexample breaks the precondition: {msg}"))
        );
        assert!(
            !f.q || valid(q.0, q.1),
            "{}",
            ctx(&format!(
                "counterexample breaks the invariant of `q`: {msg}"
            ))
        );
        let broken = match f.run(x, y, q) {
            Err(Fault) => true,
            Ok(r) if about_inv => !valid(r.0, r.1),
            Ok(r) => !f.post(x, y, r),
        };
        assert!(
            broken,
            "{}",
            ctx(&format!(
                "counterexample does not break {}: {msg}",
                f.name()
            ))
        );
    }

    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            emit(true).1.unwrap()
        } else {
            ir.clone().unwrap()
        };
        let exe = dir.join(format!("d{seed}-{all_checks}"));
        let ll = dir.join(format!("d{seed}-{all_checks}.build.ll"));
        fs::write(&ll, &ir).unwrap();
        let cc = Command::new("clang")
            .args(["-O2", "-Wno-override-module", "-o"])
            .arg(&exe)
            .arg(&ll)
            .output()
            .unwrap();
        let _ = fs::remove_file(&ll);
        assert!(
            cc.status.success(),
            "{}",
            ctx(&String::from_utf8_lossy(&cc.stderr))
        );
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
fn data_type_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
    let iters: u64 = std::env::var("BMB_SOUND_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let start: u64 = std::env::var("BMB_SOUND_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    if std::env::var("BMB_Z3_RLIMIT").is_err() {
        // SAFETY: set before any thread of this test binary reads the
        // environment; this file holds a single test.
        unsafe { std::env::set_var("BMB_Z3_RLIMIT", "500000") };
    }
    let dir = std::env::temp_dir().join(format!("bmb-sound-data-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (data): {} functions, {} verified ({} runs), {} counterexamples replayed, {} compiled runs; invariants: {} verified, {} E0823 counterexamples replayed",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.cexes,
        stats.compiled_runs,
        stats.inv_verified,
        stats.inv_cexes
    );
    assert!(stats.verified > 0 && stats.cexes > 0);
}
