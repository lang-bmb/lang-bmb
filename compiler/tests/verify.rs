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
