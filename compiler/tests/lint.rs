//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08): unsatisfiable
//! preconditions (W0901), preconditions the proof does not need (W0902) and
//! postconditions no caller uses (W0903). Opt-in: `check`/`verify` are
//! unchanged.

use std::process::Command;

use bmb::lint::{MAX_PRE_TRIALS, lint_sources};

const SRC: &str = r#"#[summary("contract lint")]
mod m {
    spec fn pos(x: i64) -> bool { x > 0 }

    spec fn neg(x: i64) -> bool { x < 0 }

    spec fn small(x: i64) -> bool { x < 100 }

    spec fn is_succ(x: i64, r: i64) -> bool { r == x + 1 }

    spec fn same(x: i64, r: i64) -> bool { r == x }

    // `pos` and `neg` cannot hold together.
    #[effects(pure)]
    #[uses(pos, neg)]
    fn never(x: i64) -> i64
        requires pos(x), neg(x),
    {
        x
    }

    // `small` rules out overflow; `pos` is not needed.
    #[effects(pure)]
    #[uses(small, is_succ)]
    fn inc(x: i64) -> (r: i64)
        requires small(x), pos(x),
        ensures is_succ(x, r),
    {
        x + 1
    }

    // Uses `inc`'s postcondition (no overflow in `+ 1`); needs both
    // preconditions to call `inc`.
    #[effects(pure)]
    #[uses(small, is_succ)]
    fn add2(x: i64) -> i64
        requires small(x), pos(x),
    {
        inc(x) + 1
    }

    #[effects(pure)]
    #[uses(same)]
    fn id(x: i64) -> (r: i64)
        ensures same(x, r),
    {
        x
    }

    // Ignores `id`'s postcondition.
    #[effects(pure)]
    fn call_id(x: i64) -> i64 {
        id(x)
    }
}
"#;

const CLEAN: &str = r#"#[summary("clean contracts")]
mod c {
    spec fn small(x: i64) -> bool { x < 100 }

    spec fn is_succ(x: i64, r: i64) -> bool { r == x + 1 }

    #[effects(pure)]
    #[uses(small, is_succ)]
    fn inc(x: i64) -> (r: i64)
        requires small(x),
        ensures is_succ(x, r),
    {
        x + 1
    }

    #[effects(pure)]
    #[uses(small, is_succ)]
    fn add2(x: i64) -> i64
        requires small(x),
    {
        inc(x) + 1
    }
}
"#;

fn one(name: &str, text: &str) -> Vec<(String, String)> {
    vec![(name.to_string(), text.to_string())]
}

fn rows(jsonl: &str) -> Vec<bmb::json::Value> {
    jsonl
        .lines()
        .map(|l| bmb::json::parse(l).unwrap_or_else(|e| panic!("{e}: {l}")))
        .collect()
}

fn s<'a>(v: &'a bmb::json::Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

fn n(v: &bmb::json::Value, k: &str) -> i64 {
    v.get(k).and_then(|x| x.as_i64()).unwrap()
}

#[test]
fn each_report_and_nothing_else() {
    let out = lint_sources(&one("l.bmb", SRC)).unwrap();
    assert!(out.ok);
    let ds = rows(&out.diagnostics);
    let got: Vec<(String, i64, i64, String)> = ds
        .iter()
        .map(|d| {
            assert_eq!(s(d, "severity"), "warning", "{}", out.diagnostics);
            assert!(d.get("fix").is_some());
            (
                s(d, "code").to_string(),
                n(d, "line"),
                n(d, "col"),
                s(d, "message").to_string(),
            )
        })
        .collect();
    assert_eq!(got.len(), 3, "{}", out.diagnostics);

    // W0901 spans both clauses of `never`.
    assert_eq!((got[0].0.as_str(), got[0].1, got[0].2), ("W0901", 17, 18));
    assert_eq!(n(&ds[0], "end_col"), 32);
    assert!(got[0].3.contains("`m::never`"), "{}", got[0].3);

    // W0902 on `pos(x)` of `inc`, not on `small(x)`, and nothing on `add2`
    // (its clauses are needed to call `inc`) or on `never`.
    assert_eq!((got[1].0.as_str(), got[1].1, got[1].2), ("W0902", 26, 28));
    assert!(
        got[1]
            .3
            .starts_with("precondition `pos(x)` is not needed by this function's proof"),
        "{}",
        got[1].3
    );

    // W0903 on `id`'s postcondition; `inc`'s is used by `add2`.
    assert_eq!((got[2].0.as_str(), got[2].1, got[2].2), ("W0903", 45, 17));
    assert!(got[2].3.contains("`same(x, r)`"), "{}", got[2].3);
    assert!(got[2].3.contains("1 caller of `m::id`"), "{}", got[2].3);

    // inc: 2 clauses, add2: 2 clauses (W0902); inc and id ensures (W0903).
    assert_eq!(out.verifications, 6);
    assert_eq!(out.skipped, 0);
    assert_eq!(out.defs, 5);
    assert_eq!(out.reports, 3);
    let report = out.report();
    let last = rows(&report).pop().unwrap();
    assert_eq!(
        report.lines().last().unwrap(),
        "{\"summary\":true,\"defs\":5,\"reports\":3,\"verifications\":6,\"skipped\":0}"
    );
    assert!(matches!(
        last.get("summary"),
        Some(bmb::json::Value::Bool(true))
    ));
}

#[test]
fn clean_file_has_no_reports() {
    let out = lint_sources(&one("c.bmb", CLEAN)).unwrap();
    assert!(out.ok);
    assert_eq!(out.diagnostics, "", "{}", out.diagnostics);
    assert_eq!(out.reports, 0);
    // inc: 1 clause, add2: 1 clause, inc's ensures (caller add2).
    assert_eq!(out.verifications, 3);
}

#[test]
fn deterministic() {
    let a = lint_sources(&one("l.bmb", SRC)).unwrap().report();
    let b = lint_sources(&one("l.bmb", SRC)).unwrap().report();
    assert_eq!(a, b);
}

#[test]
fn check_and_verify_are_unchanged() {
    let files = one("l.bmb", SRC);
    let check = bmb::check_sources(&files);
    assert!(check.ok);
    assert_eq!(check.diagnostics, "");
    let verify = bmb::verify_sources(&files);
    assert!(!verify.output.contains("\"W09"), "{}", verify.output);
    assert!(!verify.output.contains("warning"), "{}", verify.output);
}

#[test]
fn re_verifications_are_capped() {
    // MAX_PRE_TRIALS + 2 removable clauses: the last two are skipped.
    let k = MAX_PRE_TRIALS + 2;
    let clauses: Vec<String> = (0..k).map(|_| "small(x)".to_string()).collect();
    let src = format!(
        "#[summary(\"cap\")]\nmod k {{\n    spec fn small(x: i64) -> bool {{ x < 100 }}\n\n    #[effects(pure)]\n    #[uses(small)]\n    fn f(x: i64) -> i64\n        requires {},\n    {{\n        x + 1\n    }}\n}}\n",
        clauses.join(", ")
    );
    let out = lint_sources(&one("k.bmb", &src)).unwrap();
    assert_eq!(out.verifications, MAX_PRE_TRIALS);
    assert_eq!(out.skipped, 2);
    // Each duplicate is removable on its own (another copy remains).
    assert_eq!(out.reports, MAX_PRE_TRIALS);
}

#[test]
fn front_end_errors_are_returned() {
    let err = lint_sources(&one("e.bmb", "mod m { }\n")).err().unwrap();
    assert!(err.contains("\"E0704\""), "{err}");
}

#[test]
fn cli_reports_and_exit_codes() {
    let dir = std::env::temp_dir().join(format!("bmb-lint-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let good = dir.join("l.bmb");
    std::fs::write(&good, SRC).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["lint", good.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let codes: Vec<String> = rows(&text)
        .iter()
        .filter_map(|r| r.get("code").and_then(|c| c.as_str()).map(String::from))
        .collect();
    assert_eq!(codes, ["W0901", "W0902", "W0903"], "{text}");
    assert!(text.ends_with("\"skipped\":0}\n"), "{text}");

    let bad = dir.join("e.bmb");
    std::fs::write(&bad, "mod m { }\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["lint", bad.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8(out.stdout).unwrap().contains("E0704"));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn codes_are_warnings_on_the_reference_page() {
    for c in ["W0901", "W0902", "W0903"] {
        assert_eq!(bmb::diag::severity(c), "warning");
        assert!(bmb::diag::codes_markdown().contains(&format!("| {c} |")));
    }
    assert_eq!(bmb::diag::severity("E0801"), "error");
}
