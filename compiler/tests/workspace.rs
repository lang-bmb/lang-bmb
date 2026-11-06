//! Workspaces and write transactions (ADR-0009, ADR-0019 §3, backlog 1D-08, 1D-09).

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use common::cli;
use common::json::{self, Value};

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("bmb-ws-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    d
}

/// `bmb ARGS`: its success and stdout.
fn bmb(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(args)
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stdout).unwrap())
}

fn arg(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// The workspace's source files (path, text), as `bmb` reads a directory:
/// the `.bmb` files of `src/` (or of `root` itself), sorted.
fn snapshot(root: &Path) -> BTreeMap<String, String> {
    let dir = if root.join("src").is_dir() {
        root.join("src")
    } else {
        root.to_path_buf()
    };
    fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .map(|p| (p.display().to_string(), fs::read_to_string(&p).unwrap()))
        .collect()
}

/// `bmb import ROOT` of `files`.
fn import(root: &Path, files: &[(String, String)]) {
    let (ok, out) = cli::run(&["import", arg(root)], files);
    assert!(ok, "{out}");
}

/// The normal form of `files` (`bmb fmt`), unit name -> text.
fn normal_units(files: &[(String, String)]) -> BTreeMap<String, String> {
    let (ok, out) = cli::run(&["fmt"], files);
    assert!(ok, "{out}");
    cli::units(&out).into_iter().collect()
}

/// A transaction operation, as `bmb apply` reads it (ADR-0009).
#[derive(Debug)]
enum Op {
    Unit { path: String, summary: String },
    Put { unit: String, code: String },
    Remove { def: String },
    RemoveUnit { path: String },
    Rename { def: String, to: String },
    Expect { def: String, id: String },
}

impl Op {
    fn json(&self) -> String {
        let q = json::quote;
        match self {
            Op::Unit { path, summary } => format!(
                "{{\"op\":\"unit\",\"path\":{},\"summary\":{}}}",
                q(path),
                q(summary)
            ),
            Op::Put { unit, code } => {
                format!(
                    "{{\"op\":\"put\",\"unit\":{},\"code\":{}}}",
                    q(unit),
                    q(code)
                )
            }
            Op::Remove { def } => format!("{{\"op\":\"remove\",\"def\":{}}}", q(def)),
            Op::RemoveUnit { path } => {
                format!("{{\"op\":\"remove_unit\",\"path\":{}}}", q(path))
            }
            Op::Rename { def, to } => {
                format!("{{\"op\":\"rename\",\"def\":{},\"to\":{}}}", q(def), q(to))
            }
            Op::Expect { def, id } => {
                format!("{{\"op\":\"expect\",\"def\":{},\"id\":{}}}", q(def), q(id))
            }
        }
    }
}

/// What `bmb apply` reported: the diagnostics, then the commit line.
struct Commit {
    ok: bool,
    diagnostics: String,
    changed: Vec<String>,
    removed: Vec<String>,
}

/// `bmb apply ROOT OPS.json`; its error when it could not run (status 2).
fn commit(root: &Path, ops: &[Op]) -> Result<Commit, String> {
    let dir = cli::fresh_dir();
    let file = dir.join("ops.json");
    let ops: Vec<String> = ops.iter().map(Op::json).collect();
    fs::write(&file, format!("[{}]", ops.join(","))).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
        .arg("apply")
        .arg(root)
        .arg(&file)
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    if out.status.code() == Some(2) {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let text = String::from_utf8(out.stdout).unwrap();
    let all = text.trim_end();
    let (diagnostics, line) = all.rsplit_once('\n').unwrap_or(("", all));
    let line = json::parse(line).unwrap_or_else(|e| panic!("{e}: {text}"));
    let list = |k: &str| match line.get(k) {
        Some(Value::Array(xs)) => xs.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
        _ => panic!("{k}: {text}"),
    };
    Ok(Commit {
        ok: matches!(line.get("committed"), Some(Value::Bool(true))),
        diagnostics: diagnostics.to_string(),
        changed: list("changed"),
        removed: list("removed"),
    })
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
    import(&root, &[("base.bmb".into(), BASE.into())]);
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
    import(
        &other,
        &[("x.bmb".into(), b.into()), ("y.bmb".into(), a.into())],
    );
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
    let (_, status) = bmb(&["query", "surface", "core::inc", "--", arg(&root)]);
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

/// A transaction that is not a JSON array of known operations is refused
/// before anything is read or written (status 2).
#[test]
fn malformed_transactions_are_refused() {
    let root = init("malformed");
    let before = snapshot(&root);
    for ops in [
        r#"[{"op":"explode"}]"#,
        r#"{"op":"put"}"#,
        r#"[{"op":"rename","def":"core::dec"}]"#,
        r#"[{"op":"unit","path":"a"}]"#,
    ] {
        let dir = cli::fresh_dir();
        fs::write(dir.join("ops.json"), ops).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("apply")
            .arg(&root)
            .arg(dir.join("ops.json"))
            .output()
            .unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(out.status.code(), Some(2), "{ops}");
        assert_eq!(snapshot(&root), before, "{ops}");
    }
}

/// Unit and definition paths become file names: anything but `::`-separated
/// identifiers is rejected and nothing is written (backlog 1D-15).
#[test]
fn invalid_paths_are_rejected_and_write_nothing() {
    let outer = temp("traversal");
    let root = outer.join("ws");
    import(&root, &[("base.bmb".into(), BASE.into())]);
    let before = snapshot(&root);
    let listing = |d: &Path| {
        let mut v: Vec<String> = fs::read_dir(d)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    let outer_before = listing(&outer);
    let src_before = listing(&root.join("src"));
    let f = "#[effects(pure)]\nfn f() -> i64 { 1 }";
    let bad = [
        "../../evil",
        "../evil",
        "",
        "a::",
        "::a",
        "a::::b",
        "a b",
        " a",
        "a/b",
        "a\\b",
        ".a",
        "1a",
        "fn",
        "mod",
        "core::..",
        "a { } mod evil",
        "한글",
    ];
    for path in bad {
        let attempts = [
            vec![Op::Unit {
                path: path.into(),
                summary: "x".into(),
            }],
            vec![
                Op::Unit {
                    path: path.into(),
                    summary: "x".into(),
                },
                put(path, f),
            ],
            vec![put(path, f)],
            vec![Op::Remove {
                def: format!("{path}::f"),
            }],
            vec![Op::RemoveUnit { path: path.into() }],
        ];
        for ops in attempts {
            let c = commit(&root, &ops).unwrap();
            assert!(!c.ok, "{path:?} {ops:?}");
            assert!(
                c.diagnostics.contains("not a valid path"),
                "{path:?}: {}",
                c.diagnostics
            );
            assert_eq!(snapshot(&root), before, "{path:?}");
            assert_eq!(listing(&outer), outer_before, "{path:?}");
            assert_eq!(listing(&root.join("src")), src_before, "{path:?}");
        }
    }
    assert!(!outer.join("evil.bmb").exists());

    // The same through the CLI, as reported.
    let ops = outer.join("ops.json");
    fs::write(&ops, r#"[{"op":"unit","path":"../../evil","summary":"x"}]"#).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_bmb"))
        .args(["apply", root.to_str().unwrap(), ops.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(!outer.join("evil.bmb").exists());
    assert_eq!(snapshot(&root), before);
    let _ = fs::remove_dir_all(&outer);
}

/// `put` takes items only: nested units, code that closes the wrapper, and
/// code without an item are errors, not silently dropped (1D-19).
#[test]
fn put_rejects_units_and_empty_code() {
    let root = init("putshape");
    let before = snapshot(&root);
    let cases = [
        "#[summary(\"x\")]\nmod inner {\n#[effects(pure)]\nfn f() -> i64 { 1 }\n}",
        "#[effects(pure)]\nfn f() -> i64 { 1 }\n}\n#[summary(\"x\")]\nmod evil {\n#[effects(pure)]\nfn g() -> i64 { 2 }",
        "",
        "   ",
        "// only a comment",
    ];
    for code in cases {
        let c = commit(&root, &[put("core", code)]).unwrap();
        assert!(!c.ok, "{code:?}");
        assert!(
            c.diagnostics.contains("must not contain units")
                || c.diagnostics.contains("no definition"),
            "{code:?}: {}",
            c.diagnostics
        );
        assert_eq!(snapshot(&root), before);
    }
}

fn rm(def: &str) -> Op {
    Op::Remove { def: def.into() }
}

fn unit_text(root: &Path, unit: &str) -> String {
    snapshot(root)
        .into_iter()
        .find(|(k, _)| k.ends_with(&format!("{unit}.bmb")))
        .map(|(_, t)| t)
        .unwrap_or_default()
}

/// `impl T { .. }` merges function by function into T's block (ADR-0074
/// §2): a new function joins it, one of the same name replaces its
/// namesake, the others stay. The commit names the functions it wrote.
#[test]
fn put_merges_impl_functions_into_the_block() {
    let root = init_rich("putimpl");
    let get_x = "impl Pt {\n    #[effects(pure)]\n    fn get_x(self: &Pt) -> i64 { self.x }\n}";
    let c = commit(&root, &[put("geo", get_x)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.changed, ["geo::Pt::get_x"]);
    let text = unit_text(&root, "geo");
    assert!(
        text.contains("fn get_x(self: &Pt)") && text.contains("fn new(x: i64, y: i64)"),
        "{text}"
    );
    assert_eq!(text.matches("impl Pt {").count(), 1, "{text}");
    // Replacing `new` leaves `get_x`.
    let new = "impl Pt {\n    #[effects(pure)]\n    fn new(x: i64, y: i64) -> Pt { Pt { x: y, y: x } }\n}";
    let c = commit(&root, &[put("geo", new)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.changed, ["geo::Pt::new"]);
    assert!(unit_text(&root, "geo").contains("fn get_x(self: &Pt)"));
    // A block with no function writes nothing (ADR-0074 §4).
    let before = snapshot(&root);
    let c = commit(&root, &[put("geo", "impl Pt { }")]).unwrap();
    assert!(
        !c.ok
            && c.diagnostics
                .contains("op 0: `impl Pt { }` contains no definition"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);
}

/// A type is replaced whole (ADR-0074 §1): a changed field list that breaks
/// a construction site is rejected with the checker's diagnostic, and an
/// added `#[invariant(p)]` shows the type as changed (§5).
#[test]
fn put_replaces_types_whole() {
    let root = init_rich("puttype");
    let before = snapshot(&root);
    let c = commit(&root, &[put("geo", "struct Pt { x: i64, y: i64, z: i64 }")]).unwrap();
    assert!(!c.ok, "a construction site lacks `z`");
    assert!(c.changed.is_empty());
    assert_eq!(snapshot(&root), before);
    let inv = "spec fn tidy(p: &Pt) -> bool { p.x <= p.y }\n\n#[invariant(tidy)]\nstruct Pt { x: i64, y: i64 }";
    let c = commit(&root, &[put("geo", inv)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.changed, ["geo::Pt", "geo::tidy"]);
    assert!(unit_text(&root, "geo").contains("#[invariant(tidy)]"));
}

/// A trait and a trait impl are written and replaced whole, the impl at its
/// address `<unit::T as unit::Trait>` (ADR-0074 §1); a missing method is
/// the checker's E0315.
#[test]
fn put_writes_traits_and_trait_impls() {
    let root = init_rich("puttrait");
    let code = "trait Perim {\n    #[effects(pure)]\n    fn perim(self: &Self) -> i64;\n}\n\nimpl Perim for Pt {\n    #[effects(pure)]\n    fn perim(self: &Pt) -> i64 { self.x.saturating_add(self.y) }\n}";
    let c = commit(&root, &[put("geo", code)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(
        c.changed,
        [
            "<geo::Pt as geo::Perim>",
            "<geo::Pt as geo::Perim>::perim",
            "geo::Perim"
        ]
    );
    // The same address again replaces the impl; there is still one.
    let again = "impl Perim for Pt {\n    #[effects(pure)]\n    fn perim(self: &Pt) -> i64 { self.y.saturating_add(self.x) }\n}";
    let c = commit(&root, &[put("geo", again)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.changed, ["<geo::Pt as geo::Perim>::perim"]);
    assert_eq!(
        unit_text(&root, "geo").matches("impl Perim for Pt").count(),
        1
    );
    let before = snapshot(&root);
    let c = commit(&root, &[put("geo", "impl Perim for Pt { }")]).unwrap();
    assert!(
        !c.ok && c.diagnostics.contains("E0315"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);
}

/// Five kinds of item in one `code` commit together, each at its address,
/// and the commit names them all; a `use` is the checker's E0214 (ADR-0074
/// §4), the same as in a file.
#[test]
fn put_writes_every_kind_of_item_at_once() {
    let root = init("putall");
    let code = "struct Box2 { v: i64 }\n\nimpl Box2 {\n    #[effects(pure)]\n    fn get(self: &Box2) -> i64 { self.v }\n}\n\ntrait Val {\n    #[effects(pure)]\n    fn val(self: &Self) -> i64;\n}\n\nimpl Val for Box2 {\n    #[effects(pure)]\n    fn val(self: &Box2) -> i64 { self.v }\n}\n\n#[effects(pure)]\nfn one() -> i64 { 1 }";
    let c = commit(&root, &[put("app", code)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(
        c.changed,
        [
            "<app::Box2 as app::Val>",
            "<app::Box2 as app::Val>::val",
            "app::Box2",
            "app::Box2::get",
            "app::Val",
            "app::one"
        ]
    );
    let before = snapshot(&root);
    let c = commit(
        &root,
        &[put(
            "app",
            "use core::dec;\n#[effects(pure)]\nfn two() -> i64 { 2 }",
        )],
    )
    .unwrap();
    assert!(
        !c.ok && c.diagnostics.contains("E0214"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);
}

/// `remove` takes the five addresses (ADR-0074 §3): a type goes with its
/// `impl` block, the last function of a block takes the block, a trait impl
/// goes by its address; a trait method, a trait impl function and a unit
/// are refused with what to do instead.
#[test]
fn remove_takes_every_item_address() {
    let root = init("rmall");
    let code = "struct Box2 { v: i64 }\n\nimpl Box2 {\n    #[effects(pure)]\n    fn get(self: &Box2) -> i64 { self.v }\n\n    #[effects(pure)]\n    fn twice(self: &Box2) -> i64 { self.v.saturating_mul(2) }\n}\n\ntrait Val {\n    #[effects(pure)]\n    fn val(self: &Self) -> i64;\n}\n\nimpl Val for Box2 {\n    #[effects(pure)]\n    fn val(self: &Box2) -> i64 { self.v }\n}";
    assert!(commit(&root, &[put("app", code)]).unwrap().ok);
    let before = snapshot(&root);
    for (def, want) in [
        ("app::Val::val", "is a method of trait `app::Val`"),
        (
            "<app::Box2 as app::Val>::val",
            "is a function of a trait impl",
        ),
        ("app", "is a unit; remove a unit with a `remove_unit` op"),
        (
            "<app::Box2 as app::Nope>",
            "no trait impl `<app::Box2 as app::Nope>`",
        ),
        ("app::Box2::nope", "no definition `app::Box2::nope`"),
    ] {
        let c = commit(&root, &[rm(def)]).unwrap();
        assert!(
            !c.ok && c.diagnostics.contains(want),
            "{def}: {}",
            c.diagnostics
        );
        assert_eq!(snapshot(&root), before);
    }
    // A trait still implemented is the checker's to refuse (E0305).
    let c = commit(&root, &[rm("app::Val")]).unwrap();
    assert!(!c.ok, "{}", c.diagnostics);
    // One function of the block; then the impl, the trait, the type.
    let c = commit(&root, &[rm("app::Box2::twice")]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(c.removed, ["app::Box2::twice"]);
    let c = commit(
        &root,
        &[
            rm("<app::Box2 as app::Val>"),
            rm("app::Val"),
            rm("app::Box2"),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(
        c.removed,
        [
            "<app::Box2 as app::Val>",
            "<app::Box2 as app::Val>::val",
            "app::Box2",
            "app::Box2::get",
            "app::Val"
        ]
    );
    assert!(!unit_text(&root, "app").contains("Box2"));
}

/// No temporary files are left behind by a commit.
#[test]
fn commit_leaves_no_temporary_files() {
    let root = init("tmpfiles");
    let c = commit(
        &root,
        &[
            Op::Unit {
                path: "extra".into(),
                summary: "추가".into(),
            },
            put("extra", "#[effects(pure)]\nfn one() -> i64 { 1 }"),
            Op::RemoveUnit { path: "app".into() },
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let mut names: Vec<String> = fs::read_dir(root.join("src"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["core.bmb", "extra.bmb"]);
}

// ---------- rename and expect (ADR-0059 §5, backlog T1-10) ----------

/// Functions, a spec fn, a type with an `impl` and a trait impl, a trait
/// with a bound and a default body, referred to from another unit by full
/// path and from their own by short name.
const RICH: &str = r#"
#[summary("기반")]
mod geo {
    spec fn pos(x: i64) -> bool { x > 0 }

    trait Area {
        #[effects(pure)]
        fn area(self: &Self) -> i64;

        #[effects(pure)]
        fn twice(self: &Self) -> i64 {
            Area::area(self).saturating_mul(2)
        }
    }

    struct Pt { x: i64, y: i64 }

    impl Pt {
        #[effects(pure)]
        fn new(x: i64, y: i64) -> Pt {
            Pt { x: x, y: y }
        }
    }

    impl Area for Pt {
        #[effects(pure)]
        fn area(self: &Pt) -> i64 {
            self.x.saturating_mul(self.y)
        }
    }

    #[effects(pure)]
    #[uses(pos)]
    fn dec(x: i64) -> i64
        requires pos(x),
    {
        x - 1
    }

    #[effects(pure)]
    fn size<T: Copy + Area>(t: T) -> i64 {
        Area::twice(&t)
    }

    #[effects(pure)]
    fn origin() -> Pt {
        Pt::new(0, 0)
    }
}

#[summary("사용처")]
mod app {
    #[effects(pure)]
    #[uses(geo::pos)]
    fn use_dec() -> i64 { geo::dec(3) }

    #[effects(pure)]
    fn corner() -> geo::Pt {
        let p: geo::Pt = geo::Pt::new(1, 2);
        geo::Pt { x: p.y, y: p.x }
    }

    #[effects(pure)]
    fn big(p: geo::Pt) -> i64 {
        geo::size::<geo::Pt>(p)
    }
}
"#;

fn init_rich(name: &str) -> PathBuf {
    let root = temp(name);
    import(&root, &[("base.bmb".into(), RICH.into())]);
    root
}

/// `bmb hash` rows of the workspace: path -> its identity and hashes.
fn ids(root: &Path) -> BTreeMap<String, String> {
    let (ok, rows) = bmb(&["hash", arg(root)]);
    assert!(ok, "{rows}");
    rows.lines()
        .map(|l| {
            let path = l.split('"').nth(3).unwrap().to_string();
            (path, l[l.find(",\"id\"").unwrap()..].to_string())
        })
        .collect()
}

fn rename(def: &str, to: &str) -> Op {
    Op::Rename {
        def: def.into(),
        to: to.into(),
    }
}

#[test]
fn rename_rewrites_references_and_keeps_every_identity() {
    // (target, new name, how the rows of the old paths move)
    type Case<'a> = (&'a str, &'a str, &'a [(&'a str, &'a str)]);
    let cases: &[Case] = &[
        ("geo::dec", "decrement", &[("geo::dec", "geo::decrement")]),
        ("geo::pos", "positive", &[("geo::pos", "geo::positive")]),
        ("geo::Pt::new", "make", &[("geo::Pt::new", "geo::Pt::make")]),
        ("geo::size", "measure", &[("geo::size", "geo::measure")]),
        (
            "geo::Pt",
            "Point",
            &[
                ("geo::Pt::new", "geo::Point::new"),
                ("geo::Pt", "geo::Point"),
                (
                    "<geo::Pt as geo::Area>::area",
                    "<geo::Point as geo::Area>::area",
                ),
            ],
        ),
        (
            "geo::Area",
            "Surface",
            &[
                (
                    "<geo::Pt as geo::Area>::area",
                    "<geo::Pt as geo::Surface>::area",
                ),
                ("geo::Area::twice", "geo::Surface::twice"),
            ],
        ),
    ];
    for (k, (def, to, moves)) in cases.iter().enumerate() {
        let root = init_rich(&format!("rename{k}"));
        let before = ids(&root);
        let c = commit(&root, &[rename(def, to)]).unwrap();
        assert!(c.ok, "{def}: {}", c.diagnostics);
        let after = ids(&root);
        let mut want = before.clone();
        for (from, into) in *moves {
            let v = want.remove(*from).unwrap();
            want.insert(into.to_string(), v);
        }
        assert_eq!(after, want, "{def} -> {to}");
        let text: String = snapshot(&root).into_values().collect();
        let old = def.rsplit("::").next().unwrap();
        let word = |w: &str| {
            text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|x| x == w)
        };
        assert!(!word(old), "{def}: `{old}` is still written\n{text}");
        assert!(word(to), "{def}\n{text}");
        // The program still verifies (a rename changes no proof).
        let (ok, output) = bmb(&["verify", arg(&root)]);
        assert!(ok, "{def}\n{output}");
    }
}

#[test]
fn rename_that_would_collide_or_capture_is_rejected() {
    let root = init_rich("rename_bad");
    let before = snapshot(&root);
    for (def, to) in [
        // Collides with another definition of the unit.
        ("geo::dec", "origin"),
        ("geo::Pt", "Area"),
        // Not a name, not a definition, a trait method.
        ("geo::dec", "a::b"),
        ("geo::nope", "x"),
        ("geo::Area::area", "size2"),
    ] {
        let c = commit(&root, &[rename(def, to)]).unwrap();
        assert!(!c.ok, "{def} -> {to} committed");
        assert_eq!(snapshot(&root), before, "{def} -> {to}");
    }
    // Capture: `app` calls the built-in `print_i64` by its short name; a
    // definition of `app` with the same signature renamed to it would take
    // that call over (it still checks, but `show` would mean another thing).
    let c = commit(
        &root,
        &[
            put(
                "app",
                "#[effects(io)]\nfn show() -> () {\n    print_i64(1);\n}",
            ),
            put("app", "#[effects(io)]\nfn emit(v: i64) -> () {}"),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let before = snapshot(&root);
    let c = commit(&root, &[rename("app::emit", "print_i64")]).unwrap();
    assert!(!c.ok, "a capturing rename committed");
    assert!(
        c.diagnostics
            .contains("would change what other names refer to (app::show)"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);
    // Another unit's name is no capture: other units name `geo::dec` by
    // full path.
    let c = commit(
        &root,
        &[
            put("app", "#[effects(pure)]\nfn dec() -> i64 { 7 }"),
            rename("geo::Pt::new", "dec"),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
}

#[test]
fn expect_rejects_a_changed_definition_only() {
    let root = init_rich("expect");
    let id_of = |root: &Path, p: &str| {
        let row = ids(root)[p].clone();
        row.split('"').nth(3).unwrap().to_string()
    };
    let dec = id_of(&root, "geo::dec");
    let expect = |def: &str, id: &str| Op::Expect {
        def: def.into(),
        id: id.into(),
    };
    // Another writer renames `pos` and edits `origin`: `dec` keeps its
    // identity, so an edit made against it still commits.
    let c = commit(
        &root,
        &[
            rename("geo::pos", "positive"),
            put(
                "geo",
                "#[effects(pure)]\nfn origin() -> Pt {\n    Pt::new(1, 1)\n}",
            ),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(id_of(&root, "geo::dec"), dec);
    let c = commit(
        &root,
        &[
            expect("geo::dec", &dec),
            put("app", "#[effects(pure)]\nfn two() -> i64 { geo::dec(3) }"),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    // Another writer edits `dec` itself: the stale expectation is refused
    // and nothing is written.
    let c = commit(
        &root,
        &[put(
            "geo",
            "#[effects(pure)]\n#[uses(positive)]\nfn dec(x: i64) -> i64\n    requires positive(x),\n{\n    x - 1 + 0\n}",
        )],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let before = snapshot(&root);
    let c = commit(
        &root,
        &[
            expect("geo::dec", &dec),
            put("app", "#[effects(pure)]\nfn three() -> i64 { geo::dec(4) }"),
        ],
    )
    .unwrap();
    assert!(!c.ok);
    assert!(
        c.diagnostics.contains("changed since it was read"),
        "{}",
        c.diagnostics
    );
    assert_eq!(snapshot(&root), before);
    // No such definition.
    let c = commit(&root, &[expect("geo::gone", &dec)]).unwrap();
    assert!(
        !c.ok && c.diagnostics.contains("no definition"),
        "{}",
        c.diagnostics
    );
}

#[test]
fn rename_follows_an_invariant_predicate() {
    // `#[invariant(p)]` names its predicate as `#[uses]` does (ADR-0060 §1):
    // renaming the predicate rewrites the attribute and keeps every identity.
    const SRC: &str = r#"
#[summary("불변식")]
mod iv {
    #[invariant(ordered)]
    struct Span { lo: i64, hi: i64 }

    spec fn ordered(s: &Span) -> bool { s.lo <= s.hi }

    #[effects(pure)]
    #[uses(ordered)]
    fn width(s: Span) -> i64 {
        if s.hi - s.lo > 9 { 9 } else { s.hi - s.lo }
    }
}
"#;
    let root = temp("rename_inv");
    import(&root, &[("base.bmb".into(), SRC.into())]);
    let before = ids(&root);
    let c = commit(&root, &[rename("iv::ordered", "sorted_pair")]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let mut want = before.clone();
    let v = want.remove("iv::ordered").unwrap();
    want.insert("iv::sorted_pair".into(), v);
    assert_eq!(ids(&root), want);
    let text: String = snapshot(&root).into_values().collect();
    assert!(text.contains("#[invariant(sorted_pair)]"), "{text}");
    assert!(!text.contains("ordered"), "{text}");
}

/// Every corpus program can be written through transactions alone (ADR-0074
/// §7): in an empty workspace, a `unit` op per unit and a `put` per item, in
/// one commit, give the same normal-form files as importing the program.
/// The commit also names each trait impl at the address the checker uses
/// for its functions (`<A>` beside every `<A>::f`).
#[test]
fn every_corpus_program_round_trips_through_put() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests");
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in ["verify/pass", "run", "check/ok"] {
        for e in fs::read_dir(corpus.join(dir)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "bmb") {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut seen_impls = 0;
    for path in files {
        let text = fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let units = normal_units(&[(name, text)]);
        let mut ops = Vec::new();
        for (name, unit) in &units {
            let summary = unit
                .lines()
                .next()
                .and_then(|l| l.strip_prefix("#[summary(\"")?.strip_suffix("\")]"))
                .unwrap_or_else(|| panic!("{}: {unit}", path.display()))
                .replace("\\\"", "\"");
            ops.push(Op::Unit {
                path: name.clone(),
                summary,
            });
            // The unit's body, split at its top-level items (normal form:
            // items at four spaces, one blank line between them).
            let open = unit.find("{\n").unwrap() + 2;
            let body = &unit[open..unit.trim_end().len() - 1];
            let mut item = String::new();
            for line in body.lines() {
                let top = line.starts_with("    ") && !line.starts_with("     ");
                let starts = top && (line.starts_with("    #[") || !item.trim_end().ends_with(']'));
                if starts && !item.trim().is_empty() && item.ends_with("\n\n") {
                    ops.push(put(name, item.trim()));
                    item.clear();
                }
                item.push_str(line);
                item.push('\n');
            }
            if !item.trim().is_empty() {
                ops.push(put(name, item.trim()));
            }
        }
        let root = temp(&format!(
            "roundtrip-{}",
            path.file_stem().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        let c = commit(&root, &ops).unwrap();
        assert!(c.ok, "{}: {}", path.display(), c.diagnostics);
        let got: Vec<String> = snapshot(&root).into_values().collect();
        let want: Vec<String> = units.values().cloned().collect();
        assert_eq!(got, want, "{}", path.display());
        for row in &c.changed {
            if let Some(head) = row.strip_prefix('<').and_then(|r| r.rsplit_once(">::")) {
                seen_impls += 1;
                let addr = format!("<{}>", head.0);
                assert!(
                    c.changed.contains(&addr),
                    "{}: {row} without {addr}",
                    path.display()
                );
            }
        }
        let _ = fs::remove_dir_all(&root);
    }
    assert!(seen_impls > 0, "the corpus has trait impls");
}

/// Two types with the same fields have one identity (ADR-0059 §1: a type's
/// text is its structure and its trait `impl`s; backlog T1-28, ADR-0082).
/// The tools that read identities still tell them apart: `expect` names a
/// path, so a change to one type leaves an expectation on the other true;
/// `rename` compares the identities as a multiset, so renaming one of two
/// equal types commits and both identities stay.
#[test]
fn equal_types_share_an_identity_and_the_tools_tell_them_apart() {
    const TWO: &str = r#"#[summary("first")]
mod geo {
    struct Point { x: i64, y: i64 }

    #[effects(pure)]
    fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

#[summary("second")]
mod arrs {
    struct Point { x: i64, y: i64 }

    #[effects(pure)]
    fn corner() -> Point {
        Point { x: 1, y: 1 }
    }
}
"#;
    let root = temp("equal-types");
    import(&root, &[("two.bmb".into(), TWO.into())]);
    let id_of = |p: &str| ids(&root)[p].split('"').nth(3).unwrap().to_string();
    let (geo, arrs) = (id_of("geo::Point"), id_of("arrs::Point"));
    assert_eq!(geo, arrs, "same fields, same identity");
    // A rename of one: the multiset of identities is the same, so it commits.
    let c = commit(&root, &[rename("geo::Point", "Pos")]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_eq!(id_of("geo::Pos"), arrs);
    assert_eq!(id_of("arrs::Point"), arrs);
    // A change to one: an expectation on the other still holds, one on the
    // changed path does not.
    let c = commit(
        &root,
        &[
            put("geo", "struct Pos { x: i64, y: i64, z: i64 }"),
            put(
                "geo",
                "#[effects(pure)]\nfn origin() -> Pos {\n    Pos { x: 0, y: 0, z: 0 }\n}",
            ),
        ],
    )
    .unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    assert_ne!(id_of("geo::Pos"), arrs);
    let expect = |def: &str, id: &str| Op::Expect {
        def: def.into(),
        id: id.into(),
    };
    let c = commit(&root, &[expect("arrs::Point", &arrs)]).unwrap();
    assert!(c.ok, "{}", c.diagnostics);
    let c = commit(&root, &[expect("geo::Pos", &geo)]).unwrap();
    assert!(!c.ok);
    let _ = fs::remove_dir_all(&root);
}
