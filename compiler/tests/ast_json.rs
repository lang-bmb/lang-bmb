//! The JSON syntax tree of the JSON AST edit surface (ADR-0073), through the CLI and the MCP server: every program of the
//! corpus, written as its tree by `edit_ast` into an empty workspace, is the
//! same normal form, and `bmb ast` of the result is the same tree.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Programs: each corpus file alone, and each benchmark seed of `tests/corpus` as a workspace.
fn corpus() -> Vec<(String, Vec<PathBuf>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let files = |dir: &str| -> Vec<PathBuf> {
        let mut v: Vec<PathBuf> = fs::read_dir(root.join(dir))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
            .collect();
        v.sort();
        v
    };
    let mut out = Vec::new();
    for dir in [
        "tests/check/ok",
        "tests/verify/pass",
        "tests/verify/fail",
        "tests/run",
        "lib",
    ] {
        for p in files(dir) {
            out.push((p.display().to_string(), vec![p]));
        }
    }
    for dir in ["tests/corpus/seed/m1b/src", "tests/corpus/seed/bvb/src"] {
        out.push((dir.to_string(), files(dir)));
    }
    out
}

fn bmb(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(args)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

fn sources(ws: &Path) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = fs::read_dir(ws.join("src"))
        .map(|d| {
            d.map(|e| e.unwrap().path())
                .map(|p| {
                    let name = p.file_name().unwrap().to_string_lossy().to_string();
                    (name, fs::read_to_string(&p).unwrap())
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// One `edit_ast` call on a fresh `bmb mcp --ast-tools`; the result text.
fn edit_ast(ws: &Path, patch: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["mcp", "--ast-tools", ws.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(
        child.stdin.as_mut().unwrap(),
        "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{{\"name\":\"edit_ast\",\"arguments\":{{\"patch\":{patch}}}}}}}"
    )
    .unwrap();
    drop(child.stdin.take());
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

/// A repeat count written as a name belongs to the unit that writes it,
/// whatever units come after it or beside it: the document
/// of a program with such counts in a unit, its nested unit and another
/// unit commits and gives the same sources.
#[test]
fn named_repeat_counts_round_trip_with_nested_units() {
    const SRC: &str = r#"#[summary("o")]
mod outer {
    const K: usize = 2;
    const ROW: [u8; K] = [7; K];

    #[summary("i")]
    mod inner {
        const N: usize = 3;
        const COL: [u8; N] = [9; N];
    }

    #[effects(pure)]
    fn first() -> u8 {
        let a: [u8; K] = [1; K];
        a[0]
    }
}

#[summary("p")]
mod peer {
    const M: usize = 4;
    const ALL: [u8; M] = [5; M];
}
"#;
    let tmp = std::env::temp_dir().join(format!("bmb-ast-json-counts-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let (a, b) = (tmp.join("a"), tmp.join("b"));
    fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("counts.bmb");
    fs::write(&file, SRC).unwrap();
    let (ok, out) = bmb(&["import", a.to_str().unwrap(), file.to_str().unwrap()]);
    assert!(ok, "{out}");
    let (ok, doc) = bmb(&["ast", a.to_str().unwrap()]);
    assert!(ok, "{doc}");
    fs::create_dir_all(&b).unwrap();
    let out = edit_ast(
        &b,
        &format!(
            "[{{\"op\":\"add\",\"path\":\"\",\"value\":{}}}]",
            doc.trim_end()
        ),
    );
    assert!(out.contains("\\\"committed\\\":true"), "{out}");
    assert_eq!(sources(&b), sources(&a));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn every_program_round_trips_through_json() {
    let tmp = std::env::temp_dir().join(format!("bmb-ast-json-{}", std::process::id()));
    let mut n = 0;
    for (name, files) in corpus() {
        let _ = fs::remove_dir_all(&tmp);
        let (a, b) = (tmp.join("a"), tmp.join("b"));
        let mut args = vec!["import", a.to_str().unwrap()];
        let paths: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
        args.extend(paths.iter().map(String::as_str));
        // Programs the checker accepts; a file that holds a recovery form
        // (a `use`, a `let` without a type) is an error case, not a program.
        if !bmb(&args).0 {
            continue;
        }
        let (ok, doc) = bmb(&["ast", a.to_str().unwrap()]);
        assert!(ok, "{name}");
        let doc = doc.trim_end();
        fs::create_dir_all(&b).unwrap();
        let out = edit_ast(
            &b,
            &format!("[{{\"op\":\"add\",\"path\":\"\",\"value\":{doc}}}]"),
        );
        assert!(out.contains("\\\"committed\\\":true"), "{name}: {out}");
        assert_eq!(sources(&b), sources(&a), "{name}");
        assert_eq!(
            bmb(&["ast", b.to_str().unwrap()]).1.trim_end(),
            doc,
            "{name}"
        );
        n += 1;
    }
    let _ = fs::remove_dir_all(&tmp);
    assert!(n >= 130, "only {n} programs checked");
}
