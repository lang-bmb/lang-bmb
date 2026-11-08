//! Soundness fuzzing of the proof gate over traits and generics (backlog
//! 2T-05, ADR-0046).
//!
//! Each batch declares a trait `Val` (a spec method `val` and an exact
//! method `get` whose contract is `r == val(self)`), three fixed impls (`B`,
//! the generic wrapper `N<T>`, which delegates through the bound, and `M`,
//! a move type that owns a `Vec`, backlog 2T-09) and one random impl `R`
//! whose body and own postcondition are random, so its refinement
//! obligations (E0817) may fail. Every random scalar function of
//! `common::scalar` is written twice: `h` calls `get` on a `B` and an
//! `N<B>` (a known `Self`: the impls' contracts), and `f<T: Val>` calls it
//! on its parameters (a bounded `Self`: the trait's contract, proved once).
//! An independent interpreter checks every claim:
//!
//! - the fixed impls verify;
//! - a verified `R::get` returns `v` (the trait's contract) on any `v` it
//!   runs on;
//! - a verified `h` or `f`, on any input that satisfies its precondition,
//!   never overflows or divides by zero and meets its postcondition;
//! - every definite counterexample of `h`, and of `R::get`'s body, really
//!   breaks it;
//! - the compiled program, built twice (proved checks removed, and every
//!   check kept), prints what the interpreter computes for `h`, `f::<B>`,
//!   `f::<N<B>>`, `f::<M>` (its arguments move in and are dropped there)
//!   and `R::get`;
//! - a call the interpreter says faults runs as a program of its own
//!   (`common::trap`, backlog 1B-54): both builds print the line before it
//!   and end by the trap.
//!
//! Odd seeds write the trait calls as method calls (`a.get()`,
//! `self.inner.val()`, ADR-0051, backlog 2M-02), even seeds by path. `R`
//! also has an inherent `get` (`v` plus `INHERENT_SHIFT`) that a method call on an `R`
//! names before the trait's: the run prints `r.get()` (the inherent one)
//! next to `Val::get(&r)` (the trait's random body).
//!
//! It drives the `bmb` command line (HEALTH.md S5). `BMB_SOUND_ITERS`
//! raises the number of batches for longer runs.

use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::scalar::*;
use common::trap;

/// The random impl `R`: `get` returns `body` (over `v`, the field), and
/// ensures `got` (the trait's contract) or its own `post` over `v`, `r`.
struct RImpl {
    body: E,
    post: Option<P>,
}

impl RImpl {
    fn get(&self, v: i64) -> Result<i64, Fault> {
        e_eval(&self.body, &vec![("v".into(), v)])
    }

    fn text(&self) -> String {
        let (uses, ensures) = match &self.post {
            Some(p) => {
                let pre = format!(
                    "    spec fn r_post(v: i64, r: i64) -> bool {{ {} }}\n\n",
                    p_text(p)
                );
                (pre, "r_post(self.v, r)".to_string())
            }
            None => (String::new(), "got::<R>(self, r)".to_string()),
        };
        let reveal = if self.post.is_some() {
            "got, Val::val, r_post"
        } else {
            "got, Val::val"
        };
        format!(
            "{uses}    impl Val for R {{\n        spec fn val(self: &R) -> i64 {{\n            self.v\n        }}\n\n        #[effects(pure)]\n        #[uses({reveal})]\n        fn get(self: &R) -> (r: i64)\n            ensures {ensures},\n        {{\n            let v: i64 = self.v;\n            {}\n        }}\n    }}\n\n",
            e_text(&self.body)
        )
    }
}

const PRELUDE: &str = r#"    trait Val {
        spec fn val(self: &Self) -> i64;

        #[effects(pure)]
        fn get(self: &Self) -> (r: i64)
            ensures got::<Self>(self, r);
    }

    spec fn got<T: Val>(x: &T, r: i64) -> bool { r == Val::val(x) }

    struct B { v: i64 }

    struct N<T: Copy> { inner: T }

    struct R { v: i64 }

    struct M { v: i64, log: Vec<i64> }

    impl M {
        #[effects(alloc)]
        fn mk(v: i64) -> M {
            let mut log: Vec<i64> = Vec::new();
            log.push(v);
            M { v: v, log: log }
        }
    }

    impl Val for M {
        spec fn val(self: &M) -> i64 {
            self.v
        }

        #[effects(pure)]
        #[uses(got, Val::val)]
        fn get(self: &M) -> (r: i64)
            ensures got::<M>(self, r),
        {
            self.v
        }
    }

    impl R {
        #[effects(pure)]
        fn get(self: &R) -> i64 {
            self.v.wrapping_add(1000003)
        }
    }

    impl Val for B {
        spec fn val(self: &B) -> i64 {
            self.v
        }

        #[effects(pure)]
        #[uses(got, Val::val)]
        fn get(self: &B) -> (r: i64)
            ensures got::<B>(self, r),
        {
            self.v
        }
    }

    impl<T: Copy + Val> Val for N<T> {
        spec fn val(self: &N<T>) -> i64 {
            Val::val(&self.inner)
        }

        #[effects(pure)]
        #[uses(got, Val::val)]
        fn get(self: &N<T>) -> (r: i64)
            ensures got::<N<T>>(self, r),
        {
            Val::get(&self.inner)
        }
    }

"#;

/// What the inherent `R::get` adds to `v`, so it differs from the random
/// `R::get` body: that adds nothing, an interesting value or a small one
/// (`common::scalar::value`), none of which is this, or a 64-bit random one.
const INHERENT_SHIFT: i64 = 1000003;

/// `text` with each trait call by path `Val::get(&a)`, `Val::val(x)`,
/// `m::Val::get(&r0)` written as a method call (`a.get()`, `x.val()`):
/// the receiver is a path of names and fields.
fn method_calls(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("Val::") {
        let (head, tail) = rest.split_at(at);
        let head = head.strip_suffix("m::").unwrap_or(head);
        let after = &tail["Val::".len()..];
        let name_len = after
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        let (name, args) = after.split_at(name_len);
        let recv = args
            .strip_prefix('(')
            .map(|a| a.strip_prefix('&').unwrap_or(a))
            .and_then(|a| {
                let n = a.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))?;
                (a[n..].starts_with(')') && n > 0).then(|| (&a[..n], &a[n + 1..]))
            });
        match recv {
            Some((r, after_call)) if name == "get" || name == "val" => {
                out.push_str(head);
                let _ = write!(out, "{r}.{name}()");
                rest = after_call;
            }
            _ => {
                out.push_str(&rest[..at + "Val::".len()]);
                rest = &rest[at + "Val::".len()..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// `text` with the whole words `x` and `y` replaced.
fn subst_xy(text: &str, x: &str, y: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |w: &mut String, out: &mut String| {
        out.push_str(match w.as_str() {
            "x" => x,
            "y" => y,
            other => other,
        });
        w.clear();
    };
    for c in text.chars() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

/// The statements and result of `f`'s body over locals `x`, `y` (as
/// `Func::text` writes them).
fn body_text(f: &Func) -> String {
    let mut t = String::new();
    for (v, e) in &f.lets {
        let _ = writeln!(t, "        let {v}: i64 = {};", e_text(e));
    }
    if !f.arr.is_empty() {
        let _ = writeln!(t, "        let mut t: [i64; 3] = [x, y, 0];");
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
        let _ = writeln!(t, "        t = {rhs};");
    }
    let _ = writeln!(t, "        {}\n    }}\n", e_text(&f.body));
    t
}

/// `h{k}` (a known `Self`) and `f{k}<T: Val>` (a bounded `Self`) for `f`.
fn funcs_text(f: &Func) -> String {
    let k = &f.name[1..];
    let (pre, post) = (f.pre_text(), p_text(&f.post));
    let mut t = String::new();
    let _ = write!(
        t,
        "    spec fn h{k}_pre(x: i64, y: i64) -> bool {{ {pre} }}\n\n    spec fn h{k}_post(x: i64, y: i64, r: i64) -> bool {{ {post} }}\n\n"
    );
    let _ = write!(
        t,
        "    #[effects(pure)]\n    #[uses(h{k}_pre, h{k}_post, got, Val::val)]\n    fn h{k}(x0: i64, y0: i64) -> (r: i64)\n        requires h{k}_pre(x0, y0),\n        ensures h{k}_post(x0, y0, r),\n    {{\n        let a: B = B {{ v: x0 }};\n        let n: N<B> = N {{ inner: B {{ v: y0 }} }};\n        let x: i64 = Val::get(&a);\n        let y: i64 = Val::get(&n);\n{}",
        body_text(f)
    );
    let (va, vb) = ("Val::val(a)", "Val::val(b)");
    let _ = write!(
        t,
        "    spec fn f{k}_pre<T: Val>(a: &T, b: &T) -> bool {{ {} }}\n\n    spec fn f{k}_post<T: Val>(a: &T, b: &T, r: i64) -> bool {{ {} }}\n\n",
        subst_xy(&pre, va, vb),
        subst_xy(&post, va, vb)
    );
    let _ = write!(
        t,
        "    #[effects(pure)]\n    #[uses(f{k}_pre, f{k}_post, got)]\n    fn f{k}<T: Val>(a: T, b: T) -> (r: i64)\n        requires f{k}_pre::<T>(&a, &b),\n        ensures f{k}_post::<T>(&a, &b, r),\n    {{\n        let x: i64 = Val::get(&a);\n        let y: i64 = Val::get(&b);\n{}",
        body_text(f)
    );
    t
}

#[derive(Default)]
struct Stats {
    funcs: usize,
    verified: usize,
    verified_runs: usize,
    r_verified: usize,
    cexes: usize,
    generic_cexes: usize,
    compiled_runs: usize,
    /// Calls that fault in the interpreter, run as trap programs (1B-54).
    traps: usize,
    /// Batches written with method calls (ADR-0051).
    method_batches: usize,
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let funcs: Vec<Func> = (0..4).map(|k| gen_func(&mut rng, k)).collect();
    // `R::get` is `v` or `v` shifted (never faulting), and ensures the
    // trait's contract, nothing (`r <= r`), or a random property: so it
    // often meets its own contract but not the trait's (E0817).
    let r = RImpl {
        body: if rng.chance(50) {
            E::Var("v".into())
        } else {
            E::Bin(
                Op::WrapAdd,
                Box::new(E::Var("v".into())),
                Box::new(E::Lit(value(&mut rng))),
            )
        },
        post: match rng.below(10) {
            0..=3 => None,
            4..=6 => Some(P::Cmp(Cmp::Le, S::Var("r"), S::Var("r"))),
            _ => Some(gen_p(&mut rng, &["v", "r"], 1)),
        },
    };
    let mut src = String::from("#[summary(\"trait 건전성 퍼징\")]\nmod m {\n");
    src.push_str(PRELUDE);
    src.push_str(&r.text());
    for f in &funcs {
        src.push_str(&funcs_text(f));
    }
    let methods = seed % 2 == 1;
    if methods {
        src = method_calls(&src);
        assert!(
            src.contains("a.get()")
                && src.contains("self.inner.val()")
                && !src.contains("Val::get(&a)"),
            "{src}"
        );
        stats.method_batches += 1;
    }
    // The run: every function at inputs it runs on, and `R::get`.
    let mut uses: Vec<String> = funcs
        .iter()
        .flat_map(|f| {
            let k = &f.name[1..];
            [format!("m::h{k}_pre"), format!("m::f{k}_pre")]
        })
        .collect();
    uses.extend(["m::got".to_string(), "m::Val::val".to_string()]);
    src.push_str("}\n\n");
    let module = src.clone();
    let main_head = format!(
        "#[summary(\"실행\")]\nmod app {{\n    #[effects(alloc, io)]\n    #[uses({})]\n    fn main() -> () {{\n",
        uses.join(", ")
    );
    src.push_str(&main_head);
    let mut expected = String::new();
    let mut n = 0;
    // Calls that fault in the interpreter (backlog 1B-54): `main` lines for
    // `h`, `f::<B>`, `f::<N<B>>` and `f::<M>` (the kind), one form each.
    let mut faults: Vec<(usize, (usize, String))> = Vec::new();
    for (fi, f) in funcs.iter().enumerate() {
        let k = &f.name[1..];
        for (x, y) in f.inputs(&mut rng) {
            let (lx, ly) = (lit(x), lit(y));
            let Ok(res) = f.run(x, y) else {
                let form = faults.len() % 4;
                let call = match form {
                    0 => format!("        print_i64(m::h{k}({lx}, {ly}));\n"),
                    1 => format!(
                        "        let a: m::B = m::B {{ v: {lx} }};\n        let b: m::B = m::B {{ v: {ly} }};\n        print_i64(m::f{k}::<m::B>(a, b));\n"
                    ),
                    2 => format!(
                        "        let c: m::N<m::B> = m::N {{ inner: m::B {{ v: {lx} }} }};\n        let d: m::N<m::B> = m::N {{ inner: m::B {{ v: {ly} }} }};\n        print_i64(m::f{k}::<m::N<m::B>>(c, d));\n"
                    ),
                    _ => format!(
                        "        print_i64(m::f{k}::<m::M>(m::M::mk({lx}), m::M::mk({ly})));\n"
                    ),
                };
                faults.push((fi, (form, call)));
                continue;
            };
            let _ = writeln!(src, "        print_i64(m::h{k}({lx}, {ly}));");
            let _ = writeln!(
                src,
                "        let a{n}: m::B = m::B {{ v: {lx} }};\n        let b{n}: m::B = m::B {{ v: {ly} }};\n        print_i64(m::f{k}::<m::B>(a{n}, b{n}));"
            );
            let _ = writeln!(
                src,
                "        let c{n}: m::N<m::B> = m::N {{ inner: m::B {{ v: {lx} }} }};\n        let d{n}: m::N<m::B> = m::N {{ inner: m::B {{ v: {ly} }} }};\n        print_i64(m::f{k}::<m::N<m::B>>(c{n}, d{n}));"
            );
            let _ = writeln!(
                src,
                "        print_i64(m::f{k}::<m::M>(m::M::mk({lx}), m::M::mk({ly})));"
            );
            n += 1;
            for _ in 0..4 {
                let _ = writeln!(expected, "{res}");
            }
        }
    }
    let samples: Vec<i64> = (0..6).map(|_| value(&mut rng)).collect();
    for (j, s) in samples.iter().enumerate() {
        if let Ok(g) = r.get(*s) {
            let _ = writeln!(
                src,
                "        let r{j}: m::R = m::R {{ v: {} }};\n        print_i64(m::Val::get(&r{j}));",
                lit(*s)
            );
            let _ = writeln!(expected, "{g}");
            // The inherent `R::get` comes before the trait's (Rust's rule).
            if methods {
                let _ = writeln!(src, "        print_i64(r{j}.get());");
                let _ = writeln!(expected, "{}", s.wrapping_add(INHERENT_SHIFT));
            }
        }
    }
    src.push_str("    }\n}\n");
    let file = dir.join(format!("t{seed}.bmb"));
    fs::write(&file, &src).unwrap();
    let cache = dir.join(format!("cache{seed}"));
    let emit = |all_checks: bool| -> (String, Option<String>) {
        let ll = dir.join(format!("t{seed}-{all_checks}.ll"));
        let _ = fs::remove_file(&ll);
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("emit-llvm");
        if all_checks {
            cmd.arg("--all-checks");
        } else {
            cmd.arg("--cache").arg(&cache);
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
    let status = |def: &str| {
        output
            .lines()
            .find(|l| field(l, "def") == Some(def))
            .and_then(|l| field(l, "status"))
    };
    for fixed in [
        "<m::B as m::Val>::get",
        "<m::N<T> as m::Val>::get",
        "<m::M as m::Val>::get",
    ] {
        assert_eq!(status(fixed), Some("verified"), "{}", ctx(fixed));
    }

    // A verified `R::get` meets the trait's contract: it returns `v`.
    if status("<m::R as m::Val>::get") == Some("verified") {
        stats.r_verified += 1;
        for s in samples
            .iter()
            .copied()
            .chain([0, 1, -1, i64::MIN, i64::MAX])
        {
            match r.get(s) {
                Ok(g) => assert_eq!(
                    g,
                    s,
                    "{}",
                    ctx(&format!("verified R::get returns {g} for v = {s}"))
                ),
                Err(Fault) => panic!("{}", ctx(&format!("verified R::get faults at v = {s}"))),
            }
        }
    }

    for f in &funcs {
        let k = &f.name[1..];
        for def in [format!("m::h{k}"), format!("m::f{k}")] {
            stats.funcs += 1;
            if status(&def) != Some("verified") {
                continue;
            }
            stats.verified += 1;
            for (x, y) in f.inputs(&mut rng) {
                stats.verified_runs += 1;
                match f.run(x, y) {
                    Ok(res) => assert!(
                        f.post(x, y, res),
                        "{}",
                        ctx(&format!(
                            "verified {def} breaks its postcondition at x = {x}, y = {y}, r = {res}"
                        ))
                    ),
                    Err(Fault) => {
                        panic!(
                            "{}",
                            ctx(&format!("verified {def} faults at x = {x}, y = {y}"))
                        )
                    }
                }
            }
        }
    }

    for line in output.lines() {
        let Some(msg) = field(line, "message") else {
            continue;
        };
        if !msg.contains("; counterexample: ") || msg.contains("unconfirmed") {
            continue;
        }
        // `R::get`'s body against its own postcondition (a refinement
        // failure, E0817, is about contracts and is not replayed).
        if msg.contains("in `<m::R as m::Val>::get`") && field(line, "code") != Some("E0817") {
            let v: i64 = msg
                .split("R { v: ")
                .nth(1)
                .and_then(|s| s.split(' ').next())
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| panic!("{}", ctx(&format!("counterexample of R: {msg}"))));
            stats.cexes += 1;
            let broken = match r.get(v) {
                Err(Fault) => true,
                Ok(g) => r.post.as_ref().map_or(g != v, |p| {
                    !p_eval(p, &[("v", v as i128), ("r", g as i128)])
                }),
            };
            assert!(
                broken,
                "{}",
                ctx(&format!("counterexample does not break R::get: {msg}"))
            );
            continue;
        }
        let Some(f) = funcs
            .iter()
            .find(|f| msg.contains(&format!("in `m::h{}`", &f.name[1..])))
        else {
            if funcs
                .iter()
                .any(|f| msg.contains(&format!("in `m::f{}`", &f.name[1..])))
            {
                // Over the sort `T`: nothing to replay.
                stats.generic_cexes += 1;
            }
            continue;
        };
        let cex = parse_cex(msg).unwrap_or_else(|| panic!("{}", ctx(msg)));
        let get = |v: &str| cex.iter().find(|(k, _)| k == v).map(|(_, n)| *n);
        let (Some(x), Some(y)) = (get("x0"), get("y0")) else {
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
            Ok(res) => !f.post(x, y, res),
        };
        assert!(
            broken,
            "{}",
            ctx(&format!(
                "counterexample does not break h{}: {msg}",
                &f.name[1..]
            ))
        );
    }

    let lines = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            emit(true).1.unwrap()
        } else {
            ir.clone().unwrap()
        };
        let exe = dir.join(format!("t{seed}-{all_checks}"));
        let ll = dir.join(format!("t{seed}-{all_checks}.build.ll"));
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
        stats.compiled_runs += lines;
    }

    // Calls that fault in the interpreter trap in both builds.
    let chosen = trap::choose(&faults, |(form, _)| *form);
    for (j, (fi, (_, call))) in chosen.into_iter().enumerate() {
        let src = format!("{module}{main_head}{}    }}\n}}\n", trap::body(&call));
        let files = vec![(format!("t{seed}.bmb"), src.clone())];
        let exe = dir.join(format!("t{seed}-trap{j}"));
        if let Some(e) = trap::check(&files, &cache, &exe) {
            panic!("seed {seed}: {}: {e}\n{src}", funcs[fi].name);
        }
        stats.traps += 1;
    }
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn trait_proofs_counterexamples_and_builds_agree_with_the_interpreter() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-traits-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "soundness (traits): {} functions, {} verified ({} runs), R::get verified in {} batches, {} counterexamples replayed, {} over a sort, {} compiled runs, {} traps confirmed, {} batches with method calls",
        stats.funcs,
        stats.verified,
        stats.verified_runs,
        stats.r_verified,
        stats.cexes,
        stats.generic_cexes,
        stats.compiled_runs,
        stats.traps,
        stats.method_batches
    );
    assert!(stats.verified > 0 && stats.cexes > 0 && stats.traps > 0);
    assert!(iters < 2 || stats.method_batches > 0);
}
