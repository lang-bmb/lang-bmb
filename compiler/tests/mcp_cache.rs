//! The MCP server's front-end cache (backlog 3A-03) and single-definition
//! proof for `surface` (backlog 3A-04) change no answer.

use std::path::{Path, PathBuf};

use bmb::json::{self, Value};
use bmb::mcp::Server;

fn q(s: &str) -> String {
    Value::Str(s.to_string()).to_json()
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bmb-mcp-cache-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn msg(tool: &str, args: &str) -> Value {
    json::parse(&format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{{\"name\":\"{tool}\",\"arguments\":{args}}}}}"
    ))
    .unwrap()
}

/// The whole response as JSON text.
fn call(s: &mut Server, tool: &str, args: &str) -> String {
    s.handle(&msg(tool, args)).unwrap().to_json()
}

/// Text and error flag of a response.
fn text(resp: &str) -> (String, bool) {
    let v = json::parse(resp).unwrap();
    let r = v.get("result").unwrap();
    let Some(Value::Array(items)) = r.get("content") else {
        panic!("no content: {resp}")
    };
    (
        items[0].get("text").unwrap().as_str().unwrap().to_string(),
        r.get("isError") == Some(&Value::Bool(true)),
    )
}

/// Commits one transaction through `s`; panics unless it commits.
fn commit(s: &mut Server, steps: &[(&str, String)]) {
    let (t, err) = text(&call(s, "tx_begin", "{}"));
    assert!(!err, "{t}");
    let id = json::parse(&t)
        .unwrap()
        .get("tx")
        .unwrap()
        .as_i64()
        .unwrap();
    for (tool, args) in steps {
        let (t, err) = text(&call(s, tool, &format!("{{\"tx\":{id},{args}}}")));
        assert!(!err, "{t}");
    }
    let (t, err) = text(&call(s, "tx_commit", &format!("{{\"tx\":{id}}}")));
    assert!(!err, "{t}");
}

fn unit(path: &str, summary: &str) -> (&'static str, String) {
    (
        "tx_unit",
        format!("\"path\":{},\"summary\":{}", q(path), q(summary)),
    )
}

fn put(unit: &str, code: &str) -> (&'static str, String) {
    (
        "tx_put",
        format!("\"unit\":{},\"code\":{}", q(unit), q(code)),
    )
}

const SPECS: &str = "spec fn small(x: i64) -> bool { x >= 0 && x < 1000 }\n\nspec fn pos(r: i64) -> bool { r >= 0 && r < 2000 }";
const DEC: &str = "#[effects(pure)]\n#[uses(small, pos)]\nfn dec(x: i64) -> (r: i64)\n    requires small(x),\n    ensures pos(r),\n{\n    if x > 0 { x - 1 } else { x }\n}";
const DEC2: &str = "#[effects(pure)]\n#[uses(small, pos)]\nfn dec(x: i64) -> (r: i64)\n    requires small(x),\n    ensures pos(r),\n{\n    if x > 1 { x - 1 } else { x }\n}";
const BAD: &str =
    "#[effects(pure)]\nfn bad(x: i64) -> i64 {\n    let a: i64 = x + 1;\n    a * 2\n}";
const REC: &str = "#[effects(pure)]\n#[uses(small, pos)]\nfn rec(n: i64) -> (r: i64)\n    requires small(n),\n    ensures pos(r),\n    decreases n,\n{\n    if n <= 0 { 0 } else { rec(n - 1) }\n}";
const ZERO: &str = "#[effects(pure)]\nfn zero(a: &mut [i64]) -> () {\n    let mut i: usize = 0;\n    while i < a.len()\n        invariant i <= a.len(),\n        decreases a.len() - i,\n    {\n        a[i] = 0;\n        i = i + 1;\n    }\n}";
const MAIN: &str = "#[effects(io)]\nfn main() -> () {\n    let mut xs: [i64; 2] = [5, 6];\n    lib::zero(&mut xs);\n    print_i64(lib::dec(5));\n}";

/// Every read tool, with arguments for the definitions in `defs`.
fn reads(defs: &[&str]) -> Vec<(&'static str, String)> {
    let mut v: Vec<(&'static str, String)> = vec![
        ("map", "{}".into()),
        ("map", "{\"level\":\"L0\"}".into()),
        ("map", "{\"level\":\"L1\"}".into()),
        ("map", "{\"level\":\"L2\"}".into()),
        ("map", "{\"level\":\"L9\"}".into()),
        ("query", "{\"name\":\"units\"}".into()),
        ("query", "{\"name\":\"status\"}".into()),
        ("query", "{\"name\":\"duplicates\"}".into()),
        ("query", "{\"name\":\"effects\"}".into()),
        ("query", "{\"name\":\"find\",\"args\":[\"dec\"]}".into()),
        (
            "query",
            "{\"name\":\"contracts\",\"args\":[\"lib::pos\"]}".into(),
        ),
        ("query", "{\"name\":\"surface\"}".into()),
        (
            "query",
            "{\"name\":\"surface\",\"args\":[\"no::such\"]}".into(),
        ),
        ("query", "{\"name\":\"nonsense\"}".into()),
        ("verify", "{}".into()),
        ("build", "{\"name\":\"app\"}".into()),
    ];
    for d in defs {
        let d = q(d);
        for (name, extra) in [
            ("surface", ""),
            ("slice", ""),
            ("edges", ""),
            ("edges", ",\"in\""),
            ("edges", ",\"both\",\"2\""),
            ("impact", ""),
            ("impact", ",\"body\""),
            ("effects", ""),
            ("find", ""),
        ] {
            v.push((
                "query",
                format!("{{\"name\":\"{name}\",\"args\":[{d}{extra}]}}"),
            ));
        }
    }
    v
}

/// Sends every read to the long-lived server and to a new server per call;
/// the responses must be the same bytes.
fn compare(long: &mut Server, root: &Path, defs: &[&str]) -> usize {
    let mut n = 0;
    for (tool, args) in reads(defs) {
        let a = call(long, tool, &args);
        let b = call(&mut Server::new(root), tool, &args);
        assert_eq!(a, b, "{tool} {args}");
        n += 1;
    }
    n
}

/// A session of mixed reads and commits (by this server, by another
/// writer, and by editing the files directly) answered by one long-lived
/// server and by a fresh server per call: every response is the same.
#[test]
fn long_lived_server_answers_like_a_fresh_one() {
    bmb::with_stack(|| {
        let root = temp_root("session");
        let mut long = Server::new(&root);
        compare(&mut long, &root, &["lib::dec"]);

        commit(
            &mut long,
            &[
                unit("lib", "도구"),
                put("lib", SPECS),
                put("lib", DEC),
                put("lib", BAD),
                put("lib", REC),
                put("lib", ZERO),
                unit("app", "진입점"),
                put("app", MAIN),
            ],
        );
        let defs = [
            "lib::dec",
            "lib::bad",
            "lib::rec",
            "lib::zero",
            "lib::small",
            "lib::pos",
            "app::main",
        ];
        compare(&mut long, &root, &defs);
        let after_first = long.front_stats();

        // A body change in one unit: only that file is parsed again.
        commit(&mut long, &[put("lib", DEC2)]);
        compare(&mut long, &root, &defs);
        let s = long.front_stats();
        assert_eq!(s.parsed - after_first.parsed, 1, "{s:?}");
        assert_eq!(s.checks - after_first.checks, 1, "{s:?}");

        // Another writer adds a unit that sorts first: every file moves.
        let mut other = Server::new(&root);
        commit(
            &mut other,
            &[
                unit("aaa", "앞"),
                put("aaa", "#[effects(pure)]\nfn one() -> i64 { 1 }"),
            ],
        );
        compare(&mut long, &root, &defs);

        // A direct edit that breaks the syntax, then its repair.
        let lib = root.join("src/lib.bmb");
        let good = std::fs::read_to_string(&lib).unwrap();
        std::fs::write(&lib, format!("{good}\nfn broken(")).unwrap();
        compare(&mut long, &root, &defs);
        std::fs::write(&lib, &good).unwrap();
        compare(&mut long, &root, &defs);

        // A unit removed.
        commit(
            &mut long,
            &[("tx_remove_unit", "\"path\":\"aaa\"".to_string())],
        );
        compare(&mut long, &root, &defs);

        let s = long.front_stats();
        assert!(s.reused > 0, "{s:?}");
        let _ = std::fs::remove_dir_all(&root);
    });
}

fn files(root: &Path) -> Vec<(String, String)> {
    bmb::workspace::read_dir_sources(root).unwrap()
}

/// The L1 row of each definition in `map L1` (whole-program proof).
fn l1_rows(map: &str) -> Vec<(String, String)> {
    map.lines()
        .skip(1)
        .map(|l| {
            let v = json::parse(l).unwrap();
            (
                v.get("path").unwrap().as_str().unwrap().to_string(),
                l.to_string(),
            )
        })
        .collect()
}

/// `surface DEF` proves DEF alone (backlog 3A-04), with the row the
/// whole-program proof gives: without a cache, with a cold cache and with
/// a warm one. A failing definition keeps its full count of failed
/// obligations.
#[test]
fn surface_proves_one_definition_with_the_same_row() {
    bmb::with_stack(|| {
        let root = temp_root("surface");
        let mut s = Server::new(&root);
        commit(
            &mut s,
            &[
                unit("lib", "도구"),
                put("lib", SPECS),
                put("lib", DEC),
                put("lib", BAD),
                put("lib", REC),
                put("lib", ZERO),
                unit("app", "진입점"),
                put("app", MAIN),
            ],
        );
        let f = files(&root);
        let req = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let map = bmb::map_request(&f, &req(&["map", "L1"])).unwrap();
        let rows = l1_rows(&map);
        assert_eq!(rows.len(), 7);
        let bad = rows.iter().find(|(p, _)| p == "lib::bad").unwrap();
        assert!(bad.1.contains("\"failed\":2"), "{}", bad.1);

        let cold = temp_root("surface-cold");
        for (path, row) in &rows {
            let r = req(&["query", "surface", path]);
            let want = format!("{}\n{row}\n", bmb::map::HEADER);
            assert_eq!(bmb::map_request(&f, &r).unwrap(), want, "{path} uncached");
            // Cold: nothing of `path` is cached yet.
            let dir = cold.join(path.replace("::", "_"));
            assert_eq!(
                bmb::map_request_cached(&f, &r, Some(&dir)).unwrap(),
                want,
                "{path} cold"
            );
            // Only `path` was proved: the incremental cache holds at most
            // its entry.
            if let Ok(t) = std::fs::read_to_string(dir.join(bmb::incremental::FILE)) {
                for l in t.lines().skip(2) {
                    assert!(l.starts_with(&format!("P {path} ")), "{path}: {l}");
                }
            }
            assert_eq!(
                bmb::map_request_cached(&f, &r, Some(&dir)).unwrap(),
                want,
                "{path} warm"
            );
        }
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&cold);
    });
}
