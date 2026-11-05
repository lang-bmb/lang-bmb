//! Nesting limits (backlog 1A-11): deep input is a diagnostic, not a crash, and
//! programs just inside the limit go through the whole pipeline on the stack
//! the CLI uses. Through the CLI (backlog 4A-04).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// `parser::MAX_NESTING`, read from the source.
fn max_nesting() -> usize {
    let src = include_str!("../src/parser.rs");
    let line = src
        .lines()
        .find_map(|l| l.strip_prefix("pub const MAX_NESTING: usize = "))
        .expect("MAX_NESTING in src/parser.rs");
    line.trim_end_matches(';').parse().unwrap()
}

/// Writes `files` into a fresh directory named `tag` and runs `bmb args..
/// files` there.
fn bmb(tag: &str, files: &[(String, String)], args: &[&str]) -> Output {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("bmb-nesting-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(&dir)
        .args(args)
        .args(files.iter().map(|(n, _)| n))
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    out
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// `bmb check`: a deep program fails with E0107 and does not crash.
fn assert_too_deep(tag: &str, files: &[(String, String)]) {
    let out = bmb(tag, files, &["check"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout(&out).contains("\"E0107\""), "{}", stdout(&out));
}

fn program(body: &str) -> Vec<(String, String)> {
    vec![(
        "deep.bmb".to_string(),
        format!(
            "#[summary(\"깊은 중첩\")]\nmod deep {{\n    #[effects(io)]\n    fn main() -> () {{\n        let x: i64 = {body};\n        print_i64(x)\n    }}\n}}\n"
        ),
    )]
}

fn parens(n: usize) -> String {
    format!("{}1{}", "(".repeat(n), ")".repeat(n))
}

fn ifs(n: usize) -> String {
    let mut s = String::from("1");
    for _ in 0..n {
        s = format!("if true {{ {s} }} else {{ 2 }}");
    }
    s
}

fn else_ifs(n: usize) -> String {
    format!(
        "{}1 }} else {{ 2 }}",
        "if false { 0 } else ".repeat(n) + "if true {"
    )
}

fn blocks(n: usize) -> String {
    format!("{}1{}", "{ ".repeat(n), " }".repeat(n))
}

fn negs(n: usize) -> String {
    format!("{}1", "- ".repeat(n))
}

/// `0 + 0 + .. + 1`: iterative to parse, but a left-nested tree.
fn sums(n: usize) -> String {
    format!("{}1", "0 + ".repeat(n))
}

/// `[[..[1]..][0]..][0]`: nested arrays indexed back down.
fn indexes(n: usize) -> String {
    format!("{}1{}{}", "[".repeat(n), "]".repeat(n), "[0]".repeat(n))
}

/// `1.wrapping_add(0).wrapping_add(0)..`: a left-nested method chain.
fn methods(n: usize) -> String {
    format!("1{}", ".wrapping_add(0)".repeat(n))
}

/// Left-nested `||` and nested `!` (no overflow obligations, so proving stays fast).
fn ors(n: usize) -> String {
    format!("if {}true {{ 1 }} else {{ 2 }}", "false || ".repeat(n))
}

fn nots(n: usize) -> String {
    format!("if {}true {{ 1 }} else {{ 2 }}", "!".repeat(n & !1))
}

fn implications(n: usize) -> String {
    // `==>` is spec-only; the checker rejects it here, but must not crash.
    format!("{}true", "true ==> ".repeat(n))
}

#[test]
fn deep_nesting_is_a_diagnostic() {
    let n = 50_000;
    for (k, body) in [
        parens(n),
        blocks(n),
        negs(n),
        sums(n),
        indexes(n),
        methods(n),
        implications(n),
        ifs(5_000),
        else_ifs(n),
    ]
    .into_iter()
    .enumerate()
    {
        assert_too_deep(&format!("expr{k}"), &program(&body));
    }
}

#[test]
fn deep_types_and_units_are_a_diagnostic() {
    let n = 50_000;
    let ty = format!(
        "#[summary(\"t\")]\nmod t {{\n    #[effects(pure)]\n    fn f(a: &{}i64{}) -> () {{ () }}\n}}\n",
        "[".repeat(n),
        "]".repeat(n)
    );
    let units = format!(
        "{}{}",
        "#[summary(\"u\")] mod u { ".repeat(n),
        "}".repeat(n)
    );
    for (k, text) in [ty, units].into_iter().enumerate() {
        assert_too_deep(&format!("decl{k}"), &[("deep.bmb".into(), text)]);
    }
}

#[test]
fn programs_inside_the_limit_pass_the_whole_pipeline() {
    // Each kind uses at most one level per syntactic level, plus a few for
    // the surrounding function.
    let n = max_nesting() - 8;
    let half = n / 2 - 1;
    for (k, body) in [
        parens(n),
        blocks(n),
        ors(n),
        nots(n),
        methods(n),
        ifs(half),
        else_ifs(n),
    ]
    .into_iter()
    .enumerate()
    {
        let files = program(&body);
        let tag = format!("in{k}");
        for args in [
            &["check"][..],
            &["verify"],
            &["emit-llvm", "-o", "deep.ll"],
            &["fmt"],
            &["map", "all"],
        ] {
            let out = bmb(&tag, &files, args);
            assert!(
                out.status.success(),
                "{args:?} on kind {k}: {}{}",
                stdout(&out),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
}
