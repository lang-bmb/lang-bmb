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

/// Where a path is written, which decides how it can name the target.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Site {
    /// The callee of a call (`f(..)`, `T::new(..)`, `T::V(..)`,
    /// `Trait::m(..)`), or a `#[uses]` target.
    Call,
    /// A type (`T`, `geo::T<A>`).
    Type,
    /// A struct literal's type.
    Struct,
    /// A unit variant or a pattern (`T::V`).
    Variant,
    /// A trait bound or the trait of an `impl`.
    Bound,
}

/// Renames one definition and every reference to it (ADR-0059 §5).
struct Renamer<'a> {
    target: Target,
    to: &'a str,
    /// The unit being walked.
    unit: Vec<String>,
    /// Type parameters in scope (a one-segment type path that names one is
    /// no reference to a type).
    tparams: Vec<String>,
}

impl Renamer<'_> {
    /// Rewrites `segs` (as written in the current unit) if they name the
    /// target at `site`.
    fn segs(&self, segs: &mut [String], site: Site) {
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
        } else if self.unit == t.unit
            && hit(&t.local)
            && !(site == Site::Type && segs.len() == 1 && self.tparams.contains(&segs[0]))
        {
            segs[t.local.len() - 1] = self.to.to_string();
        }
    }

    fn idents(&self, path: &mut [Ident], site: Site) {
        let mut s: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
        self.segs(&mut s, site);
        for (i, n) in path.iter_mut().zip(s) {
            i.name = n;
        }
    }

    fn ty(&self, t: &mut Type) {
        match &mut t.kind {
            TypeKind::Named { path, args } => {
                self.segs(path, Site::Type);
                args.iter_mut().for_each(|a| self.ty(a));
            }
            TypeKind::Array(e, _) | TypeKind::Slice(e) | TypeKind::Vec(e) | TypeKind::Box(e) => {
                self.ty(e)
            }
            TypeKind::Ref { inner, .. } => self.ty(inner),
            _ => {}
        }
    }

    fn generics(&mut self, gs: &mut [TypeParam]) {
        for g in gs {
            self.tparams.push(g.name.name.clone());
            for b in &mut g.bounds {
                self.idents(b, Site::Bound);
            }
        }
    }

    fn function(&mut self, f: &mut FnDef) {
        let depth = self.tparams.len();
        self.generics(&mut f.generics);
        for a in f.attrs.iter_mut().filter(|a| a.name.name == "uses") {
            for x in &mut a.args {
                if let AttrArg::Ident(i) = x {
                    let mut s: Vec<String> = i.name.split("::").map(String::from).collect();
                    self.segs(&mut s, Site::Call);
                    i.name = s.join("::");
                }
            }
        }
        for p in &mut f.params {
            self.ty(&mut p.ty);
        }
        self.ty(&mut f.ret.ty);
        for e in f.requires.iter_mut().chain(&mut f.ensures) {
            self.expr(e);
        }
        if let Some(d) = &mut f.decreases {
            self.expr(d);
        }
        self.block(&mut f.body);
        self.tparams.truncate(depth);
    }

    fn block(&self, b: &mut Block) {
        for s in &mut b.stmts {
            match s {
                Stmt::Let { ty, init, .. } => {
                    self.ty(ty);
                    self.expr(init);
                }
                Stmt::Assign { target, value, .. } => {
                    self.expr(target);
                    self.expr(value);
                }
                Stmt::While {
                    cond,
                    invariants,
                    decreases,
                    body,
                    ..
                } => {
                    self.expr(cond);
                    invariants.iter_mut().for_each(|e| self.expr(e));
                    if let Some(d) = decreases {
                        self.expr(d);
                    }
                    self.block(body);
                }
                Stmt::For {
                    lo,
                    hi,
                    invariants,
                    body,
                    ..
                } => {
                    self.expr(lo);
                    self.expr(hi);
                    invariants.iter_mut().for_each(|e| self.expr(e));
                    self.block(body);
                }
                Stmt::Assert { cond, .. } => self.expr(cond),
                Stmt::Proof { body, .. } => self.block(body),
                Stmt::LetFields { path, init, .. } => {
                    self.idents(path, Site::Struct);
                    self.expr(init);
                }
                Stmt::Expr(e) => self.expr(e),
            }
        }
        if let Some(t) = &mut b.tail {
            self.expr(t);
        }
    }

    fn expr(&self, e: &mut Expr) {
        match &mut e.kind {
            ExprKind::Call { path, targs, args } => {
                self.idents(path, Site::Call);
                targs
                    .iter_mut()
                    .flat_map(|t| &mut t.args)
                    .for_each(|t| self.ty(t));
                args.iter_mut().for_each(|a| self.expr(a));
            }
            ExprKind::Method { recv, args, .. } => {
                self.expr(recv);
                args.iter_mut().for_each(|a| self.expr(a));
            }
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            ExprKind::Unary(_, x)
            | ExprKind::Ref { expr: x, .. }
            | ExprKind::Old(x)
            | ExprKind::Trigger(x)
            | ExprKind::Deref(x)
            | ExprKind::Try(x) => self.expr(x),
            ExprKind::Field { base, .. } => self.expr(base),
            ExprKind::Binary(_, l, r) => {
                self.expr(l);
                self.expr(r);
            }
            ExprKind::If { cond, then, els } => {
                self.expr(cond);
                self.block(then);
                if let Some(x) = els {
                    self.expr(x);
                }
            }
            ExprKind::Block(b) => self.block(b),
            ExprKind::Array(xs) => xs.iter_mut().for_each(|x| self.expr(x)),
            ExprKind::Repeat { value, .. } => self.expr(value),
            ExprKind::Forall { binders, body } => {
                binders.iter_mut().for_each(|b| self.ty(&mut b.ty));
                self.expr(body);
            }
            ExprKind::StructLit { path, fields, rest } => {
                self.idents(path, Site::Struct);
                fields.iter_mut().for_each(|(_, x)| self.expr(x));
                if let Some(r) = rest {
                    self.expr(r);
                }
            }
            ExprKind::Path(path) => self.idents(path, Site::Variant),
            ExprKind::Match { scrut, arms } => {
                self.expr(scrut);
                for a in arms {
                    self.idents(&mut a.pat.path, Site::Variant);
                    self.expr(&mut a.body);
                }
            }
            ExprKind::Return(x) | ExprKind::Break { value: x, .. } => {
                if let Some(x) = x {
                    self.expr(x);
                }
            }
            ExprKind::Loop {
                invariants,
                decreases,
                body,
                ..
            } => {
                invariants.iter_mut().for_each(|e| self.expr(e));
                if let Some(d) = decreases {
                    self.expr(d);
                }
                self.block(body);
            }
            ExprKind::Int(..)
            | ExprKind::ByteLit(_)
            | ExprKind::ByteStr(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Var(_)
            | ExprKind::Continue { .. }
            | ExprKind::Error => {}
        }
    }

    /// Walks unit `m` (and its sub-units): the definition itself and every
    /// reference.
    fn module(&mut self, m: &mut Mod, prefix: &[String]) {
        let mut unit = prefix.to_vec();
        unit.push(m.name.name.clone());
        for s in &mut m.mods {
            self.module(s, &unit);
        }
        self.unit = unit.clone();
        let here = unit == self.target.unit;
        let (kind, local) = (self.target.kind, self.target.local.clone());
        let to = self.to.to_string();
        for f in &mut m.fns {
            if here && kind == TargetKind::Fn && f.name.name == local[0] {
                f.name.name = to.clone();
            }
            self.function(f);
        }
        for td in &mut m.types {
            if here && kind == TargetKind::Type && td.name.name == local[0] {
                td.name.name = to.clone();
            }
            let depth = self.tparams.len();
            self.generics(&mut td.generics);
            match &mut td.kind {
                TypeDefKind::Struct(fs) => fs.iter_mut().for_each(|f| self.ty(&mut f.ty)),
                TypeDefKind::Enum(vs) => vs
                    .iter_mut()
                    .flat_map(|v| &mut v.payload)
                    .for_each(|t| self.ty(t)),
            }
            self.tparams.truncate(depth);
        }
        for i in &mut m.impls {
            let of_target = here && i.ty.name == local[0];
            if of_target && kind == TargetKind::Type {
                i.ty.name = to.clone();
            }
            let depth = self.tparams.len();
            self.generics(&mut i.generics);
            i.ty_args.iter_mut().for_each(|t| self.ty(t));
            for f in &mut i.fns {
                if of_target && kind == TargetKind::ImplFn && f.name.name == local[1] {
                    f.name.name = to.clone();
                }
                self.function(f);
            }
            self.tparams.truncate(depth);
        }
        for i in &mut m.trait_impls {
            let depth = self.tparams.len();
            self.generics(&mut i.generics);
            self.idents(&mut i.trait_path, Site::Bound);
            self.ty(&mut i.self_ty);
            for f in &mut i.fns {
                self.function(f);
            }
            self.tparams.truncate(depth);
        }
        for tr in &mut m.traits {
            if here && kind == TargetKind::Trait && tr.name.name == local[0] {
                tr.name.name = to.clone();
            }
            for tf in &mut tr.fns {
                self.function(&mut tf.f);
            }
        }
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
    let mut r = Renamer {
        target,
        to,
        unit: Vec::new(),
        tparams: Vec::new(),
    };
    for m in &mut renamed.mods {
        r.module(m, &[]);
    }
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
fn trait_paths(p: &Program) -> std::collections::BTreeSet<String> {
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
    // The qualified hashes `changed` compares, and the identities `expect`
    // reads (only of a workspace that checks).
    let (before, before_ids) = if diags.has_errors() {
        (BTreeMap::new(), None)
    } else {
        let mut d2 = Diags::default();
        let mut c = check::Checker::new(&mut d2);
        c.check_program(&program);
        let t = c.into_tables();
        let ids = (!d2.has_errors()).then(|| identities(&program, &t));
        (hashes(&program, &t.fns), ids)
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
