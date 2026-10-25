//! `bmb fix` (backlog 2D-04): applies every mechanical fix, re-checks and
//! repeats until none applies.

use std::process::Command;

const HABITS: &str = r#"#[summary("Rust habits")]
mod m {
    #[effects(pure)]
    pub fn add(x: i32, y: i64) -> i64 {
        let z = x + 1i64;
        z + y
    }

    #[effects(pure)]
    fn first(a: &'a [i64]) -> i64 {
        let v: [i64; 2] = vec![1, 2];
        v[0]
    }

    #[effects(io)]
    fn main() {
        let n = 3;
        print_i64(add(n, 2));
    }
}
"#;

const FIXED: &str = r#"#[summary("Rust habits")]
mod m {
    #[effects(pure)]
     fn add(x: i64, y: i64) -> i64 {
        let z: i64 = x + 1;
        z + y
    }

    #[effects(pure)]
    fn first(a: & [i64]) -> i64 {
        let v: [i64; 2] = [1, 2];
        v[0]
    }

    #[effects(io)]
    fn main() -> () {
        let n: i64 = 3;
        print_i64(add(n, 2));
    }
}
"#;

fn one(text: &str) -> Vec<(String, String)> {
    vec![("t.bmb".to_string(), text.to_string())]
}

#[test]
fn rust_habits_are_fixed_over_rounds() {
    let input = one(HABITS);
    assert!(!bmb::check_sources(&input).ok);
    let out = bmb::fix_sources(&input);
    assert_eq!(out.files[0].1, FIXED);
    assert!(out.ok, "{}", out.diagnostics);
    assert_eq!(out.diagnostics, "");
    assert!(bmb::check_sources(&out.files).ok);
    // `let z` gets its type only once `x` is `i64`: a second round.
    assert_eq!(out.rounds, 2);
    let mut codes: Vec<&str> = out.applied.iter().map(|a| a.code).collect();
    codes.sort();
    assert_eq!(
        codes,
        [
            "E0104", "E0201", "E0201", "E0202", "E0207", "E0208", "E0209", "E0402"
        ]
    );
    let report = out.report(&input);
    assert!(
        report.ends_with(
            "{\"summary\":true,\"rounds\":2,\"applied\":8,\"changed\":[\"t.bmb\"],\"remaining\":0,\"ok\":true}\n"
        ),
        "{report}"
    );
    // Deterministic.
    assert_eq!(bmb::fix_sources(&input).report(&input), report);
}

#[test]
fn fix_is_idempotent() {
    for text in [FIXED, HABITS] {
        let first = bmb::fix_sources(&one(text));
        let again = bmb::fix_sources(&first.files);
        assert_eq!(again.files, first.files);
        assert!(again.applied.is_empty());
        assert_eq!(again.rounds, 0);
        assert_eq!(
            again.report(&first.files),
            "{\"summary\":true,\"rounds\":0,\"applied\":0,\"changed\":[],\"remaining\":0,\"ok\":true}\n"
        );
    }
}

#[test]
fn non_mechanical_errors_remain() {
    // `return` has no mechanical fix; the suffix does.
    let text = "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        return 5i64;\n    }\n}\n";
    let out = bmb::fix_sources(&one(text));
    assert!(!out.ok);
    assert!(out.files[0].1.contains("return 5;"), "{}", out.files[0].1);
    assert_eq!(out.applied.len(), 1);
    assert!(out.diagnostics.contains("\"code\":\"E0203\""));
    assert!(!out.diagnostics.contains("E0104"));
}

#[test]
fn fixes_apply_per_file_in_a_multi_file_program() {
    let files = vec![
        (
            "a.bmb".to_string(),
            "#[summary(\"a\")]\nmod a {\n    #[effects(pure)]\n    fn f(x: i32) -> i64 { x }\n}\n".to_string(),
        ),
        (
            "b.bmb".to_string(),
            "#[summary(\"b\")]\nmod b {\n    #[effects(io)]\n    fn main() -> () { print_i64(a::f(7i64)); }\n}\n".to_string(),
        ),
    ];
    let out = bmb::fix_sources(&files);
    assert!(out.ok, "{}", out.diagnostics);
    assert!(out.files[0].1.contains("fn f(x: i64)"));
    assert!(out.files[1].1.contains("a::f(7)"));
    assert_eq!(out.changed(&files), ["a.bmb", "b.bmb"]);
}

#[test]
fn cli_reports_by_default_and_writes_only_with_write() {
    let dir = std::env::temp_dir().join(format!("bmb-fix-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("h.bmb");
    std::fs::write(&path, HABITS).unwrap();
    let bmb = env!("CARGO_BIN_EXE_bmb");

    let report = Command::new(bmb).arg("fix").arg(&path).output().unwrap();
    assert!(report.status.success());
    let stdout = String::from_utf8(report.stdout).unwrap();
    assert!(stdout.lines().last().unwrap().contains("\"applied\":8"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HABITS);

    let printed = Command::new(bmb)
        .args(["fix", "--print"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(printed.stdout).unwrap(), FIXED);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), HABITS);

    let written = Command::new(bmb)
        .args(["fix", "--write"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(written.status.success());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), FIXED);

    let bad = dir.join("r.bmb");
    std::fs::write(
        &bad,
        "#[summary(\"x\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 {\n        return 5;\n    }\n}\n",
    )
    .unwrap();
    let failed = Command::new(bmb).arg("fix").arg(&bad).output().unwrap();
    assert_eq!(failed.status.code(), Some(1));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn literal_fixes_keep_the_value() {
    // `0x10` once lost its `x10` and became `0` (2R-02 finding).
    let text = "#[summary(\"t\")]\nmod m {\n    #[effects(pure)]\n    fn f() -> i64 { 0x10 + 0o17 + 0b101 + 5i64 }\n}\n";
    let out = bmb::fix_sources(&one(text));
    assert!(out.ok, "{}", out.diagnostics);
    assert!(
        out.files[0].1.contains("{ 16 + 15 + 5 + 5 }"),
        "{}",
        out.files[0].1
    );
    // An unknown suffix gets no mechanical fix.
    let odd = text.replace("5i64", "7abc");
    let out = bmb::fix_sources(&one(&odd));
    assert!(!out.ok);
    assert!(out.files[0].1.contains("7abc"));
}
