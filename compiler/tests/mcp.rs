//! MCP server end to end over stdio (backlog 1D-11).

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use bmb::json::{self, Value};

struct Client {
    child: std::process::Child,
    out: BufReader<std::process::ChildStdout>,
    next: u64,
}

impl Client {
    fn start(root: &std::path::Path) -> Client {
        Client::start_with(root, &[])
    }

    fn start_with(root: &std::path::Path, flags: &[&str]) -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .arg("mcp")
            .args(flags)
            .arg(root.to_str().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        Client {
            child,
            out,
            next: 1,
        }
    }

    fn send(&mut self, text: &str) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{text}").unwrap();
        stdin.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: &str) -> Value {
        let id = self.next;
        self.next += 1;
        self.send(&format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"{method}\",\"params\":{params}}}"
        ));
        let mut line = String::new();
        self.out.read_line(&mut line).unwrap();
        let v = json::parse(&line).unwrap();
        assert_eq!(v.get("id").and_then(Value::as_i64), Some(id as i64));
        v
    }

    /// Calls a tool; returns (text, is_error).
    fn tool(&mut self, name: &str, args: &str) -> (String, bool) {
        let v = self.request(
            "tools/call",
            &format!("{{\"name\":\"{name}\",\"arguments\":{args}}}"),
        );
        let r = v.get("result").expect("result");
        let text = match r.get("content") {
            Some(Value::Array(items)) => items[0]
                .get("text")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            _ => panic!("no content"),
        };
        (text, r.get("isError") == Some(&Value::Bool(true)))
    }
}

fn q(s: &str) -> String {
    Value::Str(s.to_string()).to_json()
}

#[test]
fn agent_session_over_mcp() {
    let root = std::env::temp_dir().join(format!("bmb-mcp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut c = Client::start(&root);

    let init = c.request("initialize", r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}"#);
    assert_eq!(
        init.get("result")
            .unwrap()
            .get("serverInfo")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str(),
        Some("bmb")
    );
    c.send(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);

    let tools = c.request("tools/list", "{}");
    let Value::Array(list) = tools.get("result").unwrap().get("tools").unwrap() else {
        panic!()
    };
    let names: Vec<&str> = list
        .iter()
        .map(|t| t.get("name").unwrap().as_str().unwrap())
        .collect();
    for n in [
        "query",
        "map",
        "verify",
        "build",
        "tx_begin",
        "tx_put",
        "tx_commit",
        "tx_abort",
    ] {
        assert!(names.contains(&n), "{n}");
    }

    // Write a program through a transaction.
    let (t, err) = c.tool("tx_begin", "{}");
    assert!(!err && t.contains("\"tx\":1"));
    c.tool(
        "tx_unit",
        &format!("{{\"tx\":1,\"path\":\"calc\",\"summary\":{}}}", q("계산")),
    );
    let bad = "#[effects(pure)]\nfn twice(x: i64) -> bool { x }";
    c.tool(
        "tx_put",
        &format!("{{\"tx\":1,\"unit\":\"calc\",\"code\":{}}}", q(bad)),
    );
    let (t, err) = c.tool("tx_commit", "{\"tx\":1}");
    assert!(
        err && t.contains("E0401") && t.contains("\"committed\":false"),
        "{t}"
    );
    assert!(
        !root.join("src").exists(),
        "rejected commit must write nothing"
    );

    c.tool("tx_begin", "{}");
    c.tool(
        "tx_unit",
        &format!("{{\"tx\":2,\"path\":\"calc\",\"summary\":{}}}", q("계산")),
    );
    let good = "#[effects(pure)]\nfn twice(x: i64) -> i64 { x.wrapping_mul(2) }";
    c.tool(
        "tx_put",
        &format!("{{\"tx\":2,\"unit\":\"calc\",\"code\":{}}}", q(good)),
    );
    c.tool(
        "tx_unit",
        &format!("{{\"tx\":2,\"path\":\"app\",\"summary\":{}}}", q("진입점")),
    );
    let main = "#[effects(io)]\nfn main() -> () { print_i64(calc::twice(21)) }";
    c.tool(
        "tx_put",
        &format!("{{\"tx\":2,\"unit\":\"app\",\"code\":{}}}", q(main)),
    );
    let (t, err) = c.tool("tx_commit", "{\"tx\":2}");
    assert!(!err, "{t}");
    assert!(
        t.contains("\"changed\":[\"app::main\",\"calc::twice\"]"),
        "{t}"
    );

    // Read back through the maps only.
    let (t, _) = c.tool("query", "{\"name\":\"surface\",\"args\":[\"calc::twice\"]}");
    assert!(
        t.contains("\"signature\":\"fn twice(x: i64) -> i64\""),
        "{t}"
    );
    let (t, _) = c.tool(
        "query",
        "{\"name\":\"edges\",\"args\":[\"calc::twice\",\"in\"]}",
    );
    assert!(t.contains("\"from\":\"app::main\""), "{t}");

    // Prove and build; the binary prints 42.
    let (t, err) = c.tool("verify", "{}");
    assert!(!err, "{t}");
    let (t, err) = c.tool("build", "{\"name\":\"app\"}");
    assert!(!err, "{t}");
    let run = Command::new(root.join("target/app")).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42\n");

    // A stale transaction cannot overwrite newer work.
    c.tool("tx_begin", "{}");
    c.tool("tx_begin", "{}");
    c.tool(
        "tx_put",
        &format!(
            "{{\"tx\":4,\"unit\":\"calc\",\"code\":{}}}",
            q("#[effects(pure)]\nfn one() -> i64 { 1 }")
        ),
    );
    let (_, err) = c.tool("tx_commit", "{\"tx\":4}");
    assert!(!err);
    let (t, err) = c.tool("tx_commit", "{\"tx\":3}");
    assert!(err && t.contains("workspace changed"), "{t}");

    // Unknown methods are JSON-RPC errors.
    let v = c.request("no/such", "{}");
    assert!(v.get("error").is_some());

    drop(c.child.stdin.take());
    c.child.wait().unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("bmb-mcp-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn snapshot(root: &std::path::Path) -> Vec<(String, String)> {
    if root.join("src").is_dir() {
        bmb::workspace::read_dir_sources(root).unwrap()
    } else {
        Vec::new()
    }
}

/// Commits one transaction of (tool, args) steps; panics unless it commits.
fn commit_ok(c: &mut Client, steps: &[(&str, String)]) {
    let (t, err) = c.tool("tx_begin", "{}");
    assert!(!err, "{t}");
    let id = json::parse(&t)
        .unwrap()
        .get("tx")
        .unwrap()
        .as_i64()
        .unwrap();
    for (tool, args) in steps {
        let (t, err) = c.tool(tool, &format!("{{\"tx\":{id},{args}}}"));
        assert!(!err, "{t}");
    }
    let (t, err) = c.tool("tx_commit", &format!("{{\"tx\":{id}}}"));
    assert!(!err, "{t}");
}

/// Error paths of the tools (backlog 1D-19): each is a tool error
/// (`isError: true`) with a reason, and none changes the workspace.
#[test]
fn tool_errors_over_mcp() {
    let root = temp_root("errors");
    let mut c = Client::start(&root);

    // Unknown tool, missing arguments, unknown or closed transactions.
    let (t, err) = c.tool("teleport", "{}");
    assert!(err && t.contains("unknown tool"), "{t}");
    let (t, err) = c.tool("query", "{}");
    assert!(err && t.contains("missing `name`"), "{t}");
    let (t, err) = c.tool("tx_put", "{\"unit\":\"a\",\"code\":\"\"}");
    assert!(err && t.contains("missing `tx`"), "{t}");
    let (t, err) = c.tool("tx_commit", "{}");
    assert!(err && t.contains("missing `tx`"), "{t}");
    let (t, err) = c.tool("build", "{}");
    assert!(err && t.contains("missing `name`"), "{t}");
    for tool in ["tx_unit", "tx_put", "tx_remove", "tx_remove_unit"] {
        let (t, err) = c.tool(
            tool,
            "{\"tx\":99,\"path\":\"a\",\"summary\":\"s\",\"unit\":\"a\",\"code\":\"\",\"def\":\"a::f\"}",
        );
        assert!(err && t.contains("no open transaction 99"), "{tool}: {t}");
    }
    let (t, err) = c.tool("tx_unit", "{\"tx\":99,\"path\":\"a\"}");
    assert!(err && t.contains("missing `summary`"), "{t}");
    for tool in ["tx_commit", "tx_abort"] {
        let (t, err) = c.tool(tool, "{\"tx\":99}");
        assert!(err && t.contains("no open transaction 99"), "{tool}: {t}");
    }

    // Abort discards; the id is closed afterwards.
    let (t, _) = c.tool("tx_begin", "{}");
    assert!(t.contains("\"tx\":1"), "{t}");
    c.tool("tx_unit", "{\"tx\":1,\"path\":\"gone\",\"summary\":\"s\"}");
    let (t, err) = c.tool("tx_abort", "{\"tx\":1}");
    assert!(!err && t.contains("\"aborted\":true"), "{t}");
    let (t, err) = c.tool("tx_commit", "{\"tx\":1}");
    assert!(err && t.contains("no open transaction 1"), "{t}");
    assert!(!root.join("src").exists());

    // A program without `main`, then one that is not fully proved.
    let lib = "#[effects(pure)]\nfn inc(x: i64) -> i64 { x.wrapping_add(1) }";
    commit_ok(
        &mut c,
        &[
            ("tx_unit", "\"path\":\"lib\",\"summary\":\"s\"".into()),
            ("tx_put", format!("\"unit\":\"lib\",\"code\":{}", q(lib))),
        ],
    );
    let (t, err) = c.tool("build", "{\"name\":\"app\"}");
    assert!(err && t.contains("build refused: no `main`"), "{t}");
    for bad in ["../x", ".x", "a/b", "a\\\\b", "", ".."] {
        let (t, err) = c.tool("build", &format!("{{\"name\":\"{bad}\"}}"));
        assert!(err && t.contains("plain file name"), "{bad}: {t}");
    }
    assert!(!root.join("target").exists());
    assert!(!root.join("x").exists());

    let main = "#[effects(io)]\nfn main() -> () { print_i64(lib::dec(1)) }";
    let dec = "#[effects(pure)]\nfn dec(x: i64) -> i64 { x - 1 }";
    commit_ok(
        &mut c,
        &[
            ("tx_put", format!("\"unit\":\"lib\",\"code\":{}", q(dec))),
            ("tx_unit", "\"path\":\"app\",\"summary\":\"s\"".into()),
            ("tx_put", format!("\"unit\":\"app\",\"code\":{}", q(main))),
        ],
    );
    let (t, err) = c.tool("verify", "{}");
    assert!(
        err && t.contains("E0803") && t.contains("\"status\":\"failed\""),
        "{t}"
    );
    let (t, err) = c.tool("build", "{\"name\":\"app\"}");
    assert!(err && t.contains("build refused: not fully proved"), "{t}");
    assert!(!root.join("target/app").exists());

    // Maps by level.
    let (l0, err) = c.tool("map", "{\"level\":\"L0\"}");
    assert!(!err && l0.contains("\"path\":\"lib::dec\""), "{l0}");
    let (l2, err) = c.tool("map", "{\"level\":\"L2\"}");
    assert!(
        !err && l2.contains("\"from\":\"app::main\",\"to\":\"lib::dec\""),
        "{l2}"
    );
    assert_ne!(l0, l2);
    let (t, err) = c.tool("map", "{\"level\":\"L9\"}");
    assert!(err, "{t}");

    // Removal through transactions: a definition still called is a type
    // error, removing the caller first works, and missing targets are errors.
    let before = snapshot(&root);
    let (t, _) = c.tool("tx_begin", "{}");
    let id = json::parse(&t)
        .unwrap()
        .get("tx")
        .unwrap()
        .as_i64()
        .unwrap();
    c.tool(
        "tx_remove",
        &format!("{{\"tx\":{id},\"def\":\"lib::dec\"}}"),
    );
    let (t, err) = c.tool("tx_commit", &format!("{{\"tx\":{id}}}"));
    assert!(
        err && t.contains("E0304") && t.contains("\"committed\":false"),
        "{t}"
    );
    assert_eq!(snapshot(&root), before);
    let (t, _) = c.tool("tx_begin", "{}");
    let id = json::parse(&t)
        .unwrap()
        .get("tx")
        .unwrap()
        .as_i64()
        .unwrap();
    c.tool(
        "tx_remove",
        &format!("{{\"tx\":{id},\"def\":\"lib::nope\"}}"),
    );
    c.tool(
        "tx_remove_unit",
        &format!("{{\"tx\":{id},\"path\":\"nope\"}}"),
    );
    let (t, err) = c.tool("tx_commit", &format!("{{\"tx\":{id}}}"));
    assert!(
        err && t.contains("no definition `lib::nope`") && t.contains("no unit `nope`"),
        "{t}"
    );
    assert_eq!(snapshot(&root), before);
    commit_ok(
        &mut c,
        &[
            ("tx_remove_unit", "\"path\":\"app\"".into()),
            ("tx_remove", "\"def\":\"lib::dec\"".into()),
        ],
    );
    let files = snapshot(&root);
    assert_eq!(files.len(), 1);
    assert!(files[0].0.ends_with("lib.bmb") && !files[0].1.contains("fn dec"));

    // Path traversal and bad `put` code are rejected at commit (1D-15).
    let before = snapshot(&root);
    for steps in [
        vec![(
            "tx_unit",
            "\"path\":\"../../evil\",\"summary\":\"s\"".to_string(),
        )],
        vec![("tx_put", format!("\"unit\":\"lib\",\"code\":{}", q("")))],
        vec![(
            "tx_put",
            format!(
                "\"unit\":\"lib\",\"code\":{}",
                q("#[summary(\"x\")]\nmod inner {\n#[effects(pure)]\nfn f() -> i64 { 1 }\n}")
            ),
        )],
    ] {
        let (t, _) = c.tool("tx_begin", "{}");
        let id = json::parse(&t)
            .unwrap()
            .get("tx")
            .unwrap()
            .as_i64()
            .unwrap();
        for (tool, args) in &steps {
            c.tool(tool, &format!("{{\"tx\":{id},{args}}}"));
        }
        let (t, err) = c.tool("tx_commit", &format!("{{\"tx\":{id}}}"));
        assert!(err && t.contains("\"committed\":false"), "{t}");
        assert_eq!(snapshot(&root), before);
    }
    assert!(!root.parent().unwrap().join("evil.bmb").exists());

    // The server survives malformed and deeply nested input (1A-11).
    c.send(&"[".repeat(200_000));
    let mut line = String::new();
    c.out.read_line(&mut line).unwrap();
    assert!(
        line.contains("-32700") && line.contains("nesting"),
        "{line}"
    );
    let v = c.request("ping", "{}");
    assert!(v.get("result").is_some());

    drop(c.child.stdin.take());
    assert!(c.child.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&root);
}

/// Valid JSON that is not a request object, and a request without `method`,
/// are answered with JSON-RPC "invalid request" (-32600) instead of silence,
/// which left the client waiting (found by the session fuzzer, 1D-23).
#[test]
fn invalid_requests_get_an_error_response() {
    let root = temp_root("invalid");
    let mut c = Client::start(&root);
    for (line, id) in [
        ("\"str\"", "null"),
        ("42", "null"),
        ("[1,2]", "null"),
        ("[]", "null"),
        ("{\"jsonrpc\":\"2.0\",\"id\":7}", "7"),
        ("{\"jsonrpc\":\"2.0\",\"id\":8,\"method\":3}", "8"),
    ] {
        c.send(line);
        let mut out = String::new();
        c.out.read_line(&mut out).unwrap();
        let v = json::parse(&out).unwrap();
        assert_eq!(v.get("id").unwrap().to_json(), id, "{line}: {out}");
        assert_eq!(
            v.get("error")
                .and_then(|e| e.get("code"))
                .and_then(Value::as_i64),
            Some(-32600),
            "{line}: {out}"
        );
    }
    // A notification without `method` still gets no answer; the next request does.
    c.send("{\"jsonrpc\":\"2.0\"}");
    let v = c.request("ping", "{}");
    assert!(v.get("result").is_some());
    drop(c.child.stdin.take());
    assert!(c.child.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&root);
}

/// Threat model of the MCP surface (backlog 2D-20, docs/mcp-threat-model.md):
/// a workspace made by someone else cannot make the tools read or write
/// outside it through symbolic links, transactions are capped, and text from
/// the workspace stays data inside one response line.
#[cfg(unix)]
#[test]
fn untrusted_workspace_cannot_reach_outside() {
    use std::os::unix::fs::symlink;
    let base = temp_root("threats");
    let outside = base.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("victim.txt"), "secret\n").unwrap();
    std::fs::write(outside.join("keep.bmb"), "precious\n").unwrap();
    let unit = "#[summary(\"a\")]\nmod a {\n    #[effects(pure)]\n    fn f(x: i64) -> i64 {\n        x\n    }\n}\n";
    let put = (
        "tx_put",
        format!(
            "\"unit\":\"a\",\"code\":{}",
            q("#[effects(pure)]\nfn g(x: i64) -> i64 {\n    x\n}")
        ),
    );
    let untouched = |what: &str| {
        assert_eq!(
            std::fs::read_to_string(outside.join("victim.txt")).unwrap(),
            "secret\n",
            "{what}"
        );
        assert_eq!(
            std::fs::read_to_string(outside.join("keep.bmb")).unwrap(),
            "precious\n",
            "{what}"
        );
    };

    // A planted temporary file is replaced, not followed (it overwrote the
    // target before the fix).
    let ws = base.join("planted");
    std::fs::create_dir_all(ws.join("src")).unwrap();
    std::fs::write(ws.join("src/a.bmb"), unit).unwrap();
    symlink(outside.join("victim.txt"), ws.join("src/.a.bmb.tmp")).unwrap();
    let mut c = Client::start(&ws);
    commit_ok(&mut c, std::slice::from_ref(&put));
    untouched("planted temporary file");
    assert!(
        !std::fs::symlink_metadata(ws.join("src/a.bmb"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(
        std::fs::read_to_string(ws.join("src/a.bmb"))
            .unwrap()
            .contains("fn g(")
    );

    // At most MAX_OPEN_TXS open transactions (each holds a copy of the files).
    for _ in 0..bmb::mcp::MAX_OPEN_TXS {
        assert!(!c.tool("tx_begin", "{}").1);
    }
    let (t, err) = c.tool("tx_begin", "{}");
    assert!(err && t.contains("too many open transactions"), "{t}");
    assert!(!c.tool("tx_abort", "{\"tx\":2}").1);
    assert!(!c.tool("tx_begin", "{}").1);

    // Workspace text is data: a summary that looks like a protocol message
    // stays inside one response line.
    let (t, err) = c.tool("query", "{\"name\":\"units\"}");
    assert!(!err, "{t}");
    let tricky = "x\\\"}\\n{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":99}";
    let (t, _) = c.tool(
        "tx_unit",
        &format!("{{\"tx\":3,\"path\":\"b\",\"summary\":\"{tricky}\"}}"),
    );
    assert!(t.contains("\"ops\""), "{t}");
    let _ = c.tool("tx_commit", "{\"tx\":3}");
    let (t, _) = c.tool("query", "{\"name\":\"units\"}");
    assert!(!t.contains("\"id\":99}\n"), "{t}");

    // A commit after another writer changed the files is refused (T10).
    let (t, err) = c.tool("tx_begin", "{}");
    assert!(!err, "{t}");
    let id = json::parse(&t)
        .unwrap()
        .get("tx")
        .unwrap()
        .as_i64()
        .unwrap();
    let before = std::fs::read_to_string(ws.join("src/a.bmb")).unwrap();
    std::fs::write(ws.join("src/a.bmb"), before.replace("fn g(", "fn h(")).unwrap();
    let (t, err) = c.tool("tx_commit", &format!("{{\"tx\":{id}}}"));
    assert!(err && t.contains("workspace changed since tx_begin"), "{t}");
    drop(c);

    // Links in place of the directories and files the tools use.
    let cases: [(&str, &str, &str, &str); 4] = [
        ("src-link", "src", "query", "{\"name\":\"units\"}"),
        ("cache-link", ".bmb-cache", "verify", "{}"),
        ("target-link", "target", "build", "{\"name\":\"x\"}"),
        ("unit-link", "src/b.bmb", "query", "{\"name\":\"units\"}"),
    ];
    for (name, link, tool, args) in cases {
        let ws = base.join(name);
        std::fs::create_dir_all(ws.join("src")).unwrap();
        if link == "src" {
            std::fs::remove_dir(ws.join("src")).unwrap();
            symlink(&outside, ws.join("src")).unwrap();
        } else {
            std::fs::write(ws.join("src/a.bmb"), unit).unwrap();
            let target = if link.ends_with(".bmb") {
                outside.join("keep.bmb")
            } else {
                outside.clone()
            };
            symlink(target, ws.join(link)).unwrap();
        }
        let mut c = Client::start(&ws);
        let (t, err) = c.tool(tool, args);
        assert!(err && t.contains("symbolic link"), "{name}: {t}");
        let (t, err) = c.tool("tx_begin", "{}");
        assert!(err && t.contains("symbolic link"), "{name}: {t}");
        drop(c);
        untouched(name);
    }
    let _ = std::fs::remove_dir_all(&base);
}

/// `write` (backlog 2D-24): one call creates the unit, puts the definitions,
/// commits and proves; a rejected write changes nothing; a proof failure is a
/// tool error after the commit, with its counterexample.
#[test]
fn write_tool_commits_and_proves_in_one_call() {
    let root = temp_root("write");
    // Without the flag the tool list is the frozen one and `write` is unknown.
    let mut plain = Client::start(&root);
    let tools = plain.request("tools/list", "{}").to_json();
    assert!(!tools.contains("\"write\""), "{tools}");
    let (t, err) = plain.tool("write", "{\"unit\":\"calc\"}");
    assert!(err && t.contains("unknown tool"), "{t}");
    let mut c = Client::start_with(&root, &["--write-tool"]);
    c.request("initialize", r#"{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}"#);
    assert!(
        c.request("tools/list", "{}")
            .to_json()
            .contains("\"write\"")
    );

    // A type error: nothing is written.
    let bad = "#[effects(pure)]\nfn twice(x: i64) -> bool { x }";
    let (t, err) = c.tool(
        "write",
        &format!(
            "{{\"unit\":\"calc\",\"summary\":{},\"code\":{}}}",
            q("계산"),
            q(bad)
        ),
    );
    assert!(
        err && t.contains("E0401") && t.contains("\"committed\":false"),
        "{t}"
    );
    assert!(
        snapshot(&root).is_empty(),
        "a rejected write must write nothing"
    );

    // Unit, definitions and proof in one call.
    let good = "spec fn small(x: i64) -> bool { -1000 <= x && x <= 1000 }\n\n#[effects(pure)]\n#[uses(small)]\nfn twice(x: i64) -> i64\n    requires small(x),\n{\n    x + x\n}";
    let (t, err) = c.tool(
        "write",
        &format!(
            "{{\"unit\":\"calc\",\"summary\":{},\"code\":{}}}",
            q("계산"),
            q(good)
        ),
    );
    assert!(!err, "{t}");
    assert!(t.starts_with("{\"committed\":true"), "{t}");
    assert!(
        t.contains("\"def\":\"calc::twice\",\"status\":\"verified\""),
        "{t}"
    );

    // A definition that does not prove is committed, and the call says why.
    let weak = "#[effects(pure)]\nfn thrice(x: i64) -> i64 {\n    x + x + x\n}";
    let (t, err) = c.tool(
        "write",
        &format!("{{\"unit\":\"calc\",\"code\":{}}}", q(weak)),
    );
    assert!(
        err && t.contains("\"committed\":true") && t.contains("counterexample"),
        "{t}"
    );
    assert!(
        snapshot(&root)
            .iter()
            .any(|(_, text)| text.contains("fn thrice"))
    );

    // Removing it in the same kind of call proves the workspace again.
    let (t, err) = c.tool("write", "{\"unit\":\"calc\",\"remove\":[\"calc::thrice\"]}");
    assert!(!err && t.contains("\"removed\":[\"calc::thrice\"]"), "{t}");
    let (t, err) = c.tool("write", "{\"unit\":\"calc\"}");
    assert!(err && t.contains("nothing to write"), "{t}");
    let _ = std::fs::remove_dir_all(&root);
}

/// The file diff surface (`--file-tools`, backlog T1-06, ADR-0073): a
/// unified diff creates, edits and deletes unit files, then the call commits
/// and proves; a diff that does not apply, or a result with a type error or a
/// file holding another unit, writes nothing; the stored file is the normal
/// form and the call says when that differs from the diff's text.
#[test]
fn file_tools_edit_by_unified_diff() {
    let root = temp_root("file-tools");
    for flags in [&[][..], &["--write-tool"][..]] {
        let mut c = Client::start_with(&root, flags);
        let tools = c.request("tools/list", "{}").to_json();
        assert!(!tools.contains("\"edit_file\"") && !tools.contains("\"read_file\""));
        let (t, err) = c.tool("edit_file", "{\"diff\":\"\"}");
        assert!(err && t.contains("unknown tool"), "{t}");
    }
    let mut c = Client::start_with(&root, &["--file-tools"]);
    let tools = c.request("tools/list", "{}").to_json();
    assert!(
        tools.contains("\"edit_file\"") && !tools.contains("\"write\""),
        "{tools}"
    );
    let edit =
        |c: &mut Client, diff: &str| c.tool("edit_file", &format!("{{\"diff\":{}}}", q(diff)));

    // A type error: nothing is written, positions are the diff's file.
    let create = "--- /dev/null\n+++ b/src/calc.bmb\n@@ -0,0 +1,7 @@\n+#[summary(\"calc\")]\n+mod calc {\n+    #[effects(pure)]\n+    fn twice(x: i64) -> bool {\n+        x\n+    }\n+}\n";
    let (t, err) = edit(&mut c, create);
    assert!(
        err && t.contains("E0401") && t.contains("\"file\":\"src/calc.bmb\",\"line\":5"),
        "{t}"
    );
    assert!(
        snapshot(&root).is_empty(),
        "a rejected diff must write nothing"
    );
    // A file must hold the unit its name says.
    let (t, err) = edit(&mut c, &create.replace("b/src/calc.bmb", "b/src/other.bmb"));
    assert!(
        err && t.contains("src/other.bmb must hold exactly one top-level unit, `mod other`"),
        "{t}"
    );

    // Created, committed and proved; the extra blank line is normalized away.
    let good = "--- /dev/null\n+++ b/src/calc.bmb\n@@ -0,0 +1,9 @@\n+#[summary(\"calc\")]\n+mod calc {\n+    #[effects(pure)]\n+    fn twice(x: i64) -> i64 {\n+        let y: i64 = x % 1000;\n+\n+        y + y\n+    }\n+}\n";
    let (t, err) = edit(&mut c, good);
    assert!(!err, "{t}");
    assert!(t.starts_with("{\"committed\":true,\"changed\":[\"calc::twice\"],\"removed\":[],\"reformatted\":[\"src/calc.bmb\"]}"), "{t}");
    assert!(
        t.contains("\"def\":\"calc::twice\",\"status\":\"verified\""),
        "{t}"
    );
    let (file, err) = c.tool("read_file", "{\"path\":\"src/calc.bmb\"}");
    assert!(
        !err && file.contains("        let y: i64 = x % 1000;\n        y + y\n"),
        "{file}"
    );
    let (t, err) = c.tool("read_file", "{\"path\":\"src/nope.bmb\"}");
    assert!(err && t.contains("the files are: src/calc.bmb"), "{t}");

    // A hunk that does not match the file is refused and says why.
    let (t, err) = edit(
        &mut c,
        "--- a/src/calc.bmb\n+++ b/src/calc.bmb\n@@ -5,2 +5,2 @@\n-        y * y\n+        y\n",
    );
    assert!(
        err && t.contains("`        y * y` is not in the file"),
        "{t}"
    );
    // An edit that does not prove is committed, with its counterexample;
    // wrong counts are recounted.
    let (t, err) = edit(
        &mut c,
        "--- a/src/calc.bmb\n+++ b/src/calc.bmb\n@@ -5,99 +5,1 @@\n         let y: i64 = x % 1000;\n-        y + y\n+        y * 9223372036854775807\n",
    );
    assert!(
        err && t.contains("\"committed\":true") && t.contains("counterexample"),
        "{t}"
    );
    assert!(!t.contains("reformatted"), "{t}");
    // Deleting the file removes the unit.
    let (t, err) = edit(&mut c, "--- a/src/calc.bmb\n+++ /dev/null\n");
    assert!(!err && t.contains("\"removed\":[\"calc::twice\"]"), "{t}");
    assert!(snapshot(&root).is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

/// The JSON AST surface (`--ast-tools`, backlog T1-06, ADR-0073): the tree
/// is read at a pointer and edited by a JSON Patch, then committed and
/// proved; a bad patch or tree writes nothing, and diagnostics point into
/// the tree (`at`) instead of a text position.
#[test]
fn ast_tools_edit_by_json_patch() {
    let root = temp_root("ast-tools");
    let mut plain = Client::start_with(&root, &["--write-tool"]);
    assert!(
        !plain
            .request("tools/list", "{}")
            .to_json()
            .contains("\"edit_ast\"")
    );
    let mut c = Client::start_with(&root, &["--ast-tools"]);
    let tools = c.request("tools/list", "{}").to_json();
    assert!(
        tools.contains("\"read_ast\"") && tools.contains("\"edit_ast\""),
        "{tools}"
    );
    let (t, err) = c.tool("read_ast", "{}");
    assert!(!err && t == "{}", "{t}");

    let int = |v: i64| format!("{{\"kind\":\"int\",\"value\":{v}}}");
    let ty = |k: &str| format!("{{\"kind\":\"{k}\"}}");
    let twice = |ret: &str| {
        format!(
            "{{\"kind\":\"fn\",\"effects\":[\"pure\"],\"params\":[{{\"name\":\"x\",\"ty\":{}}}],\"ret\":{{\"ty\":{}}},\"body\":{{\"stmts\":[{{\"kind\":\"let\",\"name\":\"y\",\"ty\":{},\"init\":{{\"kind\":\"binary\",\"op\":\"%\",\"lhs\":{{\"kind\":\"var\",\"name\":\"x\"}},\"rhs\":{}}}}}],\"tail\":{{\"kind\":\"binary\",\"op\":\"+\",\"lhs\":{{\"kind\":\"var\",\"name\":\"y\"}},\"rhs\":{{\"kind\":\"var\",\"name\":\"y\"}}}}}}}}",
            ty("i64"),
            ty(ret),
            ty("i64"),
            int(1000)
        )
    };
    let unit = |f: &str| format!("{{\"summary\":\"calc\",\"fns\":{{\"twice\":{f}}}}}");
    let edit = |c: &mut Client, patch: &str| c.tool("edit_ast", &format!("{{\"patch\":{patch}}}"));

    // A type error: nothing is written, and the diagnostic names the node.
    let (t, err) = edit(
        &mut c,
        &format!(
            "[{{\"op\":\"add\",\"path\":\"/calc\",\"value\":{}}}]",
            unit(&twice("bool"))
        ),
    );
    assert!(
        err && t.contains("\"code\":\"E0401\"")
            && t.contains("\"at\":\"/calc/fns/twice/body/tail\""),
        "{t}"
    );
    assert!(!t.contains("\"line\""), "{t}");
    assert!(
        snapshot(&root).is_empty(),
        "a rejected patch must write nothing"
    );
    // A tree that is not one, and a patch that does not apply.
    let (t, err) = edit(
        &mut c,
        &format!(
            "[{{\"op\":\"add\",\"path\":\"/calc\",\"value\":{}}}]",
            unit(&twice("i65"))
        ),
    );
    assert!(
        err && t.contains("/calc/fns/twice/ret/ty: unknown type kind `i65`"),
        "{t}"
    );
    let (t, err) = edit(&mut c, "[{\"op\":\"remove\",\"path\":\"/calc\"}]");
    assert!(
        err && t.contains("operation 0: remove `/calc`: no such member"),
        "{t}"
    );
    // A keyword where a name goes does not print as itself.
    let (t, err) = edit(
        &mut c,
        &format!(
            "[{{\"op\":\"add\",\"path\":\"/calc\",\"value\":{}}}]",
            unit(&twice("i64")).replace("\"name\":\"y\"", "\"name\":\"loop\"")
        ),
    );
    assert!(err, "{t}");
    assert!(snapshot(&root).is_empty());
    // A tree whose text parses back as another tree says what it becomes:
    // an `if` statement at the end of a block is the block's value.
    let last_if = "{\"kind\":\"fn\",\"effects\":[\"pure\"],\"ret\":{\"ty\":{\"kind\":\"unit\"}},\"body\":{\"stmts\":[{\"kind\":\"expr\",\"expr\":{\"kind\":\"if\",\"cond\":{\"kind\":\"bool\",\"value\":true},\"then\":{}}}]}}";
    let (t, err) = edit(
        &mut c,
        &format!(
            "[{{\"op\":\"add\",\"path\":\"/calc\",\"value\":{}}}]",
            unit(last_if)
        ),
    );
    assert!(
        err && t.starts_with("/calc/fns/twice/body: this part of the tree has no text of its own: it prints and parses back as {\"tail\":{\"kind\":\"if\""),
        "{t}"
    );
    assert!(snapshot(&root).is_empty());

    // Unit, function and proof in one call; the patch may come as JSON text.
    let patch = format!(
        "[{{\"op\":\"add\",\"path\":\"/calc\",\"value\":{}}}]",
        unit(&twice("i64"))
    );
    let (t, err) = c.tool("edit_ast", &format!("{{\"patch\":{}}}", q(&patch)));
    assert!(!err, "{t}");
    assert!(
        t.starts_with("{\"committed\":true,\"changed\":[\"calc::twice\"],\"removed\":[]}"),
        "{t}"
    );
    let (t, err) = c.tool("read_ast", "{\"path\":\"/calc/fns/twice/body/tail/op\"}");
    assert!(!err && t == "\"+\"", "{t}");

    // A node replaced in place; a proof failure points at the node.
    let (t, err) = edit(
        &mut c,
        &format!(
            "[{{\"op\":\"test\",\"path\":\"/calc/fns/twice/body/tail/op\",\"value\":\"+\"}},{{\"op\":\"replace\",\"path\":\"/calc/fns/twice/body/tail/rhs\",\"value\":{}}}]",
            int(9223372036854775807i64)
        ),
    );
    assert!(
        err && t.contains("\"committed\":true")
            && t.contains("\"at\":\"/calc/fns/twice/body/tail\""),
        "{t}"
    );
    let (t, err) = c.tool("verify", "{}");
    assert!(
        err && t.contains("\"at\":\"/calc/fns/twice/body/tail\"") && !t.contains("\"line\""),
        "{t}"
    );
    let (t, err) = edit(
        &mut c,
        "[{\"op\":\"remove\",\"path\":\"/calc/fns/twice\"},{\"op\":\"add\",\"path\":\"/calc/fns/zero\",\"value\":{\"kind\":\"fn\",\"effects\":[\"pure\"],\"ret\":{\"name\":\"r\",\"ty\":{\"kind\":\"u8\"}},\"body\":{\"tail\":{\"kind\":\"int\",\"value\":0}}}}]",
    );
    assert!(!err && t.contains("\"removed\":[\"calc::twice\"]"), "{t}");
    assert!(snapshot(&root)[0].1.contains("fn zero() -> (r: u8) {"));
    let _ = std::fs::remove_dir_all(&root);
}

/// `--compact` (backlog 2D-23, ADR-0068): the same answers in the compact
/// agent view. Every surface keeps its path, status, signature and contract,
/// every definition and edge is listed, no hash or file position is shown,
/// verified definitions become a count, and the default server is unchanged.
#[test]
fn compact_view_carries_the_same_answers() {
    let root = temp_root("compact");
    let mut w = Client::start_with(&root, &["--write-tool"]);
    let code = "spec fn small(x: i64) -> bool { -1000 <= x && x <= 1000 }\n\nspec fn bigger(x: i64, r: i64) -> bool { r > x }\n\n#[effects(pure)]\n#[uses(small, bigger)]\nfn inc(x: i64) -> (r: i64)\n    requires small(x),\n    ensures bigger(x, r),\n{\n    x + 1\n}\n\n#[effects(pure)]\nfn get(a: &[i64], i: usize) -> i64 {\n    a[i]\n}";
    let (t, err) = w.tool(
        "write",
        &format!(
            "{{\"unit\":\"calc\",\"summary\":{},\"code\":{}}}",
            q("계산"),
            q(code)
        ),
    );
    assert!(err && t.contains("\"committed\":true"), "{t}");
    let mut plain = Client::start(&root);
    let mut c = Client::start_with(&root, &["--compact"]);

    // Same tool names; only the read and proof descriptions differ.
    let names = |v: &Value| -> Vec<(String, String)> {
        let Value::Array(list) = v.get("result").unwrap().get("tools").unwrap() else {
            panic!()
        };
        list.iter()
            .map(|t| {
                (
                    t.get("name").unwrap().as_str().unwrap().to_string(),
                    t.get("description").unwrap().as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    let (a, b) = (
        names(&plain.request("tools/list", "{}")),
        names(&c.request("tools/list", "{}")),
    );
    assert_eq!(a.len(), b.len());
    for ((na, da), (nb, db)) in a.iter().zip(&b) {
        assert_eq!(na, nb);
        assert_eq!(
            da != db,
            ["query", "map", "verify"].contains(&na.as_str()),
            "{na}"
        );
    }

    let rows = |text: &str| -> Vec<Value> {
        text.lines()
            .filter_map(|l| json::parse(l).ok())
            .filter(|v| v.get("schema").is_none())
            .collect()
    };
    let strs = |v: &Value, k: &str| -> Vec<String> {
        match v.get(k) {
            Some(Value::Array(xs)) => xs.iter().map(|x| x.as_str().unwrap().to_string()).collect(),
            _ => Vec::new(),
        }
    };
    let (full, _) = plain.tool("map", "{\"level\":\"all\"}");
    let (short, _) = c.tool("map", "{\"level\":\"all\"}");
    assert!(short.len() * 2 < full.len(), "{short}");
    assert!(
        !short.contains("sha256:") && !short.contains(".bmb"),
        "{short}"
    );
    for r in rows(&full) {
        let s = |k: &str| r.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        match (s("level").as_str(), s("kind").as_str()) {
            ("L0", "unit") => assert!(
                short.contains(&format!(
                    "#[summary({})] mod {};",
                    q(&s("summary")),
                    s("path")
                )),
                "{short}"
            ),
            ("L0", _) => assert!(
                short.contains(&format!("{} {}\n", s("def_kind"), s("path"))),
                "{short}"
            ),
            ("L1", _) => {
                let status = match s("status").as_str() {
                    "verified" => ": verified".to_string(),
                    "failed" => format!(
                        ": failed {} of {} obligations",
                        r.get("failed").unwrap().as_i64().unwrap(),
                        r.get("obligations").unwrap().as_i64().unwrap()
                    ),
                    _ => String::new(),
                };
                let head = format!("// {}{status}\n", s("path"));
                let at = short.find(&head).unwrap_or_else(|| panic!("{head}"));
                let surface = &short[at..];
                assert!(surface.contains(&s("signature")), "{surface}");
                for clause in ["requires", "ensures"] {
                    for item in strs(&r, clause) {
                        assert!(surface.contains(&item), "{item}");
                    }
                }
                for u in strs(&r, "uses") {
                    assert!(surface.contains(&u), "{u}");
                }
            }
            ("L2", _) => assert!(
                short.contains(&format!("{} {} {}", s("from"), s("kind"), s("to"))),
                "{short}"
            ),
            _ => {}
        }
    }

    // Verify: diagnostics as they are, failed definitions without proof
    // cost, verified ones as a count.
    let (full, e1) = plain.tool("verify", "{}");
    let (short, e2) = c.tool("verify", "{}");
    assert!(e1 && e2);
    let verified = rows(&full)
        .iter()
        .filter(|r| r.get("status").and_then(Value::as_str) == Some("verified"))
        .count();
    assert!(verified > 0);
    assert!(
        short.ends_with(&format!("{{\"verified\":{verified}}}\n")),
        "{short}"
    );
    for line in full.lines().filter(|l| l.contains("\"severity\"")) {
        assert!(short.contains(line), "{line}");
    }
    assert!(
        short.contains(
            "{\"def\":\"calc::get\",\"status\":\"failed\",\"obligations\":1,\"failed\":1}"
        ),
        "{short}"
    );
    assert!(!short.contains("rlimit"), "{short}");

    // A slice is the code itself, not an escaped JSON string.
    let (t, err) = c.tool("query", "{\"name\":\"slice\",\"args\":[\"calc::inc\"]}");
    assert!(
        !err && t.starts_with("// calc::inc\n") && t.contains("    x + 1\n"),
        "{t}"
    );
    for mut client in [w, plain, c] {
        drop(client.child.stdin.take());
        client.child.wait().unwrap();
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// `--tool-docs` (backlog 2D-28): the tool descriptions document the tools
/// on their own, so a prompt need not. Only `query` and `write` read
/// differently; names and arguments stay (`write` describes its arguments in
/// the schema only); no design reference is left in them; `--compact` still says which form results come in; without the flag
/// the list is the frozen one.
#[test]
fn tool_docs_document_the_tools_on_their_own() {
    let root = temp_root("tool-docs");
    let list = |flags: &[&str]| -> Vec<(String, String, String)> {
        let mut c = Client::start_with(&root, flags);
        let Value::Array(list) = c
            .request("tools/list", "{}")
            .get("result")
            .unwrap()
            .get("tools")
            .unwrap()
            .clone()
        else {
            panic!()
        };
        list.iter()
            .map(|t| {
                (
                    t.get("name").unwrap().as_str().unwrap().to_string(),
                    t.get("description").unwrap().as_str().unwrap().to_string(),
                    t.get("inputSchema").unwrap().to_json(),
                )
            })
            .collect()
    };
    for compact in [false, true] {
        let base: Vec<&str> = if compact {
            vec!["--write-tool", "--compact"]
        } else {
            vec!["--write-tool"]
        };
        let docs: Vec<&str> = base.iter().copied().chain(["--tool-docs"]).collect();
        let (a, b) = (list(&base), list(&docs));
        assert_eq!(a.len(), b.len());
        for ((na, da, sa), (nb, db, sb)) in a.iter().zip(&b) {
            assert_eq!(na, nb);
            // `write` describes its arguments in the schema; the arguments stay.
            assert_eq!(sa != sb, na == "write", "{na}");
            let props = |s: &str| {
                let v = json::parse(s).unwrap();
                let Some(Value::Object(p)) = v.get("properties") else {
                    panic!()
                };
                (
                    p.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                    v.get("required").unwrap().to_json(),
                )
            };
            assert_eq!(props(sa), props(sb), "{na}");
            assert_eq!(da != db, ["query", "write"].contains(&na.as_str()), "{na}");
            if da != db {
                assert!(!db.contains("ADR-") && !db.contains("backlog"), "{db}");
            }
        }
        let desc = |n: &str| b.iter().find(|t| t.0 == n).unwrap().1.clone();
        for part in [
            "`surface DEF` (signature, contract, proof status)",
            "`slice DEF` (the definition and the contracts it depends on)",
            "&mut writes",
        ] {
            assert!(desc("query").contains(part), "{part}");
        }
        assert!(desc("write").starts_with("The only way to write:"));
        assert_eq!(
            desc("query").contains("Rust-shaped text"),
            compact,
            "{}",
            desc("query")
        );
    }
    // Without `--write-tool` there is no `write`, and `--tool-docs` alone
    // changes only `query`.
    let (a, b) = (list(&[]), list(&["--tool-docs"]));
    assert!(b.iter().all(|t| t.0 != "write"));
    for ((na, da, _), (_, db, _)) in a.iter().zip(&b) {
        assert_eq!(da != db, na == "query", "{na}");
    }
    let _ = std::fs::remove_dir_all(&root);
}
