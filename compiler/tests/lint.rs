//! Contract lint (DESIGN.md §8.2 "계약 조정", backlog 2D-08): unsatisfiable
//! preconditions (W0901), preconditions the proof does not need (W0902) and
//! postconditions no caller uses (W0903). Opt-in: `check`/`verify` are
//! unchanged.

use std::process::Command;

use bmb::lint::{MAX_MUTANTS, MAX_PRE_TRIALS, lint_sources};

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
    // The satisfiability checks' cost follows (backlog T2-12): `rlimit`
    // depends on the pinned solver, nothing is undecided here.
    let line = report.lines().last().unwrap();
    assert!(
        line.starts_with(
            "{\"summary\":true,\"defs\":5,\"reports\":3,\"verifications\":6,\"skipped\":0,\"mutants\":2,\"surviving\":0,\"mutants_skipped\":0,\"pre_rlimit\":"
        ) && line.contains(",\"pre_undecided\":0,\"reach_rlimit\":")
            && line.ends_with(",\"reach_undecided\":0}"),
        "{line}"
    );
    assert!(out.pre_cost.is_some_and(|c| c.rlimit > 0));
    assert!(out.reach_cost.is_some_and(|c| c.rlimit > 0));
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
    #[uses(above_min, nonneg)]
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
    #[uses(is_sign)]
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
    // All on `abs` (line 14), none on the exactly specified `sign`.
    assert!(got.iter().all(|g| g.0 == "W0904" && g.1 == 14), "{got:?}");
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
        "    // Weak: says only that the result is not negative.\n    #[effects(pure)]\n    #[uses(above_min, nonneg)]\n    fn abs(x: i64) -> (r: i64)\n        requires above_min(x),\n        ensures nonneg(r),\n    {\n        if x < 0 { 0 - x } else { x }\n    }\n",
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
        "#[summary(\"cap\")]\nmod k {{\n    spec fn same(x: i64, r: i64) -> bool {{ r == x }}\n\n    #[effects(pure)]\n    #[uses(same)]\n    fn f(x: i64) -> (r: i64)\n        ensures same(x, r),\n    {{\n{lets}        x\n    }}\n}}\n"
    );
    let out = lint_sources(&one("k.bmb", &src)).unwrap();
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.mutants, MAX_MUTANTS);
    assert_eq!(out.mutants_skipped, 3);
    assert_eq!(out.surviving, MAX_MUTANTS);
    assert_eq!(out.reports, MAX_MUTANTS);
    // The first sites in body order are the ones examined.
    let first = rows(&out.diagnostics);
    assert_eq!(n(&first[0], "line"), 10);
    assert_eq!(
        n(&first[MAX_MUTANTS - 1], "line"),
        10 + MAX_MUTANTS as i64 - 1
    );
}

/// Backlog 2A-10: `bmb mutate` makes the W0904 mutants as programs.
#[test]
fn mutate_makes_every_lint_mutant_as_a_program() {
    let input = one("w.bmb", MUT);
    let defs = ["w::abs".to_string(), "w::sign".to_string()];
    let ms = bmb::lint::mutate_sources(&input, &defs).unwrap();
    // The same sites W0904 proves (16, see above), uncapped and in order.
    assert_eq!(ms.len(), 16);
    assert_eq!(ms.iter().filter(|m| m.def == "w::abs").count(), 5);
    assert_eq!((ms[0].def.as_str(), ms[0].site), ("w::abs", 0));
    assert_eq!((ms[5].def.as_str(), ms[5].site), ("w::sign", 0));
    let original = bmb::format_sources(&input).unwrap()["w"].clone();
    for m in &ms {
        assert_eq!(m.unit, "w");
        assert_ne!(m.text, original, "{} {}", m.before, m.after);
        // Each mutant is a program that passes the front end.
        let out = bmb::check_sources(&[("m.bmb".to_string(), m.text.clone())]);
        assert!(out.ok, "{}: {}", m.after, out.diagnostics);
    }
    // The surviving W0904 changes are among them, at the reported place.
    let survivor = ms.iter().find(|m| m.after == "1 - x").unwrap();
    assert_eq!(
        (survivor.line, survivor.col, survivor.before.as_str()),
        (14, 20, "0 - x")
    );
    assert!(survivor.text.contains("1 - x"));
    // Rows are JSON, and the result is deterministic.
    let again = bmb::lint::mutate_sources(&input, &defs).unwrap();
    let rows_of = |ms: &[bmb::lint::Mutant]| ms.iter().map(|m| m.row()).collect::<String>();
    assert_eq!(rows_of(&ms), rows_of(&again));
    assert_eq!(rows(&rows_of(&ms)).len(), 16);
    // Only executable functions are mutated.
    assert!(bmb::lint::mutate_sources(&input, &["w::nonneg".to_string()]).is_err());
    assert!(bmb::lint::mutate_sources(&input, &["w::missing".to_string()]).is_err());
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
    for c in ["W0901", "W0902", "W0903", "W0904", "W0905", "W0906"] {
        assert_eq!(bmb::diag::severity(c), "warning");
        assert!(bmb::diag::codes_markdown().contains(&format!("| {c} |")));
    }
    assert_eq!(bmb::diag::severity("E0801"), "error");
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
            let out = bmb::lint::lint_sources_cached(&input, Some(&dir)).unwrap();
            assert_eq!(out.report(), plain, "{name} {round}");
        }
    }
    assert!(dir.join(bmb::incremental::FILE).exists());
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
    #[uses(small, nonneg)]
    fn mix(x: i64, y: i64) -> (r: i64)
        requires small(x), small(y),
        ensures nonneg(r),
    {
        x * y + x / 3
    }

    spec fn is_and(a: bool, b: bool, r: bool) -> bool { r == (a && b) }

    // Exact: `&&` -> `||` is ruled out.
    #[effects(pure)]
    #[uses(is_and)]
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
    #[uses(has3_nonneg, nonneg)]
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
    #[uses(inside, is_at)]
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
    #[uses(shapes, bounded_elems, runs_below, sums_small, prefix_ok)]
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
    assert!(bmb::verify_sources(&one("w.bmb", text)).ok);
    assert!(bmb::verify_sources(&one("w.bmb", &multi)).ok);
    let out = lint_sources(&one("w.bmb", &multi)).unwrap();
    assert!(!out.diagnostics.contains("W0906"), "{}", out.diagnostics);
}
