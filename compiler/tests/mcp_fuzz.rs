//! MCP session fuzzing (backlog 1D-23): random sequences of tool calls
//! against `bmb mcp DIR` over stdio, with valid and invalid arguments,
//! interleaved, stale, closed and unknown transactions, reads and proofs in
//! between, notifications and malformed lines. Claims:
//!
//! - the server never dies or stalls: every request gets one response within
//!   a time limit, notifications get none, and it exits cleanly at the end;
//! - every response is JSON-RPC 2.0 with the request's id and exactly one of
//!   `result` / `error`; tool results carry a text item and `isError`;
//! - transaction bookkeeping follows a model: `tx_begin` ids count up, a
//!   closed or unknown id is "no open transaction", a commit after another
//!   commit changed the workspace is refused as stale;
//! - only a successful commit changes `src/` (snapshots before and after every
//!   call), and after it the workspace checks with no diagnostic and every file
//!   is already in normal form.
//!
//! `BMB_MCP_FUZZ_SESSIONS` and `BMB_MCP_FUZZ_CALLS` raise the work for longer
//! runs; `BMB_MCP_FUZZ_SEED` picks other sequences.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use bmb::json::{self, Value};

mod common;

use common::scalar::Rng;

fn env(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn q(s: &str) -> String {
    Value::Str(s.to_string()).to_json()
}

fn pick<'a>(rng: &mut Rng, xs: &[&'a str]) -> &'a str {
    xs[rng.below(xs.len())]
}

const UNITS: &[&str] = &["a", "b", "c", "a::s", "b::t"];
const BAD_UNITS: &[&str] = &[
    "",
    "../x",
    "a/b",
    "fn",
    "1a",
    "a::",
    "::a",
    "a::fn",
    "zz",
    "a::s::deep",
    ".hidden",
];
const SUMMARIES: &[&str] = &[
    "요약",
    "plain",
    "",
    "quote \" inside",
    "back\\slash",
    "line\nbreak",
    "tab\there",
    "// not a comment",
    "}",
];
const DEFS: &[&str] = &[
    "a::f", "a::g", "a::p", "b::f", "b::h", "c::m", "a::s::f", "b::t::k", "a::nope", "", "a", "f",
    "a::fn", "../f",
];

/// Definitions to `put`: valid ones (that check alone or once their callees
/// exist), and broken ones.
fn code(rng: &mut Rng) -> String {
    let n = pick(rng, &["f", "g", "h", "k", "m"]);
    let callee = pick(rng, &["a::f", "b::f", "a::s::f", "f", "g", "b::t::k"]);
    let lit = rng.below(100);
    match rng.below(16) {
        0..=3 => format!("#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ x.wrapping_add({lit}) }}"),
        4 => format!("#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ {callee}(x) }}"),
        5 => format!(
            "spec fn p(x: i64) -> bool {{ x > {lit} }}\n\n#[effects(pure)]\n#[uses(p)]\nfn {n}(x: i64) -> i64\n    requires p(x),\n{{\n    x - 1\n}}"
        ),
        6 => "#[effects(io)]\nfn main() -> () { print_i64(a::f(1)) }".to_string(),
        7 => format!("#[effects(pure)]\nfn {n}(x: i64) -> bool {{ x }}"),
        8 => format!("fn {n}(x: i64) -> i64 {{ x }}"),
        9 => format!("#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ x"),
        10 => "#[effects(pure)]\nfn f() -> i64 { 1 }\n}\nmod escape {".to_string(),
        11 => {
            "#[summary(\"x\")]\nmod inner {\n#[effects(pure)]\nfn f() -> i64 { 1 }\n}".to_string()
        }
        12 => String::new(),
        13 => "// only a comment".to_string(),
        14 => format!(
            "#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ x }}\n\n#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ x }}"
        ),
        _ => {
            let bytes = b"{}()[];:,<>=!&|*+-/%#\"\\\n x0fnmod";
            (0..rng.below(30))
                .map(|_| bytes[rng.below(bytes.len())] as char)
                .collect()
        }
    }
}

struct Server {
    child: Child,
    lines: Receiver<String>,
    root: PathBuf,
}

impl Server {
    fn start(root: &Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .args(["mcp", root.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        let (tx, lines) = channel();
        std::thread::spawn(move || {
            for line in out.lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Server {
            child,
            lines,
            root: root.to_path_buf(),
        }
    }

    fn send(&mut self, text: &str) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{text}").expect("server closed its input");
        stdin.flush().unwrap();
    }

    fn recv(&self, limit: u64, ctx: &str) -> Value {
        let line = self
            .lines
            .recv_timeout(Duration::from_secs(limit))
            .unwrap_or_else(|e| panic!("no response within {limit}s ({e}) to {ctx}"));
        json::parse(&line).unwrap_or_else(|e| panic!("response is not JSON ({e}): {line}"))
    }

    /// Nothing may arrive (after a notification).
    fn quiet(&self, ctx: &str) {
        if let Ok(line) = self.lines.recv_timeout(Duration::from_millis(20)) {
            panic!("unexpected output after {ctx}: {line}");
        }
    }
}

/// JSON-RPC 2.0 shape of one response to request `id`.
fn check_rpc(v: &Value, id: &Value, ctx: &str) {
    assert_eq!(
        v.get("jsonrpc").and_then(Value::as_str),
        Some("2.0"),
        "{ctx}"
    );
    assert_eq!(v.get("id"), Some(id), "{ctx}: id");
    let (r, e) = (v.get("result"), v.get("error"));
    assert!(r.is_some() != e.is_some(), "{ctx}: one of result/error");
    if let Some(e) = e {
        assert!(e.get("code").and_then(Value::as_i64).is_some(), "{ctx}");
        assert!(e.get("message").and_then(Value::as_str).is_some(), "{ctx}");
    }
}

/// Every file under `src/`, any name (temporary files would show up too).
fn snapshot(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Ok(rd) = std::fs::read_dir(root.join("src")) {
        for e in rd {
            let p = e.unwrap().path();
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            out.insert(name, std::fs::read_to_string(&p).unwrap());
        }
    }
    out
}

fn as_files(snap: &BTreeMap<String, String>) -> Vec<(String, String)> {
    snap.iter().map(|(n, t)| (n.clone(), t.clone())).collect()
}

/// A committed workspace: no diagnostic, and every file already in normal form.
fn check_workspace(snap: &BTreeMap<String, String>, ctx: &str) {
    for name in snap.keys() {
        assert!(
            name.ends_with(".bmb") && !name.starts_with('.'),
            "{ctx}: stray file {name}"
        );
    }
    let files = as_files(snap);
    let out = bmb::check_sources(&files);
    assert!(
        out.ok && out.diagnostics.is_empty(),
        "{ctx}: committed workspace does not check:\n{}\n{snap:#?}",
        out.diagnostics
    );
    let units = bmb::format_sources(&files).unwrap();
    let normal: BTreeMap<String, String> = units
        .into_iter()
        .map(|(u, t)| (format!("{u}.bmb"), t))
        .collect();
    assert_eq!(
        &normal, snap,
        "{ctx}: committed files are not in normal form"
    );
}

struct Model {
    next_tx: i64,
    /// Open transactions and the workspace they started from.
    open: BTreeMap<i64, BTreeMap<String, String>>,
    closed: BTreeSet<i64>,
}

/// Returns (commits, commits that changed files, transactions begun).
fn session(seed: u64, calls: u64) -> (usize, usize, usize) {
    let root = std::env::temp_dir().join(format!("bmb-mcp-fuzz-{}-{seed}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut srv = Server::start(&root);
    let mut rng = Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1);
    let mut model = Model {
        next_tx: 1,
        open: BTreeMap::new(),
        closed: BTreeSet::new(),
    };
    let mut next_id = 1i64;
    // A valid transaction played a call at a time, interleaved with random
    // calls, so commits that change files are common (`{tx}` is its id).
    let mut plan: Vec<(&str, String)> = Vec::new();
    let mut plan_tx = 0i64;
    let mut commits = 0;
    let mut changing = 0;
    let mut log: Vec<String> = Vec::new();

    let init = r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"fuzz","version":"0"}}"#;
    srv.send(&format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":0,\"method\":\"initialize\",\"params\":{init}}}"
    ));
    check_rpc(
        &srv.recv(30, "initialize"),
        &Value::Num("0".into()),
        "initialize",
    );

    for step in 0..calls {
        let before = snapshot(&root);
        // Transport-level noise now and then.
        match rng.below(40) {
            0 => {
                let line = pick(&mut rng, &["{", "[1,2", "nonsense", "\"str\"", "{\"a\":}"]);
                log.push(format!("raw {line}"));
                srv.send(line);
                let v = srv.recv(30, line);
                check_rpc(&v, &Value::Null, line);
                assert!(v.get("error").is_some(), "{line}");
                continue;
            }
            1 => {
                let line = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
                srv.send(line);
                srv.quiet(line);
                continue;
            }
            2 => {
                let line = format!(
                    "{{\"jsonrpc\":\"2.0\",\"id\":\"s{step}\",\"method\":\"{}\",\"params\":{{}}}}",
                    pick(&mut rng, &["ping", "tools/list", "no/such", "tools/call"])
                );
                srv.send(&line);
                let v = srv.recv(30, &line);
                check_rpc(&v, &Value::Str(format!("s{step}")), &line);
                continue;
            }
            _ => {}
        }

        // Transaction id: an open one, a closed one, an unknown one, or a bad value.
        let open: Vec<i64> = model.open.keys().copied().collect();
        let closed: Vec<i64> = model.closed.iter().copied().collect();
        let (mut tx_json, mut tx_val): (String, Option<i64>) = match rng.below(10) {
            0..=6 if !open.is_empty() => {
                let t = open[rng.below(open.len())];
                (t.to_string(), Some(t))
            }
            7 if !closed.is_empty() => {
                let t = closed[rng.below(closed.len())];
                (t.to_string(), Some(t))
            }
            8 => {
                let t = 1000 + rng.below(5) as i64;
                (t.to_string(), Some(t))
            }
            9 => (
                pick(&mut rng, &["\"1\"", "1.5", "null", "[]"]).to_string(),
                None,
            ),
            _ => ("999".to_string(), Some(999)),
        };
        let unit = if rng.chance(85) {
            pick(&mut rng, UNITS)
        } else {
            pick(&mut rng, BAD_UNITS)
        };
        // With nothing open, begin more often so transactions get long enough.
        let roll = if model.open.is_empty() && rng.chance(40) {
            0
        } else {
            rng.below(100)
        };
        let planned = (!plan.is_empty() && rng.chance(70)).then(|| plan.remove(0));
        let (tool, args) = match roll {
            _ if planned.is_some() => {
                let (tool, args) = planned.unwrap();
                tx_json = plan_tx.to_string();
                tx_val = Some(plan_tx);
                (tool, args.replace("{tx}", &tx_json))
            }
            0..=9 => {
                if plan.is_empty() && rng.chance(50) {
                    plan_tx = model.next_tx;
                    let u = pick(&mut rng, &["a", "b", "c"]);
                    plan.push((
                        "tx_unit",
                        format!("{{\"tx\":{{tx}},\"path\":{},\"summary\":\"s\"}}", q(u)),
                    ));
                    for _ in 0..1 + rng.below(3) {
                        let n = pick(&mut rng, &["f", "g", "h", "k", "m"]);
                        let c = format!(
                            "#[effects(pure)]\nfn {n}(x: i64) -> i64 {{ x.wrapping_add({}) }}",
                            rng.below(9)
                        );
                        plan.push((
                            "tx_put",
                            format!("{{\"tx\":{{tx}},\"unit\":{},\"code\":{}}}", q(u), q(&c)),
                        ));
                    }
                    if rng.chance(30) {
                        plan.push((
                            "tx_remove_unit",
                            format!("{{\"tx\":{{tx}},\"path\":{}}}", q(pick(&mut rng, UNITS))),
                        ));
                    }
                    plan.push(("tx_commit", "{\"tx\":{tx}}".to_string()));
                }
                ("tx_begin", "{}".to_string())
            }
            10..=21 => (
                "tx_unit",
                format!(
                    "{{\"tx\":{tx_json},\"path\":{},\"summary\":{}}}",
                    q(unit),
                    q(pick(&mut rng, SUMMARIES))
                ),
            ),
            22..=41 => (
                "tx_put",
                format!(
                    "{{\"tx\":{tx_json},\"unit\":{},\"code\":{}}}",
                    q(unit),
                    q(&code(&mut rng))
                ),
            ),
            42..=47 => (
                "tx_remove",
                format!("{{\"tx\":{tx_json},\"def\":{}}}", q(pick(&mut rng, DEFS))),
            ),
            48..=51 => (
                "tx_remove_unit",
                format!("{{\"tx\":{tx_json},\"path\":{}}}", q(unit)),
            ),
            52..=67 => ("tx_commit", format!("{{\"tx\":{tx_json}}}")),
            68..=71 => ("tx_abort", format!("{{\"tx\":{tx_json}}}")),
            72..=73 => (
                // Missing or ill-typed arguments.
                pick(
                    &mut rng,
                    &["tx_put", "tx_unit", "tx_remove", "query", "build"],
                ),
                pick(
                    &mut rng,
                    &[
                        "{}",
                        "{\"tx\":1}",
                        "{\"unit\":5,\"code\":[]}",
                        "{\"name\":7}",
                    ],
                )
                .to_string(),
            ),
            74..=86 => {
                let name = pick(
                    &mut rng,
                    &[
                        "units",
                        "find",
                        "edges",
                        "slice",
                        "contracts",
                        "impact",
                        "duplicates",
                        "effects",
                        "nope",
                        "surface",
                        "status",
                    ],
                );
                let arg = pick(&mut rng, DEFS);
                (
                    "query",
                    format!("{{\"name\":{},\"args\":[{}]}}", q(name), q(arg)),
                )
            }
            87..=92 => (
                "map",
                format!(
                    "{{\"level\":{}}}",
                    q(pick(&mut rng, &["L0", "L1", "L2", "all", "L9"]))
                ),
            ),
            93..=94 => ("verify", "{}".to_string()),
            95 => (
                "build",
                format!("{{\"name\":{}}}", q(pick(&mut rng, &["app", "../x", ""]))),
            ),
            96 => ("teleport", "{}".to_string()),
            97 => ("tx_commit", "{}".to_string()),
            _ => ("tx_abort", "{\"tx\":-1}".to_string()),
        };

        let id = next_id;
        next_id += 1;
        let line = if rng.chance(3) {
            // Arguments that are not an object.
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/call\",\"params\":{{\"name\":{},\"arguments\":[1]}}}}",
                q(tool)
            )
        } else {
            format!(
                "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"tools/call\",\"params\":{{\"name\":{},\"arguments\":{args}}}}}",
                q(tool)
            )
        };
        let ctx = format!(
            "seed {seed} step {step}: {line}\nafter:\n{}",
            log.join("\n")
        );
        srv.send(&line);
        let limit = if matches!(tool, "verify" | "build" | "query" | "map") {
            300
        } else {
            60
        };
        let v = srv.recv(limit, &ctx);
        check_rpc(&v, &Value::Num(id.to_string()), &ctx);
        let r = v
            .get("result")
            .unwrap_or_else(|| panic!("{ctx}: tools/call is never an RPC error"));
        let text = match r.get("content") {
            Some(Value::Array(items)) if items.len() == 1 => items[0]
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_else(|| panic!("{ctx}: no text"))
                .to_string(),
            _ => panic!("{ctx}: no content"),
        };
        let is_error = match r.get("isError") {
            Some(Value::Bool(b)) => *b,
            _ => panic!("{ctx}: no isError"),
        };
        log.push(format!(
            "{line}\n  -> {}{}",
            if is_error { "error: " } else { "" },
            text.lines().last().unwrap_or("")
        ));
        let after = snapshot(&root);
        let well_formed = !line.contains("\"arguments\":[1]");

        // The transaction model.
        let tx_ops = ["tx_unit", "tx_put", "tx_remove", "tx_remove_unit"];
        let uses_tx = tx_ops.contains(&tool) || matches!(tool, "tx_commit" | "tx_abort");
        if tool == "tx_begin" {
            // `tx_begin` takes no arguments, so their shape does not matter.
            assert!(!is_error, "{ctx}: {text}");
            assert_eq!(text, format!("{{\"tx\":{}}}", model.next_tx), "{ctx}");
            model.open.insert(model.next_tx, before.clone());
            model.next_tx += 1;
        } else if uses_tx && well_formed && args.starts_with(&format!("{{\"tx\":{tx_json}")) {
            match tx_val {
                Some(t) if model.open.contains_key(&t) => {
                    if tool == "tx_commit" {
                        let base = model.open.remove(&t).unwrap();
                        model.closed.insert(t);
                        if base != before {
                            assert!(
                                is_error && text.contains("workspace changed"),
                                "{ctx}: stale commit accepted: {text}"
                            );
                        } else if !is_error {
                            assert!(text.contains("\"committed\":true"), "{ctx}: {text}");
                        }
                    } else if tool == "tx_abort" {
                        model.open.remove(&t);
                        model.closed.insert(t);
                        assert!(!is_error && text.contains("\"aborted\":true"), "{ctx}");
                    } else if !is_error {
                        assert!(text.starts_with(&format!("{{\"tx\":{t},\"ops\":")), "{ctx}");
                    } else {
                        // Only a missing argument rejects an op on an open transaction.
                        assert!(text.starts_with("missing `"), "{ctx}: {text}");
                    }
                }
                Some(t) => {
                    assert!(is_error, "{ctx}: transaction {t} is not open: {text}");
                    let id = t as u64;
                    assert!(
                        text.contains(&format!("no open transaction {id}"))
                            || text.starts_with("missing `"),
                        "{ctx}: {text}"
                    );
                }
                None => {
                    assert!(is_error && text.contains("missing `tx`"), "{ctx}: {text}");
                }
            }
        }

        // Only a successful commit changes the workspace.
        let committed = tool == "tx_commit" && !is_error;
        if committed {
            commits += 1;
            changing += usize::from(before != after);
            check_workspace(&after, &ctx);
        } else {
            assert_eq!(before, after, "{ctx}: workspace changed without a commit");
        }
    }

    drop(srv.child.stdin.take());
    let status = srv.child.wait().unwrap();
    assert!(status.success(), "seed {seed}: server exit {status}");
    let _ = std::fs::remove_dir_all(&srv.root);
    (commits, changing, model.next_tx as usize - 1)
}

#[test]
fn random_mcp_sessions_keep_the_workspace_sound() {
    let sessions = env("BMB_MCP_FUZZ_SESSIONS", 20);
    let calls = env("BMB_MCP_FUZZ_CALLS", 100);
    let seed = env("BMB_MCP_FUZZ_SEED", 0);
    let (mut commits, mut changing, mut txs) = (0, 0, 0);
    for s in 0..sessions {
        let (c, ch, t) = session(seed.wrapping_mul(1_000_003).wrapping_add(s), calls);
        commits += c;
        changing += ch;
        txs += t;
    }
    eprintln!(
        "mcp fuzz: {sessions} sessions x {calls} calls, {txs} transactions, {commits} commits ({changing} changed files)"
    );
    assert!(
        changing > 0,
        "no commit changed files: the generator is too weak"
    );
}
