//! MCP server over stdio (backlog 1D-11). The agent's only path to the code:
//! map queries and slices to read, transactions to write (DESIGN.md principle 6).
//!
//! Transport: JSON-RPC 2.0, one message per line on stdin/stdout.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use crate::json::{self, Value};
use crate::workspace::{self, Op};

const PROTOCOL: &str = "2025-06-18";

struct Tx {
    ops: Vec<Op>,
    /// Workspace contents at `tx_begin`; commit refuses if they changed.
    base: Vec<(String, String)>,
}

pub struct Server {
    root: PathBuf,
    txs: BTreeMap<u64, Tx>,
    next_tx: u64,
}

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

fn obj(kv: Vec<(&str, Value)>) -> Value {
    Value::Object(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

fn schema(props: &[(&str, &str, &str)], required: &[&str]) -> Value {
    obj(vec![
        ("type", s("object")),
        (
            "properties",
            obj(props
                .iter()
                .map(|(name, ty, desc)| {
                    let mut kv = vec![("type", s(ty)), ("description", s(desc))];
                    if *ty == "array" {
                        kv.push(("items", obj(vec![("type", s("string"))])));
                    }
                    (*name, obj(kv))
                })
                .collect()),
        ),
        (
            "required",
            Value::Array(required.iter().map(|r| s(r)).collect()),
        ),
    ])
}

fn tools() -> Value {
    let t = |name: &str, desc: &str, input: Value| {
        obj(vec![
            ("name", s(name)),
            ("description", s(desc)),
            ("inputSchema", input),
        ])
    };
    Value::Array(vec![
        t(
            "query",
            "Read query on the standard maps (ADR-0020): units, surface DEF, edges DEF [out|in|both] [depth], slice DEF, find NAME, contracts PRED, status. Returns JSONL.",
            schema(
                &[
                    ("name", "string", "query name"),
                    ("args", "array", "query arguments"),
                ],
                &["name"],
            ),
        ),
        t(
            "map",
            "Standard map as JSONL: level L0 (units and definitions), L1 (surfaces), L2 (edges) or all.",
            schema(&[("level", "string", "L0, L1, L2 or all")], &[]),
        ),
        t(
            "verify",
            "Check and prove the workspace. Returns diagnostics (with counterexamples) and one status line per definition.",
            schema(&[], &[]),
        ),
        t(
            "build",
            "Build an executable from a fully proved workspace into target/NAME. Refused if any obligation is unproved.",
            schema(&[("name", "string", "executable name")], &["name"]),
        ),
        t(
            "tx_begin",
            "Start a write transaction. Returns its id.",
            schema(&[], &[]),
        ),
        t(
            "tx_unit",
            "In a transaction: create unit PATH (a or a::b) or replace its one-line summary.",
            schema(
                &[
                    ("tx", "integer", "transaction id"),
                    ("path", "string", "unit path"),
                    ("summary", "string", "one-line summary"),
                ],
                &["tx", "path", "summary"],
            ),
        ),
        t(
            "tx_put",
            "In a transaction: add or replace one definition (fn or spec fn with its attributes) in UNIT.",
            schema(
                &[
                    ("tx", "integer", "transaction id"),
                    ("unit", "string", "unit path"),
                    ("code", "string", "the definition"),
                ],
                &["tx", "unit", "code"],
            ),
        ),
        t(
            "tx_remove",
            "In a transaction: remove definition DEF (full path).",
            schema(
                &[
                    ("tx", "integer", "transaction id"),
                    ("def", "string", "definition path"),
                ],
                &["tx", "def"],
            ),
        ),
        t(
            "tx_remove_unit",
            "In a transaction: remove unit PATH and everything in it.",
            schema(
                &[
                    ("tx", "integer", "transaction id"),
                    ("path", "string", "unit path"),
                ],
                &["tx", "path"],
            ),
        ),
        t(
            "tx_commit",
            "Commit a transaction. Rejected as a whole on any structural or type error (diagnostics returned, nothing written) or if the workspace changed since tx_begin. Proofs are not required to commit.",
            schema(&[("tx", "integer", "transaction id")], &["tx"]),
        ),
        t(
            "tx_abort",
            "Discard a transaction.",
            schema(&[("tx", "integer", "transaction id")], &["tx"]),
        ),
    ])
}

impl Server {
    pub fn new(root: &Path) -> Server {
        Server {
            root: root.to_path_buf(),
            txs: BTreeMap::new(),
            next_tx: 1,
        }
    }

    fn files(&self) -> Result<Vec<(String, String)>, String> {
        if workspace::src_dir(&self.root).is_dir() {
            workspace::read_dir_sources(&self.root)
        } else {
            Ok(Vec::new())
        }
    }

    /// Runs one tool. `Ok(text)` is a normal result, `Err(text)` a tool error.
    fn call(&mut self, name: &str, a: &Value) -> Result<String, String> {
        let str_arg = |k: &str| {
            a.get(k)
                .and_then(Value::as_str)
                .map(String::from)
                .ok_or(format!("missing `{k}`"))
        };
        let tx_arg = || -> Result<u64, String> {
            a.get("tx")
                .and_then(Value::as_i64)
                .map(|v| v as u64)
                .ok_or("missing `tx`".to_string())
        };
        match name {
            "query" => {
                let mut req = vec!["query".to_string(), str_arg("name")?];
                if let Some(Value::Array(xs)) = a.get("args") {
                    req.extend(xs.iter().filter_map(|x| x.as_str().map(String::from)));
                }
                crate::map_request(&self.files()?, &req)
            }
            "map" => {
                let level = a
                    .get("level")
                    .and_then(Value::as_str)
                    .unwrap_or("all")
                    .to_string();
                crate::map_request(&self.files()?, &["map".into(), level])
            }
            "verify" => {
                let out = crate::verify_sources(&self.files()?);
                if out.ok {
                    Ok(out.output)
                } else {
                    Err(out.output)
                }
            }
            "build" => {
                let exe = str_arg("name")?;
                if exe.is_empty() || exe.contains(['/', '\\']) || exe.starts_with('.') {
                    return Err("`name` must be a plain file name".into());
                }
                let out = crate::emit_llvm_sources(&self.files()?);
                if !out.verify.ok || out.checks > 0 || !out.has_main {
                    let why = if out.has_main {
                        "not fully proved"
                    } else {
                        "no `main`"
                    };
                    return Err(format!("{}build refused: {why}", out.verify.output));
                }
                let dir = self.root.join("target");
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let path = dir.join(&exe);
                crate::toolchain::build_executable(&out.ir.unwrap(), path.to_str().unwrap())?;
                Ok(format!(
                    "{}{{\"built\":{}}}\n",
                    out.verify.output,
                    crate::diag::json_str(&path.display().to_string())
                ))
            }
            "tx_begin" => {
                let id = self.next_tx;
                self.next_tx += 1;
                self.txs.insert(
                    id,
                    Tx {
                        ops: Vec::new(),
                        base: self.files()?,
                    },
                );
                Ok(format!("{{\"tx\":{id}}}"))
            }
            "tx_unit" | "tx_put" | "tx_remove" | "tx_remove_unit" => {
                let id = tx_arg()?;
                let op = match name {
                    "tx_unit" => Op::Unit {
                        path: str_arg("path")?,
                        summary: str_arg("summary")?,
                    },
                    "tx_put" => Op::Put {
                        unit: str_arg("unit")?,
                        code: str_arg("code")?,
                    },
                    "tx_remove" => Op::Remove {
                        def: str_arg("def")?,
                    },
                    _ => Op::RemoveUnit {
                        path: str_arg("path")?,
                    },
                };
                let tx = self
                    .txs
                    .get_mut(&id)
                    .ok_or(format!("no open transaction {id}"))?;
                tx.ops.push(op);
                Ok(format!("{{\"tx\":{id},\"ops\":{}}}", tx.ops.len()))
            }
            "tx_commit" => {
                let id = tx_arg()?;
                let tx = self
                    .txs
                    .remove(&id)
                    .ok_or(format!("no open transaction {id}"))?;
                if self.files()? != tx.base {
                    return Err(format!(
                        "{{\"committed\":false,\"error\":\"workspace changed since tx_begin; begin a new transaction\",\"tx\":{id}}}"
                    ));
                }
                let c = workspace::commit(&self.root, &tx.ops)?;
                let list =
                    |xs: &[String]| Value::Array(xs.iter().map(|x| s(x)).collect()).to_json();
                let line = format!(
                    "{{\"committed\":{},\"changed\":{},\"removed\":{}}}",
                    c.ok,
                    list(&c.changed),
                    list(&c.removed)
                );
                if c.ok {
                    Ok(line)
                } else {
                    Err(format!("{}{line}", c.diagnostics))
                }
            }
            "tx_abort" => {
                let id = tx_arg()?;
                self.txs
                    .remove(&id)
                    .ok_or(format!("no open transaction {id}"))?;
                Ok(format!("{{\"tx\":{id},\"aborted\":true}}"))
            }
            other => Err(format!("unknown tool `{other}`")),
        }
    }

    /// Handles one JSON-RPC message; `None` for notifications.
    pub fn handle(&mut self, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let id = id?; // notifications get no response
        let result = match method {
            "initialize" => Ok(obj(vec![
                ("protocolVersion", s(PROTOCOL)),
                ("capabilities", obj(vec![("tools", obj(vec![]))])),
                (
                    "serverInfo",
                    obj(vec![
                        ("name", s("bmb")),
                        ("version", s(env!("CARGO_PKG_VERSION"))),
                    ]),
                ),
                (
                    "instructions",
                    s(
                        "BMB workspace tools. Read with `query`/`map` (start from `query units`, then `surface`, `edges`, `slice`). Write only through tx_begin/tx_put/tx_commit. Prove with `verify`.",
                    ),
                ),
            ])),
            "ping" => Ok(obj(vec![])),
            "tools/list" => Ok(obj(vec![("tools", tools())])),
            "tools/call" => {
                let params = msg.get("params").cloned().unwrap_or(Value::Null);
                let name = params
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let args = params.get("arguments").cloned().unwrap_or(obj(vec![]));
                let (text, is_error) = match self.call(&name, &args) {
                    Ok(t) => (t, false),
                    Err(t) => (t, true),
                };
                Ok(obj(vec![
                    (
                        "content",
                        Value::Array(vec![obj(vec![("type", s("text")), ("text", s(&text))])]),
                    ),
                    ("isError", Value::Bool(is_error)),
                ]))
            }
            other => Err((-32601, format!("method not found: {other}"))),
        };
        Some(match result {
            Ok(r) => obj(vec![("jsonrpc", s("2.0")), ("id", id), ("result", r)]),
            Err((code, m)) => obj(vec![
                ("jsonrpc", s("2.0")),
                ("id", id),
                (
                    "error",
                    obj(vec![
                        ("code", Value::Num(code.to_string())),
                        ("message", s(&m)),
                    ]),
                ),
            ]),
        })
    }
}

/// Serves stdin/stdout until end of input.
pub fn serve(root: &Path) -> std::io::Result<()> {
    let mut server = Server::new(root);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match json::parse(&line) {
            Ok(msg) => server.handle(&msg),
            Err(e) => Some(obj(vec![
                ("jsonrpc", s("2.0")),
                ("id", Value::Null),
                (
                    "error",
                    obj(vec![
                        ("code", Value::Num("-32700".into())),
                        ("message", s(&e)),
                    ]),
                ),
            ])),
        };
        if let Some(r) = reply {
            writeln!(stdout, "{}", r.to_json())?;
            stdout.flush()?;
        }
    }
    Ok(())
}
