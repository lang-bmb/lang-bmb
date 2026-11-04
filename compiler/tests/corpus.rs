//! Regression corpus for the front end.
//!
//! - `tests/check/ok/*.bmb` must produce no diagnostics.
//! - `tests/check/err/*.bmb` must produce exactly the diagnostics in the sibling `.jsonl`.
//!
//! Set `BMB_BLESS=1` to rewrite the expected `.jsonl` files.
//!
//! Through the CLI (`bmb check`, `bmb codes`), so the corpus tests whichever
//! compiler is built (backlog 4A-04).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bmb(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(args)
        .current_dir(root())
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// What `bmb check` says about a corpus file.
struct CheckOutput {
    ok: bool,
    diagnostics: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("check")
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

fn run(path: &Path) -> CheckOutput {
    let name = format!(
        "{}/{}",
        path.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
        path.file_name().unwrap().to_str().unwrap()
    );
    let (ok, diagnostics) = bmb(&["check", &name]);
    CheckOutput { ok, diagnostics }
}

#[test]
fn ok_corpus_has_no_diagnostics() {
    for p in bmb_files(&root().join("ok")) {
        let out = run(&p);
        assert!(out.ok, "{} should pass:\n{}", p.display(), out.diagnostics);
    }
}

#[test]
fn err_corpus_matches_expected() {
    let bless = std::env::var("BMB_BLESS").is_ok();
    let mut failures = Vec::new();
    for p in bmb_files(&root().join("err")) {
        let out = run(&p);
        assert!(!out.ok, "{} should fail", p.display());
        let expected_path = p.with_extension("jsonl");
        if bless {
            fs::write(&expected_path, &out.diagnostics).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&expected_path).unwrap_or_default();
        if expected != out.diagnostics {
            failures.push(format!(
                "{}\n--- expected\n{expected}--- actual\n{}",
                p.display(),
                out.diagnostics
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn output_is_deterministic() {
    for dir in ["ok", "err"] {
        for p in bmb_files(&root().join(dir)) {
            assert_eq!(run(&p).diagnostics, run(&p).diagnostics, "{}", p.display());
        }
    }
}

#[test]
fn every_code_is_documented_once() {
    let mut seen = std::collections::BTreeSet::new();
    let (ok, page) = bmb(&["codes", "--md"]);
    assert!(ok);
    let (ok, codes) = bmb(&["codes"]);
    assert!(ok);
    for row in codes.lines() {
        let field = |k: &str| {
            let at = row.find(&format!("\"{k}\":\"")).unwrap() + k.len() + 4;
            row[at..].split('"').next().unwrap().to_string()
        };
        let (c, d) = (field("code"), field("description"));
        assert!(seen.insert(c.clone()), "duplicate code {c}");
        assert!(!d.is_empty());
        assert!(
            page.contains(&format!("| {c} |")),
            "{c} missing from the reference page"
        );
    }
}
