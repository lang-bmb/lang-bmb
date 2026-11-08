//! The standard library as the tools see it (ADR-0092 §1, backlog 4B-02):
//! added to a program that names it and only then, proved and run with it,
//! shown apart in the map, and never written by a tool.

mod common;

use std::fs;
use std::process::Command;

use common::cli;

fn files(text: &str) -> Vec<(String, String)> {
    vec![("app.bmb".to_string(), text.to_string())]
}

const USES: &str = r#"#[summary("std 사용")]
mod app {
    #[effects(pure)]
    fn begins(s: &str) -> bool {
        std::str::starts_with(s, "ab")
    }
}
"#;

const PLAIN: &str = r#"#[summary("std 없음")]
mod app {
    #[effects(pure)]
    fn one() -> i64 {
        1
    }
}
"#;

/// A program that names `std::str` gets it, proved with the program; one
/// that does not, does not.
#[test]
fn the_library_is_added_where_it_is_named() {
    let (ok, out) = cli::run(&["verify"], &files(USES));
    assert!(ok, "{out}");
    assert!(
        out.contains("\"def\":\"std::str::starts_with\",\"status\":\"verified\""),
        "{out}"
    );
    let (ok, out) = cli::run(&["verify"], &files(PLAIN));
    assert!(ok, "{out}");
    assert!(!out.contains("std::"), "{out}");
}

/// The normal form and `fix --write` hold the program's units only.
#[test]
fn fmt_and_fix_leave_the_library_out() {
    let (ok, out) = cli::run(&["fmt"], &files(USES));
    assert!(ok, "{out}");
    assert!(
        !out.contains("mod std") && !out.contains("mod str"),
        "{out}"
    );
    let dir = cli::fresh_dir();
    let app = dir.join("app.bmb");
    // A fixable habit in the program: `x.len() as u64`-style casts are
    // E0205 with a replacement; the library has none to apply.
    fs::write(
        &app,
        USES.replace(
            "std::str::starts_with(s, \"ab\")",
            "std::str::starts_with(s, \"ab\") && s.len() as u64 > 0",
        ),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["fix", "--write"])
        .arg(&app)
        .output()
        .unwrap();
    let text = fs::read_to_string(&app).unwrap();
    assert!(text.contains("u64::try_from(s.len())"), "{text}\n{out:?}");
    let written: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(written, vec!["app.bmb".to_string()]);
    assert!(!dir.join("std").exists());
    let _ = fs::remove_dir_all(&dir);
}

/// The map shows the library's units and definitions apart.
#[test]
fn the_map_marks_the_library() {
    let (ok, out) = cli::run(&["map", "L0"], &files(USES));
    assert!(ok, "{out}");
    let lib: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("\"library\":\"std\""))
        .collect();
    assert!(
        lib.iter()
            .any(|l| l.contains("\"path\":\"std::str::starts_with\"")),
        "{out}"
    );
    assert!(
        out.lines()
            .filter(|l| l.contains("\"path\":\"app"))
            .all(|l| !l.contains("\"library\"")),
        "{out}"
    );
}

/// A transaction writes the program's units, reports its own changes, and
/// does not change the library.
#[test]
fn transactions_do_not_touch_the_library() {
    let root = cli::fresh_dir();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/app.bmb"), PLAIN).unwrap();
    let apply = |ops: &str| {
        let f = root.join("ops.json");
        fs::write(&f, ops).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("apply")
            .arg(&root)
            .arg(&f)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap()
    };
    let out = apply(concat!(
        r#"[{"op":"put","unit":"app","code":""#,
        "#[effects(pure)] fn begins(s: &str) -> bool { std::str::starts_with(s, \\\"ab\\\") }",
        r#""}]"#
    ));
    assert!(out.contains("\"committed\":true"), "{out}");
    assert!(out.contains("\"changed\":[\"app::begins\"]"), "{out}");
    let names: Vec<_> = fs::read_dir(root.join("src"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names, vec!["app.bmb".to_string()]);
    for op in [
        r#"[{"op":"remove","def":"std::str::starts_with"}]"#,
        r#"[{"op":"rename","def":"std::str::starts_with","to":"begins"}]"#,
        concat!(
            r#"[{"op":"put","unit":"std::str","code":""#,
            "#[effects(pure)] fn f() -> i64 { 0 }",
            r#""}]"#
        ),
        r#"[{"op":"unit","path":"std","summary":"x"}]"#,
    ] {
        let out = apply(op);
        assert!(out.contains("\"committed\":false"), "{op}: {out}");
        assert!(out.contains("standard library"), "{op}: {out}");
    }
    let _ = fs::remove_dir_all(&root);
}

/// Every library unit is proved: on its own, as a program of one unit.
#[test]
fn every_library_unit_is_proved() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lib/std");
    let mut n = 0;
    for e in fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "bmb") {
            continue;
        }
        let text = fs::read_to_string(&p).unwrap();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        let (ok, out) = cli::run(&["verify"], &[(name, text)]);
        assert!(ok, "{}: {out}", p.display());
        assert!(!out.contains("\"status\":\"failed\""), "{out}");
        n += 1;
    }
    assert!(n >= 1);
}
