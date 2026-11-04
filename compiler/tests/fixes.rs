//! Mechanical fixes (backlog 1A-06, DESIGN.md §8.2): every `replacement` replaces
//! exactly the text of its diagnostic's span, and applying them removes the
//! diagnostics they belong to. Through the CLI (`bmb check`, backlog 4A-04).

use std::process::Command;

mod common;

use common::jsonl::{num_field, str_field};

/// What `bmb check` says about `text` (as `t.bmb`).
struct Checked {
    ok: bool,
    diagnostics: String,
}

fn check(text: &str) -> Checked {
    let dir = std::env::temp_dir().join(format!(
        "bmb-fixes-{}-{:x}",
        std::process::id(),
        text.len() as u64 * 31 + text.bytes().map(u64::from).sum::<u64>()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("t.bmb"), text).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["check", "t.bmb"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    Checked {
        ok: out.status.success(),
        diagnostics: String::from_utf8_lossy(&out.stdout).into_owned(),
    }
}

/// Byte offset of a 1-based (line, column-in-characters) position.
fn offset(text: &str, line: usize, col: usize) -> usize {
    let start: usize = text
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum();
    let rest = &text[start..];
    start
        + rest
            .char_indices()
            .nth(col - 1)
            .map_or(rest.len(), |(i, _)| i)
}

/// Applies all replacements; returns the new text and the codes that had one.
fn apply(text: &str) -> (String, Vec<String>) {
    let out = check(text);
    let mut edits = Vec::new();
    for d in out.diagnostics.lines() {
        let Some(r) = str_field(d, "replacement") else {
            continue;
        };
        let n = |k: &str| num_field(d, k).unwrap() as usize;
        let (lo, hi) = (
            offset(text, n("line"), n("col")),
            offset(text, n("end_line"), n("end_col")),
        );
        edits.push((lo, hi, r.to_string(), str_field(d, "code").unwrap()));
    }
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut s = text.to_string();
    let mut codes = Vec::new();
    for (lo, hi, r, code) in edits {
        s.replace_range(lo..hi, &r);
        codes.push(code);
    }
    codes.sort();
    (s, codes)
}

fn wrap(body: &str) -> String {
    format!("#[summary(\"x\")]\nmod m {{\n{body}\n}}\n")
}

#[test]
fn each_mechanical_fix_applies_and_clears_its_error() {
    let cases = [
        ("E0104", "#[effects(pure)]\nfn f() -> i64 { 5i64 }"),
        ("E0201", "#[effects(pure)]\nfn f() -> i64 { let y = 5; y }"),
        (
            "E0201",
            "#[effects(pure)]\nfn f(a: &[bool]) -> bool { let b = a.len() > 0; b }",
        ),
        ("E0208", "#[effects(pure)]\nfn f(a: &'a [i64]) -> () {}"),
        ("E0209", "#[effects(pure)]\nfn f() { }"),
        ("E0402", "#[effects(pure)]\nfn f(x: i128) -> i64 { x }"),
        ("E0601", "#[uses()]\nfn f() -> () {}"),
        ("E0207", "#[effects(pure)]\npub fn f() -> () {}"),
        ("E0210", "#[effects(pure)]\nfn f(mut x: i64) -> i64 { x }"),
        (
            "E0604",
            "#[effects(pure)]\nspec fn p(x: i64) -> bool { x > 0 }",
        ),
        (
            "E0401",
            "#[effects(pure)]\nfn put(a: &mut [i64]) -> () {}\n#[effects(pure)]\nfn f() -> () { let mut x: [i64; 2] = [0, 0]; put(x) }",
        ),
        (
            "E0401",
            "#[effects(pure)]\nfn get(a: &[i64]) -> () {}\n#[effects(pure)]\nfn f() -> () { let x: [i64; 2] = [0, 0]; get(x) }",
        ),
    ];
    for (code, body) in cases {
        let text = wrap(body);
        let (fixed, codes) = apply(&text);
        assert!(
            codes.iter().any(|c| c == code),
            "{code}: no replacement offered for\n{text}"
        );
        let after = check(&fixed);
        assert!(
            after.ok,
            "{code}: still failing after the fix:\n{fixed}\n{}",
            after.diagnostics
        );
    }
}

/// Removing `mut` from a parameter the body assigns clears E0210 but exposes
/// the assignment: the fix message says to copy into a `let mut` local, which
/// is not mechanical, so only the original code is required to go.
#[test]
fn mut_parameter_fix_clears_e0210_and_exposes_the_assignment() {
    let text = wrap("#[effects(pure)]\nfn f(mut x: i64) -> i64 { x = x + 1; x }");
    let (fixed, codes) = apply(&text);
    assert_eq!(codes, ["E0210"], "{text}");
    let after = check(&fixed);
    assert!(
        !after.diagnostics.contains("\"E0210\""),
        "E0210 still reported after the fix:\n{fixed}\n{}",
        after.diagnostics
    );
    assert!(
        after.diagnostics.contains("\"E0408\""),
        "expected the assignment to be reported as E0408:\n{fixed}\n{}",
        after.diagnostics
    );
}
