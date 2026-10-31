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

/// Open transactions at once (backlog 2D-20: each holds a copy of the files).
pub const MAX_OPEN_TXS: usize = 64;

pub struct Server {
    root: PathBuf,
    txs: BTreeMap<u64, Tx>,
    next_tx: u64,
    /// Front-end results of the last call (backlog 3A-03): every call still
    /// reads the workspace files, so a change from any writer is seen, but
    /// unchanged files are not parsed again and an unchanged program is
    /// neither checked nor indexed again.
    front: crate::FrontCache,
    /// Offer the one-call `write` tool (backlog 2D-24).
    write_tool: bool,
    /// Show tool results in the compact agent view (`view::compact`,
    /// backlog 2D-23, ADR-0068).
    compact: bool,
    /// Tool descriptions that document the tools on their own, so a prompt
    /// need not (`--tool-docs`, backlog 2D-28).
    tool_docs: bool,
    /// The file diff edit surface: `read_file` and `edit_file`
    /// (`--file-tools`, backlog T1-06, ADR-0073).
    file_tools: bool,
    /// The JSON AST edit surface: `read_ast` and `edit_ast`, and positions
    /// as pointers into the tree (`--ast-tools`, backlog T1-06, ADR-0073).
    ast_tools: bool,
}

/// The flags `bmb mcp` takes before the workspace directory.
pub const FLAGS: &[&str] = &[
    "--write-tool",
    "--compact",
    "--tool-docs",
    "--file-tools",
    "--ast-tools",
];

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

/// The tool list. `write` is offered only when the server is started with it
/// (`bmb mcp --write-tool`): the frozen measurement task sets see the tools
/// they were frozen with (ADR-0021; compiler/README.md, MCP descriptions).
/// With `compact` (`--compact`, ADR-0068) the read and proof tools describe
/// the compact view their results come in. With `docs` (`--tool-docs`,
/// backlog 2D-28) the descriptions are the one place that documents the
/// tools: `query` says what each query returns and `write` that it is the
/// only way to write (what the frozen prompts' tool section said), each
/// `write` argument is described once (in its schema), and no design
/// reference is left.
fn tools(write: bool, compact: bool, docs: bool, files: bool, ast: bool) -> Value {
    let t = |name: &str, desc: &str, input: Value| {
        obj(vec![
            ("name", s(name)),
            ("description", s(desc)),
            ("inputSchema", input),
        ])
    };
    // What a proof result lists, in `verify` and `write`.
    let statuses = if compact {
        "one status line per definition that did not verify and the number of verified definitions"
    } else {
        "one status line per definition"
    };
    // The form query results come in.
    let query_form = if compact {
        "Units, surfaces and slices come as Rust-shaped text: a surface is the definition without its body, after a `// PATH: STATUS` line; edges as `FROM KIND TO`; other rows as JSON lines without empty fields."
    } else {
        "Returns JSONL."
    };
    let query = if docs {
        format!(
            "Read the workspace. Start with `units` (units and their definitions), then narrow down: `surface DEF` (signature, contract, proof status), `edges DEF [out|in|both] [depth]` (resolved calls, contract uses, #[uses], &mut writes), `slice DEF` (the definition and the contracts it depends on), `find NAME`, `contracts PRED`, `status`. {query_form}"
        )
    } else {
        format!(
            "Read query on the standard maps (ADR-0020): units, surface DEF, edges DEF [out|in|both] [depth], slice DEF, find NAME, contracts PRED, status. {query_form}"
        )
    };
    let mut list = vec![
        t(
            "query",
            &query,
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
            if compact {
                "Standard map as Rust-shaped text: level L0 (units and definitions), L1 (surfaces: each definition without its body, after a `// PATH: STATUS` line), L2 (edges, `FROM KIND TO`) or all."
            } else {
                "Standard map as JSONL: level L0 (units and definitions), L1 (surfaces), L2 (edges) or all."
            },
            schema(&[("level", "string", "L0, L1, L2 or all")], &[]),
        ),
        t(
            "verify",
            &format!(
                "Check and prove the workspace. Returns diagnostics (with counterexamples){} {statuses}.",
                if compact { "," } else { " and" }
            ),
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
    ];
    if write {
        // With `docs`, what each argument does is said once, in the schema.
        let (what, code) = if docs {
            (
                "The only way to write: add, replace and remove definitions in one unit, then commit and prove the whole workspace.",
                "one or more complete definitions (with attributes) to add or replace",
            )
        } else {
            (
                "Write in one call (backlog 2D-24): when `summary` is given, create unit UNIT or replace its summary; add or replace the definitions in `code` (one or more, with attributes) in UNIT; remove the definitions in `remove` (full paths); commit; then prove the workspace.",
                "definitions to add or replace",
            )
        };
        let remove = if docs {
            "full paths of definitions to remove"
        } else {
            "definition paths to remove"
        };
        list.insert(
            4,
            t(
                "write",
                &format!(
                    "{what} Rejected as a whole on any structural or type error (diagnostics returned, nothing written). After a commit, returns the commit line and the proof result (diagnostics with counterexamples, {statuses}), as an error when a proof fails."
                ),
                schema(
                    &[
                        ("unit", "string", "unit path"),
                        ("code", "string", code),
                        (
                            "summary",
                            "string",
                            "one-line summary: creates the unit or replaces its summary",
                        ),
                        ("remove", "array", remove),
                    ],
                    &["unit"],
                ),
            ),
        );
    }
    // The edit surfaces of ADR-0073, after `build` like `write`; their
    // descriptions carry no design references in either mode.
    let mut at = 4 + usize::from(write);
    if files {
        list.insert(
            at,
            t(
                "read_file",
                "Read one workspace file. `src/UNIT.bmb` holds the top-level unit UNIT (with its nested units) in normal form. Returns the file's text exactly, to write diffs against.",
                schema(
                    &[("path", "string", "the file, e.g. src/calc.bmb")],
                    &["path"],
                ),
            ),
        );
        list.insert(
            at + 1,
            t(
                "edit_file",
                &format!(
                    "The only way to write: apply a unified diff to one or more files, then commit and prove the whole workspace. Each file starts with `--- a/src/UNIT.bmb` and `+++ b/src/UNIT.bmb` (`--- /dev/null` creates a file, `+++ /dev/null` deletes one; `src/UNIT.bmb` holds exactly the top-level unit UNIT), then hunks `@@ -LINE,COUNT +LINE,COUNT @@` whose lines start with ' ' (context), '-' (remove) or '+' (add). Counts are recounted and LINE is where a hunk is looked for first; context and removed lines must match the file exactly. Rejected as a whole if a hunk does not apply or on any structural or type error (diagnostics returned, nothing written). Committed files are stored in normal form: `reformatted` lists the files whose text the normal form changed, to read again before the next diff. After a commit, returns the commit line and the proof result (diagnostics with counterexamples, {statuses}), as an error when a proof fails."
                ),
                schema(&[("diff", "string", "the unified diff")], &["diff"]),
            ),
        );
        at += 2;
    }
    if ast {
        list.insert(
            at,
            t(
                "read_ast",
                "Read the workspace's syntax tree as JSON, at a JSON Pointer `path` (default: the whole workspace). The document is keyed by top-level unit name; a unit has `summary` and, keyed by name, `units`, `types`, `traits`, `impls` and `fns`, so `/calc/fns/twice` is function `calc::twice` and `/calc/fns/twice/body/tail` its body's last expression. Every other node is an object with a `kind` (`fn`, `let`, `while`, `call`, `binary`, `var`, `int`, `i64`, `ref`, ..) and its parts by name; empty parts are left out.",
                schema(
                    &[(
                        "path",
                        "string",
                        "JSON Pointer, e.g. /calc/fns/twice (empty: the whole workspace)",
                    )],
                    &[],
                ),
            ),
        );
        list.insert(
            at + 1,
            t(
                "edit_ast",
                &format!(
                    "The only way to write: apply a JSON Patch (RFC 6902: `add`, `remove`, `replace`, `move`, `copy`, `test`, each at a JSON Pointer `path`) to the syntax tree `read_ast` shows, then commit and prove the whole workspace. `add` at `/UNIT/fns/NAME` adds or replaces a function, at `/UNIT` a unit; any node can be replaced in place. Rejected as a whole if an operation does not apply, the result is not a syntax tree, or on any structural or type error (diagnostics returned with `at`, the pointer of the node, nothing written). After a commit, returns the commit line and the proof result (diagnostics with counterexamples and `at`, {statuses}), as an error when a proof fails."
                ),
                obj(vec![
                    ("type", s("object")),
                    (
                        "properties",
                        obj(vec![(
                            "patch",
                            obj(vec![
                                ("type", s("array")),
                                (
                                    "description",
                                    s("the JSON Patch operations, applied in order"),
                                ),
                                (
                                    "items",
                                    obj(vec![
                                        ("type", s("object")),
                                        (
                                            "properties",
                                            obj(vec![
                                                (
                                                    "op",
                                                    obj(vec![
                                                        ("type", s("string")),
                                                        (
                                                            "enum",
                                                            Value::Array(
                                                                [
                                                                    "add", "remove", "replace",
                                                                    "move", "copy", "test",
                                                                ]
                                                                .iter()
                                                                .map(|x| s(x))
                                                                .collect(),
                                                            ),
                                                        ),
                                                    ]),
                                                ),
                                                ("path", obj(vec![("type", s("string"))])),
                                                ("from", obj(vec![("type", s("string"))])),
                                                ("value", obj(vec![])),
                                            ]),
                                        ),
                                        (
                                            "required",
                                            Value::Array(vec![s("op"), s("path")]),
                                        ),
                                    ]),
                                ),
                            ]),
                        )]),
                    ),
                    ("required", Value::Array(vec![s("patch")])),
                ]),
            ),
        );
    }
    Value::Array(list)
}

impl Server {
    pub fn new(root: &Path) -> Server {
        Server {
            root: root.to_path_buf(),
            txs: BTreeMap::new(),
            next_tx: 1,
            front: crate::FrontCache::default(),
            write_tool: false,
            compact: false,
            tool_docs: false,
            file_tools: false,
            ast_tools: false,
        }
    }

    /// The same server offering the file diff edit surface (`--file-tools`,
    /// ADR-0073).
    pub fn file_tools(self) -> Server {
        Server {
            file_tools: true,
            ..self
        }
    }

    /// The same server offering the JSON AST edit surface (`--ast-tools`,
    /// ADR-0073).
    pub fn ast_tools(self) -> Server {
        Server {
            ast_tools: true,
            ..self
        }
    }

    /// A server that also offers the one-call `write` tool (backlog 2D-24).
    pub fn with_write_tool(root: &Path) -> Server {
        Server {
            write_tool: true,
            ..Server::new(root)
        }
    }

    /// The same server showing tool results in the compact agent view
    /// (`bmb mcp --compact`, backlog 2D-23, ADR-0068).
    pub fn compact(self) -> Server {
        Server {
            compact: true,
            ..self
        }
    }

    /// The same server with tool descriptions that document the tools on
    /// their own (`bmb mcp --tool-docs`, backlog 2D-28).
    pub fn tool_docs(self) -> Server {
        Server {
            tool_docs: true,
            ..self
        }
    }

    /// What the front-end cache did so far.
    pub fn front_stats(&self) -> crate::FrontStats {
        self.front.stats.clone()
    }

    /// Reads the workspace and brings the front-end cache up to date.
    fn load(&mut self) -> Result<&mut crate::FrontCache, String> {
        let files = self.files()?;
        self.front.update(files);
        Ok(&mut self.front)
    }

    /// Proof cache of the workspace (backlog 2D-03): answers are the same with
    /// or without it, only faster.
    fn cache_dir(&self) -> std::path::PathBuf {
        self.root.join(".bmb-cache")
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
        // The directories the tools read and write must not lead outside the
        // workspace (backlog 2D-20, docs/mcp-threat-model.md).
        for d in ["src", ".bmb-cache", "target"] {
            workspace::refuse_symlink(&self.root.join(d))?;
        }
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
                let dir = self.cache_dir();
                self.load()?.map_request(&req, Some(&dir))
            }
            "map" => {
                let level = a
                    .get("level")
                    .and_then(Value::as_str)
                    .unwrap_or("all")
                    .to_string();
                let dir = self.cache_dir();
                self.load()?.map_request(&["map".into(), level], Some(&dir))
            }
            "verify" => {
                let dir = self.cache_dir();
                let out = self.load()?.verify(Some(&dir));
                let text = self.positions(out.output)?;
                if out.ok { Ok(text) } else { Err(text) }
            }
            "build" => {
                let exe = str_arg("name")?;
                if exe.is_empty() || exe.contains(['/', '\\', '\0']) || exe.starts_with('.') {
                    return Err("`name` must be a plain file name".into());
                }
                let dir = self.cache_dir();
                let out = self.load()?.emit_llvm(Some(&dir));
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
                let Some(path_str) = path.to_str() else {
                    return Err(format!(
                        "build output path {} is not valid UTF-8",
                        path.display()
                    ));
                };
                let Some(ir) = out.ir else {
                    return Err(format!("{}build refused: no IR", out.verify.output));
                };
                crate::toolchain::build_executable(&ir, path_str)?;
                Ok(format!(
                    "{}{{\"built\":{}}}\n",
                    out.verify.output,
                    crate::diag::json_str(&path.display().to_string())
                ))
            }
            "write" if self.write_tool => {
                // One transaction and a proof in one tool call: in the pilot
                // the separate calls (begin, unit, put, commit, verify) made
                // BMB sessions take 2.9 times the model turns of Verus.
                let unit = str_arg("unit")?;
                let mut ops = Vec::new();
                if let Some(summary) = a.get("summary").and_then(Value::as_str) {
                    ops.push(Op::Unit {
                        path: unit.clone(),
                        summary: summary.to_string(),
                    });
                }
                if let Some(code) = a.get("code").and_then(Value::as_str)
                    && !code.trim().is_empty()
                {
                    ops.push(Op::Put {
                        unit: unit.clone(),
                        code: code.to_string(),
                    });
                }
                if let Some(Value::Array(xs)) = a.get("remove") {
                    for x in xs {
                        let def = x.as_str().ok_or("`remove` holds definition paths")?;
                        ops.push(Op::Remove {
                            def: def.to_string(),
                        });
                    }
                }
                if ops.is_empty() {
                    return Err("nothing to write: give `code`, `summary` or `remove`".into());
                }
                let c = workspace::commit(&self.root, &ops)?;
                let list =
                    |xs: &[String]| Value::Array(xs.iter().map(|x| s(x)).collect()).to_json();
                let line = format!(
                    "{{\"committed\":{},\"changed\":{},\"removed\":{}}}",
                    c.ok,
                    list(&c.changed),
                    list(&c.removed)
                );
                if !c.ok {
                    return Err(format!("{}{line}", c.diagnostics));
                }
                let dir = self.cache_dir();
                let out = self.load()?.verify(Some(&dir));
                let text = format!("{line}\n{}", out.output);
                if out.ok { Ok(text) } else { Err(text) }
            }
            "read_file" if self.file_tools => {
                let name = file_name(&str_arg("path")?)?;
                let files = workspace::relative_sources(&self.root)?;
                match files.iter().find(|(n, _)| *n == name) {
                    Some((_, text)) => Ok(text.clone()),
                    None => Err(format!(
                        "no file {name}; the files are: {}",
                        files
                            .iter()
                            .map(|(n, _)| n.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }
            }
            "edit_file" if self.file_tools => {
                let diff = str_arg("diff")?;
                let old = workspace::relative_sources(&self.root)?;
                let new = apply_diff(&old, &diff)?;
                let (c, units) = workspace::plan_files(&old, &new);
                let line = commit_line(&c, &new, units.as_ref());
                if !c.ok {
                    return Err(format!("{}{line}", c.diagnostics));
                }
                workspace::write_units(&self.root, &units.unwrap_or_default())?;
                self.prove_after(line)
            }
            "read_ast" if self.ast_tools => {
                let path = a.get("path").and_then(Value::as_str).unwrap_or("");
                let segs = crate::ast_json::pointer(path)?;
                let doc = self.ast_doc()?;
                Ok(crate::ast_json::resolve(&doc, &segs)?.to_json())
            }
            "edit_ast" if self.ast_tools => {
                // A patch sent as JSON text (some clients encode nested
                // arguments) is the same patch.
                let patch = match a.get("patch") {
                    Some(Value::Str(t)) => {
                        json::parse(t).map_err(|e| format!("`patch` is not JSON: {e}"))?
                    }
                    Some(p) => p.clone(),
                    None => return Err("missing `patch`".into()),
                };
                let mut doc = self.ast_doc()?;
                crate::ast_json::apply_patch(&mut doc, &patch)?;
                let program = crate::ast_json::import(&doc)?;
                let new: Vec<(String, String)> = crate::fmt::format_units(&program)
                    .into_iter()
                    .map(|(u, t)| (format!("src/{u}.bmb"), t))
                    .collect();
                let old = workspace::relative_sources(&self.root)?;
                let (c, units) = workspace::plan_files(&old, &new);
                let line = commit_line(&c, &new, None);
                if !c.ok {
                    let diags = locate(&new, &c.diagnostics);
                    return Err(format!("{diags}{line}"));
                }
                let units = units.unwrap_or_default();
                // The tree must be what the printed program parses back to;
                // otherwise something in it has no text (ADR-0073).
                let printed: Vec<(String, String)> = units
                    .iter()
                    .map(|(u, t)| (format!("src/{u}.bmb"), t.clone()))
                    .collect();
                let (back, _, _) = workspace::parse_files(&printed);
                let want = crate::ast_json::export(&program)?;
                let got = crate::ast_json::export(&back)?;
                if let Some((at, as_text)) = first_difference(&want, &got, &mut Vec::new()) {
                    return Err(format!(
                        "{at}: this part of the tree has no text of its own: it prints and parses back as {}; write it that way (nothing written)",
                        as_text.to_json()
                    ));
                }
                workspace::write_units(&self.root, &units)?;
                self.prove_after(line)
            }
            "tx_begin" => {
                // Each open transaction holds a copy of the workspace files.
                if self.txs.len() >= MAX_OPEN_TXS {
                    return Err(format!(
                        "too many open transactions ({MAX_OPEN_TXS}); commit or abort one first"
                    ));
                }
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

    /// The workspace as a JSON syntax tree (ADR-0073).
    fn ast_doc(&self) -> Result<Value, String> {
        let files = workspace::relative_sources(&self.root)?;
        let (program, sm, diags) = workspace::parse_files(&files);
        if diags.has_errors() {
            return Err(format!(
                "{}the workspace files do not parse",
                diags.to_jsonl(&sm)
            ));
        }
        crate::ast_json::export(&program)
    }

    /// In the JSON AST surface, positions in diagnostics become pointers
    /// into the tree (`at`); otherwise `text` as it is.
    fn positions(&self, text: String) -> Result<String, String> {
        if !self.ast_tools {
            return Ok(text);
        }
        Ok(locate(&self.files()?, &text))
    }

    /// The commit line, then the proof of the whole workspace.
    fn prove_after(&mut self, line: String) -> Result<String, String> {
        let dir = self.cache_dir();
        let out = self.load()?.verify(Some(&dir));
        let text = format!("{line}\n{}", self.positions(out.output)?);
        if out.ok { Ok(text) } else { Err(text) }
    }

    /// Handles one JSON-RPC message; `None` for notifications.
    pub fn handle(&mut self, msg: &Value) -> Option<Value> {
        // Anything but a request object (a string, a number, an array: MCP
        // has no batches) is an invalid request with a null id (JSON-RPC 2.0
        // §5); before backlog 1D-23 it got no answer and the client waited.
        if !matches!(msg, Value::Object(_)) {
            return Some(rpc_error(
                Value::Null,
                -32600,
                "invalid request: expected a JSON-RPC request object",
            ));
        }
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str);
        let id = id?; // notifications get no response
        let Some(method) = method else {
            return Some(rpc_error(id, -32600, "invalid request: missing `method`"));
        };
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
            "tools/list" => Ok(obj(vec![(
                "tools",
                tools(
                    self.write_tool,
                    self.compact,
                    self.tool_docs,
                    self.file_tools,
                    self.ast_tools,
                ),
            )])),
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
                let text = if self.compact {
                    crate::view::compact(&text)
                } else {
                    text
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
            Err((code, m)) => rpc_error(id, code, &m),
        })
    }
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    obj(vec![
        ("jsonrpc", s("2.0")),
        ("id", id),
        (
            "error",
            obj(vec![
                ("code", Value::Num(code.to_string())),
                ("message", s(message)),
            ]),
        ),
    ])
}

/// Serves stdin/stdout until end of input. `flags` (each one of `FLAGS`):
/// `--write-tool` offers `write` too, `--compact` shows results in the
/// compact agent view, `--tool-docs` gives the self-contained tool
/// descriptions, `--file-tools` and `--ast-tools` offer the file diff and
/// JSON AST edit surfaces.
pub fn serve(root: &Path, flags: &[&str]) -> std::io::Result<()> {
    let has = |f: &str| flags.contains(&f);
    let mut server = if has("--write-tool") {
        Server::with_write_tool(root)
    } else {
        Server::new(root)
    };
    if has("--compact") {
        server = server.compact();
    }
    if has("--tool-docs") {
        server = server.tool_docs();
    }
    if has("--file-tools") {
        server = server.file_tools();
    }
    if has("--ast-tools") {
        server = server.ast_tools();
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match json::parse(&line) {
            Ok(msg) => server.handle(&msg),
            Err(e) => Some(rpc_error(Value::Null, -32700, &e)),
        };
        if let Some(r) = reply {
            writeln!(stdout, "{}", r.to_json())?;
            stdout.flush()?;
        }
    }
    Ok(())
}

/// A file path as the file diff surface names files: `src/UNIT.bmb`, from
/// that, `a/src/UNIT.bmb`, `UNIT.bmb` or a path ending in `/src/UNIT.bmb`
/// (as diagnostics print them).
fn file_name(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    let stem = p.file_stem().and_then(|x| x.to_str()).unwrap_or("");
    let dir = p.parent().and_then(|d| d.to_str()).unwrap_or("");
    let ok_dir = dir.is_empty() || dir == "src" || dir.ends_with("/src");
    if ok_dir && p.extension().is_some_and(|x| x == "bmb") && crate::parser::is_ident(stem) {
        Ok(format!("src/{stem}.bmb"))
    } else {
        Err(format!(
            "`{path}` is not a workspace file: files are src/UNIT.bmb, one per top-level unit"
        ))
    }
}

/// The files after `diff` (ADR-0073): every file it names exists (or is
/// created by it), each at most once; a file left empty is removed.
fn apply_diff(old: &[(String, String)], diff: &str) -> Result<Vec<(String, String)>, String> {
    let patches = crate::udiff::parse(diff)?;
    if patches.is_empty() {
        return Err(
            "no file in the diff: each file starts with `--- a/src/UNIT.bmb` and `+++ b/src/UNIT.bmb`"
                .into(),
        );
    }
    let mut files: BTreeMap<String, String> = old.iter().cloned().collect();
    let mut seen = std::collections::BTreeSet::new();
    for p in &patches {
        let old_name = p.old.as_deref().map(file_name).transpose()?;
        let new_name = p.new.as_deref().map(file_name).transpose()?;
        let name = match (&old_name, &new_name) {
            (Some(a), Some(b)) if a != b => {
                return Err(format!(
                    "`--- {a}` and `+++ {b}` name different files; a diff edits files in place"
                ));
            }
            (_, Some(n)) | (Some(n), None) => n.clone(),
            (None, None) => return Err("`--- /dev/null` with `+++ /dev/null`".into()),
        };
        if !seen.insert(name.clone()) {
            return Err(format!(
                "{name} appears twice; put all its hunks under one `---`/`+++` pair"
            ));
        }
        let text = match (old_name.is_some(), files.get(&name)) {
            (true, Some(t)) => t.clone(),
            (true, None) => {
                return Err(format!(
                    "{name} does not exist (create it with `--- /dev/null`)"
                ));
            }
            (false, Some(_)) => return Err(format!("{name} already exists")),
            (false, None) => String::new(),
        };
        let after = crate::udiff::apply(&text, &p.hunks).map_err(|e| format!("{name}: {e}"))?;
        if new_name.is_none() || after.trim().is_empty() {
            files.remove(&name);
        } else {
            files.insert(name, after);
        }
    }
    Ok(files.into_iter().collect())
}

/// `{"committed","changed","removed"}`, with `reformatted` (files whose
/// stored normal form differs from the text written) when `units` is given.
fn commit_line(
    c: &workspace::Commit,
    new: &[(String, String)],
    units: Option<&BTreeMap<String, String>>,
) -> String {
    let list = |xs: &[String]| Value::Array(xs.iter().map(|x| s(x)).collect()).to_json();
    let mut line = format!(
        "{{\"committed\":{},\"changed\":{},\"removed\":{}",
        c.ok,
        list(&c.changed),
        list(&c.removed)
    );
    if let Some(units) = units.filter(|_| c.ok) {
        let re: Vec<String> = new
            .iter()
            .filter(|(n, t)| {
                let unit = Path::new(n)
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .unwrap_or("");
                units.get(unit) != Some(t)
            })
            .map(|(n, _)| n.clone())
            .collect();
        if !re.is_empty() {
            line.push_str(&format!(",\"reformatted\":{}", list(&re)));
        }
    }
    line.push('}');
    line
}

/// Diagnostics (JSONL) with each position in `files` replaced by `at`, the
/// JSON Pointer of the innermost syntax-tree node that holds it (ADR-0073).
/// Rows whose position is in no node keep it.
fn locate(files: &[(String, String)], jsonl: &str) -> String {
    let (program, _, _) = workspace::parse_files(files);
    let mut spans = Vec::new();
    if crate::ast_json::export_spans(&program, &mut spans).is_err() {
        return jsonl.to_string();
    }
    let ids: BTreeMap<&str, u32> = files
        .iter()
        .enumerate()
        .map(|(i, (n, _))| (n.as_str(), i as u32))
        .collect();
    let offset = |file: u32, line: i64, col: i64| -> Option<u32> {
        let text = &files[file as usize].1;
        let start: usize = text
            .split_inclusive('\n')
            .take(usize::try_from(line - 1).ok()?)
            .map(str::len)
            .sum();
        let rest = text.get(start..)?;
        let skip: usize = rest
            .chars()
            .take(usize::try_from(col - 1).ok()?)
            .map(char::len_utf8)
            .sum();
        u32::try_from(start + skip).ok()
    };
    let mut out = String::new();
    for row in jsonl.lines() {
        let at = json::parse(row).ok().and_then(|v| {
            let num = |k: &str| v.get(k).and_then(Value::as_i64);
            let file = *ids.get(v.get("file")?.as_str()?)?;
            let lo = offset(file, num("line")?, num("col")?)?;
            let hi = offset(file, num("end_line")?, num("end_col")?)?;
            let p = crate::ast_json::node_at(&spans, file, lo, hi)?.to_string();
            let Value::Object(kv) = v else { return None };
            let kv: Vec<(String, Value)> = kv
                .into_iter()
                .filter_map(|(k, x)| match k.as_str() {
                    "file" => Some(("at".to_string(), Value::Str(p.clone()))),
                    "line" | "col" | "end_line" | "end_col" => None,
                    _ => Some((k, x)),
                })
                .collect();
            Some(Value::Object(kv).to_json())
        });
        out.push_str(at.as_deref().unwrap_or(row));
        out.push('\n');
    }
    out
}

/// The first place, in document order, where `got` differs from `want`:
/// its pointer and what `got` has there.
fn first_difference(want: &Value, got: &Value, at: &mut Vec<String>) -> Option<(String, Value)> {
    let here = |at: &[String]| -> String {
        let p: String = at
            .iter()
            .map(|s| format!("/{}", crate::ast_json::escape(s)))
            .collect();
        if p.is_empty() { "/".into() } else { p }
    };
    match (want, got) {
        (Value::Object(a), Value::Object(b)) => {
            // A part only one side has: the object as `got` has it.
            if a.iter().any(|(k, _)| got.get(k).is_none())
                || b.iter().any(|(k, _)| want.get(k).is_none())
            {
                return Some((here(at), got.clone()));
            }
            for (k, v) in a {
                at.push(k.clone());
                let d = first_difference(v, got.get(k)?, at);
                at.pop();
                if d.is_some() {
                    return d;
                }
            }
            None
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (k, (v, w)) in a.iter().zip(b).enumerate() {
                at.push(k.to_string());
                let d = first_difference(v, w, at);
                at.pop();
                if d.is_some() {
                    return d;
                }
            }
            None
        }
        _ if want == got => None,
        _ => Some((here(at), got.clone())),
    }
}
