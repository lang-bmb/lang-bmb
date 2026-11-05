//! The `impact` query (backlog 2D-02) is complete: when a definition's body
//! changes, every definition whose proof script changes is in the reported
//! set. The proof scripts are exactly what the solver sees, so they are the
//! ground truth for "this proof must be redone". Through the CLI (backlog
//! 4A-04): scripts from `bmb verify --emit-smt`, maps and queries from
//! `bmb map` and `bmb query`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

mod common;

use common::jsonl::str_field;

/// A scratch directory for `tag`, created empty.
fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("bmb-impact-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// `bmb args.. c.bmb` on `text`, written as `c.bmb` in `dir`.
fn bmb(dir: &Path, text: &str, args: &[&str]) -> Output {
    fs::write(dir.join("c.bmb"), text).unwrap();
    Command::new(env!("CARGO_BIN_EXE_bmb"))
        .current_dir(dir)
        .args(args)
        .arg("c.bmb")
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

/// Whether `text` verifies, its output, and each definition's proof script
/// (`--emit-smt` names a script `unit.name.smt2`).
fn verify(dir: &Path, text: &str) -> (bool, String, BTreeMap<String, String>) {
    let smt = dir.join("smt");
    let _ = fs::remove_dir_all(&smt);
    let out = bmb(dir, text, &["verify", "--emit-smt", "smt"]);
    let mut scripts = BTreeMap::new();
    for e in fs::read_dir(&smt).unwrap() {
        let p = e.unwrap().path();
        let Some(stem) = p
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_suffix(".smt2")
        else {
            continue;
        };
        scripts.insert(stem.replace('.', "::"), fs::read_to_string(&p).unwrap());
    }
    (out.status.success(), stdout(&out), scripts)
}

/// The rows of `bmb map`/`bmb query` (`args` before the file).
fn query(dir: &Path, text: &str, args: &[&str]) -> Vec<String> {
    let out = bmb(dir, text, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout(&out).lines().map(str::to_string).collect()
}

fn s(row: &str, key: &str) -> Option<String> {
    str_field(row, key)
}

/// Index of the `{` that opens the body of `unit::name`.
fn body_open(text: &str, unit: &str, name: &str) -> usize {
    let m = text.find(&format!("mod {unit} {{")).unwrap();
    let f = m + text[m..].find(&format!("fn {name}(")).unwrap();
    f + text[f..].find('{').unwrap()
}

fn matching_close(text: &str, open: usize) -> usize {
    let mut depth = 0;
    for (i, c) in text[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + i;
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced braces")
}

/// The same program with the body of `path` changed but its meaning kept.
fn change_body(text: &str, path: &str, spec: bool) -> String {
    let (unit, name) = path.rsplit_once("::").unwrap();
    let open = body_open(text, unit, name);
    if spec {
        let close = matching_close(text, open);
        let body = &text[open + 1..close];
        // The other branch is never taken; a second copy of `body` would bind
        // its quantified names twice.
        let header = &text[..open];
        let other = if header[header.rfind("fn ").unwrap()..].contains("-> bool") {
            "false"
        } else {
            "0"
        };
        format!(
            "{}{{ if true {{ {body} }} else {{ {other} }} }}{}",
            &text[..open],
            &text[close + 1..]
        )
    } else {
        format!(
            "{} let impact_probe: i64 = 0;{}",
            &text[..=open],
            &text[open + 1..]
        )
    }
}

#[test]
fn body_changes_reach_only_the_reported_definitions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let dir = scratch("body");
    let mut reached_others = 0;
    for rel in ["run/search.bmb"] {
        let text = fs::read_to_string(root.join(rel)).unwrap();
        let base = verify(&dir, &text);
        assert!(base.0, "{rel}: {}", base.1);
        let defs: Vec<(String, bool)> = query(&dir, &text, &["map", "L0"])
            .iter()
            .filter(|r| s(r, "kind").as_deref() == Some("def"))
            .map(|r| {
                (
                    s(r, "path").unwrap(),
                    s(r, "def_kind").as_deref() == Some("spec fn"),
                )
            })
            .collect();
        assert!(defs.len() > 10, "{rel}");
        for (path, spec) in &defs {
            let changed = change_body(&text, path, *spec);
            let after = verify(&dir, &changed);
            assert!(
                after.0,
                "{rel}: changing {path} broke it:\n{changed}\n{}",
                after.1
            );
            let touched: BTreeSet<&String> = base
                .2
                .keys()
                .chain(after.2.keys())
                .filter(|k| base.2.get(*k) != after.2.get(*k))
                .collect();
            let reported: BTreeSet<String> =
                query(&dir, &text, &["query", "impact", path, "body", "--"])
                    .iter()
                    .filter_map(|r| s(r, "path"))
                    .collect();
            reached_others += touched.iter().filter(|t| **t != path).count();
            for t in &touched {
                assert!(
                    reported.contains(*t),
                    "{rel}: changing the body of {path} changed the proof of {t}, which impact does not report ({reported:?})"
                );
            }
        }
    }
    let _ = fs::remove_dir_all(&dir);
    // Not vacuous: spec fn bodies do reach other proofs.
    assert!(reached_others > 0);
}

#[test]
fn contract_changes_reach_callers_and_spec_bodies_reach_revealers() {
    let text = r#"
#[summary("u")]
mod u {
    spec fn p(x: i64) -> bool { x > 0 }

    spec fn q(x: i64) -> bool { p(x) }

    #[effects(pure)]
    #[uses(p)]
    fn f(x: i64) -> i64
        requires p(x),
    {
        x - 1
    }

    #[effects(pure)]
    #[uses(p)]
    fn g(x: i64) -> i64
        requires p(x),
    {
        f(x)
    }

    #[effects(pure)]
    fn h(x: i64) -> i64
        requires q(x),
    {
        x
    }
}
"#;
    let dir = scratch("contract");
    let ask = |def: &str, change: &str| -> Vec<(String, String)> {
        query(&dir, text, &["query", "impact", def, change, "--"])
            .iter()
            .filter_map(|r| Some((s(r, "path")?, s(r, "reason")?)))
            .collect()
    };
    let pairs = |xs: &[(&str, &str)]| -> Vec<(String, String)> {
        xs.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    };
    assert_eq!(ask("u::f", "body"), pairs(&[("u::f", "changed")]));
    assert_eq!(
        ask("u::f", "iface"),
        pairs(&[("u::f", "changed"), ("u::g", "calls")])
    );
    // `h` names `q`, whose body calls `p`, but reveals neither: unaffected.
    assert_eq!(
        ask("u::p", "body"),
        pairs(&[("u::f", "uses"), ("u::g", "uses")])
    );
    assert_eq!(
        ask("u::p", "iface"),
        pairs(&[
            ("u::f", "uses"),
            ("u::g", "uses"),
            ("u::p", "changed"),
            ("u::q", "names")
        ])
    );
    let usage = bmb(&dir, text, &["query", "impact", "u::f", "both", "--"]);
    assert_eq!(usage.status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}
