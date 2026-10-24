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

mod common;

use common::scalar::*;

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
