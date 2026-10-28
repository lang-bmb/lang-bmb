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

/// Unit and definition paths become file names: anything but `::`-separated
/// identifiers is rejected and nothing is written (backlog 1D-15).
#[test]
fn invalid_paths_are_rejected_and_write_nothing() {
    let outer = temp("traversal");
    let root = outer.join("ws");
    let units = bmb::format_sources(&[("base.bmb".into(), BASE.into())]).unwrap();
    write_units(&root, &units).unwrap();
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

    // `write_units` itself refuses a name that is not an identifier.
    let mut evil = BTreeMap::new();
    evil.insert("../../evil".to_string(), "x".to_string());
    assert!(write_units(&root, &evil).is_err());
    assert_eq!(listing(&outer), outer_before);
    assert_eq!(snapshot(&root), before);

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

/// `put` takes definitions only: nested units, code that closes the wrapper,
/// and code without a definition are errors, not silently dropped (1D-19).
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
            c.diagnostics.contains("only `fn`") || c.diagnostics.contains("no definition"),
            "{code:?}: {}",
            c.diagnostics
        );
        assert_eq!(snapshot(&root), before);
    }
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
    let units = bmb::format_sources(&[("base.bmb".into(), RICH.into())]).unwrap();
    write_units(&root, &units).unwrap();
    root
}

/// `bmb hash` rows of the workspace: path -> its identity and hashes.
fn ids(root: &Path) -> BTreeMap<String, String> {
    bmb::hash_sources(&read_dir_sources(root).unwrap())
        .unwrap()
        .lines()
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
        let v = bmb::verify_sources(&read_dir_sources(&root).unwrap());
        assert!(v.ok, "{def}\n{}", v.output);
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
fn rename_and_expect_parse_from_json() {
    let ops = bmb::workspace::parse_ops(
        r#"[{"op":"rename","def":"a::f","to":"g"},{"op":"expect","def":"a::g","id":"sha256:00"}]"#,
    )
    .unwrap();
    assert!(matches!(&ops[0], Op::Rename { def, to } if def == "a::f" && to == "g"));
    assert!(matches!(&ops[1], Op::Expect { def, id } if def == "a::g" && id == "sha256:00"));
    assert!(bmb::workspace::parse_ops(r#"[{"op":"rename","def":"a::f"}]"#).is_err());
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
    let units = bmb::format_sources(&[("base.bmb".into(), SRC.into())]).unwrap();
    write_units(&root, &units).unwrap();
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
