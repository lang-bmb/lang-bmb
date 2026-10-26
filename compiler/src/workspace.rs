//! Workspace and write transactions (ADR-0009, ADR-0019 §3, backlog 1D-08, 1D-09).
//!
//! A workspace is a directory whose `src/` holds one normal-form file per
//! top-level unit. Agents never edit those files; they submit transactions.
//! A transaction commits only if the resulting program has no structural or
//! type errors. Proofs are not required to commit; proof status is visible in
//! the maps.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::diag::Diags;
use crate::source::SourceMap;
use crate::{check, fmt, lexer, parser};

#[derive(Debug, Clone)]
pub enum Op {
    /// Create a unit (path `a` or `a::b`) or replace its summary.
    Unit { path: String, summary: String },
    /// Add or replace one definition (`fn` / `spec fn`, attributes included) in a unit.
    Put { unit: String, code: String },
    /// Remove a definition by full path.
    Remove { def: String },
    /// Remove a unit and everything in it.
    RemoveUnit { path: String },
}

/// Outcome of a commit.
pub struct Commit {
    pub ok: bool,
    /// Diagnostics (JSONL) when rejected; empty when committed.
    pub diagnostics: String,
    /// Definitions whose body hash changed, was added, or was removed.
    pub changed: Vec<String>,
    pub removed: Vec<String>,
}

pub fn src_dir(root: &Path) -> PathBuf {
    root.join("src")
}

/// Fails when `p` exists and is a symbolic link (backlog 2D-20). A workspace's
/// files are laid out by the tools (ADR-0011), and its contents may come from
/// someone else: a link would make reads and writes reach outside it.
pub fn refuse_symlink(p: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(p) {
        Ok(m) if m.file_type().is_symlink() => Err(format!(
            "{} is a symbolic link; workspace files must be plain files and directories",
            p.display()
        )),
        _ => Ok(()),
    }
}

/// Writes `text` to a new file at `path`, removing whatever was there first
/// (a planted link is removed, not followed; backlog 2D-20).
pub fn write_new(path: &Path, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(text.as_bytes())
}

/// Reads the `.bmb` files of a workspace (or of any directory), sorted by name.
pub fn read_dir_sources(dir: &Path) -> Result<Vec<(String, String)>, String> {
    refuse_symlink(&src_dir(dir))?;
    let dir = if src_dir(dir).is_dir() {
        src_dir(dir)
    } else {
        dir.to_path_buf()
    };
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "bmb"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|p| {
            refuse_symlink(&p)?;
            std::fs::read_to_string(&p)
                .map(|t| (p.display().to_string(), t))
                .map_err(|e| format!("cannot read {}: {e}", p.display()))
        })
        .collect()
}

fn parse_into(sm: &mut SourceMap, diags: &mut Diags, name: &str, text: &str) -> Program {
    let id = sm.add(name.to_string(), text.to_string());
    let toks = lexer::lex(id, text, diags);
    parser::Parser::new(toks, diags, text).parse_program()
}

fn find_mod<'a>(mods: &'a mut [Mod], path: &[&str]) -> Option<&'a mut Mod> {
    let (first, rest) = path.split_first()?;
    let m = mods.iter_mut().find(|m| m.name.name == *first)?;
    if rest.is_empty() {
        Some(m)
    } else {
        find_mod(&mut m.mods, rest)
    }
}

fn summary_attr(summary: &str) -> Attr {
    let sp = crate::source::Span::default();
    Attr {
        name: Ident {
            name: "summary".into(),
            span: sp,
        },
        args: vec![AttrArg::Str(summary.to_string(), sp)],
        span: sp,
    }
}

fn new_mod(name: &str, summary: &str) -> Mod {
    let sp = crate::source::Span::default();
    Mod {
        types: Vec::new(),
        impls: Vec::new(),
        uses: Vec::new(),
        name: Ident {
            name: name.to_string(),
            span: sp,
        },
        attrs: vec![summary_attr(summary)],
        fns: Vec::new(),
        mods: Vec::new(),
        span: sp,
    }
}

/// Splits a `::` path whose segments must all be identifiers (backlog 1D-15:
/// unit names become file names, so `../x` must never get through).
fn segments<'p>(k: usize, what: &str, path: &'p str) -> Result<Vec<&'p str>, String> {
    let segs: Vec<&str> = path.split("::").collect();
    if segs.iter().all(|s| parser::is_ident(s)) {
        Ok(segs)
    } else {
        Err(format!(
            "op {k}: {what} `{path}` is not a valid path (each `::` segment must be an identifier, not a keyword)"
        ))
    }
}

fn hashes(p: &Program, fns: &BTreeMap<String, check::FnSig>) -> BTreeMap<String, String> {
    fmt::hashes(p, fns)
        .into_iter()
        .map(|h| (h.path, h.body))
        .collect()
}

/// Applies `ops` to the program in `files` and checks the result. Nothing is
/// written; `Ok` carries the new normal-form files (unit name -> text).
pub fn plan(files: &[(String, String)], ops: &[Op]) -> (Commit, Option<BTreeMap<String, String>>) {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = Program { mods: Vec::new() };
    for (name, text) in files {
        program
            .mods
            .extend(parse_into(&mut sm, &mut diags, name, text).mods);
    }
    let before = if diags.has_errors() {
        BTreeMap::new()
    } else {
        let mut d2 = Diags::default();
        let mut c = check::Checker::new(&mut d2);
        c.check_program(&program);
        let t = c.into_tables();
        hashes(&program, &t.fns)
    };

    let mut problems: Vec<String> = Vec::new();
    for (k, op) in ops.iter().enumerate() {
        match op {
            Op::Unit { path, summary } => {
                let segs = match segments(k, "unit", path) {
                    Ok(s) => s,
                    Err(e) => {
                        problems.push(e);
                        continue;
                    }
                };
                if let Some(m) = find_mod(&mut program.mods, &segs) {
                    m.attrs.retain(|a| a.name.name != "summary");
                    m.attrs.insert(0, summary_attr(summary));
                } else {
                    let (last, parent) = segs.split_last().unwrap();
                    let target = if parent.is_empty() {
                        Some(&mut program.mods)
                    } else {
                        find_mod(&mut program.mods, parent).map(|m| &mut m.mods)
                    };
                    match target {
                        Some(list) => list.push(new_mod(last, summary)),
                        None => {
                            problems.push(format!("op {k}: parent unit of `{path}` does not exist"))
                        }
                    }
                }
            }
            Op::Put { unit, code } => {
                let segs = match segments(k, "unit", unit) {
                    Ok(s) => s,
                    Err(e) => {
                        problems.push(e);
                        continue;
                    }
                };
                let last = segs.last().copied().unwrap_or("");
                // Wrap on the first line so diagnostic lines match the submitted code.
                let wrapped = format!("#[summary(\"\")] mod {last} {{ {code}\n}}\n");
                let before = diags.items.len();
                let parsed = parse_into(&mut sm, &mut diags, &format!("tx/op{k}"), &wrapped);
                let syntax_ok = diags.items.len() == before;
                // The code must stay inside the wrapper and hold only definitions:
                // anything else would be dropped silently.
                if syntax_ok && (parsed.mods.len() != 1 || !parsed.mods[0].mods.is_empty()) {
                    problems.push(format!(
                        "op {k}: `code` must contain only `fn` / `spec fn` definitions (create units with a `unit` op)"
                    ));
                    continue;
                }
                let new_fns: Vec<FnDef> = parsed.mods.into_iter().flat_map(|m| m.fns).collect();
                if syntax_ok && new_fns.is_empty() {
                    problems.push(format!("op {k}: `code` contains no definition"));
                    continue;
                }
                let Some(m) = find_mod(&mut program.mods, &segs) else {
                    problems.push(format!(
                        "op {k}: unit `{unit}` does not exist (create it with a `unit` op)"
                    ));
                    continue;
                };
                for f in new_fns {
                    m.fns.retain(|g| g.name.name != f.name.name);
                    m.fns.push(f);
                }
            }
            Op::Remove { def } => {
                let segs = match segments(k, "definition", def) {
                    Ok(s) => s,
                    Err(e) => {
                        problems.push(e);
                        continue;
                    }
                };
                let (name, unit) = segs.split_last().unwrap();
                match find_mod(&mut program.mods, unit) {
                    Some(m) if m.fns.iter().any(|f| f.name.name == *name) => {
                        m.fns.retain(|f| f.name.name != *name)
                    }
                    _ => problems.push(format!("op {k}: no definition `{def}`")),
                }
            }
            Op::RemoveUnit { path } => {
                let segs = match segments(k, "unit", path) {
                    Ok(s) => s,
                    Err(e) => {
                        problems.push(e);
                        continue;
                    }
                };
                let (last, parent) = segs.split_last().unwrap();
                let list = if parent.is_empty() {
                    Some(&mut program.mods)
                } else {
                    find_mod(&mut program.mods, parent).map(|m| &mut m.mods)
                };
                match list {
                    Some(l) if l.iter().any(|m| m.name.name == *last) => {
                        l.retain(|m| m.name.name != *last)
                    }
                    _ => problems.push(format!("op {k}: no unit `{path}`")),
                }
            }
        }
    }

    let mut c = check::Checker::new(&mut diags);
    c.check_program(&program);
    let tables = c.into_tables();
    let mut diagnostics = diags.to_jsonl(&sm);
    for p in &problems {
        diagnostics.push_str(&format!("{{\"error\":{}}}\n", crate::diag::json_str(p)));
    }
    if !diagnostics.is_empty() {
        let commit = Commit {
            ok: false,
            diagnostics,
            changed: Vec::new(),
            removed: Vec::new(),
        };
        return (commit, None);
    }
    let after = hashes(&program, &tables.fns);
    let changed = after
        .iter()
        .filter(|(p, h)| before.get(*p) != Some(*h))
        .map(|(p, _)| p.clone())
        .collect();
    let removed = before
        .keys()
        .filter(|p| !after.contains_key(*p))
        .cloned()
        .collect();
    let commit = Commit {
        ok: true,
        diagnostics: String::new(),
        changed,
        removed,
    };
    (commit, Some(fmt::format_units(&program)))
}

/// Commits `ops` to the workspace at `root`: files change only on success.
pub fn commit(root: &Path, ops: &[Op]) -> Result<Commit, String> {
    let dir = src_dir(root);
    let files = if dir.is_dir() {
        read_dir_sources(root)?
    } else {
        Vec::new()
    };
    let (result, units) = plan(&files, ops);
    if let Some(units) = units {
        write_units(root, &units)?;
    }
    Ok(result)
}

/// Writes the normal-form files and removes files of units that no longer exist.
///
/// Every unit name must be an identifier (it becomes a file name). New contents
/// go to temporary files first and are renamed into place only when all of them
/// were written, so a failed write leaves the old files untouched; stale files
/// are removed last (backlog 1D-19).
pub fn write_units(root: &Path, units: &BTreeMap<String, String>) -> Result<(), String> {
    if let Some(bad) = units.keys().find(|u| !parser::is_ident(u)) {
        return Err(format!("unit name `{bad}` is not an identifier"));
    }
    let dir = src_dir(root);
    refuse_symlink(&dir)?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let tmp = |unit: &str| dir.join(format!(".{unit}.bmb.tmp"));
    let mut written = Vec::new();
    for (unit, text) in units {
        let t = tmp(unit);
        if let Err(e) = write_new(&t, text) {
            let _ = std::fs::remove_file(&t);
            for w in &written {
                let _ = std::fs::remove_file(w);
            }
            return Err(format!("cannot write {}: {e}", t.display()));
        }
        written.push(t);
    }
    for (unit, t) in units.keys().zip(&written) {
        let p = dir.join(format!("{unit}.bmb"));
        std::fs::rename(t, &p)
            .map_err(|e| format!("cannot rename {} to {}: {e}", t.display(), p.display()))?;
    }
    for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        let stem = p
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if p.extension().is_some_and(|x| x == "bmb") && !units.contains_key(&stem) {
            std::fs::remove_file(&p).map_err(|e| format!("cannot remove {}: {e}", p.display()))?;
        }
    }
    Ok(())
}

/// Parses transaction operations from a JSON array, e.g.
/// `[{"op":"unit","path":"a","summary":"..."},{"op":"put","unit":"a","code":"fn .."},
///   {"op":"remove","def":"a::f"},{"op":"remove_unit","path":"a"}]`.
pub fn parse_ops(json: &str) -> Result<Vec<Op>, String> {
    let v = crate::json::parse(json)?;
    let crate::json::Value::Array(items) = v else {
        return Err("transaction must be a JSON array of operations".into());
    };
    let mut ops = Vec::new();
    for (k, it) in items.iter().enumerate() {
        let get = |key: &str| -> Result<String, String> {
            it.get(key)
                .and_then(|x| x.as_str())
                .map(String::from)
                .ok_or(format!("op {k}: missing string field `{key}`"))
        };
        let op = match get("op")?.as_str() {
            "unit" => Op::Unit {
                path: get("path")?,
                summary: get("summary")?,
            },
            "put" => Op::Put {
                unit: get("unit")?,
                code: get("code")?,
            },
            "remove" => Op::Remove { def: get("def")? },
            "remove_unit" => Op::RemoveUnit { path: get("path")? },
            other => return Err(format!("op {k}: unknown op `{other}`")),
        };
        ops.push(op);
    }
    Ok(ops)
}
