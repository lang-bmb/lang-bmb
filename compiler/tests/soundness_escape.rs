//! Soundness fuzzing of escapes (ADR-0029 §8, backlog 2E-05).
//!
//! Random integer functions (`common::scalar`) are mixed with escapes whose
//! trusted facts may be false:
//!
//! - `a*`: `assume(q(x, y))` at the top of the body, `q` a random predicate
//!   (often a box the inputs leave);
//! - `e*`: `#[external_body]`, whose random body may break its `ensures`;
//! - `c*`: callers of a plain `f*` or an external `e*` on their own inputs,
//!   with the callee's precondition assumed, so the call's obligation and the
//!   callee's checks rest on a fact that may be false (§4, downward
//!   propagation).
//!
//! Properties, against an interpreter that ignores `assume` and runs every
//! body as written:
//!
//! 1. a `proved` definition (verified, no `trust` key) never faults and meets
//!    its postcondition on any input its precondition admits, whatever the
//!    trusted facts are;
//! 2. deleting every `assume` leaves the set of `proved` definitions as it
//!    is (the first pass does not see trusted facts, §2);
//! 3. both builds print what the interpreter computes, and a call the
//!    interpreter says faults traps in both (`common::trap`): a false fact
//!    ends in a trap, never in a wrong value;
//! 4. in the IR of the definitions alone (no `main` joining them into one
//!    call component), an `assumed` or `external` definition and everything
//!    a checked definition calls carry no `; proved` (no check was removed).
//!
//! It drives the `bmb` command line (`common::cli`, HEALTH.md S5).
//! `BMB_SOUND_ITERS` raises the number of batches.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::process::Command;

mod common;

use common::cli;
use common::scalar::*;
use common::trap;

#[derive(Default)]
struct Stats {
    defs: usize,
    proved: usize,
    assumed: usize,
    external: usize,
    proved_runs: usize,
    compiled_runs: usize,
    traps: usize,
    checked_bodies: usize,
}

const RATIONALE: &str =
    "        #[rationale(kind = solver_limit, hash = \"0000000000000000\", \"fuzz\")]\n";

/// What a definition is, for rendering and for the interpreter.
enum Kind {
    /// A plain function.
    Plain,
    /// `assume(q(x, y))` first.
    Assume(P),
    /// `#[external_body]`.
    External,
    /// `assume(callee_pre(x, y)); let v: i64 = callee(x, y);` then `body`
    /// over `x`, `y`, `v` (`Func::body`); the index is into the batch.
    Caller(usize),
}

struct Def {
    f: Func,
    kind: Kind,
}

impl Def {
    fn has_escape(&self) -> bool {
        !matches!(self.kind, Kind::Plain)
    }

    /// The source. `assumes` false leaves every `assume` out (property 2).
    fn text(&self, defs: &[Def], assumes: bool) -> String {
        let n = &self.f.name;
        let plain = self.f.text();
        let effects = if self.has_escape() { "panic" } else { "pure" };
        let mut t = plain.replace("#[effects(pure)]", &format!("#[effects({effects})]"));
        let head = "\n    {\n";
        let at = t.find(head).expect("a body") + head.len();
        match &self.kind {
            Kind::Plain => {}
            Kind::Assume(q) => {
                let spec = format!(
                    "    spec fn {n}_q(x: i64, y: i64) -> bool {{ {} }}\n\n",
                    p_text(q)
                );
                let body = if assumes {
                    format!("{RATIONALE}        assume({n}_q(x, y));\n")
                } else {
                    String::new()
                };
                t.insert_str(at, &body);
                t = t.replace(
                    &format!("#[uses({n}_pre, {n}_post)]"),
                    &format!("#[uses({n}_pre, {n}_post, {n}_q)]"),
                );
                t.insert_str(0, &spec);
            }
            Kind::External => {
                t = t.replace(
                    "    #[effects(panic)]",
                    "    #[rationale(kind = boundary, hash = \"0000000000000000\", \"fuzz\")]\n    #[external_body]\n    #[effects(panic)]",
                );
            }
            Kind::Caller(j) => {
                let g = &defs[*j].f.name;
                let mut body = String::new();
                if assumes {
                    let _ = writeln!(body, "{RATIONALE}        assume({g}_pre(x, y));");
                }
                let _ = writeln!(body, "        let v: i64 = {g}(x, y);");
                t.insert_str(at, &body);
                t = t.replace(
                    &format!("#[uses({n}_pre, {n}_post)]"),
                    &format!("#[uses({n}_pre, {n}_post, {g}_pre, {g}_post)]"),
                );
            }
        }
        t
    }

    /// The interpreter: `assume` is not run, a callee runs on any input.
    fn run(&self, defs: &[Def], x: i64, y: i64) -> Result<i64, Fault> {
        match self.kind {
            Kind::Caller(j) => {
                let v = defs[j].run(defs, x, y)?;
                let env: Env = vec![("x".into(), x), ("y".into(), y), ("v".into(), v)];
                e_eval(&self.f.body, &env)
            }
            _ => self.f.run(x, y),
        }
    }
}

fn gen_def(rng: &mut Rng, k: usize, defs: &[Def]) -> Def {
    let mut f = gen_func(rng, k);
    let kind = match k % 4 {
        0 => Kind::Plain,
        1 => Kind::Assume(if rng.chance(60) {
            let (lx, hx) = bounds(rng);
            let (ly, hy) = bounds(rng);
            let s = |v: i64| S::Lit(v);
            let and = |a: P, b: P| P::And(Box::new(a), Box::new(b));
            and(
                and(
                    P::Cmp(Cmp::Le, s(lx), S::Var("x")),
                    P::Cmp(Cmp::Le, S::Var("x"), s(hx)),
                ),
                and(
                    P::Cmp(Cmp::Le, s(ly), S::Var("y")),
                    P::Cmp(Cmp::Le, S::Var("y"), s(hy)),
                ),
            )
        } else {
            gen_p(rng, &["x", "y"], 1)
        }),
        2 => Kind::External,
        _ => {
            // A caller of an earlier plain or external definition.
            let targets: Vec<usize> = (0..defs.len())
                .filter(|&i| matches!(defs[i].kind, Kind::Plain | Kind::External))
                .collect();
            let j = targets[rng.below(targets.len())];
            // Its own body reads the callee's result; no locals or arrays.
            f.lets.clear();
            f.arr.clear();
            let vars = ["x", "y", "v"].map(String::from);
            // Mostly a safe body and a true postcondition, so the caller
            // verifies (`assumed`) and its callee's checks rest on the fact
            // alone: a failed caller already keeps its component's checks.
            f.body = if rng.chance(60) {
                E::If(
                    Box::new(C::Cmp(gen_cmp(rng), E::Var("v".into()), E::Lit(value(rng)))),
                    Box::new(E::Var("x".into())),
                    Box::new(E::Lit(0)),
                )
            } else {
                gen_e(rng, &vars, 2)
            };
            if rng.chance(60) {
                f.post = P::Cmp(Cmp::Le, S::Var("r"), S::Var("r"));
            }
            Kind::Caller(j)
        }
    };
    Def { f, kind }
}

/// The `proved` set and each definition's trust, from emit-llvm's rows.
fn trusts(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter_map(|l| {
            let name = field(l, "def")?.strip_prefix("m::")?.to_string();
            let trust = match (field(l, "status"), field(l, "trust")) {
                (Some("verified"), None) => "proved",
                (_, Some(t)) => t,
                (Some(s), None) => s,
                _ => return None,
            };
            Some((name, trust.to_string()))
        })
        .collect()
}

fn ir_body<'a>(ir: &'a str, name: &str) -> &'a str {
    let head = format!("@\"m::{name}\"(");
    let start = ir
        .lines()
        .position(|l| l.starts_with("define") && l.contains(&head))
        .unwrap_or_else(|| panic!("no IR for m::{name}"));
    let from: usize = ir.lines().take(start).map(|l| l.len() + 1).sum();
    let rest = &ir[from..];
    &rest[..rest.find("\n}\n").map_or(rest.len(), |e| e + 3)]
}

fn batch(seed: u64, stats: &mut Stats, dir: &std::path::Path) {
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut defs: Vec<Def> = Vec::new();
    for k in 0..12 {
        let d = gen_def(&mut rng, k, &defs);
        defs.push(d);
    }
    let module = |assumes: bool| {
        let mut m = String::from("#[summary(\"탈출구 건전성 퍼징\")]\nmod m {\n");
        for d in &defs {
            m.push_str(&d.text(&defs, assumes));
        }
        m.push_str("}\n\n");
        m
    };
    let main_head =
        "#[summary(\"실행\")]\nmod app {\n    #[effects(io, panic)]\n    fn main() -> () {\n";
    let mut src = format!("{}{main_head}", module(true));
    let mut expected = String::new();
    let mut faults: Vec<(usize, (i64, i64))> = Vec::new();
    for (i, d) in defs.iter().enumerate() {
        for (x, y) in d.f.inputs(&mut rng) {
            let Ok(r) = d.run(&defs, x, y) else {
                faults.push((i, (x, y)));
                continue;
            };
            let _ = writeln!(
                src,
                "        print_i64(m::{}({}, {}));",
                d.f.name,
                lit(x),
                lit(y)
            );
            let _ = writeln!(expected, "{r}");
        }
    }
    src.push_str("    }\n}\n");
    let files = vec![("s.bmb".to_string(), src.clone())];
    let cache = dir.join(format!("cache{seed}"));
    let out = cli::emit_llvm(&files, &dir.join(format!("src{seed}")), Some(&cache), false);
    let ctx = |what: &str| format!("seed {seed}: {what}\n{src}\n{}", out.output);
    let ir = out.ir.clone().unwrap_or_else(|| panic!("{}", ctx("no IR")));
    let trust = trusts(&out.output);
    let of = |name: &str| {
        trust
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t.as_str())
            .unwrap_or_else(|| panic!("{}", ctx(&format!("no row for m::{name}"))))
    };

    // 1. A proved definition is safe and meets its postcondition.
    for d in &defs {
        stats.defs += 1;
        match of(&d.f.name) {
            "assumed" => stats.assumed += 1,
            "external" => stats.external += 1,
            "proved" => {
                stats.proved += 1;
                for (x, y) in d.f.inputs(&mut rng) {
                    stats.proved_runs += 1;
                    match d.run(&defs, x, y) {
                        Ok(r) => assert!(
                            d.f.post(x, y, r),
                            "{}",
                            ctx(&format!(
                                "proved {} breaks its postcondition at x = {x}, y = {y}, r = {r}",
                                d.f.name
                            ))
                        ),
                        Err(Fault) => panic!(
                            "{}",
                            ctx(&format!("proved {} faults at x = {x}, y = {y}", d.f.name))
                        ),
                    }
                }
            }
            _ => {}
        }
    }

    // 2. Without its `assume`s the program proves the same definitions.
    let proved = |rows: &[(String, String)]| -> BTreeSet<String> {
        rows.iter()
            .filter(|(_, t)| t == "proved")
            .map(|(n, _)| n.clone())
            .collect()
    };
    let bare = vec![(
        "s.bmb".to_string(),
        format!("{}{main_head}    }}\n}}\n", module(false)),
    )];
    let without = cli::emit_llvm(&bare, &dir.join(format!("bare{seed}")), None, false);
    assert_eq!(
        proved(&trust),
        proved(&trusts(&without.output)),
        "{}",
        ctx(&format!(
            "the proved set changes without `assume`\n{}",
            without.output
        ))
    );

    // 4. Checked definitions keep every check: trusted ones and their callees.
    // Without `main`, which calls every definition: one call-connected
    // component with a failed definition keeps all its checks (backlog
    // 1B-31), so in the batch program that would hide a removed check.
    let lib = vec![("s.bmb".to_string(), module(true))];
    let lib_out = cli::emit_llvm(&lib, &dir.join(format!("lib{seed}")), Some(&cache), false);
    let lib_ir = lib_out
        .ir
        .unwrap_or_else(|| panic!("{}", ctx("no library IR")));
    let mut checked: BTreeSet<usize> = (0..defs.len())
        .filter(|&i| of(&defs[i].f.name) != "proved")
        .collect();
    for i in checked.clone() {
        if let Kind::Caller(j) = defs[i].kind {
            checked.insert(j);
        }
    }
    for &i in &checked {
        let body = ir_body(&lib_ir, &defs[i].f.name);
        assert!(
            !body.contains("; proved"),
            "{}",
            ctx(&format!(
                "checked m::{} has a removed check:\n{body}",
                defs[i].f.name
            ))
        );
        stats.checked_bodies += 1;
    }

    // 3. Both builds print what the interpreter computes.
    let n = expected.lines().count();
    for all_checks in [false, true] {
        let ir = if all_checks {
            cli::emit_llvm(&files, &dir.join(format!("src{seed}")), None, true)
                .ir
                .unwrap()
        } else {
            ir.clone()
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

    // ... and a call that faults traps, escapes first (a false fact).
    faults.sort_by_key(|(i, _)| !defs[*i].has_escape());
    for (j, (i, (x, y))) in trap::choose(&faults, |_| ()).into_iter().enumerate() {
        let d = &defs[i];
        let call = format!(
            "        print_i64(m::{}({}, {}));\n",
            d.f.name,
            lit(x),
            lit(y)
        );
        let src = format!(
            "{}{main_head}{}    }}\n}}\n",
            module(true),
            trap::body(&call)
        );
        let files = vec![("s.bmb".to_string(), src.clone())];
        let exe = dir.join(format!("s{seed}-trap{j}"));
        if let Some(e) = trap::check(&files, &cache, &exe) {
            panic!("seed {seed}: m::{}({x}, {y}): {e}\n{src}", d.f.name);
        }
        stats.traps += 1;
    }
    let _ = fs::remove_dir_all(&cache);
}

#[test]
fn false_trusted_facts_end_in_a_trap_never_a_wrong_value() {
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
    let dir = std::env::temp_dir().join(format!("bmb-sound-escape-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut stats = Stats::default();
    for seed in start..start + iters {
        let t = std::time::Instant::now();
        batch(seed, &mut stats, &dir);
        eprintln!("seed {seed}: {:?}", t.elapsed());
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!(
        "escape soundness: {} definitions, {} proved ({} runs), {} assumed, {} external, {} checked bodies, {} compiled runs, {} traps confirmed",
        stats.defs,
        stats.proved,
        stats.proved_runs,
        stats.assumed,
        stats.external,
        stats.checked_bodies,
        stats.compiled_runs,
        stats.traps
    );
    // The generator must reach every outcome.
    assert!(stats.proved > 0 && stats.assumed > 0 && stats.external > 0);
    assert!(stats.traps > 0 && stats.checked_bodies > 0);
}
