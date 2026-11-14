//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08): unsatisfiable
//! preconditions (W0901), preconditions the proof does not need (W0902) and
//! postconditions no caller uses (W0903). Opt-in: `check`/`verify` are
//! unchanged.

mod common;

use std::path::Path;
use std::process::Command;

use common::cli;
use common::json::{self, Value};

// The caps `compiler/README.md` documents for `bmb lint`: re-verifications
// per check (W0902, W0903) and mutants per run (W0904).
const MAX_PRE_TRIALS: usize = 32;
const MAX_MUTANTS: usize = 64;

/// What `bmb lint` printed: the diagnostics, then the summary's counts.
struct Lint {
    ok: bool,
    diagnostics: String,
    report: String,
    defs: usize,
    reports: usize,
    verifications: usize,
    skipped: usize,
    mutants: usize,
    surviving: usize,
    mutants_skipped: usize,
}

impl Lint {
    fn report(&self) -> String {
        self.report.clone()
    }
}

/// `bmb lint` on `files`; the front end's diagnostics when it stops there.
fn lint_sources(files: &[(String, String)]) -> Result<Lint, String> {
    lint_args(&["lint"], files)
}

/// `bmb lint --cache DIR` on `files`.
fn lint_cached(files: &[(String, String)], dir: &Path) -> Result<Lint, String> {
    lint_args(&["lint", "--cache", dir.to_str().unwrap()], files)
}

fn lint_args(args: &[&str], files: &[(String, String)]) -> Result<Lint, String> {
    let (ok, report) = cli::run(args, files);
    let all = report.trim_end();
    let (diagnostics, summary) = all.rsplit_once('\n').unwrap_or(("", all));
    let summary = json::parse(summary).map_err(|_| report.clone())?;
    if summary.get("summary").is_none() {
        return Err(report);
    }
    let count = |k: &str| usize::try_from(n(&summary, k)).unwrap();
    let diagnostics = if diagnostics.is_empty() {
        String::new()
    } else {
        format!("{diagnostics}\n")
    };
    Ok(Lint {
        ok,
        defs: count("defs"),
        reports: count("reports"),
        verifications: count("verifications"),
        skipped: count("skipped"),
        mutants: count("mutants"),
        surviving: count("surviving"),
        mutants_skipped: count("mutants_skipped"),
        diagnostics,
        report,
    })
}

const SRC: &str = r#"#[summary("contract lint")]
mod m {
    spec fn pos(x: i64) -> bool { x > 0 }

    spec fn neg(x: i64) -> bool { x < 0 }

    spec fn small(x: i64) -> bool { x < 100 }

    spec fn is_succ(x: i64, r: i64) -> bool { r == x + 1 }

    spec fn same(x: i64, r: i64) -> bool { r == x }

    // `pos` and `neg` cannot hold together.
    #[effects(pure)]
    fn never(x: i64) -> i64
        requires pos(x), neg(x),
    {
        x
    }

    // `small` rules out overflow; `pos` is not needed.
    #[effects(pure)]
    fn inc(x: i64) -> (r: i64)
        requires small(x), pos(x),
        ensures is_succ(x, r),
    {
        x + 1
    }

    // Uses `inc`'s postcondition (no overflow in `+ 1`); needs both
    // preconditions to call `inc`.
    #[effects(pure)]
    fn add2(x: i64) -> i64
        requires small(x), pos(x),
    {
        inc(x) + 1
    }

    #[effects(pure)]
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
    fn inc(x: i64) -> (r: i64)
        requires small(x),
        ensures is_succ(x, r),
    {
        x + 1
    }

    #[effects(pure)]
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

fn rows(jsonl: &str) -> Vec<Value> {
    jsonl
        .lines()
        .map(|l| json::parse(l).unwrap_or_else(|e| panic!("{e}: {l}")))
        .collect()
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("")
}

fn n(v: &Value, k: &str) -> i64 {
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
    assert_eq!((got[0].0.as_str(), got[0].1, got[0].2), ("W0901", 16, 18));
    assert_eq!(n(&ds[0], "end_col"), 32);
    assert!(got[0].3.contains("`m::never`"), "{}", got[0].3);

    // W0902 on `pos(x)` of `inc`, not on `small(x)`, and nothing on `add2`
    // (its clauses are needed to call `inc`) or on `never`.
    assert_eq!((got[1].0.as_str(), got[1].1, got[1].2), ("W0902", 24, 28));
    assert!(
        got[1]
            .3
            .starts_with("precondition `pos(x)` is not needed by this function's proof"),
        "{}",
        got[1].3
    );

    // W0903 on `id`'s postcondition; `inc`'s is used by `add2`.
    assert_eq!((got[2].0.as_str(), got[2].1, got[2].2), ("W0903", 41, 17));
    assert!(got[2].3.contains("`same(x, r)`"), "{}", got[2].3);
    assert!(got[2].3.contains("1 caller of `m::id`"), "{}", got[2].3);

    // inc: 2 clauses, add2: 2 clauses (W0902); inc and id ensures (W0903).
    assert_eq!(out.verifications, 6);
    assert_eq!(out.skipped, 0);
    assert_eq!(out.defs, 5);
    assert_eq!(out.reports, 3);
    let report = out.report();
    let last = rows(&report).pop().unwrap();
    // The satisfiability checks' cost follows (backlog T2-12): `rlimit`
    // depends on the pinned solver, nothing is undecided here.
    let line = report.lines().last().unwrap();
    assert!(
        line.starts_with(
            "{\"summary\":true,\"defs\":5,\"reports\":3,\"verifications\":6,\"skipped\":0,\"mutants\":2,\"surviving\":0,\"mutants_skipped\":0,\"pre_rlimit\":"
        ) && line.contains(",\"pre_undecided\":0,\"reach_rlimit\":")
            && line.ends_with(",\"reach_undecided\":0,\"det_rlimit\":null,\"det_undecided\":null}"),
        "{line}"
    );
    for k in ["pre_rlimit", "reach_rlimit"] {
        assert!(last.get(k).and_then(|v| v.as_i64()).is_some_and(|r| r > 0));
    }
    assert!(matches!(last.get("summary"), Some(Value::Bool(true))));
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

/// An `#[external_body]` body is not proved: no clause it "does not need",
/// no mutant it "does not distinguish", no point it "cannot reach" (backlog
/// 2E-05, found writing `tests/run/collatz.bmb`).
#[test]
fn external_bodies_are_not_judged_by_their_proof() {
    let src = "#[summary(\"ext\")]\nmod x {\n    spec fn small(n: i64) -> bool { 1 <= n && n <= 10 }\n\n    spec fn low(r: i64) -> bool { 0 <= r && r <= 5 }\n\n    #[rationale(kind = boundary, hash = \"0000000000000000\", \"outside\")]\n    #[external_body]\n    #[effects(panic)]\n    fn f(n: i64) -> (r: i64)\n        requires small(n),\n        ensures low(r),\n    {\n        n + 1\n    }\n}\n";
    let out = lint_sources(&one("e.bmb", src)).unwrap();
    for code in ["W0902", "W0904", "W0905"] {
        assert!(
            !out.diagnostics.contains(code),
            "{code}:\n{}",
            out.diagnostics
        );
    }
    assert_eq!((out.mutants, out.surviving), (0, 0), "{}", out.diagnostics);
    // The rationale itself is still judged (W1001: its hash is not current).
    assert!(out.diagnostics.contains("W1001"), "{}", out.diagnostics);
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
    assert_eq!(cli::run(&["check"], &files), (true, String::new()));
    let (_, verify) = cli::run(&["verify"], &files);
    assert!(!verify.contains("\"W09"), "{verify}");
    assert!(!verify.contains("warning"), "{verify}");
}

#[test]
fn re_verifications_are_capped() {
    // MAX_PRE_TRIALS + 2 removable clauses: the last two are skipped.
    let k = MAX_PRE_TRIALS + 2;
    let clauses: Vec<String> = (0..k).map(|_| "small(x)".to_string()).collect();
    let src = format!(
        "#[summary(\"cap\")]\nmod k {{\n    spec fn small(x: i64) -> bool {{ x < 100 }}\n\n    #[effects(pure)]\n    fn f(x: i64) -> i64\n        requires {},\n    {{\n        x + 1\n    }}\n}}\n",
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
    assert!(
        text.contains("\"skipped\":0,\"mutants\":2,\"surviving\":0,\"mutants_skipped\":0,"),
        "{text}"
    );

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

// Backlog 2D-09: body mutations the specification does not distinguish.
const MUT: &str = r#"#[summary("weak and exact specifications")]
mod w {
    spec fn above_min(x: i64) -> bool { x > 0 - 9223372036854775807 }

    spec fn nonneg(r: i64) -> bool { r >= 0 }

    // Weak: says only that the result is not negative.
    #[effects(pure)]
    fn abs(x: i64) -> (r: i64)
        requires above_min(x),
        ensures nonneg(r),
    {
        if x < 0 { 0 - x } else { x }
    }

    spec fn is_sign(x: i64, r: i64) -> bool {
        (x > 0 ==> r == 1) && (x == 0 ==> r == 0) && (x < 0 ==> r == 0 - 1)
    }

    // Exact, and no change below is equivalent: none survives.
    #[effects(pure)]
    fn sign(x: i64) -> (r: i64)
        ensures is_sign(x, r),
    {
        if x > 0 { 1 } else { if x == 0 { 0 } else { 0 - 1 } }
    }
}
"#;

#[test]
fn weak_specification_lets_mutants_survive() {
    let out = lint_sources(&one("w.bmb", MUT)).unwrap();
    assert!(out.ok);
    let ds = rows(&out.diagnostics);
    let got: Vec<(String, i64, i64, String)> = ds
        .iter()
        .map(|d| {
            assert_eq!(s(d, "severity"), "warning");
            assert!(d.get("fix").is_some());
            (
                s(d, "code").to_string(),
                n(d, "line"),
                n(d, "col"),
                s(d, "message").to_string(),
            )
        })
        .collect();
    // All on `abs` (line 13), none on the exactly specified `sign`.
    assert!(got.iter().all(|g| g.0 == "W0904" && g.1 == 13), "{got:?}");
    assert!(got.iter().all(|g| g.3.contains("`w::abs` still verifies")));
    // `0 - x` -> `1 - x` is a real change `nonneg` does not see.
    assert!(
        got.iter().any(|g| g.2 == 20
            && g.3.starts_with(
                "the specification does not distinguish this change: `0 - x` -> `1 - x`"
            )),
        "{got:?}"
    );
    // `x < 0` -> `x <= 0` and `x < 1` also survive; they are equivalent
    // changes (at x == 0 both branches give 0), which the lint does not
    // decide, so the reports are informational.
    let msgs: Vec<&str> = got.iter().map(|g| g.3.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("`x < 0` -> `x <= 0`")));
    assert!(msgs.iter().any(|m| m.contains("`x < 0` -> `x < 1`")));
    assert_eq!(got.len(), 3, "{got:?}");
    // abs: if, `<`, `0`, `-`, `0`; sign: 2 ifs, `>`, `0`, `1`, `==`, `0`,
    // `0`, `-`, `0`, `1`.
    assert_eq!(out.mutants, 16);
    assert_eq!(out.surviving, 3);
    assert_eq!(out.mutants_skipped, 0);
    assert_eq!(out.reports, 3);
    assert!(
        out.report()
            .contains("\"skipped\":0,\"mutants\":16,\"surviving\":3,\"mutants_skipped\":0,")
    );
    // Deterministic.
    assert_eq!(
        out.report(),
        lint_sources(&one("w.bmb", MUT)).unwrap().report()
    );
}

#[test]
fn exact_specification_kills_every_mutant() {
    let exact = MUT.replace(
        "    // Weak: says only that the result is not negative.\n    #[effects(pure)]\n    fn abs(x: i64) -> (r: i64)\n        requires above_min(x),\n        ensures nonneg(r),\n    {\n        if x < 0 { 0 - x } else { x }\n    }\n",
        "",
    );
    assert_ne!(exact, MUT);
    let out = lint_sources(&one("s.bmb", &exact)).unwrap();
    assert_eq!(out.diagnostics, "", "{}", out.diagnostics);
    assert_eq!((out.mutants, out.surviving), (11, 0));
    // CLEAN: `inc`'s `x + 1` -> `x - 1` and `x + 2` are both ruled out.
    let out = lint_sources(&one("c.bmb", CLEAN)).unwrap();
    assert_eq!((out.mutants, out.surviving), (2, 0));
}

#[test]
fn mutants_are_capped() {
    // MAX_MUTANTS + 3 dead literals: each change survives, three are skipped.
    let k = MAX_MUTANTS + 3;
    let lets: String = (0..k)
        .map(|i| format!("        let v{i}: i64 = 5;\n"))
        .collect();
    let src = format!(
        "#[summary(\"cap\")]\nmod k {{\n    spec fn same(x: i64, r: i64) -> bool {{ r == x }}\n\n    #[effects(pure)]\n    fn f(x: i64) -> (r: i64)\n        ensures same(x, r),\n    {{\n{lets}        x\n    }}\n}}\n"
    );
    let out = lint_sources(&one("k.bmb", &src)).unwrap();
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.mutants, MAX_MUTANTS);
    assert_eq!(out.mutants_skipped, 3);
    assert_eq!(out.surviving, MAX_MUTANTS);
    assert_eq!(out.reports, MAX_MUTANTS);
    // The first sites in body order are the ones examined.
    let first = rows(&out.diagnostics);
    assert_eq!(n(&first[0], "line"), 9);
    assert_eq!(
        n(&first[MAX_MUTANTS - 1], "line"),
        9 + MAX_MUTANTS as i64 - 1
    );
}

/// Backlog 2A-10: `bmb mutate` makes the W0904 mutants as programs.
#[test]
fn mutate_makes_every_lint_mutant_as_a_program() {
    let input = one("w.bmb", MUT);
    let mutate = |defs: &[&str]| {
        let args: Vec<&str> = std::iter::once("mutate")
            .chain(defs.iter().flat_map(|d| ["--def", d]))
            .collect();
        cli::run(&args, &input)
    };
    let (ok, out) = mutate(&["w::abs", "w::sign"]);
    assert!(ok, "{out}");
    let mut ms = rows(&out);
    let summary = ms.pop().unwrap();
    // The same sites W0904 proves (16, see above), uncapped and in order.
    assert_eq!(n(&summary, "mutants"), 16);
    assert_eq!(ms.len(), 16);
    assert_eq!(ms.iter().filter(|m| s(m, "def") == "w::abs").count(), 5);
    assert_eq!((s(&ms[0], "def"), n(&ms[0], "site")), ("w::abs", 0));
    assert_eq!((s(&ms[5], "def"), n(&ms[5], "site")), ("w::sign", 0));
    let (_, normal) = cli::run(&["fmt"], &input);
    let original = cli::units(&normal).remove(0).1;
    for m in &ms {
        assert_eq!(s(m, "unit"), "w");
        let text = s(m, "text");
        assert_ne!(text, original, "{} {}", s(m, "before"), s(m, "after"));
        // Each mutant is a program that passes the front end.
        let (ok, diags) = cli::run(&["check"], &one("m.bmb", text));
        assert!(ok, "{}: {diags}", s(m, "after"));
    }
    // The surviving W0904 changes are among them, at the reported place.
    let survivor = ms.iter().find(|m| s(m, "after") == "1 - x").unwrap();
    assert_eq!(
        (
            n(survivor, "line"),
            n(survivor, "col"),
            s(survivor, "before")
        ),
        (13, 20, "0 - x")
    );
    assert!(s(survivor, "text").contains("1 - x"));
    // The result is deterministic.
    assert_eq!(mutate(&["w::abs", "w::sign"]), (true, out));
    // Only executable functions are mutated.
    assert!(!mutate(&["w::nonneg"]).0);
    assert!(!mutate(&["w::missing"]).0);
}

#[test]
fn mutate_and_all_checks_cli() {
    let dir = std::env::temp_dir().join(format!("bmb-mutate-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("w.bmb");
    std::fs::write(&file, MUT).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["mutate", "--def", "w::abs"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let rs = rows(&text);
    assert_eq!(rs.len(), 6);
    assert_eq!(
        text.lines().last().unwrap(),
        "{\"summary\":true,\"mutants\":5}"
    );
    assert_eq!(s(&rs[0], "def"), "w::abs");
    // No `--def`: usage error.
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("mutate")
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    // emit-llvm --all-checks: IR without running the prover; every
    // obligation keeps its runtime check.
    let ll = dir.join("w.ll");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["emit-llvm", "--all-checks"])
        .arg(&file)
        .arg("-o")
        .arg(&ll)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        !String::from_utf8(out.stdout)
            .unwrap()
            .contains("\"status\"")
    );
    let ir = std::fs::read_to_string(&ll).unwrap();
    assert!(ir.contains("unproved"), "{ir}");
    assert!(!ir.contains("; proved"), "{ir}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn codes_are_warnings_on_the_reference_page() {
    let (_, page) = cli::run(&["codes", "--md"], &[]);
    let (_, codes) = cli::run(&["codes"], &[]);
    let severity = |c: &str| {
        let row = rows(&codes)
            .into_iter()
            .find(|r| s(r, "code") == c)
            .unwrap();
        s(&row, "severity").to_string()
    };
    for c in ["W0901", "W0902", "W0903", "W0904", "W0905", "W0906"] {
        assert_eq!(severity(c), "warning");
        assert!(page.contains(&format!("| {c} |")));
    }
    assert_eq!(severity("E0801"), "error");
}

/// With `--cache DIR` (backlog 2D-15) the report is the same, cold and warm.
#[test]
fn cache_does_not_change_the_report() {
    let dir = std::env::temp_dir().join(format!("bmb-lint-cache-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (name, src) in [("l.bmb", SRC), ("w.bmb", MUT)] {
        let input = one(name, src);
        let plain = lint_sources(&input).unwrap().report();
        for round in ["cold", "warm"] {
            let out = lint_cached(&input, &dir).unwrap();
            assert_eq!(out.report(), plain, "{name} {round}");
        }
    }
    // The incremental store (`incremental::FILE`).
    assert!(dir.join("defs").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

// Backlog 2D-16: `*` -> `+`, `/` <-> `%`, `&&` <-> `||`, and an index shifted
// by one either way.
const OPS: &str = r#"#[summary("new mutation operators")]
mod o {
    spec fn small(x: i64) -> bool { x >= 0 && x < 1000 }

    spec fn nonneg(r: i64) -> bool { r >= 0 }

    // Weak: `*` -> `+` and `/` -> `%` keep the result non-negative.
    #[effects(pure)]
    fn mix(x: i64, y: i64) -> (r: i64)
        requires small(x), small(y),
        ensures nonneg(r),
    {
        x * y + x / 3
    }

    spec fn is_and(a: bool, b: bool, r: bool) -> bool { r == (a && b) }

    // Exact: `&&` -> `||` is ruled out.
    #[effects(pure)]
    fn both(a: bool, b: bool) -> (r: bool)
        ensures is_and(a, b, r),
    {
        a && b
    }

    spec fn has3_nonneg(a: &[i64]) -> bool {
        a.len() >= 3 && forall|i: usize| i < a.len() ==> #[trigger] a[i] >= 0
    }

    // Weak: says nothing about which element is read.
    #[effects(pure)]
    fn mid(a: &[i64]) -> (r: i64)
        requires has3_nonneg(a),
        ensures nonneg(r),
    {
        a[1]
    }

    spec fn inside(a: &[i64], i: usize) -> bool { i < a.len() }

    spec fn is_at(a: &[i64], i: usize, r: i64) -> bool { i < a.len() && r == a[i] }

    // Exact: the shifted reads are out of bounds or differ.
    #[effects(pure)]
    fn at(a: &[i64], i: usize) -> (r: i64)
        requires inside(a, i),
        ensures is_at(a, i, r),
    {
        a[i]
    }
}
"#;

#[test]
fn operator_and_index_mutants() {
    let out = lint_sources(&one("o.bmb", OPS)).unwrap();
    assert!(out.ok, "{}", out.diagnostics);
    let msgs: Vec<String> = rows(&out.diagnostics)
        .iter()
        .map(|d| {
            assert_eq!(s(d, "code"), "W0904");
            let m = s(d, "message");
            m[m.find('`').unwrap()..m.find(" (").unwrap()].to_string()
        })
        .collect();
    // mix: `+` -> `-` can go negative (killed); `*` -> `+`, `/` -> `%` and
    // `3` -> `4` keep the result non-negative. both: `&&` -> `||` is killed
    // by the exact postcondition. mid: both shifted reads stay in bounds
    // and non-negative, as does `1` -> `2`. at: `a[i + 1]` and `a[i - 1]`
    // are killed (bounds proof or postcondition).
    assert_eq!(
        msgs,
        [
            "`x * y` -> `x + y`",
            "`x / 3` -> `x % 3`",
            "`x / 3` -> `x / 4`",
            "`a[1]` -> `a[1 + 1]`",
            "`a[1]` -> `a[1 - 1]`",
            "`1` -> `2`",
        ]
    );
    // mix 4 sites, both 1, mid 3 (two index shifts and the literal), at 2.
    assert_eq!((out.mutants, out.surviving), (10, 6));
    assert_eq!(
        out.report(),
        lint_sources(&one("o.bmb", OPS)).unwrap().report()
    );
}

/// Backlog 1B-45: a `forall` whose only trigger matches another term of its
/// body at other arguments (a matching loop) is W0906; a multi-pattern, a
/// term without bound variables or another array is not.
#[test]
fn self_matching_triggers() {
    let text = r#"#[summary("triggers")]
mod t {
    spec fn up(a: &[i64]) -> bool { forall|k: usize| k + 1 < a.len() ==> #[trigger] a[k] <= a[k + 1] }

    spec fn down(a: &[i64]) -> bool { forall|k: usize| 0 < k && k < a.len() ==> #[trigger] a[k] == a[k - 1] }

    spec fn both(a: &[i64]) -> bool { forall|k: usize| k + 1 < a.len() ==> #[trigger] a[k] <= #[trigger] a[k + 1] }

    spec fn ground(a: &[i64]) -> bool { a.len() > 0 && (forall|k: usize| k < a.len() ==> #[trigger] a[k] >= a[0]) }

    spec fn other(a: &[i64], b: &[i64]) -> bool { a.len() == b.len() && (forall|k: usize| k + 1 < a.len() ==> #[trigger] a[k] == b[k + 1]) }

    spec fn sq(i: u64) -> u64 { i * i }

    spec fn grows(n: u64) -> bool { forall|k: u64| k < n ==> #[trigger] sq(k) <= sq(k + 1) }

    spec fn inner(a: &[i64], b: &[i64]) -> bool {
        forall|i: usize| i < a.len() ==> #[trigger] a[i] >= 0 && (forall|j: usize| j < i ==> #[trigger] b[j] <= b[j + 1])
    }
}
"#;
    let out = lint_sources(&one("t.bmb", text)).unwrap();
    assert!(out.ok, "{}", out.diagnostics);
    let got: Vec<(i64, String, String)> = rows(&out.diagnostics)
        .iter()
        .filter(|d| s(d, "code") == "W0906")
        .map(|d| {
            assert_eq!(s(d, "severity"), "warning");
            let fix = d.get("fix").unwrap();
            (
                n(d, "line"),
                s(d, "message").to_string(),
                s(fix, "replacement").to_string(),
            )
        })
        .collect();
    let lines: Vec<i64> = got.iter().map(|g| g.0).collect();
    assert_eq!(lines, [3, 5, 15, 18], "{}", out.diagnostics);
    assert!(got[0].1.starts_with("`a[k + 1]` matches"), "{}", got[0].1);
    assert!(got[0].1.contains("only trigger `a[k]`"), "{}", got[0].1);
    assert_eq!(got[0].2, "#[trigger] a[k + 1]");
    assert_eq!(got[1].2, "#[trigger] a[k - 1]");
    assert_eq!(got[2].2, "#[trigger] sq(k + 1)");
    assert_eq!(got[3].2, "#[trigger] b[j + 1]");

    // The replacement turns the loop into a multi-pattern: no report.
    let mut fixed = text.to_string();
    for (from, to) in [
        ("<= a[k + 1] }", "<= #[trigger] a[k + 1] }"),
        ("== a[k - 1] }", "== #[trigger] a[k - 1] }"),
        ("<= sq(k + 1) }", "<= #[trigger] sq(k + 1) }"),
        ("<= b[j + 1])", "<= #[trigger] b[j + 1])"),
    ] {
        assert_eq!(fixed.matches(from).count(), 1, "{from}");
        fixed = fixed.replace(from, to);
    }
    let out = lint_sources(&one("t.bmb", &fixed)).unwrap();
    assert!(!out.diagnostics.contains("W0906"), "{}", out.diagnostics);
}

/// The prefix-sum specification of bench m1b `w22` (the loop 1B-42 met) is
/// reported in each spec function that states the step; with both reads as
/// triggers the program still verifies.
#[test]
fn prefix_sum_specification_is_reported() {
    let text = r#"#[summary("prefix sums")]
mod tasks {
    spec fn shapes(a: &[i64], out: &[i64]) -> bool { out.len() == a.len() }

    spec fn bounded_elems(a: &[i64]) -> bool { a.len() <= 1000000 && (forall|k: usize| k < a.len() ==> -1000 <= #[trigger] a[k] && a[k] <= 1000) }

    spec fn runs_below(a: &[i64], out: &[i64], n: usize) -> bool { forall|k: usize| k + 1 < n ==> #[trigger] out[k + 1] == out[k] + a[k + 1] }

    spec fn sums_small(out: &[i64], n: usize) -> bool { forall|k: usize| k < n ==> -1000 * (k + 1) <= #[trigger] out[k] && out[k] <= 1000 * (k + 1) }

    spec fn prefix_ok(a: &[i64], out: &[i64]) -> bool { out.len() == a.len() && (a.len() > 0 ==> out[0] == a[0]) && (forall|k: usize| k + 1 < a.len() ==> #[trigger] out[k + 1] == out[k] + a[k + 1]) }

    #[effects(pure)]
    #[uses(runs_below, sums_small)]
    fn prefix_sums(a: &[i64], out: &mut [i64]) -> ()
        requires shapes(a, out), bounded_elems(a),
        ensures prefix_ok(a, out),
    {
        if a.len() > 0 {
            out[0] = a[0];
            let mut k: usize = 1;
            while k < a.len()
                invariant 1 <= k, k <= a.len(), out.len() == a.len(), out[0] == a[0], runs_below(a, out, k), sums_small(out, k),
                decreases a.len() - k,
            {
                out[k] = out[k - 1] + a[k];
                k = k + 1;
            }
        }
    }
}
"#;
    let out = lint_sources(&one("w.bmb", text)).unwrap();
    let lines: Vec<i64> = rows(&out.diagnostics)
        .iter()
        .filter(|d| s(d, "code") == "W0906")
        .map(|d| n(d, "line"))
        .collect();
    assert_eq!(lines, [7, 11], "{}", out.diagnostics);
    let multi = text.replace(
        "#[trigger] out[k + 1] == out[k] +",
        "#[trigger] out[k + 1] == #[trigger] out[k] +",
    );
    assert!(cli::run(&["verify"], &one("w.bmb", text)).0);
    assert!(cli::run(&["verify"], &one("w.bmb", &multi)).0);
    let out = lint_sources(&one("w.bmb", &multi)).unwrap();
    assert!(!out.diagnostics.contains("W0906"), "{}", out.diagnostics);
}

/// ADR-0075: the satisfiability checks retry an undecided query with ground
/// instances, specialized under quantifiers, and take its complete `sat`.
/// An index predicate used as a trigger left the precondition check of
/// `pick` and six reach points undecided before.
#[test]
fn index_predicates_leave_no_satisfiability_check_undecided() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/verify/pass/index_preds.bmb");
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["lint", "--select", "W0901,W0905", file.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let summary = rows(&text).pop().unwrap();
    assert_eq!(n(&summary, "reports"), 0, "{text}");
    assert_eq!(n(&summary, "pre_undecided"), 0, "{text}");
    assert_eq!(n(&summary, "reach_undecided"), 0, "{text}");
}

/// Backlog T2-13: a satisfiability check that the probe leaves undecided
/// writes the definitions its query reaches as recursive functions, which
/// the solver unfolds where a quantified recursive definition left its
/// model search `unknown`. `reach` needs a model of `count(n) == 3`;
/// `never` has none of `count(n) == 5` at `n == 2`, and is still reported.
#[test]
fn recursive_definitions_leave_no_satisfiability_check_undecided() {
    let text = "#[summary(\"t\")]
mod m {
    spec fn count(n: u64) -> u64
        decreases n,
    {
        if n == 0 { 0 } else { count(n - 1) + 1 }
    }

    #[effects(pure)]
    fn reach(n: u64) -> (r: u64)
        requires count(n) == 3,
        ensures r == n,
    {
        n
    }

    #[effects(pure)]
    fn never(n: u64) -> (r: u64)
        requires n == 2, count(n) == 5,
        ensures r == n,
    {
        n
    }
}
";
    let (_, out) = cli::run(&["lint", "--select", "W0901,W0905"], &one("r.bmb", text));
    let mut reports = rows(&out);
    let summary = reports.pop().unwrap();
    assert_eq!(n(&summary, "pre_undecided"), 0, "{out}");
    assert_eq!(n(&summary, "reach_undecided"), 0, "{out}");
    let got: Vec<(&str, bool)> = reports
        .iter()
        .map(|r| (s(r, "code"), s(r, "message").contains("m::never")))
        .collect();
    assert_eq!(got, [("W0901", true)], "{out}");

    // The recursive types and the proofs by induction of the corpus: four
    // checks of `recursive.bmb` and eleven of `proofs.bmb` were undecided.
    for (name, undecided) in [("recursive.bmb", 0), ("proofs.bmb", 1)] {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/verify/pass")
            .join(name);
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .args(["lint", "--select", "W0901,W0905", file.to_str().unwrap()])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
        let text = String::from_utf8(out.stdout).unwrap();
        let summary = rows(&text).pop().unwrap();
        assert_eq!(n(&summary, "reports"), 0, "{text}");
        assert_eq!(
            n(&summary, "pre_undecided") + n(&summary, "reach_undecided"),
            undecided,
            "{name}: {text}"
        );
    }
}

/// A line of `scripts/toolchain.env` (`KEY=value`).
fn pinned(key: &str) -> String {
    let env = Path::new(env!("CARGO_MANIFEST_DIR")).join("../scripts/toolchain.env");
    std::fs::read_to_string(env)
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")).map(str::to_string))
        .unwrap()
}

/// Backlog T2-16: the queries after the first verification are advisory and
/// their solver processes have the advisory memory limit. In the owned-tree
/// arm of the port-shape experiment a changed body or a dropped clause makes
/// an obligation false under the recursive predicate `spans_in` (a
/// quantifier inside it), and so do its reach points and two precondition
/// checks: Z3 neither refutes nor reaches its `rlimit` there, it grows. Each
/// such query ends at the limit and is undecided: an undecided mutant is not
/// a surviving one, an undecided check is counted. With the gate's limit for
/// these queries the same lint took 328 s.
#[test]
fn advisory_queries_end_at_their_own_memory_limit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let tree = root.join("dev-docs/experiments/port-shape/tree.bmb");
    let tmp = std::env::temp_dir().join(format!("bmb-lint-advisory-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let bin = tmp.join("bin");
    std::fs::create_dir_all(&bin).unwrap();

    // The pinned solver behind a script that writes down every start.
    let path = std::env::var("PATH").unwrap();
    let z3 = std::env::split_paths(&path)
        .map(|d| d.join("z3"))
        .find(|p| p.is_file())
        .expect("z3 must be installed");
    let log = tmp.join("starts");
    std::fs::write(
        bin.join("z3"),
        format!(
            "#!/bin/sh\n\
             if [ \"$1\" != --version ]; then echo \"$@\" >> {}; fi\n\
             exec {} \"$@\"\n",
            log.display(),
            z3.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(bin.join("z3"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let logged = format!("{}:{path}", bin.display());

    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .env("PATH", &logged)
        .arg("lint")
        .arg(&tree)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    let mut reports = rows(&text);
    let summary = reports.pop().unwrap();
    assert_eq!(
        (
            n(&summary, "mutants"),
            n(&summary, "surviving"),
            n(&summary, "mutants_skipped")
        ),
        (64, 13, 58),
        "{text}"
    );
    assert_eq!(n(&summary, "pre_undecided"), 1, "{text}");
    assert_eq!(n(&summary, "reach_undecided"), 6, "{text}");
    assert_eq!(
        reports.iter().filter(|r| s(r, "code") == "W0904").count(),
        13
    );

    // The first verification ran under the gate's limit, everything after
    // it under the advisory one: processes that answer one query after
    // another under half the limit, and each query that ended one asked
    // again alone under the limit itself (ADR-0129).
    let limit = |k: &str| pinned(k).parse::<u64>().unwrap();
    let (gate, advisory) = (limit("Z3_MEMORY_MB"), limit("Z3_ADVISORY_MEMORY_MB"));
    assert!(advisory < gate / 2);
    let of = |l: &str, mb: u64| {
        l.contains(&format!("-memory:{mb} ")) || l.ends_with(&format!("-memory:{mb}"))
    };
    let starts = std::fs::read_to_string(&log).unwrap();
    let starts: Vec<&str> = starts.lines().collect();
    let first = starts
        .iter()
        .take_while(|l| of(l, gate / 2) || of(l, gate))
        .count();
    assert!(first >= 1, "{starts:?}");
    assert!(
        starts[first..]
            .iter()
            .all(|l| of(l, advisory / 2) || of(l, advisory)),
        "{starts:?}"
    );
    assert!(
        starts[first..].iter().filter(|l| of(l, advisory)).count() > 40,
        "{starts:?}"
    );

    // One of the undecided mutants, `*pos = s + 2` in `term`: not reported,
    // and the gate does not decide it either (here at a smaller limit, the
    // sooner to reach it), so it is neither a survivor nor killed.
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["mutate", "--def", "tree::term"])
        .arg(&tree)
        .output()
        .unwrap();
    assert!(out.status.success());
    let mutants = rows(&String::from_utf8(out.stdout).unwrap());
    let m = mutants
        .iter()
        .find(|m| (s(m, "before"), s(m, "after")) == ("s + 1", "s + 2"))
        .unwrap();
    assert!(
        !reports.iter().any(|r| s(r, "code") == "W0904"
            && (n(r, "line"), n(r, "col")) == (n(m, "line"), n(m, "col"))),
        "{text}"
    );
    let file = tmp.join("tree.bmb");
    std::fs::write(&file, s(m, "text")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .env("BMB_Z3_MEMORY_MB", "64")
        .args(["verify", "--def", "tree::term"])
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let verdict = String::from_utf8(out.stdout).unwrap();
    assert!(
        verdict.contains("in `tree::term`: solver could not decide (memory limit reached, 64 MB)"),
        "{verdict}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// Backlog T2-17 (d): a proof in `--cache DIR` answers an advisory query
/// only when it was found under the advisory memory limit (the cache's key
/// covers the limit, ADR-0129), so the lint's output is the same with and
/// without a cache. Before, a proof the gate had stored for the same text
/// (here: of the program with the clause already removed) made the lint
/// report what it leaves undecided without the cache. The solver here ends
/// at once under the advisory limits and is the pinned one under the
/// gate's, so every advisory query is undecided unless the cache answers
/// it.
#[test]
fn a_cached_gate_proof_does_not_answer_an_advisory_query() {
    let tmp = std::env::temp_dir().join(format!("bmb-lint-cache-limit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let bin = tmp.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let with = "        requires x < 100, x < 1000,\n";
    let text = format!(
        "#[summary(\"더하기\")]\nmod m {{\n    #[effects(pure)]\n    fn add1(x: u64) -> (r: u64)\n{with}        ensures r == x + 1,\n    {{\n        x + 1\n    }}\n}}\n"
    );
    let file = tmp.join("p.bmb");
    std::fs::write(&file, &text).unwrap();
    let removed = tmp.join("removed");
    std::fs::create_dir_all(&removed).unwrap();
    std::fs::write(
        removed.join("p.bmb"),
        text.replace(with, "        requires x < 100,\n"),
    )
    .unwrap();
    let bmb = |args: &[&str], path: Option<&str>| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        if let Some(p) = path {
            cmd.env("PATH", p);
        }
        let out = cmd.args(args).output().unwrap();
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).unwrap()
    };
    let w0902 = |out: &str| rows(out).iter().filter(|r| s(r, "code") == "W0902").count();

    // With the pinned solver either clause is reported: the other one
    // keeps `x + 1` in range.
    let cache = tmp.join("cache");
    let (file, cache) = (file.to_str().unwrap(), cache.to_str().unwrap());
    assert_eq!(w0902(&bmb(&["lint", file], None)), 2);

    // The gate proves the program without `x < 1000` and stores its proofs.
    bmb(
        &[
            "verify",
            "--cache",
            cache,
            removed.join("p.bmb").to_str().unwrap(),
        ],
        None,
    );

    let path = std::env::var("PATH").unwrap();
    let z3 = std::env::split_paths(&path)
        .map(|d| d.join("z3"))
        .find(|p| p.is_file())
        .expect("z3 must be installed");
    let limit = pinned("Z3_ADVISORY_MEMORY_MB").parse::<u64>().unwrap();
    std::fs::write(
        bin.join("z3"),
        format!(
            "#!/bin/sh\n\
             case \" $* \" in *\" -memory:{} \"*|*\" -memory:{limit} \"*) exec {z3} -in -smt2 -memory:1;; esac\n\
             exec {z3} \"$@\"\n",
            limit / 2,
            z3 = z3.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(bin.join("z3"), std::fs::Permissions::from_mode(0o755)).unwrap();
    let starved = format!("{}:{path}", bin.display());
    let without = bmb(&["lint", file], Some(&starved));
    assert_eq!(w0902(&without), 0, "{without}");
    let cold = bmb(&["lint", "--cache", cache, file], Some(&starved));
    let warm = bmb(&["lint", "--cache", cache, file], Some(&starved));
    assert_eq!(cold, without);
    assert_eq!(warm, without);
    let _ = std::fs::remove_dir_all(&tmp);
}

/// The program of the two tests below: either `requires` clause can go
/// (the other keeps `x + 1` in range), so the lint re-verifies it twice.
const TWO_CLAUSES: &str = "#[summary(\"더하기\")]\nmod m {\n    #[effects(pure)]\n    fn add1(x: u64) -> (r: u64)\n        requires x < 100, x < 1000,\n        ensures r == x + 1,\n    {\n        x + 1\n    }\n}\n";

/// The pinned solver on `PATH`.
fn pinned_z3() -> (String, std::path::PathBuf) {
    let path = std::env::var("PATH").unwrap();
    let z3 = std::env::split_paths(&path)
        .map(|d| d.join("z3"))
        .find(|p| p.is_file())
        .expect("z3 must be installed");
    (path, z3)
}

fn executable(file: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(file, text).unwrap();
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// Runs `bmb args..` with the pinned solver behind a script that keeps
/// what each process was started with and sent (the filter ends with the
/// solver, at `(exit)`). Gives the output and, for the processes of the
/// gate (`[0]`) and the advisory ones (`[1]`, told apart by their memory
/// limit, ADR-0126): the `rlimit` of each query header, and every `rlimit`
/// sent.
type Limits = [(Vec<u64>, std::collections::BTreeSet<u64>); 2];
fn sent_limits(tmp: &Path, args: &[&str]) -> (String, Limits) {
    let (bin, sent) = (tmp.join("bin"), tmp.join("sent"));
    let _ = std::fs::remove_dir_all(&sent);
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&sent).unwrap();
    let (path, z3) = pinned_z3();
    executable(
        &bin.join("z3"),
        &format!(
            "#!/bin/sh\n\
             if [ \"$1\" = --version ]; then exec {z3} \"$@\"; fi\n\
             echo \"$@\" > {sent}/$$.args\n\
             sed -u -e 'w {sent}/'$$.in -e '/^(exit)$/q' | {z3} \"$@\"\n",
            z3 = z3.display(),
            sent = sent.display()
        ),
    );
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .env("PATH", format!("{}:{path}", bin.display()))
        .args(args)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let limit = |k: &str| pinned(k).parse::<u64>().unwrap();
    let (gate_mb, advisory_mb) = (limit("Z3_MEMORY_MB"), limit("Z3_ADVISORY_MEMORY_MB"));
    assert!(advisory_mb < gate_mb / 2);
    let mut limits: Limits = Default::default();
    for e in std::fs::read_dir(&sent).unwrap() {
        let args = e.unwrap().path();
        if args.extension().and_then(|x| x.to_str()) != Some("args") {
            continue;
        }
        let started = std::fs::read_to_string(&args).unwrap();
        let of = |mb: u64| format!("{} ", started.trim()).contains(&format!("-memory:{mb} "));
        let which = if of(advisory_mb) || of(advisory_mb / 2) {
            1
        } else {
            assert!(of(gate_mb) || of(gate_mb / 2), "{started}");
            0
        };
        let input = std::fs::read_to_string(args.with_extension("in")).unwrap();
        let lines: Vec<&str> = input.lines().collect();
        let value = |l: &str| {
            l.strip_prefix("(set-option :rlimit ")
                .and_then(|r| r.strip_suffix(')'))
                .map(|r| r.parse::<u64>().unwrap())
        };
        for (i, l) in lines.iter().enumerate() {
            if let Some(v) = value(l) {
                limits[which].1.insert(v);
                // The header: the limit follows the seeds.
                if lines[i - 1].starts_with("(set-option :sat.random_seed ") {
                    limits[which].0.push(v);
                }
            }
        }
    }
    (String::from_utf8(out.stdout).unwrap(), limits)
}

/// Backlog T2-18 (ADR-0131): the full queries after the lint's first
/// verification carry the advisory `rlimit` in their header, the first
/// verification's the gate's; a probe has the same number in both (a share
/// of the gate's limit). Read from what each solver process was sent. The
/// output is the same bytes with a cache, cold and warm.
#[test]
fn advisory_queries_run_under_their_own_rlimit() {
    let tmp = std::env::temp_dir().join(format!("bmb-lint-rlimit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("p.bmb");
    std::fs::write(&file, TWO_CLAUSES).unwrap();
    let (plain, [gates, advisories]) = sent_limits(&tmp, &["lint", file.to_str().unwrap()]);
    let w0902 = rows(&plain)
        .iter()
        .filter(|r| s(r, "code") == "W0902")
        .count();
    assert_eq!(w0902, 2, "{plain}");

    let limit = |k: &str| pinned(k).parse::<u64>().unwrap();
    let (gate, advisory) = (limit("Z3_RLIMIT"), limit("Z3_ADVISORY_RLIMIT"));
    assert!(advisory < gate);
    assert!(!gates.0.is_empty() && gates.0.iter().all(|v| *v == gate));
    assert!(
        advisories.0.len() >= 2 && advisories.0.iter().all(|v| *v == advisory),
        "{advisories:?}"
    );
    // No advisory process was sent the gate's limit and no process of the
    // gate the advisory one; an advisory query's probe (the precondition
    // check's) has the gate's share.
    assert!(!advisories.1.contains(&gate), "{advisories:?}");
    assert!(
        gates.1.iter().all(|v| [gate, gate / 100].contains(v)),
        "{gates:?}"
    );
    assert!(advisories.1.contains(&(gate / 100)), "{advisories:?}");

    let cache = tmp.join("cache");
    for round in ["cold", "warm"] {
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .args(["lint", "--cache"])
            .arg(&cache)
            .arg(&file)
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(out.stdout).unwrap(), plain, "{round}");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

/// ADR-0131: `bmb examples` reads its rows from models, and a model may
/// need more than the lint's advisory `rlimit`, so its queries keep the
/// gate's `rlimit` under the advisory memory limit: every process it starts
/// is an advisory one, and every header and limit it sends is the gate's
/// (or the probe's share of it).
#[test]
fn examples_queries_keep_the_gates_rlimit() {
    let tmp = std::env::temp_dir().join(format!("bmb-examples-rlimit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("p.bmb");
    std::fs::write(&file, TWO_CLAUSES).unwrap();
    let (out, [gates, advisories]) =
        sent_limits(&tmp, &["examples", "m::add1", "--", file.to_str().unwrap()]);
    assert!(
        rows(&out).iter().any(|r| s(r, "kind") == "allowed"),
        "{out}"
    );
    let limit = |k: &str| pinned(k).parse::<u64>().unwrap();
    let (gate, advisory) = (limit("Z3_RLIMIT"), limit("Z3_ADVISORY_RLIMIT"));
    assert!(gates.0.is_empty() && gates.1.is_empty(), "{gates:?}");
    assert!(
        !advisories.0.is_empty() && advisories.0.iter().all(|v| *v == gate),
        "{advisories:?}"
    );
    assert!(
        advisories.1.iter().all(|v| [gate, gate / 100].contains(v)),
        "{advisories:?}"
    );
    assert!(!advisories.1.contains(&advisory));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// Backlog T2-18 (ADR-0131): an advisory query that reaches the advisory
/// `rlimit` is undecided and nothing is reported for it, while the first
/// verification, under the gate's `rlimit`, proves the program; and a proof
/// the gate stored in `--cache DIR` does not answer the advisory query of
/// the same obligation (the limit is in the text the key is made of), so
/// the output is the same with and without the cache. The solver here is
/// the pinned one with the advisory limit and the probe's share rewritten
/// to 1 in what it is sent, and the memory limits are made equal
/// (`BMB_Z3_MEMORY_MB`), so only the `rlimit` tells the gate's queries from
/// the advisory ones.
#[test]
fn a_query_at_the_advisory_rlimit_is_undecided_with_and_without_a_cache() {
    let tmp = std::env::temp_dir().join(format!("bmb-lint-rlimit-end-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let bin = tmp.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let file = tmp.join("p.bmb");
    std::fs::write(&file, TWO_CLAUSES).unwrap();
    let removed = tmp.join("removed");
    std::fs::create_dir_all(&removed).unwrap();
    let without_clause = TWO_CLAUSES.replace("x < 100, x < 1000,", "x < 100,");
    assert_ne!(without_clause, TWO_CLAUSES);
    std::fs::write(removed.join("p.bmb"), without_clause).unwrap();
    let memory = pinned("Z3_ADVISORY_MEMORY_MB");
    let bmb = |args: &[&str], path: Option<&str>| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bmb"));
        cmd.env("BMB_Z3_MEMORY_MB", &memory);
        if let Some(p) = path {
            cmd.env("PATH", p);
        }
        let out = cmd.args(args).output().unwrap();
        assert!(out.status.success(), "{out:?}");
        String::from_utf8(out.stdout).unwrap()
    };
    let w0902 = |out: &str| rows(out).iter().filter(|r| s(r, "code") == "W0902").count();
    let cache = tmp.join("cache");
    let (file, cache) = (file.to_str().unwrap(), cache.to_str().unwrap());
    assert_eq!(w0902(&bmb(&["lint", file], None)), 2);

    // The gate proves the program without `x < 1000` and stores its proofs.
    bmb(
        &[
            "verify",
            "--cache",
            cache,
            removed.join("p.bmb").to_str().unwrap(),
        ],
        None,
    );

    let limit = |k: &str| pinned(k).parse::<u64>().unwrap();
    let (gate, advisory) = (limit("Z3_RLIMIT"), limit("Z3_ADVISORY_RLIMIT"));
    assert!(advisory != gate && gate / 100 != gate);
    let (path, z3) = pinned_z3();
    executable(
        &bin.join("z3"),
        &format!(
            "#!/bin/sh\n\
             if [ \"$1\" = --version ]; then exec {z3} \"$@\"; fi\n\
             sed -u -e 's/^(set-option :rlimit {advisory})$/(set-option :rlimit 1)/' \
             -e 's/^(set-option :rlimit {probe})$/(set-option :rlimit 1)/' \
             -e '/^(exit)$/q' | {z3} \"$@\"\n",
            z3 = z3.display(),
            probe = gate / 100
        ),
    );
    let starved = format!("{}:{path}", bin.display());
    let without = bmb(&["lint", file], Some(&starved));
    assert_eq!(w0902(&without), 0, "{without}");
    let summary = rows(&without).pop().unwrap();
    assert_eq!(n(&summary, "reports"), 0, "{without}");
    let cold = bmb(&["lint", "--cache", cache, file], Some(&starved));
    let warm = bmb(&["lint", "--cache", cache, file], Some(&starved));
    assert_eq!(cold, without);
    assert_eq!(warm, without);
    // The gate's own queries are not the advisory ones: it proves both
    // programs behind the same solver.
    bmb(&["verify", file], Some(&starved));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// The index shift (`a[i]` to `a[i + 1]`, `a[i - 1]`) is of an element's
/// index: a subslice `&b[lo..hi]` has no such mutant (`(lo..hi) + 1` is not
/// a program, and the lint's re-verification of it ended in the encoder's
/// `unreachable!`). Its bounds are sites of their own.
#[test]
fn a_subslice_has_no_index_shift() {
    const SUB: &str = "#[summary(\"부분 슬라이스\")]\nmod m {\n    #[effects(pure)]\n    fn len(b: &[u8]) -> (r: usize)\n        ensures r == b.len(),\n    {\n        b.len()\n    }\n\n    #[effects(pure)]\n    fn part(b: &[u8], lo: usize) -> (r: usize)\n        requires lo + 2 <= b.len(),\n        ensures r <= b.len(),\n    {\n        len(&b[lo..lo + 2])\n    }\n}\n";
    let tmp = std::env::temp_dir().join(format!("bmb-lint-subslice-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("p.bmb");
    std::fs::write(&file, SUB).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["mutate", "--def", "m::part"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let mut mutants = rows(&String::from_utf8(out.stdout).unwrap());
    mutants.retain(|m| m.get("text").is_some());
    let changes: Vec<(&str, &str)> = mutants
        .iter()
        .map(|m| (s(m, "before"), s(m, "after")))
        .collect();
    // The bounds: `+` to `-`, and the literal.
    assert_eq!(
        changes,
        [("lo + 2", "lo - 2"), ("lo + 2", "lo + 3")],
        "{changes:?}"
    );
    // Every mutant is a program the checker accepts.
    for m in &mutants {
        let mutant = tmp.join("mutant.bmb");
        std::fs::write(&mutant, s(m, "text")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("check")
            .arg(&mutant)
            .output()
            .unwrap();
        assert!(out.status.success(), "{out:?}");
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("lint")
        .arg(&file)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    let summary = rows(&text).pop().unwrap();
    assert_eq!(n(&summary, "mutants"), 2, "{text}");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// `bmb lint` ends with a report over every file of the proved corpus and
/// the standard library: exit status 0 or 1, never a panic (a mutant or a
/// changed clause the encoder does not expect), and every line of its
/// output is a JSON object, the last one the summary. Through the binary,
/// as a panic there is an exit status. A subslice's index mutant went
/// unseen in `tests/verify/pass/subslices.bmb` until the lint was run over
/// that file by hand.
#[test]
fn lint_ends_with_a_report_over_the_corpora() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut paths = Vec::new();
    for dir in ["tests/verify/pass", "lib/std"] {
        let mut found: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        found.sort();
        paths.extend(found);
    }
    assert!(paths.len() > 60, "{}", paths.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failures = std::sync::Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map_or(2, |n| n.get());
    std::thread::scope(|sc| {
        for _ in 0..workers {
            sc.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(p) = paths.get(k) else { break };
                    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
                        .arg("lint")
                        .arg(p)
                        .output()
                        .unwrap();
                    let text = String::from_utf8_lossy(&out.stdout).into_owned();
                    let lines: Vec<&str> = text.lines().collect();
                    let well_formed = lines
                        .iter()
                        .all(|l| matches!(json::parse(l), Ok(Value::Object(_))))
                        && lines.last().is_some_and(|l| l.contains("\"summary\":true"));
                    if !matches!(out.status.code(), Some(0 | 1)) || !well_formed {
                        failures.lock().unwrap().push(format!(
                            "{}: {:?}\n{}",
                            p.display(),
                            out.status,
                            String::from_utf8_lossy(&out.stderr)
                        ));
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// W0909 (ADR-0030 §3, backlog 2F-02): a declared effect the body does not
/// need, from its own operations or its callees' declared effects. Not for
/// `main`'s `io`, a trait's method or an `impl`'s function of a trait. The
/// fix writes what the body needs.
#[test]
fn unneeded_effects() {
    let text = r#"#[summary("effects")]
mod w {
    #[effects(alloc, io)]
    fn wide(x: i64) -> i64 { x }

    #[effects(alloc)]
    fn grows() -> usize {
        let mut v: Vec<i64> = Vec::new();
        v.push(1);
        v.len()
    }

    #[effects(alloc, io)]
    fn calls() -> usize {
        grows()
    }

    trait Shape {
        #[effects(io)]
        fn show(self: &Self) -> ();
    }

    struct Sq { s: i64 }

    impl Shape for Sq {
        #[effects(io)]
        fn show(self: &Sq) -> () {}
    }

    #[effects(alloc, io)]
    fn main() -> () {}
}
"#;
    let out = lint_sources(&one("w.bmb", text)).unwrap();
    let found: Vec<(i64, String)> = rows(&out.diagnostics)
        .iter()
        .filter(|d| s(d, "code") == "W0909")
        .map(|d| {
            let fix = d
                .get("fix")
                .and_then(|f| f.get("replacement"))
                .and_then(|r| r.as_str());
            (n(d, "line"), fix.unwrap_or_default().to_string())
        })
        .collect();
    assert_eq!(
        found,
        [
            (3, "#[effects(pure)]".to_string()),
            (13, "#[effects(alloc)]".to_string()),
            (30, "#[effects(io)]".to_string()),
        ],
        "{}",
        out.diagnostics
    );
}

/// W0910 (backlog 2D-30, from T0-34): a printing loop of an `io` function
/// that decides and computes its state itself (the agents of T0-30 wrote
/// the ledger this way) gets the suggestion to move the update into a pure
/// function: the state, the loop's own inputs and the value it prints. The
/// same loop over an extracted step, with a counter and a sentinel test,
/// gets none (the BRB-EFFECT reference solutions get none).
#[test]
fn pure_state_in_printing_loops() {
    let text = r#"#[summary("w0910")]
mod w {
    #[effects(io)]
    fn ledger(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
        let mut bal: i64 = start % 1000;
        let mut k: usize = 0;
        while k < kinds.len()
            decreases kinds.len() - k,
        {
            let kind: i64 = kinds[k] % 4;
            let amt: i64 = amounts[k] % 1000;
            let mut out: i64 = -1;
            if kind == 0 && amt >= 0 {
                bal = bal % 1000 + amt;
                out = bal;
            }
            print_i64(out);
            k = k + 1;
        }
        bal
    }

    #[effects(pure)]
    fn step(bal: i64, kind: i64, amt: i64) -> i64 {
        if kind == 0 && amt >= 0 { bal % 1000 + amt % 1000 } else { -1 }
    }

    #[effects(io)]
    fn shell(kinds: &[i64], amounts: &[i64], start: i64) -> i64 {
        let mut bal: i64 = start % 1000;
        let mut k: usize = 0;
        while k < kinds.len()
            decreases kinds.len() - k,
        {
            let n: i64 = step(bal, kinds[k] % 4, amounts[k]);
            if n < 0 {
                print_i64(-1);
            } else {
                bal = n;
                print_i64(bal);
            }
            k = k + 1;
        }
        bal
    }

    #[effects(io)]
    fn main() -> () {}
}
"#;
    let out = lint_sources(&one("w.bmb", text)).unwrap();
    let found: Vec<(i64, String, String)> = rows(&out.diagnostics)
        .iter()
        .filter(|d| s(d, "code") == "W0910")
        .map(|d| {
            let fix = d
                .get("fix")
                .and_then(|f| f.get("message"))
                .and_then(|r| r.as_str());
            (
                n(d, "line"),
                s(d, "message").to_string(),
                fix.unwrap_or_default().to_string(),
            )
        })
        .collect();
    assert_eq!(found.len(), 1, "{}", out.diagnostics);
    assert_eq!(found[0].0, 7);
    assert!(found[0].1.contains("its state `bal`"), "{}", found[0].1);
    assert!(
        found[0]
            .2
            .contains("fn ledger_step(bal: i64, kind: i64, amt: i64) -> (i64, i64)"),
        "{}",
        found[0].2
    );
}

/// W0918 (backlog T1-44a): asked by `--select`, a contract that fixes its
/// result says nothing; one that bounds it shows an input and two results
/// its postconditions both allow; a clause through a spec fn seen by name
/// only (`huge`, which `big`'s body names, ADR-0096) is not judged (counted
/// undecided), nor a function with a `&mut` parameter. A plain
/// run does not ask.
#[test]
fn determinacy_shows_two_results_on_request() {
    let src = r#"#[summary("t")]
mod d {
    spec fn big(r: i64) -> bool { huge(r) }

    spec fn huge(r: i64) -> bool { r > 10 }

    #[effects(pure)]
    fn twice(x: i64) -> (r: i64)
        requires x < 1000 && x > -1000,
        ensures r == 2 * x,
    {
        2 * x
    }

    #[effects(pure)]
    fn above(x: i64) -> (r: i64)
        requires x < 1000,
        ensures r >= x,
    {
        x + 1
    }

    #[effects(pure)]
    fn opaque() -> (r: i64)
        ensures big(r),
    {
        11
    }

    #[effects(pure)]
    fn bump(a: &mut i64) -> (r: i64)
        requires *a < 1000,
        ensures r >= 0,
    {
        *a = *a + 1;
        0
    }
}
"#;
    let files = one("d.bmb", src);
    let out = lint_args(&["lint", "--select", "W0918"], &files).unwrap();
    let ds = rows(&out.diagnostics);
    assert_eq!(ds.len(), 1, "{}", out.diagnostics);
    assert_eq!(s(&ds[0], "code"), "W0918");
    let msg = s(&ds[0], "message");
    assert!(
        msg.starts_with("the postconditions of `d::above` allow two results for one input at x = "),
        "{msg}"
    );
    assert!(
        msg.contains(" and r = ") && msg.ends_with(" both hold"),
        "{msg}"
    );
    assert_eq!((n(&ds[0], "line"), n(&ds[0], "col")), (18, 17));
    let last = rows(&out.report()).pop().unwrap();
    assert_eq!(n(&last, "det_undecided"), 1, "{}", out.report());
    let plain = lint_sources(&files).unwrap();
    assert!(
        !plain.diagnostics.contains("W0918"),
        "{}",
        plain.diagnostics
    );
}

/// W0918 over the corpora (backlog T1-44a): every program of `check/ok`,
/// `verify/pass` and `run` lints with `--select W0918` to its summary line
/// and exit status 0 (no panic, no failure hidden by a report count).
#[test]
fn determinacy_runs_over_the_corpora() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut n = 0;
    for dir in ["check/ok", "verify/pass", "run"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        paths.sort();
        for p in paths {
            let files = one("p.bmb", &std::fs::read_to_string(&p).unwrap());
            let out = lint_args(&["lint", "--select", "W0918"], &files)
                .unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert!(out.ok, "{}", p.display());
            n += 1;
        }
    }
    assert!(n > 100, "{n}");
}
