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
use crate::refs::{Decl, Rewrite, Site};
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
    /// Rename a function, `spec fn`, `impl` function, type or trait (full
    /// path) to `to` (one identifier). The tool rewrites the definition and
    /// every reference to it; no identity changes (ADR-0059 §5), and a
    /// rename that would change one (a new name that captures other
    /// references) is rejected.
    Rename { def: String, to: String },
    /// Commit only if the definition at `def` (full path) has identity `id`
    /// in the workspace the transaction starts from (ADR-0059 §5:
    /// optimistic concurrency).
    Expect { def: String, id: String },
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
        traits: Vec::new(),
        trait_impls: Vec::new(),
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

/// The items of `m` other than `fn` / `spec fn` definitions (what `put`
/// cannot write), named as written, in source order.
fn non_fn_items(m: &Mod) -> Vec<String> {
    let fns = |fs: &[FnDef]| {
        fs.iter()
            .map(|f| format!("`fn {}`", f.name.name))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let args = |ts: &[Type]| {
        if ts.is_empty() {
            String::new()
        } else {
            let ts: Vec<String> = ts.iter().map(fmt::type_str).collect();
            format!("<{}>", ts.join(", "))
        }
    };
    let mut items: Vec<(u32, String)> = Vec::new();
    for t in &m.types {
        let kind = match t.kind {
            TypeDefKind::Struct(_) => "struct",
            TypeDefKind::Enum(_) => "enum",
        };
        items.push((t.span.lo, format!("`{kind} {}`", t.name.name)));
    }
    for i in &m.impls {
        let head = format!("impl {}{}", i.ty.name, args(&i.ty_args));
        let text = if i.fns.is_empty() {
            format!("`{head}`")
        } else {
            format!("`{head}` ({})", fns(&i.fns))
        };
        items.push((i.span.lo, text));
    }
    for t in &m.traits {
        items.push((t.span.lo, format!("`trait {}`", t.name.name)));
    }
    for i in &m.trait_impls {
        let tr: Vec<&str> = i.trait_path.iter().map(|s| s.name.as_str()).collect();
        let head = format!("impl {} for {}", tr.join("::"), fmt::type_str(&i.self_ty));
        let text = if i.fns.is_empty() {
            format!("`{head}`")
        } else {
            format!("`{head}` ({})", fns(&i.fns))
        };
        items.push((i.span.lo, text));
    }
    for u in &m.uses {
        let p: Vec<&str> = u.path.iter().map(|s| s.name.as_str()).collect();
        items.push((u.span.lo, format!("`use {}`", p.join("::"))));
    }
    items.sort();
    items.into_iter().map(|(_, s)| s).collect()
}

/// What a commit compares: each definition's own text, other definitions
/// by path (`fmt::qualified_hashes`). A body edit reports that definition
/// alone, though the identities of its referrers change too (ADR-0059).
fn hashes(p: &Program, fns: &BTreeMap<String, check::FnSig>) -> BTreeMap<String, String> {
    fmt::qualified_hashes(p, fns)
}

/// The identity of every definition, type and trait, by path (ADR-0059 §1).
fn identities(p: &Program, t: &crate::verify::Tables) -> BTreeMap<String, String> {
    let ids = crate::identity::Ids::new(p, t);
    let keys: Vec<crate::identity::Key> = ids.keys().cloned().collect();
    keys.into_iter()
        .filter_map(|k| Some((k.path().to_string(), ids.get(&k)?.id)))
        .collect()
}

/// Checks a copy of `p`: its tables, or `None` when it does not check.
fn checked(p: &Program) -> Option<crate::verify::Tables> {
    let mut d = Diags::default();
    let mut c = check::Checker::new(&mut d);
    c.check_program(p);
    let t = c.into_tables();
    (!d.has_errors()).then_some(t)
}

/// What `rename` renames: the full path of its unit, and the segments that
/// name it there (`f`, `T::f`, `T`, `Trait`).
struct Target {
    unit: Vec<String>,
    local: Vec<String>,
    kind: TargetKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    Fn,
    ImplFn,
    Type,
    Trait,
}

/// Renames one definition and every reference to it (ADR-0059 §5).
struct Renamer<'a> {
    target: Target,
    to: &'a str,
}

impl Rewrite for Renamer<'_> {
    /// Rewrites `segs` (as written in `unit`) if they name the target at
    /// `site`.
    fn path(&mut self, segs: &mut Vec<String>, site: Site, unit: &[String], tparams: &[String]) {
        let t = &self.target;
        let full: Vec<String> = t.unit.iter().chain(&t.local).cloned().collect();
        // How many segments past the target's name the path has at `site`.
        let extra: &[usize] = match (t.kind, site) {
            (TargetKind::Fn | TargetKind::ImplFn, Site::Call) => &[0],
            (TargetKind::Type, Site::Call | Site::Variant) => &[1],
            (TargetKind::Type, Site::Type | Site::Struct) => &[0],
            (TargetKind::Trait, Site::Call) => &[1],
            (TargetKind::Trait, Site::Bound) => &[0],
            _ => &[],
        };
        let hit = |prefix: &[String]| {
            extra.iter().any(|e| segs.len() == prefix.len() + e) && segs.starts_with(prefix)
        };
        if hit(&full) {
            segs[full.len() - 1] = self.to.to_string();
        } else if unit == t.unit.as_slice()
            && hit(&t.local)
            && !(site == Site::Type && segs.len() == 1 && tparams.contains(&segs[0]))
        {
            segs[t.local.len() - 1] = self.to.to_string();
        }
    }

    /// The target's own declaration (a type's also in its `impl` head).
    fn decl(&mut self, unit: &[String], d: Decl<'_>) {
        if unit != self.target.unit.as_slice() {
            return;
        }
        let (kind, local) = (self.target.kind, &self.target.local);
        let name = match d {
            Decl::Fn(n) if kind == TargetKind::Fn && n.name == local[0] => n,
            Decl::Type(n) | Decl::ImplTy(n) if kind == TargetKind::Type && n.name == local[0] => n,
            Decl::ImplFn { ty, name }
                if kind == TargetKind::ImplFn && ty == local[0] && name.name == local[1] =>
            {
                name
            }
            Decl::Trait(n) if kind == TargetKind::Trait && n.name == local[0] => n,
            _ => return,
        };
        name.name = self.to.to_string();
    }
}

/// Renames the definition at `def` to `to` in `p` (ADR-0059 §5): its
/// declaration and every reference, which the checker resolves first. Every
/// identity must stay as it was; otherwise (the new name captures other
/// references, or the target is not one `rename` handles) it fails and `p`
/// is left as it was.
fn rename(p: &mut Program, def: &str, to: &str) -> Result<(), String> {
    let t = checked(p).ok_or("`rename` needs a program that checks; this one does not")?;
    let fn_sig = t.fns.get(def).filter(|s| !s.builtin);
    let segs: Vec<String> = def.split("::").map(String::from).collect();
    let (kind, n_local) = if t.adts.contains_key(def) {
        (TargetKind::Type, 1)
    } else if fn_sig.is_some_and(|s| s.trait_of.is_some()) {
        return Err(format!(
            "`{def}` is a trait method: `rename` takes functions, types and traits"
        ));
    } else if fn_sig.is_some_and(|s| s.in_impl) {
        (TargetKind::ImplFn, 2)
    } else if fn_sig.is_some() {
        (TargetKind::Fn, 1)
    } else if trait_paths(p).contains(def) {
        (TargetKind::Trait, 1)
    } else {
        return Err(format!("no function, type or trait `{def}`"));
    };
    if segs.len() <= n_local {
        return Err(format!("`{def}` is not a full path"));
    }
    let before = identities(p, &t);
    let target = Target {
        unit: segs[..segs.len() - n_local].to_vec(),
        local: segs[segs.len() - n_local..].to_vec(),
        kind,
    };
    let mut renamed = p.clone();
    let mut r = Renamer { target, to };
    crate::refs::walk(&mut renamed, &mut r);
    // The program after the rename is checked with the transaction's other
    // operations; if it checks here, no identity may have moved.
    if let Some(t2) = checked(&renamed) {
        let after = identities(&renamed, &t2);
        let mut a: Vec<&String> = before.values().collect();
        let mut b: Vec<&String> = after.values().collect();
        a.sort();
        b.sort();
        if a != b {
            let moved: Vec<&str> = after
                .iter()
                .filter(|(_, id)| !before.values().any(|x| x == *id))
                .map(|(p, _)| p.as_str())
                .collect();
            return Err(format!(
                "renaming `{def}` to `{to}` would change what other names refer to ({})",
                moved.join(", ")
            ));
        }
    }
    *p = renamed;
    Ok(())
}

/// The full paths of the trait declarations.
pub(crate) fn trait_paths(p: &Program) -> std::collections::BTreeSet<String> {
    fn walk(m: &Mod, prefix: &str, out: &mut std::collections::BTreeSet<String>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            walk(s, &format!("{mp}::"), out);
        }
        out.extend(m.traits.iter().map(|t| format!("{mp}::{}", t.name.name)));
    }
    let mut out = std::collections::BTreeSet::new();
    for m in &p.mods {
        walk(m, "", &mut out);
    }
    out
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
    let (before, before_ids) = before_state(&program, &diags);

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
                // `put` writes functions only; anything else in `code` rejects
                // the transaction rather than being dropped (backlog T0-20).
                if let Some(m) = parsed.mods.first().filter(|_| syntax_ok) {
                    let others = non_fn_items(m);
                    if !others.is_empty() {
                        problems.push(format!(
                            "op {k}: `put` writes only `fn` and `spec fn` definitions, and `code` holds {}{}; no transaction operation adds or edits types, `impl` blocks, traits, trait impls or `use` items",
                            others.join(", "),
                            if m.fns.is_empty() {
                                ""
                            } else {
                                " besides its functions"
                            },
                        ));
                        continue;
                    }
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
            Op::Rename { def, to } => {
                if let Err(e) = segments(k, "definition", def) {
                    problems.push(e);
                    continue;
                }
                if !parser::is_ident(to) {
                    problems.push(format!(
                        "op {k}: new name `{to}` is not an identifier (rename keeps the unit; give the last segment only)"
                    ));
                    continue;
                }
                if let Err(e) = rename(&mut program, def, to) {
                    problems.push(format!("op {k}: {e}"));
                }
            }
            Op::Expect { def, id } => match before_ids.as_ref().map(|m| m.get(def.as_str())) {
                None => problems.push(format!(
                    "op {k}: `expect` needs a workspace that checks; it does not"
                )),
                Some(None) => problems.push(format!("op {k}: no definition `{def}`")),
                Some(Some(cur)) if cur != id => problems.push(format!(
                    "op {k}: `{def}` has identity {cur}, not {id}: it changed since it was read"
                )),
                Some(Some(_)) => {}
            },
        }
    }

    finish(program, sm, diags, &problems, &before)
}

/// The qualified hashes `changed` compares, and the identities `expect`
/// reads (only of a workspace that checks).
fn before_state(
    program: &Program,
    diags: &Diags,
) -> (BTreeMap<String, String>, Option<BTreeMap<String, String>>) {
    if diags.has_errors() {
        return (BTreeMap::new(), None);
    }
    let mut d2 = Diags::default();
    let mut c = check::Checker::new(&mut d2);
    c.check_program(program);
    let t = c.into_tables();
    let ids = (!d2.has_errors()).then(|| identities(program, &t));
    (hashes(program, &t.fns), ids)
}

/// Checks the new program and reports what changed against `before`.
fn finish(
    program: Program,
    sm: SourceMap,
    mut diags: Diags,
    problems: &[String],
    before: &BTreeMap<String, String>,
) -> (Commit, Option<BTreeMap<String, String>>) {
    let mut c = check::Checker::new(&mut diags);
    c.check_program(&program);
    let tables = c.into_tables();
    let mut diagnostics = diags.to_jsonl(&sm);
    for p in problems {
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

/// The workspace files `src/UNIT.bmb` as `(name, text)` with `name`
/// relative to the workspace (`src/calc.bmb`), sorted by name.
pub fn relative_sources(root: &Path) -> Result<Vec<(String, String)>, String> {
    if !src_dir(root).is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<(String, String)> = read_dir_sources(root)?
        .into_iter()
        .map(|(p, t)| {
            let name = Path::new(&p)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(p);
            (format!("src/{name}"), t)
        })
        .collect();
    out.sort();
    Ok(out)
}

/// Replaces the workspace's files with `new` (the file diff and JSON AST
/// edit surfaces, ADR-0073): `old` and `new` are whole files
/// `(src/UNIT.bmb, text)`. As a transaction (ADR-0009): the result must
/// have no structural or type error, and each file must hold exactly the
/// top-level unit its name says (the tools lay files out, ADR-0011).
/// Nothing is written; `Ok` carries the new normal-form files.
pub fn plan_files(
    old: &[(String, String)],
    new: &[(String, String)],
) -> (Commit, Option<BTreeMap<String, String>>) {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut before_program = Program { mods: Vec::new() };
    for (name, text) in old {
        before_program
            .mods
            .extend(parse_into(&mut sm, &mut diags, name, text).mods);
    }
    let (before, _) = before_state(&before_program, &diags);
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = Program { mods: Vec::new() };
    let mut problems = Vec::new();
    for (name, text) in new {
        let parsed = parse_into(&mut sm, &mut diags, name, text);
        let unit = Path::new(name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let units: Vec<&str> = parsed.mods.iter().map(|m| m.name.name.as_str()).collect();
        if units != [unit.as_str()] && !diags.has_errors() {
            problems.push(format!(
                "{name} must hold exactly one top-level unit, `mod {unit}` (it holds {})",
                if units.is_empty() {
                    "none".to_string()
                } else {
                    units
                        .iter()
                        .map(|u| format!("`mod {u}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ));
        }
        program.mods.extend(parsed.mods);
    }
    finish(program, sm, diags, &problems, &before)
}

/// Parses `files` without checking them: the program, its source map and
/// the syntax diagnostics.
pub fn parse_files(files: &[(String, String)]) -> (Program, SourceMap, Diags) {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = Program { mods: Vec::new() };
    for (name, text) in files {
        program
            .mods
            .extend(parse_into(&mut sm, &mut diags, name, text).mods);
    }
    (program, sm, diags)
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
///   {"op":"remove","def":"a::f"},{"op":"remove_unit","path":"a"},
///   {"op":"rename","def":"a::f","to":"g"},{"op":"expect","def":"a::g","id":"sha256:.."}]`.
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
            "rename" => Op::Rename {
                def: get("def")?,
                to: get("to")?,
            },
            "expect" => Op::Expect {
                def: get("def")?,
                id: get("id")?,
            },
            other => return Err(format!("op {k}: unknown op `{other}`")),
        };
        ops.push(op);
    }
    Ok(ops)
}
