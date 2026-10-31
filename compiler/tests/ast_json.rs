//! The JSON syntax tree of the JSON AST edit surface (backlog T1-06,
//! ADR-0073), through the CLI and the MCP server: every program of the
//! corpus, written as its tree by `edit_ast` into an empty workspace, is the
//! same normal form, and `bmb ast` of the result is the same tree.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Programs: each corpus file alone, and each bench seed as a workspace.
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
    for dir in ["bench/m1b/seed/src", "bench/bvb/seed/src"] {
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
