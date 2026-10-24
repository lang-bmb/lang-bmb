//! Mechanical fixes (backlog 1A-06, DESIGN.md §8.2): every `replacement` replaces
//! exactly the text of its diagnostic's span, and applying them removes the
//! diagnostics they belong to.

use bmb::json::{self, Value};

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
    let out = bmb::check_sources(&[("t.bmb".into(), text.into())]);
    let mut edits = Vec::new();
    for line in out.diagnostics.lines() {
        let d = json::parse(line).unwrap();
        let Some(r) = d
            .get("fix")
            .and_then(|f| f.get("replacement"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let n = |k: &str| d.get(k).unwrap().as_i64().unwrap() as usize;
        let (lo, hi) = (
            offset(text, n("line"), n("col")),
            offset(text, n("end_line"), n("end_col")),
        );
        edits.push((
            lo,
            hi,
            r.to_string(),
            d.get("code").unwrap().as_str().unwrap().to_string(),
        ));
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
        (
            "E0202",
            "#[effects(pure)]\nfn f() -> i64 { let v: [i64; 2] = vec![1, 2]; v[0] }",
        ),
        ("E0206", "#[effects(pure)]\nfn f(x: i64) -> i64 { x? }"),
        ("E0208", "#[effects(pure)]\nfn f(a: &'a [i64]) -> () {}"),
        ("E0209", "#[effects(pure)]\nfn f() { }"),
        ("E0402", "#[effects(pure)]\nfn f(x: i32) -> i64 { x }"),
        ("E0601", "#[uses()]\nfn f() -> () {}"),
    ];
    for (code, body) in cases {
        let text = wrap(body);
        let (fixed, codes) = apply(&text);
        assert!(
            codes.iter().any(|c| c == code),
            "{code}: no replacement offered for\n{text}"
        );
        let after = bmb::check_sources(&[("t.bmb".into(), fixed.clone())]);
        assert!(
            after.ok,
            "{code}: still failing after the fix:\n{fixed}\n{}",
            after.diagnostics
        );
    }
}
