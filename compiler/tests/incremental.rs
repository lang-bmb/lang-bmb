//! Contract-firewall incremental build (backlog 2D-10): with the cache,
//! outputs are byte-identical to an uncached run, cold or warm; an unchanged
//! definition is neither encoded nor lowered again; a body change touches only
//! that definition; a contract change re-encodes exactly the `impact` set;
//! damaged cache lines and another compiler identity cause rebuilds.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use common::cli;
use common::json::{self, Value};

/// What the incremental cache did in one run (`--stats`; paths sorted).
#[derive(Default, Debug)]
struct Stats {
    encoded: Vec<String>,
    reused: Vec<String>,
    lowered: Vec<String>,
    ir_reused: Vec<String>,
    audit_mismatches: Vec<String>,
    damaged: usize,
    runtime_checks: u32,
}

fn read_stats(path: &Path) -> Stats {
    let text = fs::read_to_string(path).unwrap();
    let v = json::parse(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    let list = |k: &str| match v.get(k) {
        Some(Value::Array(xs)) => xs.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
        _ => panic!("{k}: {text}"),
    };
    let num = |k: &str| v.get(k).and_then(Value::as_i64).unwrap_or(0);
    Stats {
        encoded: list("encoded"),
        reused: list("reused"),
        lowered: list("lowered"),
        ir_reused: list("ir_reused"),
        audit_mismatches: list("audit_mismatches"),
        damaged: usize::try_from(num("damaged")).unwrap(),
        runtime_checks: u32::try_from(num("runtime_checks")).unwrap(),
    }
}

/// `bmb ARGS NAMES..` in the new directory `src` holding `f`: its success
/// and stdout.
fn bmb_in(src: &Path, args: &[&str], f: &[(String, String)]) -> (bool, String) {
    for (name, text) in f {
        fs::write(src.join(name), text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(src)
        .args(args)
        .args(f.iter().map(|(n, _)| n))
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

/// `bmb emit-llvm` of `f` through the caches in `cache` (and `--audit`):
/// its rows, its IR and the stats.
fn emit_cached(
    f: &[(String, String)],
    cache: &Path,
    audit: bool,
) -> (String, Option<String>, Stats) {
    let src = cli::fresh_dir();
    let (ll, st) = (src.join("out.ll"), src.join("stats.json"));
    let mut args = vec!["emit-llvm", "--cache", cache.to_str().unwrap()];
    args.extend(["--stats", st.to_str().unwrap(), "-o", ll.to_str().unwrap()]);
    if audit {
        args.push("--audit");
    }
    let (_, output) = bmb_in(&src, &args, f);
    let ir = fs::read_to_string(&ll).ok();
    let stats = read_stats(&st);
    let _ = fs::remove_dir_all(&src);
    (output, ir, stats)
}

/// `bmb verify` of `f` through the caches in `cache`.
struct Verified {
    ok: bool,
    output: String,
    incr: Stats,
}

fn verify_cached(f: &[(String, String)], cache: &Path) -> Verified {
    let src = cli::fresh_dir();
    let st = src.join("stats.json");
    let (ok, output) = bmb_in(
        &src,
        &[
            "verify",
            "--cache",
            cache.to_str().unwrap(),
            "--stats",
            st.to_str().unwrap(),
        ],
        f,
    );
    let incr = read_stats(&st);
    let _ = fs::remove_dir_all(&src);
    Verified { ok, output, incr }
}

/// The script of each definition (`bmb verify --emit-smt`), by path.
fn scripts(f: &[(String, String)]) -> BTreeMap<String, String> {
    let src = cli::fresh_dir();
    bmb_in(&src, &["verify", "--emit-smt", "smt"], f);
    let mut out = BTreeMap::new();
    for e in fs::read_dir(src.join("smt")).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_str().unwrap().to_string();
        if let Some(def) = name.strip_suffix(".smt2") {
            out.insert(def.replace('.', "::"), fs::read_to_string(&p).unwrap());
        }
    }
    let _ = fs::remove_dir_all(&src);
    out
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("bmb-incr-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

fn files(text: &str) -> Vec<(String, String)> {
    vec![("g.bmb".to_string(), text.to_string())]
}

/// A generated program: per unit two spec fns used by every function, one
/// spare predicate, and five functions (straight line, a call, a loop, a
/// recursion, and a call into the previous unit).
fn generate(units: usize) -> String {
    let mut s = String::new();
    for k in 0..units {
        let (prev_uses, prev_call, combine) = if k == 0 {
            (String::new(), "0".to_string(), "a")
        } else {
            (
                format!(", m{}::small, m{}::pos", k - 1, k - 1),
                format!("m{}::f1(x)", k - 1),
                "if a > b { a } else { b }",
            )
        };
        s.push_str(&format!(
            r#"
#[summary("generated unit {k}")]
mod m{k} {{
    spec fn small(x: i64) -> bool {{ x >= 0 && x < 1000 }}

    spec fn pos(r: i64) -> bool {{ r >= 0 && r < 2000 }}

    spec fn pos2(r: i64) -> bool {{ r >= 0 && r < 2000 }}

    #[effects(pure)]
    #[uses(small, pos, pos2)]
    fn f0(x: i64) -> (r: i64)
        requires small(x),
        ensures pos(r),
    {{
        x + {k}
    }}

    #[effects(pure)]
    #[uses(small, pos, pos2)]
    fn f1(x: i64) -> (r: i64)
        requires small(x),
        ensures pos(r),
    {{
        let y: i64 = f0(x);
        if y > 5 {{ y - 5 }} else {{ y }}
    }}

    #[effects(pure)]
    #[uses(small)]
    fn f2(n: i64) -> i64
        requires small(n),
    {{
        let mut c: i64 = 0;
        let mut i: i64 = 0;
        while i < n
            invariant i <= n, c == i,
            decreases n - i,
        {{
            c = c + 1;
            i = i + 1;
        }}
        c
    }}

    #[effects(pure)]
    #[uses(small, pos)]
    fn rec(n: i64) -> (r: i64)
        requires small(n),
        ensures pos(r),
        decreases n,
    {{
        if n <= 0 {{ 0 }} else {{ rec(n - 1) }}
    }}

    #[effects(pure)]
    #[uses(small, pos{prev_uses})]
    fn g(x: i64) -> (r: i64)
        requires small(x),
        ensures pos(r),
    {{
        let a: i64 = f1(x);
        let b: i64 = {prev_call};
        {combine}
    }}
}}
"#
        ));
    }
    s
}

/// Everything a run outputs, uncached: the rows, the IR, its runtime
/// checks and the maps.
fn plain(f: &[(String, String)]) -> (String, Option<String>, u32, String) {
    let l = cli::emit(f, false);
    let (_, map) = cli::run(&["map", "all"], f);
    (l.output, l.ir, l.checks, map)
}

/// The same through the cache in `dir`; also returns the stats of the
/// `emit-llvm` run.
fn cached(f: &[(String, String)], dir: &Path) -> ((String, Option<String>, u32, String), Stats) {
    cached_with(f, dir, false)
}

/// [`cached`] with `emit-llvm --audit`: every reused proof and IR is
/// computed again and compared.
fn audited(f: &[(String, String)], dir: &Path) -> ((String, Option<String>, u32, String), Stats) {
    cached_with(f, dir, true)
}

fn cached_with(
    f: &[(String, String)],
    dir: &Path,
    audit: bool,
) -> ((String, Option<String>, u32, String), Stats) {
    let (output, ir, stats) = emit_cached(f, dir, audit);
    let (_, map) = cli::run(&["map", "--cache", dir.to_str().unwrap(), "all"], f);
    let v = verify_cached(f, dir);
    // `emit-llvm` adds only that it cannot lower data types yet (2B-06).
    let emitted: String = output
        .lines()
        .filter(|x| !x.contains("not compiled yet"))
        .map(|x| format!("{x}\n"))
        .collect();
    assert_eq!(v.output, emitted, "verify and emit-llvm agree");
    let checks = stats.runtime_checks;
    ((output, ir, checks, map), stats)
}

fn set(v: &[String]) -> BTreeSet<String> {
    v.iter().cloned().collect()
}

/// The definitions of `generate(units)` that a run with an empty cache
/// encodes. Proof keys name nothing (ADR-0063 §4, backlog T1-49), so a
/// definition whose read set is another's up to names reuses its result:
/// units 2.. repeat unit 1 except `f0`'s constant, and unit 1 repeats unit 0
/// except `f0` and `g` (the call into the previous unit).
fn alpha_distinct(units: usize) -> BTreeSet<String> {
    let mut s: BTreeSet<String> = ["f0", "f1", "f2", "rec", "g"]
        .iter()
        .map(|f| format!("m0::{f}"))
        .collect();
    for k in 1..units {
        s.insert(format!("m{k}::f0"));
    }
    if units > 1 {
        s.insert("m1::g".into());
    }
    s
}

fn failed_defs(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .filter(|l| l.contains("\"status\":\"failed\""))
        .map(|l| l.split('"').nth(3).unwrap().to_string())
        .collect()
}

#[test]
fn outputs_are_identical_with_and_without_the_cache() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut inputs = vec![("generated".to_string(), generate(4))];
    for sub in ["verify/pass", "verify/fail", "run"] {
        let mut paths: Vec<PathBuf> = fs::read_dir(root.join(sub))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        paths.sort();
        for p in paths {
            inputs.push((p.display().to_string(), fs::read_to_string(&p).unwrap()));
        }
    }
    for (name, text) in inputs {
        let dir = tmp("same");
        let f = files(&text);
        let expect = plain(&f);
        let (cold, _) = cached(&f, &dir);
        assert_eq!(expect, cold, "{name} (cold)");
        let (warm, stats) = audited(&f, &dir);
        assert_eq!(expect, warm, "{name} (warm, audited)");
        assert!(stats.audit_mismatches.is_empty(), "{name}: {stats:?}");
        let (warm, stats) = cached(&f, &dir);
        assert_eq!(expect, warm, "{name} (warm)");
        // Warm: only failed definitions are encoded again, and with an
        // escape (ADR-0029) those that may rest on trusted facts, which
        // are never stored: every one except the `#[external_body]` ones.
        if text.contains("assume(") || text.contains("#[external_body]") {
            let externals: BTreeSet<String> = expect
                .0
                .lines()
                .filter(|l| l.contains("\"trust\":\"external\""))
                .map(|l| l.split('"').nth(3).unwrap().to_string())
                .collect();
            assert!(set(&stats.encoded).is_disjoint(&externals), "{name}");
        } else {
            assert_eq!(set(&stats.encoded), failed_defs(&expect.0), "{name}");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn a_body_change_relowers_only_that_definition() {
    let dir = tmp("body");
    let base = generate(6);
    let (_, cold) = cached(&files(&base), &dir);
    assert_eq!(set(&cold.encoded), alpha_distinct(6));
    assert_eq!(cold.encoded.len() + cold.reused.len(), 30);
    assert!(cold.ir_reused.is_empty());
    assert_eq!(cold.lowered.len(), 30);
    let (_, warm) = cached(&files(&base), &dir);
    assert!(
        warm.encoded.is_empty() && warm.lowered.is_empty(),
        "{warm:?}"
    );
    assert_eq!(warm.reused.len(), 30);
    assert_eq!(warm.ir_reused.len(), 30);

    let changed = base.replacen("        x + 3\n", "        3 + x\n", 1);
    assert_ne!(changed, base);
    let f = files(&changed);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert_eq!(stats.encoded, vec!["m3::f0".to_string()]);
    assert_eq!(stats.lowered, vec!["m3::f0".to_string()]);
    let _ = fs::remove_dir_all(&dir);
}

/// A rename re-encodes nothing (ADR-0063 §4, backlog T1-49): a spec fn, an
/// executable function and its callers' calls, a parameter and a local, in
/// one unit. Every stored result is reused, the audit (which proves each
/// definition again and compares the script as the solver receives it, the
/// status and the proved obligations) finds no difference, and the output is
/// the uncached one.
#[test]
fn a_rename_reencodes_nothing() {
    let dir = tmp("rename");
    let base = generate(6);
    cached(&files(&base), &dir);
    let unit = |text: &str, k: usize| -> (usize, usize) {
        let lo = text.find(&format!("mod m{k} {{")).unwrap();
        let hi = text[lo + 1..]
            .find("mod m")
            .map_or(text.len(), |h| lo + 1 + h);
        (lo, hi)
    };
    let (lo, hi) = unit(&base, 3);
    let renamed_unit = base[lo..hi]
        .replace("m2::pos", "M2_POS")
        .replace("pos(", "nonneg(")
        .replace("pos,", "nonneg,")
        .replace("pos)", "nonneg)")
        .replace("fn f0(", "fn first(")
        .replace("f0(x)", "first(x)")
        .replace("let y: i64", "let v: i64")
        .replace(
            "if y > 5 { y - 5 } else { y }",
            "if v > 5 { v - 5 } else { v }",
        )
        .replace("fn f2(n: i64)", "fn f2(m: i64)")
        .replace(
            "requires small(n),\n    {\n        let mut c",
            "requires small(m),\n    {\n        let mut c",
        )
        .replace(
            "while i < n\n            invariant i <= n,",
            "while i < m\n            invariant i <= m,",
        )
        .replace("decreases n - i,", "decreases m - i,")
        .replace("M2_POS", "m2::pos");
    // The next unit's `#[uses]` follows the renamed spec fn.
    let renamed =
        format!("{}{renamed_unit}{}", &base[..lo], &base[hi..]).replace("m3::pos", "m3::nonneg");
    assert!(
        renamed.contains("fn first(")
            && renamed.contains("spec fn nonneg(")
            && renamed.contains("let v: i64")
    );
    assert!(renamed.contains("while i < m"));
    let f = files(&renamed);
    let expect = plain(&f);
    assert!(
        failed_defs(&expect.0).is_empty() && !expect.0.contains("error"),
        "{}",
        expect.0
    );
    let (out, stats) = audited(&f, &dir);
    assert_eq!(out, expect);
    assert!(stats.audit_mismatches.is_empty(), "{stats:?}");
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, expect);
    assert!(stats.encoded.is_empty(), "{stats:?}");
    assert_eq!(stats.reused.len(), 30);
    let _ = fs::remove_dir_all(&dir);
}

fn impact(f: &[(String, String)], def: &str, change: &str) -> BTreeSet<String> {
    let (ok, rows) = cli::run(&["query", "impact", def, change, "--"], f);
    assert!(ok, "{rows}");
    rows.lines()
        .filter_map(|l| l.split("\"path\":\"").nth(1))
        .map(|r| r.split('"').next().unwrap().to_string())
        .collect()
}

#[test]
fn a_contract_change_reencodes_exactly_the_impact_set() {
    let dir = tmp("contract");
    let base = generate(6);
    cached(&files(&base), &dir);

    // Contract of an executable function: itself and its callers.
    let ensures = base.replacen(
        "    fn f0(x: i64) -> (r: i64)\n        requires small(x),\n        ensures pos(r),\n    {\n        x + 3\n",
        "    fn f0(x: i64) -> (r: i64)\n        requires small(x),\n        ensures pos2(r),\n    {\n        x + 3\n",
        1,
    );
    assert_ne!(ensures, base);
    let f = files(&ensures);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert!(failed_defs(&out.0).is_empty(), "{}", out.0);
    let expect = impact(&f, "m3::f0", "iface");
    assert_eq!(
        expect,
        ["m3::f0", "m3::f1"].map(String::from).into_iter().collect()
    );
    assert_eq!(set(&stats.encoded), expect);
    // IR: the function and the callers that call it (they read its signature).
    assert_eq!(set(&stats.lowered), expect);

    // Body of a spec fn: the definitions that reveal it.
    let spec = ensures.replacen(
        "spec fn pos(r: i64) -> bool { r >= 0 && r < 2000 }",
        "spec fn pos(r: i64) -> bool { r < 2000 && r >= 0 }",
        4,
    );
    let spec = spec.replacen(
        "spec fn pos(r: i64) -> bool { r < 2000 && r >= 0 }",
        "spec fn pos(r: i64) -> bool { r >= 0 && r < 2000 }",
        3,
    );
    let f = files(&spec);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    let expect = impact(&f, "m3::pos", "body");
    assert_eq!(expect.len(), 5, "{expect:?}");
    assert_eq!(set(&stats.encoded), expect);
    // No signature and no verification state changed: no IR is lowered.
    assert!(stats.lowered.is_empty(), "{stats:?}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_failure_in_a_component_relowers_it_with_checks() {
    let dir = tmp("component");
    let base = generate(3);
    cached(&files(&base), &dir);
    // `m1::f2` breaks its loop invariant: it fails, and its call component
    // (only itself: nothing calls f2) keeps runtime checks.
    let broken = base.replacen("            c = c + 1;\n", "            c = c + 2;\n", 2);
    let broken = broken.replacen("            c = c + 2;\n", "            c = c + 1;\n", 1);
    let f = files(&broken);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert_eq!(failed_defs(&out.0), ["m1::f2".to_string()].into());
    assert_eq!(stats.lowered, vec!["m1::f2".to_string()]);
    // `m1::f0` fails: the whole call component (m0, m1, m2 are connected
    // through `g`) loses its proof facts, so all of it is lowered again.
    let f0 = base.replacen("        x + 1\n", "        x + 1500\n", 1);
    let f = files(&f0);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert_eq!(failed_defs(&out.0), ["m1::f0".to_string()].into());
    // `m1::f2` is back to the text whose proof is stored.
    assert_eq!(stats.encoded, vec!["m1::f0".to_string()]);
    let component: BTreeSet<String> = ["m0", "m1", "m2"]
        .iter()
        .flat_map(|u| ["f0", "f1", "g"].map(|d| format!("{u}::{d}")))
        .collect();
    let mut expect = component.clone();
    expect.insert("m1::f2".into());
    assert_eq!(set(&stats.lowered), expect);
    // And back: the component is proved again and lowered without checks.
    let (out, _) = cached(&files(&base), &dir);
    assert_eq!(out, plain(&files(&base)));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn edits_that_shift_spans_or_change_context_keep_outputs_exact() {
    let dir = tmp("edits");
    let base = generate(4);
    let edits = [
        // A comment shifts every span after it.
        base.replacen("mod m1 {", "mod m1 {\n    // moved\n", 1),
        // A new spec fn that a definition of the unit names.
        base.replacen(
            "mod m2 {",
            "mod m2 {\n    spec fn extra(x: i64) -> bool { x > 7 }\n",
            1,
        )
        .replacen("ensures pos(r),\n    {\n        x + 2\n", "ensures pos(r) || extra(r),\n    {\n        x + 2\n", 1),
        // An unrelated new spec fn (no query mentions it).
        base.replacen(
            "mod m2 {",
            "mod m2 {\n    spec fn extra(x: i64) -> bool { x > 7 }\n",
            1,
        ),
        // A new local definition changes what a short name resolves to.
        base.replacen(
            "mod m0 {",
            "mod m0 {\n    #[summary(\"inner\")]\n    mod inner {\n        spec fn unused(x: i64) -> bool { x > 1 }\n    }\n",
            1,
        ),
        // Recursion measure changes.
        base.replacen("        decreases n,\n", "        decreases n + 1,\n", 1),
        // A callee's `#[uses]` loses an entry it does not need, then the ones
        // its proof needs: its callers keep their keys (backlog 2D-22).
        base.replacen("#[uses(small, pos, pos2)]", "#[uses(small, pos)]", 1),
        base.replacen("#[uses(small, pos, pos2)]", "#[uses(pos2)]", 1),
        // Layout and comments inside a definition: its proof is reused,
        // its obligations relocated by node position (ADR-0059 §4).
        base.replacen(
            "        let mut c: i64 = 0;\n",
            "        // counter\n        let   mut c :i64=0;\n\n",
            1,
        ),
        // Renames: a definition (and its callers' references), a
        // parameter, a local (backlog T1-10).
        base.replace("f0(", "first("),
        base.replacen("fn f2(n: i64) -> i64\n        requires small(n),", "fn f2(count: i64) -> i64\n        requires small(count),", 1)
            .replacen("        while i < n\n            invariant i <= n, c == i,\n            decreases n - i,", "        while i < count\n            invariant i <= count, c == i,\n            decreases count - i,", 1),
        base.replace("let y: i64 = f0(x);\n        if y > 5 { y - 5 } else { y }", "let yy: i64 = f0(x);\n        if yy > 5 { yy - 5 } else { yy }"),
        base.clone(),
    ];
    cached(&files(&base), &dir);
    for (i, text) in edits.iter().enumerate() {
        let f = files(text);
        let (out, stats) = audited(&f, &dir);
        assert_eq!(out, plain(&f), "edit {i}");
        assert!(stats.audit_mismatches.is_empty(), "edit {i}: {stats:?}");
        let (again, _) = cached(&f, &dir);
        assert_eq!(again, out, "edit {i} (warm)");
    }
    let _ = fs::remove_dir_all(&dir);
}

/// Layout and comments inside a definition do not encode it again: the
/// proof key holds its qualified normal form, and the stored obligations
/// are node positions relocated to the new text (ADR-0059 §4, backlog
/// T1-10). Its IR is lowered again (the IR key keeps the source).
#[test]
fn layout_inside_a_definition_reencodes_nothing() {
    let dir = tmp("layout");
    let base = generate(3);
    cached(&files(&base), &dir);
    let changed = base
        .replacen(
            "        let mut c: i64 = 0;\n",
            "        // counter\n        let   mut c :i64=0;\n\n",
            1,
        )
        .replacen(
            "            c = c + 1;\n",
            "            c = c\n                + 1; // next\n",
            1,
        );
    assert_ne!(changed, base);
    let f = files(&changed);
    let (out, stats) = audited(&f, &dir);
    assert_eq!(out, plain(&f));
    assert!(stats.audit_mismatches.is_empty(), "{stats:?}");
    let (again, stats) = cached(&f, &dir);
    assert_eq!(again, out);
    assert!(stats.encoded.is_empty(), "{stats:?}");
    let _ = fs::remove_dir_all(&dir);
}

/// A query declares only the spec fns it mentions (backlog 2D-14), so an
/// unrelated spec fn changes no query: nothing is encoded or lowered again,
/// and every definition's script is byte-identical.
#[test]
fn adding_an_unrelated_spec_fn_reencodes_nothing() {
    let dir = tmp("unrelated");
    let base = generate(4);
    cached(&files(&base), &dir);
    let added = base.replacen(
        "mod m2 {",
        "mod m2 {\n    spec fn extra(x: i64, y: bool) -> bool { y && x > 7 }\n",
        1,
    );
    let f = files(&added);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert!(
        stats.encoded.is_empty() && stats.lowered.is_empty(),
        "{stats:?}"
    );
    assert_eq!(stats.reused.len(), 20);
    let before = scripts(&files(&base));
    let after = scripts(&f);
    assert!(!before.is_empty());
    assert_eq!(before, after);
    // The definitions that mention a spec fn declare it (and define it,
    // when they reveal it: a macro, backlog T2-04); the others do not. The
    // scripts name nothing (canonical symbols, ADR-0063): `m1::f0` mentions
    // `small` and `pos`, so no query of it declares a third predicate
    // (`pos2`, which it reveals but does not mention, or `m0::small`), and
    // the one of its postcondition declares both.
    let s = &after["m1::f0"];
    let mut queries: Vec<usize> = Vec::new();
    for line in s.lines() {
        if line.starts_with("; ") {
            queries.push(0);
        } else if line.starts_with("(declare-fun |$") && line.ends_with(" (Int) Bool)") {
            *queries.last_mut().unwrap() += 1;
        }
    }
    assert_eq!(queries.iter().max(), Some(&2), "{s}");
    assert!(s.contains("(< |$") && s.contains(" 2000)"), "{s}");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn damaged_cache_lines_are_ignored() {
    let dir = tmp("damaged");
    let base = generate(3);
    let f = files(&base);
    let expect = plain(&f);
    cached(&f, &dir);
    // The store (`incremental::FILE`).
    let path = dir.join("defs");
    let text = fs::read_to_string(&path).unwrap();
    // Flip one byte of every third entry's payload: its digest no longer matches.
    let mut damaged = 0;
    let lines: Vec<String> = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i >= 2 && i % 3 == 0 {
                damaged += 1;
                let mut b = l.as_bytes().to_vec();
                let k = b.len() - 1;
                b[k] = if b[k] == b'0' { b'1' } else { b'0' };
                String::from_utf8(b).unwrap()
            } else {
                l.to_string()
            }
        })
        .collect();
    fs::write(&path, lines.join("\n") + "\n").unwrap();
    let ((out, ir, checks, map), stats) = cached(&f, &dir);
    assert_eq!((out, ir, checks, map), expect);
    assert_eq!(stats.damaged, damaged);
    assert!(!stats.encoded.is_empty() || !stats.lowered.is_empty());
    // Truncated file, garbage, and another format version.
    for junk in [
        text[..text.len() / 2].to_string(),
        "garbage\n\u{0}\u{1}".to_string(),
        text.replace("cache v1", "cache v0"),
    ] {
        fs::write(&path, junk).unwrap();
        let (out, _) = cached(&f, &dir);
        assert_eq!(out, expect);
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn another_compiler_identity_invalidates_everything() {
    let dir = tmp("identity");
    let f = files(&generate(2));
    let expect = plain(&f);
    cached(&f, &dir);
    // The store as another compiler build wrote it: its second line names
    // the build (`id VERSION+DIGEST`).
    let path = dir.join("defs");
    let text = fs::read_to_string(&path).unwrap();
    let (magic, rest) = text.split_once('\n').unwrap();
    let (id, entries) = rest.split_once('\n').unwrap();
    assert!(id.starts_with("id "), "{id}");
    fs::write(&path, format!("{magic}\nid other\n{entries}")).unwrap();
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, expect);
    // As a run with an empty cache: only the run's own results are reused.
    assert!(stats.ir_reused.is_empty());
    assert_eq!(set(&stats.encoded), alpha_distinct(2));
    assert_eq!(stats.encoded.len() + stats.reused.len(), 10);
    // That run stored its results under this build's identity.
    let (_, stats) = cached(&f, &dir);
    assert_eq!(stats.reused.len(), 10);
    let _ = fs::remove_dir_all(&dir);
}

/// Measurement (backlog 2D-10): `cargo test --release --test incremental
/// -- --ignored --nocapture`. `BMB_INCR_UNITS` sets the size (7 definitions
/// per unit).
#[test]
#[ignore]
fn measure_large_program() {
    let units: usize = std::env::var("BMB_INCR_UNITS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(45);
    let base = generate(units);
    let f = files(&base);
    let dir = tmp("measure");
    // Each run is a `bmb` process: the times include starting it.
    let time = |label: &str, f: &dyn Fn() -> (String, Option<String>, Stats)| {
        let t = Instant::now();
        let out = f();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let s = &out.2;
        println!(
            "{label:<34} {ms:>9.1} ms  encoded {:>4} reused {:>4} lowered {:>4} ir_reused {:>4}",
            s.encoded.len(),
            s.reused.len(),
            s.lowered.len(),
            s.ir_reused.len(),
        );
        out
    };
    println!(
        "{units} units, {} definitions ({} executable)",
        units * 8,
        units * 5
    );
    let t = Instant::now();
    assert!(cli::run(&["check"], &f).0);
    println!(
        "{:<34} {:>9.1} ms  (parse and check only: the floor)",
        "front end",
        t.elapsed().as_secs_f64() * 1000.0
    );
    let t = Instant::now();
    let p = cli::emit(&f, false);
    println!(
        "{:<34} {:>9.1} ms",
        "no cache",
        t.elapsed().as_secs_f64() * 1000.0
    );
    let c = time("cold cache", &|| emit_cached(&f, &dir, false));
    assert_eq!(p.ir, c.1);
    let w = time("warm", &|| emit_cached(&f, &dir, false));
    assert_eq!(p.ir, w.1);
    let t = Instant::now();
    let v = verify_cached(&f, &dir);
    println!(
        "{:<34} {:>9.1} ms  (verify only)",
        "warm, incremental",
        t.elapsed().as_secs_f64() * 1000.0,
    );
    assert!(v.ok);
    let k = units / 2;
    let changed = base.replacen(
        &format!("        x + {k}\n"),
        &format!("        {k} + x\n"),
        1,
    );
    let f2 = files(&changed);
    let b = time("one body changed", &|| emit_cached(&f2, &dir, false));
    assert_eq!(b.1, cli::emit(&f2, false).ir);
    let changed = changed.replacen(
        &format!("ensures pos(r),\n    {{\n        {k} + x\n"),
        &format!("ensures pos2(r),\n    {{\n        {k} + x\n"),
        1,
    );
    let f3 = files(&changed);
    let c = time("one contract changed", &|| emit_cached(&f3, &dir, false));
    assert!(c.1.is_some() && c.2.runtime_checks == 0);
    let size = fs::metadata(dir.join("defs")).unwrap().len();
    println!("cache file {size} bytes");
    let _ = fs::remove_dir_all(&dir);
}

/// `bmb verify --stats FILE` (backlog 2A-14, BCB) names the definitions that
/// were encoded and reused; stdout is the same as without it.
#[test]
fn verify_stats_name_the_reencoded_definitions() {
    let dir = tmp("stats");
    fs::create_dir_all(&dir).unwrap();
    let src = dir.join("g.bmb");
    let cache = dir.join("cache");
    let stats = dir.join("stats.json");
    let run = |with_stats: bool| {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.arg("verify").arg("--cache").arg(&cache);
        if with_stats {
            cmd.arg("--stats").arg(&stats);
        }
        let out = cmd.arg(&src).output().unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap()
    };
    let field = |name: &str| -> Vec<String> {
        let v = json::parse(&fs::read_to_string(&stats).unwrap()).unwrap();
        match v.get(name) {
            Some(Value::Array(xs)) => xs.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
            other => panic!("{name}: {other:?}"),
        }
    };
    let base = generate(6);
    fs::write(&src, &base).unwrap();
    run(true);
    assert_eq!(set(&field("encoded")), alpha_distinct(6));
    assert_eq!(field("encoded").len() + field("reused").len(), 30);
    assert_eq!(run(true), run(false));
    assert!(field("encoded").is_empty());
    assert_eq!(field("reused").len(), 30);

    let body = base.replacen("        x + 3\n", "        3 + x\n", 1);
    fs::write(&src, &body).unwrap();
    run(true);
    assert_eq!(field("encoded"), vec!["m3::f0".to_string()]);

    // `#[uses]` reveals bodies only in the function's own queries, so
    // dropping an entry it does not need re-encodes it alone, not its
    // callers (backlog 2D-22; before, `#[uses]` was in the callers' keys).
    let f0 = "    #[effects(pure)]\n    #[uses(small, pos, pos2)]\n    fn f0(x: i64) -> (r: i64)\n        requires small(x),\n        ensures pos(r),\n    {\n        x + 4\n";
    assert!(body.contains(f0));
    fs::write(
        &src,
        body.replacen(f0, &f0.replace("small, pos, pos2", "small, pos"), 1),
    )
    .unwrap();
    run(true);
    assert_eq!(field("encoded"), vec!["m4::f0".to_string()]);

    // `emit-llvm` (and `build`) take it too: the first run lowers, the next reuses.
    let emit = || {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("emit-llvm")
            .arg("--cache")
            .arg(&cache)
            .arg("--stats")
            .arg(&stats)
            .arg(&src)
            .arg("-o")
            .arg(dir.join("out.ll"))
            .output()
            .unwrap();
        assert!(out.status.success());
    };
    emit();
    assert_eq!(field("lowered").len(), 30);
    emit();
    assert!(field("lowered").is_empty() && field("encoded").is_empty());
    assert_eq!(field("ir_reused").len(), 30);
    let _ = fs::remove_dir_all(&dir);
}

/// What a proof reads of data types (backlog 2B-05a): the contract of a
/// `T::f` callee and the field types of the structs it names. Changing
/// either re-encodes the reader; an unrelated type does not.
#[test]
fn data_type_changes_reencode_their_readers() {
    const BASE: &str = r#"#[summary("types")]
mod geo {
    struct Point { x: i64, y: i64 }

    struct Other { z: i64 }

    spec fn kept(p: Point, q: Point) -> bool { p.y == q.y }

    impl Point {
        #[effects(pure)]
        #[uses(kept)]
        fn reset(self: &mut Point) -> ()
            ensures kept(*old(self), *self),
        {
            self.x = 0;
        }
    }

    #[effects(pure)]
    #[uses(kept)]
    fn caller(p: Point) -> (r: Point)
        ensures kept(p, r),
    {
        let mut q: Point = p;
        Point::reset(&mut q);
        q
    }

    #[effects(pure)]
    fn reader(p: Point) -> i64 {
        p.x
    }

    #[effects(pure)]
    fn other(o: Other) -> i64 {
        o.z
    }
}
"#;
    // `Point::reset` is a call edge of `caller` (map and impact, 2B-07).
    assert!(impact(&files(BASE), "geo::Point::reset", "iface").contains("geo::caller"));
    let dir = tmp("types");
    let run = |text: &str| verify_cached(&files(text), &dir);
    let first = run(BASE);
    assert!(first.ok, "{}", first.output);
    assert!(run(BASE).incr.encoded.is_empty());
    // The callee's contract: `caller` reads it through `Point::reset`.
    let weaker = BASE.replace("            ensures kept(*old(self), *self),\n", "");
    let out = run(&weaker);
    assert!(!out.ok, "the caller no longer proves:\n{}", out.output);
    assert!(
        set(&out.incr.encoded).contains("geo::caller"),
        "{:?}",
        out.incr.encoded
    );
    run(BASE);
    // A field type: `reader` names `Point`, `other` does not.
    let retyped = BASE.replace(
        "struct Point { x: i64, y: i64 }",
        "struct Point { x: i64, y: u64 }",
    );
    let out = run(&retyped);
    assert!(out.ok, "{}", out.output);
    let enc = set(&out.incr.encoded);
    assert!(
        enc.contains("geo::reader") && enc.contains("geo::caller"),
        "{enc:?}"
    );
    assert!(!enc.contains("geo::other"), "{enc:?}");
    // What was re-proved lies inside the type's impact set (H-MAINT).
    let hit = impact(&files(BASE), "geo::Point", "iface");
    assert!(enc.is_subset(&hit), "{enc:?} not in {hit:?}");
    assert!(!hit.contains("geo::other"), "{hit:?}");
    let _ = fs::remove_dir_all(&dir);
}

/// A definition proved only by the ground-instance retry (backlog 2V-03):
/// its stored verdict must match the recomputed one.
#[test]
fn retried_proofs_are_reused_exactly() {
    let text = include_str!("data/retry.bmb");
    let dir = tmp("retry");
    let f = files(text);
    let expect = plain(&f);
    let (cold, _) = cached(&f, &dir);
    assert_eq!(expect, cold, "cold");
    let (warm, stats) = audited(&f, &dir);
    assert_eq!(expect, warm, "warm, audited");
    assert!(stats.audit_mismatches.is_empty(), "{stats:?}");
}

/// Instances are stored under their own keys (backlog 2T-04): a warm build
/// lowers none of them, a hit still brings the instances it calls, and a
/// change to a type argument's declaration relowers only the instances it
/// names.
#[test]
fn instances_are_reused_and_follow_their_type_arguments() {
    const BASE: &str = r#"#[summary("instances")]
mod g {
    struct Pt { x: i64, y: i64 }

    struct Cell<T: Copy> { v: T }

    #[effects(pure)]
    fn id<T: Copy>(x: T) -> T {
        x
    }

    #[effects(pure)]
    fn wrap<T: Copy>(x: T) -> Cell<T> {
        Cell { v: id::<T>(x) }
    }

    #[effects(io)]
    fn main() -> () {
        let p: Pt = Pt { x: 1, y: 2 };
        let c: Cell<Pt> = wrap::<Pt>(p);
        print_i64(c.v.y);
        print_i64(id::<i64>(7));
    }
}
"#;
    let dir = tmp("instances");
    let f = files(BASE);
    let (out, cold) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    let lowered = set(&cold.lowered);
    for k in ["g::wrap<g::Pt>", "g::id<g::Pt>", "g::id<i64>", "g::main"] {
        assert!(lowered.contains(k), "{lowered:?}");
    }
    let (out, warm) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    assert!(warm.lowered.is_empty(), "{warm:?}");
    assert!(set(&warm.ir_reused).contains("g::id<g::Pt>"), "{warm:?}");
    // A field type of `Pt`: the instances that take it, and the function
    // that reads it, are lowered again; `id<i64>` is not.
    let retyped = BASE.replace(
        "struct Pt { x: i64, y: i64 }",
        "struct Pt { x: u64, y: i64 }",
    );
    let f = files(&retyped);
    let (out, stats) = cached(&f, &dir);
    assert_eq!(out, plain(&f));
    let lowered = set(&stats.lowered);
    assert!(
        lowered.contains("g::wrap<g::Pt>") && lowered.contains("g::id<g::Pt>"),
        "{lowered:?}"
    );
    assert!(!lowered.contains("g::id<i64>"), "{lowered:?}");
    let _ = fs::remove_dir_all(&dir);
}

/// A lemma is behind the contract firewall like an executable callee
/// (ADR-0062 §3): a change to its proof re-encodes it alone, a change to
/// its contract re-encodes it and every definition that calls it. Outputs
/// stay exact, and the audit finds no stale entry.
#[test]
fn lemma_edits_follow_the_contract_firewall() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/run");
    let base = fs::read_to_string(root.join("max_list.bmb")).unwrap();
    let body = base.replacen(
        "        if a[n - 1] != r {\n",
        "        if r != a[n - 1] {\n",
        1,
    );
    assert_ne!(body, base);
    let contract = base.replacen(
        "        ensures sum_le(a, i, j),\n",
        "        ensures sum_le(a, i, j), sum_le(a, i, j),\n",
        1,
    );
    assert_ne!(contract, base);
    let dir = tmp("lemmas");
    cached(&files(&base), &dir);
    for text in [&body, &base, &contract, &base] {
        let f = files(text);
        let (out, stats) = audited(&f, &dir);
        assert_eq!(out, plain(&f));
        assert!(stats.audit_mismatches.is_empty(), "{stats:?}");
    }
    let _ = fs::remove_dir_all(&dir);
    // From a warm cache of `base`, exactly what each edit reaches.
    for (i, (text, want)) in [
        (&body, vec!["ml::contains_has_index"]),
        (&contract, vec!["ml::sum_grows", "ml::total"]),
    ]
    .into_iter()
    .enumerate()
    {
        let dir = tmp(&format!("lemmas{i}"));
        cached(&files(&base), &dir);
        let (_, stats) = cached(&files(text), &dir);
        let want: BTreeSet<String> = want.iter().map(|s| s.to_string()).collect();
        assert_eq!(set(&stats.encoded), want, "edit {i}");
        let _ = fs::remove_dir_all(&dir);
    }
}

/// A type invariant (ADR-0060 §3, backlog T1-11) is assumed and proved
/// wherever a value of the type passes, also where no text names the type
/// (`relay` sees it only through its callees' signatures). Changing the
/// predicate or the attribute re-encodes inside the type's `impact` set,
/// outputs stay exact (audited), an unrelated definition is not touched,
/// and the type's identity follows the predicate, not its name.
#[test]
fn invariant_edits_reencode_where_values_pass() {
    const BASE: &str = r#"#[summary("invariants")]
mod iv {
    #[invariant(ordered)]
    struct Span { lo: i64, hi: i64 }

    spec fn ordered(s: &Span) -> bool { s.lo <= s.hi }

    spec fn small(x: i64) -> bool { 0 <= x && x <= 1000 }

    #[effects(pure)]
    #[uses(ordered, small)]
    fn make(a: i64) -> Span
        requires small(a),
    {
        Span { lo: a, hi: a + 1 }
    }

    #[effects(pure)]
    #[uses(ordered)]
    fn width(s: Span) -> i64 {
        if s.hi - s.lo > 100 { 100 } else { s.hi - s.lo }
    }

    #[effects(pure)]
    #[uses(small)]
    fn relay(a: i64) -> i64
        requires small(a),
    {
        width(make(a))
    }

    #[effects(pure)]
    fn other(x: i64) -> i64 {
        x / 2
    }
}
"#;
    let stronger = BASE.replace("{ s.lo <= s.hi }", "{ s.lo < s.hi }");
    let without = BASE.replace("    #[invariant(ordered)]\n", "");
    for t in [&stronger, &without] {
        assert_ne!(t, BASE);
    }
    let dir = tmp("invariants");
    cached(&files(BASE), &dir);
    for text in [&stronger, BASE, &without, BASE] {
        let f = files(text);
        let (out, stats) = audited(&f, &dir);
        assert_eq!(out, plain(&f));
        assert!(stats.audit_mismatches.is_empty(), "{stats:?}");
    }
    let _ = fs::remove_dir_all(&dir);
    let hit = impact(&files(BASE), "iv::Span", "iface");
    assert!(
        hit.contains("iv::relay") && !hit.contains("iv::other"),
        "{hit:?}"
    );
    for (i, text) in [&stronger, &without].into_iter().enumerate() {
        let dir = tmp(&format!("invariants{i}"));
        cached(&files(BASE), &dir);
        let (_, stats) = cached(&files(text), &dir);
        let enc = set(&stats.encoded);
        assert!(enc.contains("iv::relay"), "edit {i}: {enc:?}");
        assert!(!enc.contains("iv::other"), "edit {i}: {enc:?}");
        assert!(enc.is_subset(&hit), "edit {i}: {enc:?} not in {hit:?}");
        let _ = fs::remove_dir_all(&dir);
    }
    // The type's identity holds its invariant (ADR-0059 §1): it changes
    // with the predicate and the attribute, not with the predicate's name.
    let id = |text: &str| -> String {
        let (ok, h) = cli::run(&["hash"], &files(text));
        assert!(ok, "{h}");
        h.lines()
            .find(|l| l.contains("\"def\":\"iv::Span\""))
            .and_then(|l| l.split("\"id\":\"").nth(1))
            .map(|r| r.split('"').next().unwrap().to_string())
            .unwrap()
    };
    let base_id = id(BASE);
    assert_ne!(id(&stronger), base_id);
    assert_ne!(id(&without), base_id);
    assert_eq!(id(&BASE.replace("ordered", "sorted_pair")), base_id);
}

/// The proof key of `def` (`bmb reads`).
fn proof_key(src: &str, def: &str) -> String {
    let (ok, rows) = cli::run(&["reads"], &files(src));
    assert!(ok, "{rows}");
    let line = rows
        .lines()
        .find(|l| l.starts_with(&format!("{{\"def\":\"{def}\"")))
        .unwrap_or_else(|| panic!("{def}: {rows}"));
    line.split("\"key\":\"").nth(1).unwrap()[..64].to_string()
}

/// Nameless proof keys (ADR-0063 §4, backlog T1-49) keep apart what the
/// queries read: a key names nothing, but a renaming is the only change
/// that keeps it. Each pair below has the same shape up to names; the first
/// differs only by names (same key), the others in what a query reads
/// (different keys).
#[test]
fn nameless_keys_tell_apart_what_queries_read() {
    const BASE: &str = r#"#[summary("k")]
mod u {
    struct S { a: i64, b: i64, c: i64 }

    spec fn p(x: i64) -> bool { x >= 0 }

    spec fn q(x: i64) -> bool { x >= 1 }

    spec fn small(x: i64) -> bool { 0 <= x && x < 100 }

    spec fn both(s: S) -> bool { small(s.a) && small(s.b) }

    #[effects(pure)]
    #[uses(small, p)]
    fn f(x: i64) -> (r: i64)
        requires small(x),
        ensures p(r),
    { x }

    #[effects(pure)]
    #[uses(small, p)]
    fn g(x: i64) -> (r: i64)
        requires small(x),
        ensures p(r),
    { x }

    #[effects(pure)]
    #[uses(both, small, p)]
    fn d(s: S) -> (r: i64)
        requires both(s),
        ensures p(r),
    {
        let y: i64 = f(s.a);
        let z: i64 = g(s.b);
        y + z
    }
}
"#;
    let key = |src: &str| proof_key(src, "u::d");
    let base = key(BASE);
    // Names only: a spec fn, a callee, the type, a field, a local.
    let renamed = BASE
        .replace("spec fn p(", "spec fn nonneg(")
        .replace("p(r)", "nonneg(r)")
        .replace(", p)]", ", nonneg)]")
        .replace("fn f(", "fn first(")
        .replace("f(s.a)", "first(s.a)")
        .replace("struct S { a: i64", "struct Pair { left: i64")
        .replace("(s: S)", "(s: Pair)")
        .replace("s.a", "s.left")
        .replace("let y: i64", "let w: i64")
        .replace("y + z", "w + z");
    assert!(!renamed.contains("fn p(") && !renamed.contains("s.a") && !renamed.contains("let y"));
    assert_eq!(key(&renamed), base, "a rename keeps the key");
    for (what, changed) in [
        // A callee's contract names an unrevealed spec fn of the same
        // signature instead of the revealed one.
        (
            "callee contract",
            BASE.replacen("ensures p(r),\n    { x }", "ensures q(r),\n    { x }", 1),
        ),
        // The same callee twice instead of two alike.
        ("same callee twice", BASE.replace("g(s.b)", "f(s.b)")),
        // The other field.
        ("field", BASE.replace("f(s.a)", "f(s.b)")),
        // The revealed spec fn's body.
        ("revealed body", BASE.replace("{ x >= 0 }", "{ x >= 2 }")),
        // A field's type.
        ("field type", BASE.replace("c: i64 }", "c: i32 }")),
    ] {
        assert_ne!(changed, BASE, "{what}");
        assert!(cli::run(&["reads"], &files(&changed)).0, "{what}");
        assert_ne!(key(&changed), base, "{what}");
    }
    assert_eq!(key(&BASE.replace("{ x >= 1 }", "{ x >= 3 }")), base);
}

/// A definition that reuses another's result by key (equal up to names)
/// keeps it under its own path: editing the other one later re-encodes
/// only that one (`bench/bcb` found the loss).
#[test]
fn a_result_reused_by_key_stays_when_its_source_changes() {
    let dir = tmp("twin");
    let base = generate(3);
    let (_, cold) = cached(&files(&base), &dir);
    // `m1::f1` and `m2::f1` are `m0::f1` up to names: reused in the cold run.
    for d in ["m1::f1", "m2::f1"] {
        assert!(cold.reused.contains(&d.to_string()), "{cold:?}");
    }
    // Edit `m0::f1`, the definition they took the result from.
    let edited = base.replacen(
        "        if y > 5 { y - 5 } else { y }",
        "        if y > 6 { y - 6 } else { y }",
        1,
    );
    assert!(edited.find("y > 6").unwrap() < edited.find("mod m1 {").unwrap());
    let (_, stats) = cached(&files(&edited), &dir);
    assert_eq!(stats.encoded, vec!["m0::f1".to_string()], "{stats:?}");
    let _ = fs::remove_dir_all(&dir);
}
