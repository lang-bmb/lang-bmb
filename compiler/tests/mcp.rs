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
