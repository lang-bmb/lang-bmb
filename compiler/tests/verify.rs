//! Proof gate corpus (backlog 1B-12).
//! - `tests/check/ok` and `tests/verify/pass`: every definition is proved.
//! - `tests/verify/fail`: output matches the sibling `.jsonl`
//!   (`BMB_BLESS=1` rewrites it), and every non-helper definition fails.

use std::fs;
use std::path::{Path, PathBuf};

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

fn verify(p: &Path) -> bmb::VerifyOutput {
    bmb::verify_sources(&[(name(p), fs::read_to_string(p).unwrap())])
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
    // The libraries, each alone and all of them with each program that uses them.
    let libs: Vec<(String, String)> = bmb_files(&bench.join("..").join("lib"))
        .iter()
        .map(|p| (name(p), fs::read_to_string(p).unwrap()))
        .collect();
    for lib in &libs {
        programs.push((lib.0.clone(), vec![lib.clone()]));
    }
    for p in bmb_files(&root().join("lib_run")) {
        let mut files = vec![(name(&p), fs::read_to_string(&p).unwrap())];
        files.extend(libs.iter().cloned());
        programs.push((name(&p), files));
    }
    for (label, files) in programs {
        let points = bmb::vacuous_points(&files);
        assert!(points.is_empty(), "{label}: {points:?}");
    }
}

/// The Benchmarks Game programs (`bench/game`, backlog T3-01) are proved with
/// the libraries. Running them takes tens of seconds, so the gate proves them
/// and `bench/game/README.md` runs them.
#[test]
fn game_programs_are_proved() {
    let repo = root().join("..");
    let libs: Vec<(String, String)> = bmb_files(&repo.join("lib"))
        .iter()
        .map(|p| (name(p), fs::read_to_string(p).unwrap()))
        .collect();
    let games = bmb_files(&repo.join("bench").join("game"));
    assert!(games.len() >= 2);
    for p in games {
        let mut files = vec![(name(&p), fs::read_to_string(&p).unwrap())];
        files.extend(libs.iter().cloned());
        let out = bmb::verify_sources(&files);
        assert!(out.ok, "{}: {}", p.display(), out.output);
    }
}

/// Every library (`lib/`: bytes, ADR-0044 §4; UTF-8, CRC-32, SHA-256, base64,
/// T4) is proved completely, each on its own.
#[test]
fn libraries_are_proved() {
    let libs = bmb_files(&root().join("..").join("lib"));
    assert!(libs.len() >= 5);
    for p in libs {
        let out = bmb::verify_sources(&[(name(&p), fs::read_to_string(&p).unwrap())]);
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

#[test]
fn queries_and_output_are_deterministic() {
    let mut files = bmb_files(&root().join("check").join("ok"));
    files.extend(bmb_files(&root().join("verify").join("fail")));
    for p in files {
        let (a, b) = (verify(&p), verify(&p));
        assert_eq!(a.scripts, b.scripts, "{}", p.display());
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
    let out = bmb::verify_sources(&files);
    assert!(out.ok, "seed workspace should verify:\n{}", out.output);
}

#[test]
fn hidden_predicate_hint_leads_to_a_proof() {
    let src = r#"
#[summary("힌트")]
mod h {
    spec fn small(x: i64) -> bool { x < 100 }

    spec fn bigger(x: i64, r: i64) -> bool { r > x }

    #[effects(pure)]
    #[uses(small, bigger)]
    fn inc(x: i64) -> (r: i64)
        requires small(x),
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
    let run = |uses: &str| bmb::verify_sources(&[("h.bmb".into(), src.replace("USES", uses))]);
    let first = run("bigger");
    assert!(!first.ok);
    // The precondition `small(5)` fails; the hint names `small` and nothing else.
    let hint = first
        .output
        .lines()
        .find(|l| l.contains("\"code\":\"E0801\""))
        .expect("precondition failure");
    assert!(hint.contains("add to #[uses(small)]"), "{hint}");
    assert!(run("bigger, small").ok);
}

#[test]
fn nonlinear_detection() {
    use bmb::verify::has_nonlinear_product;
    assert!(has_nonlinear_product("(assert (= r (* x y)))"));
    assert!(has_nonlinear_product("(assert (> (+ 1 (* x x x)) 0))"));
    assert!(!has_nonlinear_product("(assert (= r (* 2 x)))"));
    assert!(!has_nonlinear_product("(assert (= r (* x (- 3))))"));
    assert!(!has_nonlinear_product("(assert (|f| x y))"));
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
            let out = bmb::verify_sources_opts(&files, None, false);
            assert!(out.ok, "{}", out.output);
            out.scripts["t::f"].clone()
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
            let out = bmb::verify_sources_opts(&files, None, false);
            assert!(out.ok, "{}", out.output);
            out.scripts["t::f"].clone()
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
    let task = bmb::json::parse(&task).unwrap();
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
        bmb::verify_sources(&[
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
