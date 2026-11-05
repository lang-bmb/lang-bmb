//! The MCP server's front-end cache (backlog 3A-03) and single-definition
//! proof for `surface` (backlog 3A-04) change no answer. Through the CLI
//! (backlog 4A-04): a long-lived `bmb mcp --stats` process, a new process
//! per call, and `bmb map`/`bmb query`.

mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};

use common::jsonl::{num_field, str_field};

fn bmb() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bmb"))
}

/// `s` as a JSON string.
fn q(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bmb-mcp-cache-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn msg(tool: &str, args: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/call\",\"params\":{{\"name\":\"{tool}\",\"arguments\":{args}}}}}"
    )
}

/// The front-end cache's counts `--stats` writes after each reply.
#[derive(Debug, Clone, Copy, Default)]
struct Stats {
    parsed: i64,
    reused: i64,
    checks: i64,
}

/// A long-lived `bmb mcp --stats` process.
struct Server {
    child: Child,
    input: Option<ChildStdin>,
    out: BufReader<ChildStdout>,
    err: BufReader<ChildStderr>,
    stats: Stats,
}

impl Server {
    fn new(root: &Path) -> Server {
        let mut child = bmb()
            .args(["mcp", "--stats"])
            .arg(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Server {
            input: child.stdin.take(),
            out: BufReader::new(child.stdout.take().unwrap()),
            err: BufReader::new(child.stderr.take().unwrap()),
            child,
            stats: Stats::default(),
        }
    }

    /// The whole response as JSON text.
    fn call(&mut self, tool: &str, args: &str) -> String {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{}", msg(tool, args)).unwrap();
        input.flush().unwrap();
        let mut line = String::new();
        self.out.read_line(&mut line).unwrap();
        let mut st = String::new();
        self.err.read_line(&mut st).unwrap();
        let n = |k: &str| num_field(&st, k).unwrap_or_else(|| panic!("no {k} in {st:?}"));
        self.stats = Stats {
            parsed: n("parsed"),
            reused: n("reused"),
            checks: n("checks"),
        };
        line.trim_end().to_string()
    }

    fn front_stats(&self) -> Stats {
        self.stats
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        drop(self.input.take());
        let _ = self.child.wait();
    }
}

/// The response of a new `bmb mcp` process to one call.
fn fresh(root: &Path, tool: &str, args: &str) -> String {
    let mut child = bmb()
        .arg("mcp")
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", msg(tool, args)).unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout)
        .unwrap()
        .trim_end()
        .to_string()
}

/// Text and error flag of a response.
fn text(resp: &str) -> (String, bool) {
    let t = str_field(resp, "text").unwrap_or_else(|| panic!("no content: {resp}"));
    (t, resp.contains("\"isError\":true"))
}

/// Commits one transaction through `s`; panics unless it commits.
fn commit(s: &mut Server, steps: &[(&str, String)]) {
    let (t, err) = text(&s.call("tx_begin", "{}"));
    assert!(!err, "{t}");
    let id = num_field(&t, "tx").unwrap();
    for (tool, args) in steps {
        let (t, err) = text(&s.call(tool, &format!("{{\"tx\":{id},{args}}}")));
        assert!(!err, "{t}");
    }
    let (t, err) = text(&s.call("tx_commit", &format!("{{\"tx\":{id}}}")));
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
        let a = long.call(tool, &args);
        let b = fresh(root, tool, &args);
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

    // A body change in one unit: only that file is parsed again (the
    // counts after the commit's reply, before any read).
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
    drop(other);
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
    drop(long);
    let _ = std::fs::remove_dir_all(&root);
}

/// `bmb` with `args`; its stdout, which must be a success.
fn run(args: &[&str]) -> String {
    let out = bmb().args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success(), "{args:?}: {text}");
    text
}

/// `surface DEF` proves DEF alone (backlog 3A-04), with the row the
/// whole-program proof gives: without a cache, with a cold cache and with
/// a warm one. A failing definition keeps its full count of failed
/// obligations.
#[test]
fn surface_proves_one_definition_with_the_same_row() {
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
    drop(s);
    let ws = root.to_str().unwrap();
    let map = run(&["map", "L1", ws]);
    let mut lines = map.lines();
    let header = lines.next().unwrap();
    // The L1 row of each definition (whole-program proof).
    let rows: Vec<(String, String)> = lines
        .map(|l| (str_field(l, "path").unwrap(), l.to_string()))
        .collect();
    assert_eq!(rows.len(), 7);
    let bad = rows.iter().find(|(p, _)| p == "lib::bad").unwrap();
    assert!(bad.1.contains("\"failed\":2"), "{}", bad.1);

    let cold = temp_root("surface-cold");
    for (path, row) in &rows {
        let want = format!("{header}\n{row}\n");
        assert_eq!(
            run(&["query", "surface", path, "--", ws]),
            want,
            "{path} uncached"
        );
        // Cold: nothing of `path` is cached yet.
        let dir = cold.join(path.replace("::", "_"));
        let d = dir.to_str().unwrap();
        let cached = || run(&["query", "--cache", d, "surface", path, "--", ws]);
        assert_eq!(cached(), want, "{path} cold");
        // Only `path` was proved: the incremental cache (`defs`) holds at
        // most its entry.
        if let Ok(t) = std::fs::read_to_string(dir.join("defs")) {
            for l in t.lines().skip(2) {
                assert!(l.starts_with(&format!("P {path} ")), "{path}: {l}");
            }
        }
        assert_eq!(cached(), want, "{path} warm");
    }
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&cold);
}
