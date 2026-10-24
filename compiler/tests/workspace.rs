//! Workspaces and write transactions (ADR-0009, ADR-0019 §3, backlog 1D-08, 1D-09).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use bmb::workspace::{Op, commit, read_dir_sources, write_units};

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("bmb-ws-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

fn snapshot(root: &Path) -> BTreeMap<String, String> {
    read_dir_sources(root).unwrap().into_iter().collect()
}

const BASE: &str = r#"
#[summary("기반")]
mod core {
    spec fn pos(x: i64) -> bool { x > 0 }

    #[effects(pure)]
    #[uses(pos)]
    fn dec(x: i64) -> i64
        requires pos(x),
    {
        x - 1
    }
}

#[summary("사용처")]
mod app {
    #[effects(pure)]
    #[uses(core::pos)]
    fn use_dec() -> i64 { core::dec(3) }
}
"#;

fn init(name: &str) -> PathBuf {
    let root = temp(name);
    let units = bmb::format_sources(&[("base.bmb".into(), BASE.into())]).unwrap();
    write_units(&root, &units).unwrap();
    root
}

fn put(unit: &str, code: &str) -> Op {
    Op::Put {
        unit: unit.into(),
        code: code.into(),
    }
}

#[test]
fn import_places_one_normal_form_file_per_unit() {
    let root = init("import");
    let names: Vec<String> = snapshot(&root)
        .keys()
        .map(|k| k.rsplit('/').next().unwrap().to_string())
        .collect();
    assert_eq!(names, ["app.bmb", "core.bmb"]);
    // The layout is a function of the program, not of how it was split before.
    let other = temp("import2");
    let (a, b) = BASE.split_at(BASE.find("#[summary(\"사용처").unwrap());
    let units =
        bmb::format_sources(&[("x.bmb".into(), b.into()), ("y.bmb".into(), a.into())]).unwrap();
    write_units(&other, &units).unwrap();
    let strip = |m: BTreeMap<String, String>| m.into_values().collect::<Vec<_>>();
    assert_eq!(strip(snapshot(&root)), strip(snapshot(&other)));
}

#[test]
fn good_transaction_commits_and_reports_changes() {
    let root = init("good");
    let c = commit(
        &root,
        &[
            Op::Unit { path: "extra".into(), summary: "추가".into() },
            put("extra", "#[effects(pure)]\nfn one() -> i64 { 1 }"),
            put("core", "#[effects(pure)]\n#[uses(pos)]\nfn dec(x: i64) -> i64\n    requires pos(x),\n{\n    x - 1 + 0\n}"),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.changed, ["core::dec", "extra::one"]);
    let files = snapshot(&root);
    assert!(files.keys().any(|k| k.ends_with("extra.bmb")));
    let core = files
        .iter()
        .find(|(k, _)| k.ends_with("core.bmb"))
        .unwrap()
        .1;
    assert!(core.contains("x - 1 + 0"));
}

#[test]
fn rejected_transaction_changes_nothing() {
    let root = init("reject");
    let before = snapshot(&root);
    let c = commit(
        &root,
        &[put(
            "core",
            "#[effects(pure)]\nfn bad(x: i64) -> bool {\n    x + 1\n}",
        )],
    )
    .unwrap();
    assert!(!c.ok);
    assert!(
        c.diagnostics.contains("\"file\":\"tx/op0\",\"line\":3"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);

    // Removing a definition that another one calls is a type error too.
    let c = commit(
        &root,
        &[Op::Remove {
            def: "core::dec".into(),
        }],
    )
    .unwrap();
    assert!(!c.ok);
    assert!(c.diagnostics.contains("E0304"), "{}", c.diagnostics);
    assert_eq!(snapshot(&root), before);

    // Unknown targets are rejected without touching files.
    let c = commit(
        &root,
        &[put("nowhere", "#[effects(pure)]\nfn f() -> () {}")],
    )
    .unwrap();
    assert!(!c.ok);
    assert!(c.diagnostics.contains("does not exist"));
    assert_eq!(snapshot(&root), before);
}

#[test]
fn unproved_definitions_commit_and_show_as_failed() {
    let root = init("unproved");
    let c = commit(
        &root,
        &[put(
            "core",
            "#[effects(pure)]\nfn inc(x: i64) -> i64 { x + 1 }",
        )],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let files = read_dir_sources(&root).unwrap();
    let status = bmb::map_request(
        &files,
        &["query".into(), "surface".into(), "core::inc".into()],
    )
    .unwrap();
    assert!(status.contains("\"status\":\"failed\""), "{status}");
}

#[test]
fn removing_a_unit_removes_its_file() {
    let root = init("rmunit");
    let c = commit(&root, &[Op::RemoveUnit { path: "app".into() }]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.removed, ["app::use_dec"]);
    assert!(!snapshot(&root).keys().any(|k| k.ends_with("app.bmb")));
}

#[test]
fn ops_parse_from_json() {
    let ops = bmb::workspace::parse_ops(
        r#"[{"op":"unit","path":"a","summary":"s"},{"op":"put","unit":"a","code":"fn f() -> () {}"},
            {"op":"remove","def":"a::f"},{"op":"remove_unit","path":"a"}]"#,
    )
    .unwrap();
    assert_eq!(ops.len(), 4);
    assert!(bmb::workspace::parse_ops(r#"[{"op":"explode"}]"#).is_err());
    assert!(bmb::workspace::parse_ops(r#"{"op":"put"}"#).is_err());
}
