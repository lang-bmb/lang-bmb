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
        let mut child = Command::new(env!("CARGO_BIN_EXE_bmb"))
            .args(["mcp", root.to_str().unwrap()])
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
