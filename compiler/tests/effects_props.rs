//! Effect checking against a reference on random call graphs (ADR-0030 §8,
//! backlog 2F-05).
//!
//! Each seed builds a module of functions `f0..fN` over `n: u64`, each with
//! random own effects (`alloc`: `Vec::new()`, `io`: `print_i64`, `panic`: an
//! `assume`, ADR-0029) and random calls, recursion and mutual recursion
//! included (`if n > 0 { f_j(n - 1); }`). Two programs:
//!
//! 1. random declarations: `bmb check` reports E0606 on a function exactly
//!    when its declaration misses part of `req(f)` = own effects ∪ the
//!    callees' declared effects (§3), except a `pure` function whose only gap
//!    is `io`, and E0603 once per `io` call (built-in or callee) of a `pure`
//!    function (ADR-0030 implementation notes);
//! 2. declarations closed over the calls plus random extra effects (the
//!    program checks): `status`'s `effects_required` is the fixpoint
//!    `eff*(f)` = own ∪ `eff*` of the callees, not the wider declarations,
//!    and `query effect E` lists each function that declares or requires
//!    `E`, with the source (`own` or `callee`).
//!
//! It drives the `bmb` command line (`common::cli`). `BMB_EFFECT_SEEDS`
//! raises the number of seeds.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

mod common;

use common::cli;
use common::json::{self, Value};
use common::scalar::Rng;

const ATOMS: [&str; 3] = ["alloc", "io", "panic"];

type Set = BTreeSet<&'static str>;

struct Def {
    own: Set,
    /// Callees by index, in call order (a callee may repeat).
    calls: Vec<usize>,
}

fn gen_defs(rng: &mut Rng) -> Vec<Def> {
    let n = 4 + rng.below(6);
    (0..n)
        .map(|_| {
            let own = ATOMS.iter().copied().filter(|_| rng.chance(30)).collect();
            let calls = (0..rng.below(4)).map(|_| rng.below(n)).collect();
            Def { own, calls }
        })
        .collect()
}

fn text(defs: &[Def], decl: &[Set]) -> String {
    let mut t = String::from(
        "#[summary(\"효과 성질\")]\nmod m {\n    spec fn t(n: u64) -> bool { n < 100 }\n\n",
    );
    for (i, (d, e)) in defs.iter().zip(decl).enumerate() {
        let effects = if e.is_empty() {
            "pure".to_string()
        } else {
            e.iter().copied().collect::<Vec<_>>().join(", ")
        };
        let _ = writeln!(t, "    #[effects({effects})]");
        if d.own.contains("panic") {
            t.push_str("    #[uses(t)]\n");
        }
        let _ = writeln!(t, "    fn f{i}(n: u64) -> ()\n        decreases n,\n    {{");
        if d.own.contains("alloc") {
            t.push_str("        let v: Vec<i64> = Vec::new();\n");
        }
        if d.own.contains("io") {
            t.push_str("        print_i64(1);\n");
        }
        if d.own.contains("panic") {
            t.push_str("        #[rationale(kind = solver_limit, hash = \"0000000000000000\", \"effects\")]\n        assume(t(n));\n");
        }
        if !d.calls.is_empty() {
            t.push_str("        if n > 0 {\n");
            for j in &d.calls {
                let _ = writeln!(t, "            f{j}(n - 1);");
            }
            t.push_str("        }\n");
        }
        t.push_str("    }\n\n");
    }
    t.push_str("}\n");
    t
}

/// `req(f)`: own effects and the callees' declared effects (§3).
fn req(defs: &[Def], decl: &[Set], i: usize) -> Set {
    let mut r = defs[i].own.clone();
    for &j in &defs[i].calls {
        r.extend(decl[j].iter().copied());
    }
    r
}

/// The least fixpoint of `step` from `start` (for `eff*` and closed
/// declarations).
fn fixpoint(defs: &[Def], start: Vec<Set>, step: impl Fn(&[Set], usize) -> Set) -> Vec<Set> {
    let mut cur = start;
    loop {
        let next: Vec<Set> = (0..defs.len()).map(|i| step(&cur, i)).collect();
        if next == cur {
            return cur;
        }
        cur = next;
    }
}

fn rows(out: &str) -> Vec<Value> {
    out.lines().filter_map(|l| json::parse(l).ok()).collect()
}

fn s<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(Value::as_str)
}

fn check_diagnostics(seed: u64, defs: &[Def], decl: &[Set]) {
    let src = text(defs, decl);
    let (_, out) = cli::run(&["check"], &[("e.bmb".to_string(), src.clone())]);
    let mut e0606: BTreeSet<usize> = BTreeSet::new();
    let mut e0603: BTreeMap<usize, usize> = BTreeMap::new();
    for v in rows(&out) {
        let msg = s(&v, "message").unwrap_or("");
        match s(&v, "code") {
            Some("E0606") => {
                let name = msg.strip_prefix("`f").and_then(|r| r.split('`').next());
                let i: usize = name.and_then(|n| n.parse().ok()).unwrap_or_else(|| {
                    panic!("seed {seed}: E0606 without a function: {msg}\n{src}")
                });
                assert!(e0606.insert(i), "seed {seed}: two E0606 on f{i}\n{out}");
            }
            Some("E0603") => {
                let i: usize = msg
                    .rsplit("`m::f")
                    .next()
                    .and_then(|r| r.split('`').next())
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| panic!("seed {seed}: E0603 without a function: {msg}"));
                *e0603.entry(i).or_default() += 1;
            }
            Some(code) => panic!("seed {seed}: unexpected {code}: {msg}\n{src}"),
            None => {}
        }
    }
    for i in 0..defs.len() {
        let r = req(defs, decl, i);
        let missing: Set = r.difference(&decl[i]).copied().collect();
        let pure = decl[i].is_empty();
        let want_0606 = !missing.is_empty() && !(pure && missing == ["io"].into());
        assert_eq!(
            e0606.contains(&i),
            want_0606,
            "seed {seed}: f{i} declares {:?}, needs {r:?}\n{src}\n{out}",
            decl[i]
        );
        let io_calls = if pure {
            usize::from(defs[i].own.contains("io"))
                + defs[i]
                    .calls
                    .iter()
                    .filter(|&&j| decl[j].contains("io"))
                    .count()
        } else {
            0
        };
        assert_eq!(
            e0603.get(&i).copied().unwrap_or(0),
            io_calls,
            "seed {seed}: E0603 on f{i}\n{src}\n{out}"
        );
    }
}

fn check_map(seed: u64, rng: &mut Rng, defs: &[Def]) {
    // Declarations closed over the calls, some wider than needed.
    let start: Vec<Set> = defs
        .iter()
        .map(|d| {
            let mut e = d.own.clone();
            e.extend(ATOMS.iter().copied().filter(|_| rng.chance(15)));
            e
        })
        .collect();
    let decl = fixpoint(defs, start.clone(), |cur, i| {
        let mut e = start[i].clone();
        for &j in &defs[i].calls {
            e.extend(cur[j].iter().copied());
        }
        e
    });
    let eff = fixpoint(
        defs,
        defs.iter().map(|d| d.own.clone()).collect(),
        |cur, i| {
            let mut e = defs[i].own.clone();
            for &j in &defs[i].calls {
                e.extend(cur[j].iter().copied());
            }
            e
        },
    );
    let src = text(defs, &decl);
    let files = [("e.bmb".to_string(), src.clone())];
    let (ok, out) = cli::run(&["check"], &files);
    assert!(
        ok,
        "seed {seed}: closed declarations must check\n{src}\n{out}"
    );

    let (_, status) = cli::run(&["query", "status", "--"], &files);
    let mut seen = 0;
    for v in rows(&status) {
        let Some(i) = s(&v, "path")
            .and_then(|p| p.strip_prefix("m::f"))
            .and_then(|n| n.parse::<usize>().ok())
        else {
            continue;
        };
        let got: BTreeSet<String> = match v.get("effects_required") {
            Some(Value::Array(xs)) => xs.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
            _ => panic!("seed {seed}: no effects_required for f{i}\n{status}"),
        };
        let want: BTreeSet<String> = eff[i].iter().map(|e| e.to_string()).collect();
        assert_eq!(
            got, want,
            "seed {seed}: eff*(f{i}), declared {:?}\n{src}",
            decl[i]
        );
        seen += 1;
    }
    assert_eq!(seen, defs.len(), "seed {seed}\n{status}");

    for atom in ATOMS {
        let (_, q) = cli::run(&["query", "effect", atom, "--"], &files);
        let mut listed = BTreeSet::new();
        for v in rows(&q) {
            let Some(i) = s(&v, "def")
                .and_then(|p| p.strip_prefix("m::f"))
                .and_then(|n| n.parse::<usize>().ok())
            else {
                continue;
            };
            listed.insert(i);
            let flag = |k: &str| matches!(v.get(k), Some(Value::Bool(true)));
            assert_eq!(
                flag("declares"),
                decl[i].contains(atom),
                "seed {seed}: {atom} f{i}\n{q}"
            );
            assert_eq!(
                flag("requires"),
                eff[i].contains(atom),
                "seed {seed}: {atom} f{i}\n{q}"
            );
            if eff[i].contains(atom) {
                let source = if defs[i].own.contains(atom) {
                    "own"
                } else {
                    "callee"
                };
                assert_eq!(
                    s(&v, "source"),
                    Some(source),
                    "seed {seed}: {atom} f{i}\n{q}"
                );
            }
        }
        let want: BTreeSet<usize> = (0..defs.len())
            .filter(|&i| decl[i].contains(atom) || eff[i].contains(atom))
            .collect();
        assert_eq!(listed, want, "seed {seed}: query effect {atom}\n{src}\n{q}");
    }
}

#[test]
fn effects_agree_with_the_reference_on_random_call_graphs() {
    let seeds: u64 = std::env::var("BMB_EFFECT_SEEDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);
    for seed in 1..=seeds {
        let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
        let defs = gen_defs(&mut rng);
        let decl: Vec<Set> = defs
            .iter()
            .map(|_| ATOMS.iter().copied().filter(|_| rng.chance(40)).collect())
            .collect();
        check_diagnostics(seed, &defs, &decl);
        check_map(seed, &mut rng, &defs);
    }
}
