//! Proof gate corpus (backlog 1B-12).
//! - `tests/check/ok` and `tests/verify/pass`: every definition is proved.
//! - `tests/verify/fail`: output matches the sibling `.jsonl`
//!   (`BMB_BLESS=1` rewrites it), and every non-helper definition fails.

//!
//! Through the command line (backlog 4A-04): `bmb verify` (its stdout and
//! exit status), `--emit-smt` for the scripts and `bmb lint` for the
//! vacuity points.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

mod common;

use common::cli;
use common::json;

/// What `bmb verify` printed (diagnostics, then a status line per
/// definition) and whether it succeeded.
struct Out {
    ok: bool,
    output: String,
}

fn verify_files(files: &[(String, String)]) -> Out {
    let (ok, output) = cli::run(&["verify"], files);
    Out { ok, output }
}

/// `bmb verify --emit-smt`: its result and the script of every
/// definition, by path.
fn verify_scripts(files: &[(String, String)]) -> (Out, BTreeMap<String, String>) {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "bmb-verify-smt-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("smt")).unwrap();
    for (name, text) in files {
        let p = dir.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(&dir)
        .args(["verify", "--emit-smt", "smt"])
        .args(files.iter().map(|(n, _)| n))
        .output()
        .unwrap();
    let mut scripts = BTreeMap::new();
    for e in fs::read_dir(dir.join("smt")).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_str().unwrap().to_string();
        if let Some(def) = name.strip_suffix(".smt2") {
            scripts.insert(def.replace('.', "::"), fs::read_to_string(&p).unwrap());
        }
    }
    let _ = fs::remove_dir_all(&dir);
    let res = Out {
        ok: out.status.success(),
        output: String::from_utf8(out.stdout).unwrap(),
    };
    (res, scripts)
}

/// The reach points `bmb lint` finds refuted by the program's own proofs
/// (W0905).
fn vacuous_points(files: &[(String, String)]) -> Vec<String> {
    let (_, out) = cli::run(&["lint", "--select", "W0905"], files);
    out.lines()
        .filter(|l| l.contains("\"code\":\"W0905\""))
        .map(str::to_string)
        .collect()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
}

fn bmb_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    v.sort();
    v
}

fn name(p: &Path) -> String {
    let parent = p.parent().unwrap().file_name().unwrap().to_str().unwrap();
    format!("{parent}/{}", p.file_name().unwrap().to_str().unwrap())
}

fn verify(p: &Path) -> Out {
    verify_files(&[(name(p), fs::read_to_string(p).unwrap())])
}

#[test]
fn pass_corpus_is_proved() {
    let mut files = bmb_files(&root().join("check").join("ok"));
    files.extend(bmb_files(&root().join("verify").join("pass")));
    for p in files {
        let out = verify(&p);
        assert!(out.ok, "{} should verify:\n{}", p.display(), out.output);
    }
}

/// No reach point of the proved corpora, the run programs or the bench
/// seed and baseline programs is refuted by its own proof (backlog 1B-44:
/// a loop that kept a `Vec`'s length made two corpus proofs vacuous and
/// nothing said so). One program per file; a bench `seed/src` directory is
/// one program.
#[test]
fn proofs_are_not_vacuous() {
    let mut programs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let one = |p: &Path| (name(p), vec![(name(p), fs::read_to_string(p).unwrap())]);
    for dir in [root().join("verify").join("pass"), root().join("run")] {
        programs.extend(bmb_files(&dir).iter().map(|p| one(p)));
    }
    let bench = root().join("..").join("bench");
    programs.extend(bmb_files(&bench.join("baseline")).iter().map(|p| one(p)));
    for set in ["m1", "m1b", "bvb"] {
        let src = bench.join(set).join("seed").join("src");
        let files = bmb_files(&src)
            .iter()
            .map(|p| (name(p), fs::read_to_string(p).unwrap()))
            .collect();
        programs.push((format!("{set}/seed"), files));
    }
    // BRB-ENCAP (backlog T0-09): one seed unit per task directory.
    let mut encap: Vec<PathBuf> = fs::read_dir(bench.join("brb-encap").join("seed"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    encap.sort();
    for dir in encap {
        programs.extend(bmb_files(&dir).iter().map(|p| one(p)));
    }
    // The standard library's units, each alone, and each program that uses
    // them (the compiler adds them, ADR-0092).
    for lib in bmb_files(&bench.join("..").join("lib").join("std")) {
        programs.push(one(&lib));
    }
    for p in bmb_files(&root().join("lib_run")) {
        programs.push(one(&p));
    }
    for (label, files) in programs {
        let points = vacuous_points(&files);
        assert!(points.is_empty(), "{label}: {points:?}");
    }
}

/// The Benchmarks Game programs (`bench/game`, backlog T3-01) are proved with
/// the standard library units they use. Running them takes tens of seconds,
/// so the gate proves them and `bench/game/README.md` runs them.
#[test]
fn game_programs_are_proved() {
    let repo = root().join("..");
    let games = bmb_files(&repo.join("bench").join("game"));
    assert!(games.len() >= 2);
    for p in games {
        let files = vec![(name(&p), fs::read_to_string(&p).unwrap())];
        let out = verify_files(&files);
        assert!(out.ok, "{}: {}", p.display(), out.output);
    }
}

/// Every standard library unit (`lib/std/`: bytes, ADR-0044 §4; UTF-8,
/// CRC-32, SHA-256, base64, T4; str, ADR-0092) is proved completely, each on
/// its own.
#[test]
fn libraries_are_proved() {
    let libs = bmb_files(&root().join("..").join("lib").join("std"));
    assert!(libs.len() >= 6);
    for p in libs {
        let out = verify_files(&[(name(&p), fs::read_to_string(&p).unwrap())]);
        assert!(out.ok, "{}: {}", p.display(), out.output);
    }
}

#[test]
fn fail_corpus_matches_expected() {
    let bless = std::env::var("BMB_BLESS").is_ok();
    let mut failures = Vec::new();
    for p in bmb_files(&root().join("verify").join("fail")) {
        let out = verify(&p);
        assert!(!out.ok, "{} should fail", p.display());
        let expected_path = p.with_extension("jsonl");
        if bless {
            fs::write(&expected_path, &out.output).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();
        if expected != out.output {
            failures.push(format!(
                "{}\n--- expected\n{expected}--- actual\n{}",
                p.display(),
                out.output
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// An obligation that reveals a scalar leaf `spec fn` is first probed with
/// the definition as a macro under 1% of the solver limit, then, undecided,
/// solved with the quantified definition (backlog T2-05, ADR-0067). The
/// counterexample comes from the probe, far below the limit (with the
/// quantified definition alone the query reached it, `contracts::wrong_post`
/// in the fail corpus).
#[test]
fn scalar_leaf_definitions_are_probed_as_macros() {
    let src = r#"
#[summary("탐침")]
mod m {
    spec fn is_succ(x: i64, r: i64) -> bool { r == x + 1 }

    spec fn small(x: i64) -> bool { x < 100 }

    #[effects(pure)]
    #[uses(is_succ)]
    fn wrong(x: i64) -> (r: i64)
        ensures is_succ(x, r),
    {
        x
    }

    #[effects(pure)]
    #[uses(is_succ, small)]
    fn right(x: i64) -> (r: i64)
        requires small(x),
        ensures is_succ(x, r),
    {
        x + 1
    }
}
"#;
    let (out, scripts) = verify_scripts(&[("m.bmb".into(), src.into())]);
    for def in ["m::wrong", "m::right"] {
        let script = &scripts[def];
        // The postcondition's probe comes just before its full query.
        let lines: Vec<&str> = script.lines().collect();
        let section = |prefix: &str| {
            let at = lines
                .iter()
                .position(|l| l.starts_with(prefix) && l.contains(": E0802 "))
                .expect(script);
            let len = lines[at + 1..]
                .iter()
                .take_while(|l| !l.starts_with("; "))
                .count();
            (at, lines[at..at + 1 + len].join("\n"))
        };
        let (probe_at, probe) = section("; probe of obligation ");
        let (full_at, full) = section("; obligation ");
        assert_eq!(probe_at + probe.lines().count(), full_at, "{script}");
        // The scripts are what the solver gets: canonical symbols (`|$k|`,
        // ADR-0063). The probe defines the predicate as a macro; the full
        // query states it by a quantified definition.
        assert!(
            probe.contains("(define-fun |$") && probe.contains("(set-option :rlimit 50000)"),
            "{script}"
        );
        assert!(
            full.contains("(forall ((|$") && !full.contains("(define-fun |$"),
            "{script}"
        );
    }
    let row = |def: &str| {
        out.output
            .lines()
            .find(|l| l.starts_with(&format!(r#"{{"def":"{def}""#)))
            .unwrap()
            .to_string()
    };
    let rlimit = |row: &str| -> u64 {
        row.split(r#""rlimit":"#)
            .nth(1)
            .and_then(|r| r.split(',').next())
            .and_then(|n| n.parse().ok())
            .unwrap()
    };
    let wrong = row("m::wrong");
    assert!(wrong.contains(r#""status":"failed""#), "{wrong}");
    assert!(rlimit(&wrong) < 50_000, "{wrong}");
    assert!(
        out.output.contains("counterexample: x = "),
        "{}",
        out.output
    );
    let right = row("m::right");
    assert!(right.contains(r#""status":"verified""#), "{right}");
}

#[test]
fn queries_and_output_are_deterministic() {
    let mut files = bmb_files(&root().join("check").join("ok"));
    files.extend(bmb_files(&root().join("verify").join("fail")));
    for p in files {
        let file = [(name(&p), fs::read_to_string(&p).unwrap())];
        let ((a, sa), (b, sb)) = (verify_scripts(&file), verify_scripts(&file));
        assert_eq!(sa, sb, "{}", p.display());
        assert_eq!(a.output, b.output, "{}", p.display());
    }
}

#[test]
fn benchmark_programs_are_proved() {
    let bench = root().join("..").join("bench");
    for p in bmb_files(&bench.join("baseline")) {
        let out = verify(&p);
        assert!(out.ok, "{} should verify:\n{}", p.display(), out.output);
    }
    // The M1 seed workspace is one program spread over unit files.
    let seed = bench.join("m1").join("seed").join("src");
    let files: Vec<(String, String)> = bmb_files(&seed)
        .into_iter()
        .map(|p| (name(&p), fs::read_to_string(&p).unwrap()))
        .collect();
    let out = verify_files(&files);
    assert!(out.ok, "seed workspace should verify:\n{}", out.output);
}

#[test]
fn hidden_predicate_hint_leads_to_a_proof() {
    let src = r#"
#[summary("힌트")]
mod h {
    spec fn small(x: i64) -> bool { x < 100 }

    spec fn fits(x: i64) -> bool { small(x) }

    spec fn bigger(x: i64, r: i64) -> bool { r > x }

    #[effects(pure)]
    #[uses(small, bigger)]
    fn inc(x: i64) -> (r: i64)
        requires fits(x),
        ensures bigger(x, r),
    {
        x + 1
    }

    #[effects(pure)]
    #[uses(USES)]
    fn caller() -> i64 {
        let y: i64 = inc(5);
        assert(bigger(5, y) && y > 5);
        y
    }
}
"#;
    let run = |uses: &str| verify_files(&[("h.bmb".into(), src.replace("USES", uses))]);
    let first = run("bigger");
    assert!(!first.ok);
    // The precondition `fits(5)` fails: the default context (ADR-0095 A1)
    // reveals `fits` and `bigger`, the callee's contract, but not `small`
    // in `fits`'s body. The hint names `small` and nothing else.
    let hint = first
        .output
        .lines()
        .find(|l| l.contains("\"code\":\"E0801\""))
        .expect("precondition failure");
    assert!(hint.contains("add to #[uses(small)]"), "{hint}");
    assert!(run("bigger, small").ok);
}

/// A range `for` is encoded as its `while` expansion (ADR-0049 §2, backlog
/// 2K-01d and 2R-04f): `let mut i = lo; while i < hi invariant lo <= i &&
/// (i <= hi || i == lo), P decreases hi - i { body; i = i + 1; }`, with
/// `if i == hi { break; }` before the step of `lo..=hi` and the step before
/// each `continue`. The proof scripts are byte for byte the same.
#[test]
fn for_queries_are_the_while_expansion() {
    let unit = |body: &str| {
        format!(
            "#[summary(\"t\")]\nmod t {{\n    spec fn few(n: u64) -> bool {{\n        n < 1000\n    }}\n\n    #[effects(pure)]\n    #[uses(few)]\n    fn f(a: &[u64], n: u64) -> u64\n        requires few(n),\n    {{\n        let mut s: u64 = 0;\n{body}        s\n    }}\n}}\n"
        )
    };
    let cases = [
        (
            "        for i in 0..n\n            invariant s <= 1000 * i,\n        {\n            s = s + i;\n        }\n",
            "        let mut i: u64 = 0;\n        while i < n\n            invariant 0 <= i && (i <= n || i == 0), s <= 1000 * i,\n            decreases n - i,\n        {\n            s = s + i;\n            i = i + 1;\n        }\n",
        ),
        (
            "        for i in 1..=n\n            invariant s <= 1000 * i,\n        {\n            s = s + i;\n        }\n",
            "        let mut i: u64 = 1;\n        while i <= n\n            invariant 1 <= i && (i <= n || i == 1), s <= 1000 * i,\n            decreases n - i,\n        {\n            s = s + i;\n            if i == n {\n                break;\n            }\n            i = i + 1;\n        }\n",
        ),
        (
            "        for i in 0..a.len()\n            invariant s <= i,\n        {\n            if a[i] == 0 {\n                continue;\n            }\n            if a[i] == 7 {\n                break;\n            }\n            s = s + 1;\n        }\n",
            "        let mut i: usize = 0;\n        while i < a.len()\n            invariant 0 <= i && (i <= a.len() || i == 0), s <= i,\n            decreases a.len() - i,\n        {\n            if a[i] == 0 {\n                i = i + 1;\n                continue;\n            }\n            if a[i] == 7 {\n                break;\n            }\n            s = s + 1;\n            i = i + 1;\n        }\n",
        ),
    ];
    for (for_body, while_body) in cases {
        let script = |body: &str| {
            let files = [("t.bmb".to_string(), unit(body))];
            let (out, scripts) = verify_scripts(&files);
            assert!(out.ok, "{}", out.output);
            scripts["t::f"].clone()
        };
        assert_eq!(script(for_body), script(while_body), "{for_body}");
    }
}

/// `x?` is encoded as `match x { None => return None, Some(v) => v }` (and
/// `Ok(v) => v, Err(e) => return Err(e)`, ADR-0049 §3, backlog 2K-01e): the
/// proof scripts are byte for byte the same.
#[test]
fn try_queries_are_the_match_expansion() {
    let unit = |body: &str| {
        format!(
            "#[summary(\"t\")]\nmod t {{\n    spec fn ok(r: Result<u64, u64>) -> bool {{\n        match r {{\n            Ok(v) => v < 100,\n            Err(c) => c > 7,\n        }}\n    }}\n\n    #[effects(pure)]\n    fn half(x: u64) -> Option<u64> {{\n        if x % 2 == 0 {{ Some(x / 2) }} else {{ None }}\n    }}\n\n    #[effects(pure)]\n    #[uses(ok)]\n    fn get(x: u64) -> (r: Result<u64, u64>)\n        ensures ok(r),\n    {{\n        if x < 100 {{ Ok(x) }} else {{ Err(x) }}\n    }}\n\n{body}}}\n"
        )
    };
    let cases = [
        (
            "    #[effects(pure)]\n    fn f(x: u64) -> Option<u64> {\n        let h: u64 = half(x)?;\n        Some(half(h)?)\n    }\n",
            "    #[effects(pure)]\n    fn f(x: u64) -> Option<u64> {\n        let h: u64 = match half(x) {\n            None => return None,\n            Some(v) => v,\n        };\n        Some(match half(h) {\n            None => return None,\n            Some(w) => w,\n        })\n    }\n",
        ),
        (
            "    #[effects(pure)]\n    #[uses(ok)]\n    fn f(x: u64) -> (r: Result<u64, u64>)\n        ensures ok(r),\n    {\n        let a: u64 = get(x)?;\n        let b: u64 = get(a + 1)?;\n        Ok(b)\n    }\n",
            "    #[effects(pure)]\n    #[uses(ok)]\n    fn f(x: u64) -> (r: Result<u64, u64>)\n        ensures ok(r),\n    {\n        let a: u64 = match get(x) {\n            Ok(v) => v,\n            Err(e) => return Err(e),\n        };\n        let b: u64 = match get(a + 1) {\n            Ok(w) => w,\n            Err(e2) => return Err(e2),\n        };\n        Ok(b)\n    }\n",
        ),
    ];
    for (try_body, match_body) in cases {
        let script = |body: &str| {
            let files = [("t.bmb".to_string(), unit(body))];
            let (out, scripts) = verify_scripts(&files);
            assert!(out.ok, "{}", out.output);
            scripts["t::f"].clone()
        };
        assert_eq!(script(try_body), script(match_body), "{try_body}");
    }
}

/// An agent's answer to BVB `vc-vv0039-max-of-list` (backlog T1-14): the
/// contract states membership with a recursive predicate, the hidden one
/// with a quantifier. `{ENSURES}` and `{PROOF}` are filled per case.
const VV0039: &str = r#"#[summary("과제 구현")]
mod tasks {
    spec fn non_empty(a: &[i64]) -> bool {
        a.len() > 0
    }

    spec fn contains_prefix(a: &[i64], n: usize, r: i64) -> bool
        decreases n,
    {
        if n == 0 { false } else { a[n - 1] == r || contains_prefix(a, n - 1, r) }
    }

    spec fn contains(a: &[i64], r: i64) -> bool {
        contains_prefix(a, a.len(), r)
    }

    spec fn ge_all(a: &[i64], r: i64) -> bool {
        forall|j: usize| j < a.len() ==> #[trigger] a[j] <= r
    }

    spec fn max_upto(a: &[i64], k: usize, m: i64) -> bool {
        1 <= k && k <= a.len() && contains_prefix(a, k, m) && (forall|j: usize| j < k ==> #[trigger] a[j] <= m)
    }

    spec fn prefix_ok(a: &[i64], n: usize, r: i64) -> bool {
        n <= a.len() && contains_prefix(a, n, r)
    }

    spec fn has_index_below(a: &[i64], n: usize, r: i64) -> bool {
        !(forall|i: usize| i < n ==> #[trigger] a[i] != r)
    }

    #[uses(contains_prefix, has_index_below, prefix_ok)]
    proof fn contains_has_index(a: &[i64], n: usize, r: i64)
        requires prefix_ok(a, n, r),
        ensures has_index_below(a, n, r),
        decreases n,
    {
        if a[n - 1] != r {
            contains_has_index(a, n - 1, r);
        }
    }

    #[effects(pure)]
    #[uses(contains, contains_prefix, ge_all, has_index_below, max_upto, non_empty, prefix_ok)]
    fn max_of_list(lst: &[i64]) -> (r: i64)
        requires non_empty(lst),
        ensures {ENSURES},
    {
        let mut m: i64 = lst[0];
        for k in 1..lst.len()
            invariant max_upto(lst, k, m),
        {
            if lst[k] > m {
                m = lst[k];
            }
        }
        {PROOF}
        m
    }
}
"#;

/// The hidden contract check of the BVB judge for `vc-vv0039` (the text
/// `bench/m1/harness.py` `judge_target` writes), with `body` as the
/// check's body.
fn vv0039_judge(body: &str) -> String {
    let task =
        fs::read_to_string(root().join("../bench/bvb/tasks/vc-vv0039-max-of-list.json")).unwrap();
    let task = json::parse(&task).unwrap();
    let accept = task.get("accept").unwrap();
    let field = |k: &str| accept.get(k).unwrap().as_str().unwrap().to_string();
    let specs = [
        "contains",
        "contains_prefix",
        "ge_all",
        "has_index_below",
        "max_upto",
        "non_empty",
        "prefix_ok",
    ]
    .map(|s| format!("tasks::{s}"))
    .join(", ");
    format!(
        "#[summary(\"hidden acceptance check\")]\nmod bmb_accept {{\n{}\n\n#[effects(pure)]\n#[uses(ref_post, ref_pre, {specs})]\nfn check(lst: &[i64]) -> (r: i64)\n    requires {},\n    ensures {},\n{{\n{body}\n}}\n}}\n",
        field("specs"),
        field("pre"),
        field("post")
    )
}

/// ADR-0062 P5: the same-meaning contract the judge could not decide
/// (T1-14) is proved with a lemma, both where the agent states the
/// recursive form and where it proves the quantifier form itself.
#[test]
fn the_vv0039_equivalent_contract_is_proved_with_a_lemma() {
    let recursive = VV0039
        .replace("{ENSURES}", "contains(lst, r), ge_all(lst, r)")
        .replace("{PROOF}\n", "");
    let quantified = VV0039
        .replace(
            "{ENSURES}",
            "has_index_below(lst, lst.len(), r), ge_all(lst, r)",
        )
        .replace(
            "{PROOF}",
            "proof {\n            contains_has_index(lst, lst.len(), m);\n        }",
        );
    let run = |agent: &str, body: &str| {
        verify_files(&[
            ("ws.bmb".to_string(), agent.to_string()),
            ("accept.bmb".to_string(), vv0039_judge(body)),
        ])
    };
    let call = "    tasks::max_of_list(lst)";
    // The judge as it is: the implication from the recursive contract
    // needs an induction (the pilot's failure).
    let out = run(&recursive, call);
    assert!(!out.ok, "{}", out.output);
    assert!(
        out.output.contains("solver could not decide")
            && out.output.contains("in `bmb_accept::check`"),
        "{}",
        out.output
    );
    // The agent's lemma closes it at the check...
    let out = run(
        &recursive,
        "    let x: i64 = tasks::max_of_list(lst);\n    proof {\n        tasks::contains_has_index(lst, lst.len(), x);\n    }\n    x",
    );
    assert!(out.ok, "{}", out.output);
    // ... or in the agent's own proof, which then states the quantifier
    // form: the judge as it is accepts that.
    let out = run(&quantified, call);
    assert!(out.ok, "{}", out.output);
}

/// The values a counterexample in `msg` gives the `f64` inputs `names`.
fn f64_cex(msg: &str, names: &[&str]) -> Vec<f64> {
    let cex = msg
        .split("; counterexample: ")
        .nth(1)
        .unwrap_or_else(|| panic!("no counterexample: {msg}"));
    names
        .iter()
        .map(|n| {
            let v = cex
                .split(", ")
                .find_map(|kv| kv.strip_prefix(&format!("{n} = ")))
                .unwrap_or_else(|| panic!("no `{n}` in {msg}"));
            v.split('"')
                .next()
                .unwrap_or(v)
                .parse()
                .unwrap_or_else(|_| panic!("`{v}` is not an f64"))
        })
        .collect()
}

/// G1 for `f64` (ADR-0061 §5, backlog T2-03): outside `by (bit_vector)`
/// the operations are uninterpreted, so the solver's model may be no IEEE
/// arithmetic at all (`1.0 + 1.0 == 2.0` is satisfiable with `f64.add`
/// free; `x + y == y + x` first gets both zeros with `f64.eq` false). Such
/// a model is recomputed with Rust's `f64` arithmetic and not shown; a
/// counterexample comes only from a model IEEE arithmetic confirms, here
/// checked again by running the assertion on the shown values. The same
/// assertions show what is and is not proved where: `!(x < x)` holds in
/// IEEE arithmetic, is not proved in an ordinary query (the gate-power
/// mutant `v-f64-lt-interpreted` would prove it) and is proved by
/// `by (bit_vector)`.
#[test]
fn f64_counterexamples_are_confirmed_by_ieee_arithmetic() {
    let text = "#[summary(\"g1\")]
mod g {
    #[effects(pure)]
    fn two() -> () {
        let one: f64 = 1.0;
        assert(one + one == 2.0);
    }

    #[effects(pure)]
    fn commutes(x: f64, y: f64) -> () {
        assert(x + y == y + x);
    }

    #[effects(pure)]
    fn below(x: f64) -> () {
        assert(x * x >= 0.0);
    }

    #[effects(pure)]
    fn irreflexive(x: f64) -> () {
        assert(!(x < x));
    }

    #[effects(pure)]
    fn exactly(x: f64) -> () {
        assert(!(x < x)) by (bit_vector);
        assert(x * x >= 0.0 || x.is_nan()) by (bit_vector);
    }
}
";
    let out = verify_files(&[("g.bmb".to_string(), text.to_string())]);
    let lines: Vec<&str> = out.output.lines().collect();
    let diag = |def: &str| -> &str {
        lines
            .iter()
            .find(|l| l.contains(&format!("in `g::{def}`")))
            .unwrap_or_else(|| panic!("no diagnostic for {def}:\n{}", out.output))
    };
    // A true assertion: undecided, and no counterexample.
    for def in ["two", "irreflexive"] {
        let d = diag(def);
        assert!(
            d.contains("\"code\":\"E0811\"") && !d.contains("; counterexample:"),
            "{d}"
        );
        assert!(d.contains("it holds with IEEE 754 arithmetic"), "{d}");
    }
    // False ones: the shown values break the assertion in Rust.
    let v = f64_cex(diag("commutes"), &["x", "y"]);
    let (x, y) = (v[0], v[1]);
    assert!(x + y != y + x, "{x} {y}");
    let x = f64_cex(diag("below"), &["x"])[0];
    assert!(x * x < 0.0 || (x * x).is_nan(), "{x}");
    assert!(
        out.output
            .contains("{\"def\":\"g::exactly\",\"status\":\"verified\""),
        "{}",
        out.output
    );
}

/// `--bounded-search` (ADR-0108 §1, backlog 2D-33): an obligation the solver
/// cannot decide is asked once more with the inputs bounded. A model there
/// is a counterexample of the obligation itself (E0802 with the values);
/// none is the E0811 field `bounded`. Without the flag both stay E0811.
#[test]
fn bounded_search_turns_undecided_into_a_verdict() {
    let src = "#[summary(\"t\")]\nmod nb {\n    #[effects(pure)]\n    fn f(x: i64, y: i64, z: i64) -> i64\n        requires 0 < x && x < 1000 && 0 < y && y < 1000 && 0 < z && z < 1000,\n        ensures x * x * x + y * y * y != z * z * z,\n    {\n        0\n    }\n\n    #[effects(pure)]\n    fn g(x: i64, y: i64, z: i64) -> i64\n        requires 0 < x && x < 1000 && 0 < y && y < 1000 && 0 < z && z < 1000,\n        ensures x * x * x + y * y * y + 2 != z * z * z * x,\n    {\n        0\n    }\n}\n";
    let files = vec![("nb.bmb".to_string(), src.to_string())];
    let lines = |args: &[&str]| -> Vec<String> {
        let (ok, out) = cli::run(args, &files);
        assert!(!ok, "{out}");
        out.lines()
            .filter(|l| l.starts_with("{\"code\""))
            .map(String::from)
            .collect()
    };
    let plain = lines(&["verify"]);
    assert_eq!(plain.len(), 2, "{plain:?}");
    assert!(
        plain
            .iter()
            .all(|l| l.contains("\"code\":\"E0811\"") && !l.contains("\"bounded\"")),
        "{plain:?}"
    );
    let searched = lines(&["verify", "--bounded-search"]);
    assert_eq!(searched.len(), 2, "{searched:?}");
    let f = searched.iter().find(|l| l.contains("nb::f")).unwrap();
    assert!(
        f.contains("\"code\":\"E0811\"") && f.contains("\"bounded\":{\"b\":8,\"l\":4}"),
        "{f}"
    );
    let g = searched.iter().find(|l| l.contains("nb::g")).unwrap();
    assert!(
        g.contains("\"code\":\"E0802\"")
            && g.contains("counterexample: x = 5, y = 2, z = 3 (found by the search within"),
        "{g}"
    );
}
