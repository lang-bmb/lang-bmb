//! Standard maps L0-L2 and read queries (ADR-0020, backlog 1D-05, 1D-06).

use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::json_str;
use crate::source::{SourceMap, Span};
use crate::verify::{Outcome, Tables};

pub const HEADER: &str = "{\"schema\":\"bmb.map\",\"version\":1}";

/// A JSON value with keys kept in insertion order.
pub enum J {
    S(String),
    N(u64),
    B(bool),
    Null,
    A(Vec<J>),
    O(Vec<(&'static str, J)>),
}

impl J {
    pub fn render(&self) -> String {
        match self {
            J::S(s) => json_str(s),
            J::N(n) => n.to_string(),
            J::B(b) => b.to_string(),
            J::Null => "null".into(),
            J::A(xs) => format!(
                "[{}]",
                xs.iter().map(J::render).collect::<Vec<_>>().join(",")
            ),
            J::O(kv) => format!(
                "{{{}}}",
                kv.iter()
                    .map(|(k, v)| format!("{}:{}", json_str(k), v.render()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

fn strs<I: IntoIterator<Item = String>>(xs: I) -> J {
    J::A(xs.into_iter().map(J::S).collect())
}

/// One definition of an `Index`: what the maps know of it without proof
/// results.
#[derive(Clone)]
pub struct Entry {
    pub path: String,
    pub unit: String,
    /// Position of the definition in program walk order (`fn_order`).
    idx: usize,
    pub file: String,
    pub line: u32,
    /// Body and interface hashes (ADR-0019 §2), computed on first use: most
    /// queries never read them (backlog 3A-03).
    hashes: OnceCell<(String, String)>,
}

/// A definition as a query sees it: its index entry, its syntax and its
/// proof status.
pub struct Def<'m> {
    e: &'m Entry,
    pub f: &'m FnDef,
    pub status: &'static str,
    pub obligations: usize,
    pub failed: usize,
    pub rlimit_max: u64,
}

impl std::ops::Deref for Def<'_> {
    type Target = Entry;
    fn deref(&self) -> &Entry {
        self.e
    }
}

/// A type declaration of an `Index` (ADR-0026 §6).
#[derive(Clone)]
pub struct TypeEntry {
    pub path: String,
    pub unit: String,
    /// `"struct"`, `"enum"` or `"trait"`.
    pub kind: &'static str,
    /// The declaration in normal form on one line.
    pub text: String,
    pub hash: String,
    pub file: String,
    pub line: u32,
    /// Types its fields or payloads name.
    pub names: BTreeSet<String>,
}

#[derive(Clone)]
pub struct Unit {
    pub path: String,
    pub summary: String,
    pub units: Vec<String>,
    pub defs: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: &'static str,
    pub arg: Option<usize>,
    /// The type arguments of a call of a generic definition (ADR-0046 §3).
    pub targs: Option<String>,
}

/// Everything the maps need from a checked program except proof results.
/// It holds no reference into the program, so a long-lived MCP server keeps
/// it between calls while the program is unchanged (backlog 3A-03) and
/// answers each query through `Model::view`.
#[derive(Clone)]
pub struct Index {
    pub units: BTreeMap<String, Unit>,
    pub defs: BTreeMap<String, Entry>,
    /// Struct and enum declarations by path.
    pub types: BTreeMap<String, TypeEntry>,
    pub edges: BTreeSet<Edge>,
    /// Executable definitions with effect `io`, built-ins (`print_i64`)
    /// included.
    pub io: BTreeSet<String>,
    /// `edges` by target, each list in edge order.
    incoming: BTreeMap<String, Vec<Edge>>,
}

/// The maps of one program with one proof outcome.
pub struct Model<'p> {
    ix: Cow<'p, Index>,
    /// The program's definitions in walk order (`Entry::idx`).
    fns: Vec<&'p FnDef>,
    sigs: &'p BTreeMap<String, crate::check::FnSig>,
    /// Proof results by path: (obligations, failed, rlimit_max).
    status: BTreeMap<String, (usize, usize, u64)>,
}

impl std::ops::Deref for Model<'_> {
    type Target = Index;
    fn deref(&self) -> &Index {
        &self.ix
    }
}

fn status_map(outcome: &Outcome) -> BTreeMap<String, (usize, usize, u64)> {
    outcome
        .statuses
        .iter()
        .map(|s| (s.path.clone(), (s.obligations, s.failed, s.rlimit_max)))
        .collect()
}

fn resolve(path: &[Ident], unit: &str, t: &Tables) -> String {
    let plain = path
        .iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    // `f`, `T::f` (an `impl` function of the unit's type) and `Trait::f` (a
    // method of the unit's trait) are local, as the checker resolves them.
    let local = format!("{unit}::{plain}");
    match t.fns.get(&local) {
        Some(sig)
            if path.len() == 1 || (path.len() == 2 && (sig.in_impl || sig.trait_of.is_some())) =>
        {
            local
        }
        _ => plain,
    }
}

/// Calls in an expression, with whether they occur in specification context.
fn calls_in_expr<'e>(
    e: &'e Expr,
    spec: bool,
    out: &mut Vec<(&'e [Ident], &'e [Expr], bool, Span)>,
) {
    match &e.kind {
        ExprKind::Call { path, args, .. } => {
            out.push((path, args, spec, e.span));
            args.iter().for_each(|a| calls_in_expr(a, spec, out));
        }
        ExprKind::Method { recv, args, .. } => {
            calls_in_expr(recv, spec, out);
            args.iter().for_each(|a| calls_in_expr(a, spec, out));
        }
        ExprKind::Index { base, index } => {
            calls_in_expr(base, spec, out);
            calls_in_expr(index, spec, out);
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x) => calls_in_expr(x, spec, out),
        ExprKind::Binary(_, l, r) => {
            calls_in_expr(l, spec, out);
            calls_in_expr(r, spec, out);
        }
        ExprKind::If { cond, then, els } => {
            calls_in_expr(cond, spec, out);
            calls_in_block(then, spec, out);
            if let Some(x) = els {
                calls_in_expr(x, spec, out);
            }
        }
        ExprKind::Block(b) => calls_in_block(b, spec, out),
        ExprKind::Array(xs) => xs.iter().for_each(|x| calls_in_expr(x, spec, out)),
        ExprKind::Repeat { value, .. } => calls_in_expr(value, spec, out),
        ExprKind::Forall { body, .. } => calls_in_expr(body, true, out),
        ExprKind::Field { base, .. } | ExprKind::Deref(base) => calls_in_expr(base, spec, out),
        ExprKind::StructLit { fields, .. } => {
            fields.iter().for_each(|(_, x)| calls_in_expr(x, spec, out))
        }
        ExprKind::Match { scrut, arms } => {
            calls_in_expr(scrut, spec, out);
            arms.iter().for_each(|a| calls_in_expr(&a.body, spec, out));
        }
        ExprKind::Int(_)
        | ExprKind::ByteLit(_)
        | ExprKind::ByteStr(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Path(_)
        | ExprKind::Error => {}
    }
}

fn calls_in_block<'e>(
    b: &'e Block,
    spec: bool,
    out: &mut Vec<(&'e [Ident], &'e [Expr], bool, Span)>,
) {
    for s in &b.stmts {
        match s {
            Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => calls_in_expr(init, spec, out),
            Stmt::Assign { target, value, .. } => {
                calls_in_expr(target, spec, out);
                calls_in_expr(value, spec, out);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                calls_in_expr(cond, spec, out);
                invariants.iter().for_each(|i| calls_in_expr(i, true, out));
                if let Some(d) = decreases {
                    calls_in_expr(d, true, out);
                }
                calls_in_block(body, spec, out);
            }
            Stmt::Assert { cond, .. } => calls_in_expr(cond, true, out),
            Stmt::Expr(e) => calls_in_expr(e, spec, out),
        }
    }
    if let Some(t) = &b.tail {
        calls_in_expr(t, spec, out);
    }
}

fn walk(m: &Mod, prefix: &str, t: &Tables, units: &mut BTreeMap<String, Unit>) {
    let mp = format!("{prefix}{}", m.name.name);
    let summary = m
        .attrs
        .iter()
        .find_map(|a| match a.args.first() {
            Some(AttrArg::Str(s, _)) if a.name.name == "summary" => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let mut sub: Vec<String> = m
        .mods
        .iter()
        .map(|s| format!("{mp}::{}", s.name.name))
        .collect();
    sub.sort();
    // Functions, `impl` functions (`unit::T::f`) and types (ADR-0026 §6).
    let mut defs: Vec<String> = m
        .fns
        .iter()
        .map(|f| format!("{mp}::{}", f.name.name))
        .chain(
            m.impls
                .iter()
                .flat_map(|i| i.fns.iter().map(move |f| (i, f)))
                .map(|(i, f)| format!("{mp}::{}::{}", i.ty.name, f.name.name)),
        )
        .chain(m.types.iter().map(|t| format!("{mp}::{}", t.name.name)))
        .chain(m.traits.iter().map(|t| format!("{mp}::{}", t.name.name)))
        .chain(
            m.trait_impls
                .iter()
                .flat_map(|i| i.fns.iter())
                .filter_map(|f| {
                    t.fns
                        .values()
                        .find(|s| s.in_impl && s.span == f.span && s.path.starts_with('<'))
                        .map(|s| s.path.clone())
                }),
        )
        .collect();
    defs.sort();
    units.insert(
        mp.clone(),
        Unit {
            path: mp.clone(),
            summary,
            units: sub,
            defs,
        },
    );
    for s in &m.mods {
        walk(s, &format!("{mp}::"), t, units);
    }
}

/// The type declarations of a unit and its sub-units: (path, unit, decl).
fn walk_types<'p>(m: &'p Mod, prefix: &str, out: &mut Vec<(String, String, &'p TypeDef)>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        walk_types(s, &format!("{mp}::"), out);
    }
    for t in &m.types {
        out.push((format!("{mp}::{}", t.name.name), mp.clone(), t));
    }
}

/// The trait declarations of a program: (path, unit, decl).
fn walk_traits(p: &Program) -> Vec<(String, String, &TraitDef)> {
    fn walk<'p>(m: &'p Mod, prefix: &str, out: &mut Vec<(String, String, &'p TraitDef)>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            walk(s, &format!("{mp}::"), out);
        }
        for t in &m.traits {
            out.push((format!("{mp}::{}", t.name.name), mp.clone(), t));
        }
    }
    let mut out = Vec::new();
    for m in &p.mods {
        walk(m, "", &mut out);
    }
    out
}

/// The data types a type names (full paths, declared ones only).
fn named_types(t: &TypeKind, tb: &Tables, out: &mut BTreeSet<String>) {
    match t {
        TypeKind::Named { path, args } => {
            let p = path.join("::");
            if tb.adts.contains_key(&p) {
                out.insert(p);
            }
            args.iter().for_each(|a| named_types(&a.kind, tb, out));
        }
        TypeKind::Array(e, _) | TypeKind::Slice(e) => named_types(&e.kind, tb, out),
        TypeKind::Ref { inner, .. } => named_types(&inner.kind, tb, out),
        _ => {}
    }
}

/// Every expression of a definition (contracts and body), for the types
/// their values have.
fn exprs_of(f: &FnDef) -> Vec<&Expr> {
    fn block<'e>(b: &'e Block, out: &mut Vec<&'e Expr>) {
        for s in &b.stmts {
            match s {
                Stmt::Let { init, .. } | Stmt::LetFields { init, .. } => expr(init, out),
                Stmt::Assign { target, value, .. } => {
                    expr(target, out);
                    expr(value, out);
                }
                Stmt::While {
                    cond,
                    invariants,
                    decreases,
                    body,
                    ..
                } => {
                    expr(cond, out);
                    invariants.iter().for_each(|x| expr(x, out));
                    decreases.iter().for_each(|x| expr(x, out));
                    block(body, out);
                }
                Stmt::Assert { cond, .. } => expr(cond, out),
                Stmt::Expr(e) => expr(e, out),
            }
        }
        if let Some(t) = &b.tail {
            expr(t, out);
        }
    }
    fn expr<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
        out.push(e);
        match &e.kind {
            ExprKind::Call { args, .. } | ExprKind::Array(args) => {
                args.iter().for_each(|a| expr(a, out))
            }
            ExprKind::Method { recv, args, .. } => {
                expr(recv, out);
                args.iter().for_each(|a| expr(a, out));
            }
            ExprKind::Index { base, index } => {
                expr(base, out);
                expr(index, out);
            }
            ExprKind::Unary(_, x)
            | ExprKind::Ref { expr: x, .. }
            | ExprKind::Old(x)
            | ExprKind::Trigger(x)
            | ExprKind::Deref(x)
            | ExprKind::Field { base: x, .. }
            | ExprKind::Repeat { value: x, .. }
            | ExprKind::Forall { body: x, .. } => expr(x, out),
            ExprKind::Binary(_, l, r) => {
                expr(l, out);
                expr(r, out);
            }
            ExprKind::If { cond, then, els } => {
                expr(cond, out);
                block(then, out);
                els.iter().for_each(|x| expr(x, out));
            }
            ExprKind::Block(b) => block(b, out),
            ExprKind::StructLit { fields, rest, .. } => {
                fields.iter().for_each(|(_, x)| expr(x, out));
                rest.iter().for_each(|x| expr(x, out));
            }
            ExprKind::Match { scrut, arms } => {
                expr(scrut, out);
                arms.iter().for_each(|a| expr(&a.body, out));
            }
            ExprKind::Path(_)
            | ExprKind::Int(_)
            | ExprKind::ByteLit(_)
            | ExprKind::ByteStr(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Var(_)
            | ExprKind::Error => {}
        }
    }
    let mut out = Vec::new();
    for c in f
        .requires
        .iter()
        .chain(&f.ensures)
        .chain(f.decreases.iter())
    {
        expr(c, &mut out);
    }
    block(&f.body, &mut out);
    out
}

impl Index {
    /// Builds the index of a checked program.
    pub fn build(p: &Program, t: &Tables, sm: &SourceMap) -> Index {
        let mut units = BTreeMap::new();
        for m in &p.mods {
            walk(m, "", t, &mut units);
        }
        let list = crate::check::defs_of(p, &t.fns);
        let mut defs = BTreeMap::new();
        let mut edges = BTreeSet::new();
        for (idx, (path, unit, f)) in list.into_iter().enumerate() {
            let (line, _) = sm.line_col(f.name.span.file, f.name.span.lo);

            // Edges.
            let mut calls = Vec::new();
            let mut contract = Vec::new();
            for c in f
                .requires
                .iter()
                .chain(&f.ensures)
                .chain(f.decreases.iter())
            {
                calls_in_expr(c, true, &mut contract);
            }
            calls_in_block(&f.body, f.kind == FnKind::Spec, &mut calls);
            let targs_of = |sp: &Span| {
                t.targs.get(&(sp.file, sp.lo, sp.hi)).map(|ts| {
                    ts.iter()
                        .map(crate::check::ty_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
            };
            for (cp, _, _, sp) in &contract {
                edges.insert(Edge {
                    from: path.clone(),
                    to: resolve(cp, &unit, t),
                    kind: "contract",
                    arg: None,
                    targs: targs_of(sp),
                });
            }
            for (cp, args, spec, sp) in &calls {
                let mut to = resolve(cp, &unit, t);
                // A trait call with a known `Self` names its impl's
                // function (ADR-0051: the resolved target).
                if let Some(ts) = t.targs.get(&(sp.file, sp.lo, sp.hi))
                    && let Some((imp, _)) = crate::check::dispatch(&t.fns, &t.trait_impls, &to, ts)
                {
                    to = imp;
                }
                // A variant constructor (`Some(x)`, `E::V(..)`) is no call.
                if !t.fns.contains_key(&to) {
                    continue;
                }
                let kind = if f.kind == FnKind::Spec {
                    "spec_calls"
                } else if *spec {
                    "contract"
                } else {
                    "calls"
                };
                if kind == "calls"
                    && let Some(sig) = t.fns.get(&to)
                {
                    for (k, pt) in sig.params.iter().enumerate() {
                        if matches!(pt, TypeKind::Ref { mutable: true, .. }) && k < args.len() {
                            edges.insert(Edge {
                                from: path.clone(),
                                to: to.clone(),
                                kind: "writes",
                                arg: Some(k),
                                targs: None,
                            });
                        }
                    }
                }
                edges.insert(Edge {
                    from: path.clone(),
                    to,
                    kind,
                    arg: None,
                    targs: targs_of(sp),
                });
            }
            // The data types the definition names: its signature, locals
            // and the values of its expressions (ADR-0026 §6).
            let mut named = BTreeSet::new();
            if let Some(sig) = t.fns.get(&path) {
                sig.params
                    .iter()
                    .chain([&sig.ret])
                    .for_each(|x| named_types(x, t, &mut named));
                // The traits its type parameters are bounded by (ADR-0046):
                // a slice holds their declarations (principle 5).
                for g in &sig.generics {
                    named.extend(g.bounds.iter().filter(|b| b.contains("::")).cloned());
                }
            }
            for (cp, _, _, _) in calls.iter().chain(&contract) {
                if let Some(tr) = t
                    .fns
                    .get(&resolve(cp, &unit, t))
                    .and_then(|s| s.trait_of.clone())
                {
                    named.insert(tr);
                }
            }
            if let Some(ls) = t.locals.get(&path) {
                ls.values().for_each(|x| named_types(x, t, &mut named));
            }
            for e in exprs_of(f) {
                if let Some(x) = t.types.get(&(e.span.file, e.span.lo, e.span.hi)) {
                    named_types(x, t, &mut named);
                }
            }
            for ty in named {
                edges.insert(Edge {
                    from: path.clone(),
                    to: ty,
                    kind: "types",
                    arg: None,
                    targs: None,
                });
            }
            for a in f.attrs.iter().filter(|a| a.name.name == "uses") {
                for x in &a.args {
                    if let AttrArg::Ident(i) = x {
                        let to = crate::check::uses_target(&i.name, &unit, &t.fns)
                            .unwrap_or_else(|| i.name.clone());
                        edges.insert(Edge {
                            from: path.clone(),
                            to,
                            kind: "uses",
                            arg: None,
                            targs: None,
                        });
                    }
                }
            }
            defs.insert(
                path.clone(),
                Entry {
                    path,
                    unit,
                    idx,
                    file: sm.name(f.name.span.file).to_string(),
                    line,
                    hashes: OnceCell::new(),
                },
            );
        }
        let hashes: BTreeMap<String, String> = crate::fmt::type_hashes(p)
            .into_iter()
            .map(|h| (h.path, h.body))
            .collect();
        let mut list = Vec::new();
        for m in &p.mods {
            walk_types(m, "", &mut list);
        }
        let mut types = BTreeMap::new();
        for (path, unit, tr) in walk_traits(p) {
            let (line, _) = sm.line_col(tr.name.span.file, tr.name.span.lo);
            let text = crate::fmt::trait_text(tr);
            types.insert(
                path.clone(),
                TypeEntry {
                    kind: "trait",
                    hash: crate::sha256::hex(crate::fmt::trait_text_as(tr, "_").as_bytes()),
                    text,
                    file: sm.name(tr.name.span.file).to_string(),
                    line,
                    names: BTreeSet::new(),
                    unit,
                    path,
                },
            );
        }
        for (path, unit, td) in list {
            let (line, _) = sm.line_col(td.name.span.file, td.name.span.lo);
            let mut names = BTreeSet::new();
            if let Some(a) = t.adts.get(&path) {
                match &a.kind {
                    crate::check::AdtKind::Struct(fs) => {
                        fs.iter().for_each(|(_, x)| named_types(x, t, &mut names))
                    }
                    crate::check::AdtKind::Enum(vs) => vs
                        .iter()
                        .flat_map(|(_, ps)| ps.iter())
                        .for_each(|x| named_types(x, t, &mut names)),
                }
            }
            types.insert(
                path.clone(),
                TypeEntry {
                    kind: match td.kind {
                        TypeDefKind::Struct(_) => "struct",
                        TypeDefKind::Enum(_) => "enum",
                    },
                    text: crate::fmt::type_text(td),
                    hash: hashes.get(&path).cloned().unwrap_or_default(),
                    file: sm.name(td.name.span.file).to_string(),
                    line,
                    names,
                    unit,
                    path,
                },
            );
        }
        let io = t
            .fns
            .iter()
            .filter(|(_, s)| s.kind == FnKind::Exec && s.effect == crate::check::Effect::Io)
            .map(|(p, _)| p.clone())
            .collect();
        let mut incoming: BTreeMap<String, Vec<Edge>> = BTreeMap::new();
        for e in &edges {
            incoming.entry(e.to.clone()).or_default().push(e.clone());
        }
        Index {
            units,
            defs,
            types,
            edges,
            io,
            incoming,
        }
    }
}

impl<'p> Model<'p> {
    /// Builds the index of a checked program and its maps with `outcome`.
    pub fn build(p: &'p Program, t: &'p Tables, sm: &SourceMap, outcome: &Outcome) -> Model<'p> {
        let ix = Index::build(p, t, sm);
        Model::with(Cow::Owned(ix), p, t, outcome)
    }

    /// The maps of `p` with `outcome` over an index built before from the
    /// same program (`Index::build(p, t, ..)`). Costs one walk over the
    /// definitions; nothing is recomputed (backlog 3A-03).
    pub fn view(ix: &'p Index, p: &'p Program, t: &'p Tables, outcome: &Outcome) -> Model<'p> {
        Model::with(Cow::Borrowed(ix), p, t, outcome)
    }

    fn with(ix: Cow<'p, Index>, p: &'p Program, t: &'p Tables, outcome: &Outcome) -> Model<'p> {
        // Walk order, as `Index::build`'s entries count it.
        let fns = crate::check::defs_of(p, &t.fns)
            .into_iter()
            .map(|(_, _, f)| f)
            .collect();
        Model {
            ix,
            fns,
            sigs: &t.fns,
            status: status_map(outcome),
        }
    }

    /// `e` as a query sees it.
    pub fn view_def<'m>(&'m self, e: &'m Entry) -> Def<'m> {
        let (obligations, failed, rlimit_max) =
            self.status.get(&e.path).copied().unwrap_or((0, 0, 0));
        let status = if failed > 0 {
            "failed"
        } else if obligations > 0 {
            "verified"
        } else {
            "none"
        };
        Def {
            e,
            f: self.fns[e.idx],
            status,
            obligations,
            failed,
            rlimit_max,
        }
    }

    /// The definition at `path`.
    pub fn def(&self, path: &str) -> Option<Def<'_>> {
        self.ix.defs.get(path).map(|e| self.view_def(e))
    }

    /// Every definition, by path.
    pub fn all_defs(&self) -> impl Iterator<Item = Def<'_>> {
        self.ix.defs.values().map(|e| self.view_def(e))
    }

    /// Body and interface hashes of `d`.
    fn hashes<'m>(&'m self, d: &Def<'m>) -> &'m (String, String) {
        let e: &'m Entry = d.e;
        e.hashes
            .get_or_init(|| crate::fmt::def_hash(d.f, &e.unit, self.sigs))
    }

    // ---------- rows ----------

    fn l0_unit(&self, u: &Unit) -> J {
        J::O(vec![
            ("level", J::S("L0".into())),
            ("kind", J::S("unit".into())),
            ("path", J::S(u.path.clone())),
            ("summary", J::S(u.summary.clone())),
            ("units", strs(u.units.clone())),
            ("defs", strs(u.defs.clone())),
        ])
    }

    fn def_kind(d: &Def) -> &'static str {
        if d.f.kind == FnKind::Spec {
            "spec fn"
        } else {
            "fn"
        }
    }

    fn l0_def(&self, d: &Def) -> J {
        J::O(vec![
            ("level", J::S("L0".into())),
            ("kind", J::S("def".into())),
            ("path", J::S(d.path.clone())),
            ("def_kind", J::S(Self::def_kind(d).into())),
            (
                "hash",
                J::S(self.hashes(d).0.chars().take(7 + 12).collect()),
            ),
        ])
    }

    fn l0_type(ty: &TypeEntry) -> J {
        J::O(vec![
            ("level", J::S("L0".into())),
            ("kind", J::S("def".into())),
            ("path", J::S(ty.path.clone())),
            ("def_kind", J::S(ty.kind.into())),
            ("hash", J::S(ty.hash.chars().take(7 + 12).collect())),
        ])
    }

    /// A type's L1 row: its declaration is its surface (ADR-0026 §6).
    fn l1_type(ty: &TypeEntry) -> J {
        J::O(vec![
            ("level", J::S("L1".into())),
            ("path", J::S(ty.path.clone())),
            ("def_kind", J::S(ty.kind.into())),
            ("signature", J::S(ty.text.clone())),
            ("body", J::S(ty.hash.clone())),
            ("iface", J::S(ty.hash.clone())),
            ("file", J::S(ty.file.clone())),
            ("line", J::N(ty.line as u64)),
        ])
    }

    fn effects(d: &Def) -> String {
        if d.f.kind == FnKind::Spec {
            return "spec".into();
        }
        d.f.attrs
            .iter()
            .find(|a| a.name.name == "effects")
            .and_then(|a| match a.args.first() {
                Some(AttrArg::Ident(i)) => Some(i.name.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn l1(&self, d: &Def) -> J {
        let (sig, req, ens, dec) = crate::fmt::surface_parts(d.f);
        let uses: Vec<String> = self
            .out_edges(&d.path)
            .filter(|e| e.kind == "uses")
            .map(|e| e.to.clone())
            .collect();
        let (body, iface) = self.hashes(d);
        J::O(vec![
            ("level", J::S("L1".into())),
            ("path", J::S(d.path.clone())),
            ("def_kind", J::S(Self::def_kind(d).into())),
            ("signature", J::S(sig)),
            ("requires", strs(req)),
            ("ensures", strs(ens)),
            ("decreases", dec.map_or(J::Null, J::S)),
            ("effects", J::S(Self::effects(d))),
            ("uses", strs(uses)),
            ("status", J::S(d.status.into())),
            ("obligations", J::N(d.obligations as u64)),
            ("failed", J::N(d.failed as u64)),
            ("body", J::S(body.clone())),
            ("iface", J::S(iface.clone())),
            ("file", J::S(d.file.clone())),
            ("line", J::N(d.line as u64)),
            // Added after version 1 (backward compatible, ADR-0020 §1).
            ("rlimit_max", J::N(d.rlimit_max)),
        ])
    }

    /// The edges from `path`, in order: a range of the ordered edge set, so
    /// the cost follows the definition's own edges, not the program's.
    fn out_edges<'m>(&'m self, path: &str) -> impl Iterator<Item = &'m Edge> + use<'m> {
        let from = Edge {
            from: path.to_string(),
            to: String::new(),
            kind: "",
            arg: None,
            targs: None,
        };
        let path = from.from.clone();
        self.edges.range(from..).take_while(move |e| e.from == path)
    }

    /// The edges into `path`, in edge order.
    fn in_edges<'m>(&'m self, path: &str) -> impl Iterator<Item = &'m Edge> + use<'m> {
        self.incoming.get(path).into_iter().flatten()
    }

    fn l2(e: &Edge) -> J {
        let mut kv = vec![
            ("level", J::S("L2".into())),
            ("from", J::S(e.from.clone())),
            ("to", J::S(e.to.clone())),
            ("kind", J::S(e.kind.into())),
        ];
        if let Some(a) = e.arg {
            kv.push(("arg", J::N(a as u64)));
        }
        if let Some(ts) = &e.targs {
            kv.push(("type_args", J::S(ts.clone())));
        }
        J::O(kv)
    }

    fn lines(rows: impl IntoIterator<Item = J>) -> String {
        let mut out = String::new();
        for r in rows {
            out.push_str(&r.render());
            out.push('\n');
        }
        out
    }

    // ---------- maps ----------

    pub fn l0_rows(&self) -> Vec<J> {
        let mut rows: Vec<(String, J)> = Vec::new();
        for u in self.units.values() {
            rows.push((u.path.clone(), self.l0_unit(u)));
        }
        for d in self.all_defs() {
            rows.push((d.path.clone(), self.l0_def(&d)));
        }
        for ty in self.types.values() {
            rows.push((ty.path.clone(), Self::l0_type(ty)));
        }
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows.into_iter().map(|(_, j)| j).collect()
    }

    pub fn map(&self, level: &str) -> String {
        let mut out = format!("{HEADER}\n");
        if level == "L0" || level == "all" {
            out.push_str(&Self::lines(self.l0_rows()));
        }
        if level == "L1" || level == "all" {
            let mut rows: Vec<(String, J)> = self
                .all_defs()
                .map(|d| (d.path.clone(), self.l1(&d)))
                .collect();
            rows.extend(
                self.types
                    .values()
                    .map(|ty| (ty.path.clone(), Self::l1_type(ty))),
            );
            rows.sort_by(|a, b| a.0.cmp(&b.0));
            out.push_str(&Self::lines(rows.into_iter().map(|(_, j)| j)));
        }
        if level == "L2" || level == "all" {
            out.push_str(&Self::lines(self.edges.iter().map(Self::l2)));
        }
        out
    }

    // ---------- queries ----------

    /// Runs one ADR-0020 query. `Err` is a usage error message.
    pub fn query(&self, name: &str, args: &[String]) -> Result<String, String> {
        let def = |i: usize| -> Result<Def, String> {
            let p = args.get(i).ok_or("missing definition path")?;
            self.def(p).ok_or(format!("unknown definition `{p}`"))
        };
        let ty = args.first().and_then(|p| self.types.get(p));
        let body = match name {
            "units" => Self::lines(self.l0_rows()),
            "surface" if ty.is_some() => Self::lines([Self::l1_type(ty.unwrap())]),
            "impact" if ty.is_some() => {
                let change = args.get(1).map(String::as_str).unwrap_or("iface");
                if !matches!(change, "body" | "iface") {
                    return Err(format!("change must be body or iface, not `{change}`"));
                }
                Self::lines(self.type_impact(ty.unwrap(), change))
            }
            "edges" if ty.is_some() => {
                let dir = args.get(1).map(String::as_str).unwrap_or("out");
                if !matches!(dir, "out" | "in" | "both") {
                    return Err(format!("direction must be out, in or both, not `{dir}`"));
                }
                let depth: usize = match args.get(2) {
                    Some(x) => x.parse().map_err(|_| format!("bad depth `{x}`"))?,
                    None => 1,
                };
                Self::lines(
                    self.reach(&ty.unwrap().path, dir, depth)
                        .iter()
                        .map(Self::l2),
                )
            }
            "surface" => Self::lines([self.l1(&def(0)?)]),
            "edges" => {
                let d = def(0)?;
                let dir = args.get(1).map(String::as_str).unwrap_or("out");
                if !matches!(dir, "out" | "in" | "both") {
                    return Err(format!("direction must be out, in or both, not `{dir}`"));
                }
                let depth: usize = match args.get(2) {
                    Some(x) => x.parse().map_err(|_| format!("bad depth `{x}`"))?,
                    None => 1,
                };
                Self::lines(self.reach(&d.path, dir, depth).iter().map(Self::l2))
            }
            "slice" => Self::lines([self.slice(&def(0)?)]),
            "find" => {
                let n = args.first().ok_or("missing name")?;
                let hit = |p: &str| p == n || p.rsplit("::").next() == Some(n.as_str());
                let mut rows: Vec<(String, J)> = self
                    .all_defs()
                    .filter(|d| hit(&d.path))
                    .map(|d| (d.path.clone(), self.l0_def(&d)))
                    .collect();
                rows.extend(
                    self.types
                        .values()
                        .filter(|ty| hit(&ty.path))
                        .map(|ty| (ty.path.clone(), Self::l0_type(ty))),
                );
                rows.sort_by(|a, b| a.0.cmp(&b.0));
                Self::lines(rows.into_iter().map(|(_, j)| j))
            }
            "contracts" => {
                let p = args.first().ok_or("missing predicate path")?;
                let by = |clause: fn(&FnDef) -> &Vec<Expr>| -> Vec<String> {
                    self.all_defs()
                        .filter(|d| {
                            let mut v = Vec::new();
                            for c in clause(d.f) {
                                calls_in_expr(c, true, &mut v);
                            }
                            v.iter()
                                .any(|(cp, _, _, _)| self.resolves_to(cp, &d.unit, p))
                        })
                        .map(|d| d.path.clone())
                        .collect()
                };
                let required = by(|f| &f.requires);
                let ensured = by(|f| &f.ensures);
                let uses: Vec<String> = self
                    .in_edges(p)
                    .filter(|e| e.kind == "uses")
                    .map(|e| e.from.clone())
                    .collect();
                Self::lines([J::O(vec![
                    ("pred", J::S(p.clone())),
                    ("required_by", strs(required)),
                    ("ensured_by", strs(ensured)),
                    ("uses_by", strs(uses)),
                ])])
            }
            "status" => {
                let mut rows: Vec<J> = self
                    .all_defs()
                    .map(|d| {
                        J::O(vec![
                            ("path", J::S(d.path.clone())),
                            ("status", J::S(d.status.into())),
                            ("obligations", J::N(d.obligations as u64)),
                            ("failed", J::N(d.failed as u64)),
                            ("effects", J::S(Self::effects(&d))),
                            ("rlimit_max", J::N(d.rlimit_max)),
                        ])
                    })
                    .collect();
                let verified = self.all_defs().filter(|d| d.status == "verified").count();
                let failed = self.all_defs().filter(|d| d.status == "failed").count();
                let io = self.all_defs().filter(|d| Self::effects(d) == "io").count();
                rows.push(J::O(vec![
                    ("summary", J::B(true)),
                    ("defs", J::N(self.defs.len() as u64)),
                    ("verified", J::N(verified as u64)),
                    ("failed", J::N(failed as u64)),
                    ("io_defs", J::N(io as u64)),
                ]));
                Self::lines(rows)
            }
            "impact" => {
                let d = def(0)?;
                let change = args.get(1).map(String::as_str).unwrap_or("iface");
                if !matches!(change, "body" | "iface") {
                    return Err(format!("change must be body or iface, not `{change}`"));
                }
                Self::lines(self.impact(&d, change))
            }
            "duplicates" => Self::lines(self.duplicates()),
            "tests" => Self::lines(self.tests_needed()),
            "effects" => match args.first() {
                Some(_) => Self::lines(self.effects_of(&def(0)?)),
                None => Self::lines(self.effects_all()),
            },
            other => return Err(format!("unknown query `{other}`")),
        };
        Ok(body)
    }

    /// The `io` definitions reachable from `start` over executable `calls`
    /// edges (`start` itself included when it is `io`), each with a shortest
    /// call path from `start`, in breadth-first order. Callees are taken in
    /// path order, so each path is the lexicographically smallest of the
    /// shortest ones and the first entry is the nearest `io` definition.
    fn io_paths(&self, start: &str) -> Vec<(String, Vec<String>)> {
        let mut parent: BTreeMap<String, Option<String>> = BTreeMap::new();
        parent.insert(start.to_string(), None);
        let mut queue = std::collections::VecDeque::from([start.to_string()]);
        let mut found = Vec::new();
        while let Some(at) = queue.pop_front() {
            if self.io.contains(&at) {
                let mut path = vec![at.clone()];
                while let Some(Some(p)) = parent.get(path.last().unwrap()) {
                    path.push(p.clone());
                }
                path.reverse();
                found.push((at.clone(), path));
            }
            for e in self.out_edges(&at).filter(|e| e.kind == "calls") {
                if !parent.contains_key(&e.to) {
                    parent.insert(e.to.clone(), Some(at.clone()));
                    queue.push_back(e.to.clone());
                }
            }
        }
        found
    }

    fn effect_row(def: &str, path: Vec<String>) -> J {
        J::O(vec![
            ("def", J::S(def.into())),
            ("effect", J::S("io".into())),
            ("path", strs(path)),
        ])
    }

    /// `effects DEF` (backlog 2D-11, DESIGN.md §7.2 effect view): one row
    /// `{"def","effect":"io","path"}` per `io` definition DEF reaches
    /// (`def` is DEF, `path` a shortest call path from DEF that ends at the
    /// `io` definition), ordered by that definition; then a summary row.
    fn effects_of(&self, d: &Def) -> Vec<J> {
        let mut found = self.io_paths(&d.path);
        found.sort();
        let n = found.len();
        let mut rows: Vec<J> = found
            .into_iter()
            .map(|(_, path)| Self::effect_row(&d.path, path))
            .collect();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("def", J::S(d.path.clone())),
            ("io_reached", J::N(n as u64)),
            ("defs", J::N(self.defs.len() as u64)),
        ]));
        rows
    }

    /// `effects` without a definition: one row per definition whose calls
    /// reach a built-in `io` operation (`print_i64`), by path, where `def`
    /// is that definition and `path` a shortest call path to the nearest
    /// such operation; then a summary row. The effect check (E0603) already
    /// makes every such definition `io`; a definition declared `io` that
    /// reaches no operation has no row.
    fn effects_all(&self) -> Vec<J> {
        let mut rows = Vec::new();
        for d in self.defs.values() {
            let nearest = self
                .io_paths(&d.path)
                .into_iter()
                .find(|(to, _)| !self.defs.contains_key(to));
            if let Some((_, path)) = nearest {
                rows.push(Self::effect_row(&d.path, path));
            }
        }
        let n = rows.len();
        let io = self.defs.keys().filter(|p| self.io.contains(*p)).count();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("io_reaching", J::N(n as u64)),
            ("io_defs", J::N(io as u64)),
            ("defs", J::N(self.defs.len() as u64)),
        ]));
        rows
    }

    /// `tests` (backlog 2D-21, ADR-0003, DESIGN.md principle 4): the
    /// executable definitions that still need tests, with why. M1 has two
    /// reasons: `io` (its calls reach a built-in `io` operation, whose order
    /// and content no contract states) and `unproved` (an obligation failed,
    /// so the contract itself is not established). One row
    /// `{"path","test_reasons","sites","lines"}` per such definition, by
    /// path: `sites` counts its own calls of built-in `io` operations (where
    /// a test aims), `lines` its normal-form lines. Then a summary row with
    /// the area in definitions, sites and lines.
    fn tests_needed(&self) -> Vec<J> {
        let builtins: Vec<&String> = self
            .io
            .iter()
            .filter(|p| !self.defs.contains_key(*p))
            .collect();
        let mut rows = Vec::new();
        let (mut exec, mut sites_total, mut lines_total) = (0, 0, 0);
        let (mut io_n, mut unproved_n) = (0, 0);
        for d in self.all_defs() {
            if d.f.kind == FnKind::Spec {
                continue;
            }
            exec += 1;
            let mut reasons = Vec::new();
            let io = self
                .io_paths(&d.path)
                .iter()
                .any(|(to, _)| !self.defs.contains_key(to));
            if io {
                reasons.push("io".to_string());
                io_n += 1;
            }
            if d.status == "failed" {
                reasons.push("unproved".to_string());
                unproved_n += 1;
            }
            if reasons.is_empty() {
                continue;
            }
            let mut calls = Vec::new();
            calls_in_block(&d.f.body, false, &mut calls);
            let sites = calls
                .iter()
                .filter(|(cp, _, spec, _)| {
                    !spec && builtins.iter().any(|b| self.resolves_to(cp, &d.unit, b))
                })
                .count();
            let lines = crate::fmt::def_text(d.f).lines().count();
            sites_total += sites;
            lines_total += lines;
            rows.push(J::O(vec![
                ("path", J::S(d.path.clone())),
                ("test_reasons", strs(reasons)),
                ("sites", J::N(sites as u64)),
                ("lines", J::N(lines as u64)),
            ]));
        }
        let n = rows.len();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("exec_defs", J::N(exec as u64)),
            ("test_needed_defs", J::N(n as u64)),
            ("io_defs", J::N(io_n as u64)),
            ("unproved_defs", J::N(unproved_n as u64)),
            ("test_sites", J::N(sites_total as u64)),
            ("test_lines", J::N(lines_total as u64)),
        ]));
        rows
    }

    /// Groups of definitions with the same body hash (backlog 2D-06). The body
    /// hash leaves out the definition's own name (ADR-0019 §2), so equal
    /// hashes are the same definition under different names or units. One row
    /// `{"hash","defs"}` per group of two or more, by hash, then a summary row.
    fn duplicates(&self) -> Vec<J> {
        let mut groups: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for d in self.all_defs() {
            groups
                .entry(self.hashes(&d).0.as_str())
                .or_default()
                .push(d.path.clone());
        }
        let mut rows = Vec::new();
        let mut dup_defs = 0;
        for (hash, mut defs) in groups {
            if defs.len() < 2 {
                continue;
            }
            defs.sort();
            dup_defs += defs.len();
            rows.push(J::O(vec![
                ("hash", J::S(hash.into())),
                ("defs", strs(defs)),
            ]));
        }
        let groups = rows.len();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("groups", J::N(groups as u64)),
            ("duplicate_defs", J::N(dup_defs as u64)),
            ("defs", J::N(self.defs.len() as u64)),
        ]));
        rows
    }

    /// Definitions whose proofs must be redone when `d` changes (backlog
    /// 2D-02). Proofs see callees only through their contracts and spec fns
    /// only through `#[uses]`, which does not reach further (ADR-0017 §4),
    /// so the set never grows past one step:
    ///
    /// - body of a `fn`: the definition itself;
    /// - contract or signature of a `fn` (`iface`): itself and its callers;
    /// - body of a `spec fn`: the definitions that reveal it with `#[uses]`,
    ///   and itself when it is recursive (termination);
    /// - signature of a `spec fn`: every definition that names it (they are
    ///   re-checked, and re-proved where they reveal it).
    fn impact(&self, d: &Def, change: &str) -> Vec<J> {
        let spec = d.f.kind == FnKind::Spec;
        let mut hit: BTreeMap<String, &'static str> = BTreeMap::new();
        if !spec || d.f.decreases.is_some() || change == "iface" {
            hit.insert(d.path.clone(), "changed");
        }
        for e in self.in_edges(&d.path).filter(|e| e.from != d.path) {
            let reason = match (spec, change, e.kind) {
                (false, "iface", "calls") => "calls",
                (true, _, "uses") => "uses",
                (true, "iface", "contract" | "spec_calls") => "names",
                _ => continue,
            };
            let slot = hit.entry(e.from.clone()).or_insert(reason);
            if *slot == "names" {
                *slot = reason;
            }
        }
        let n = hit.len();
        let mut rows: Vec<J> = hit
            .into_iter()
            .map(|(path, reason)| J::O(vec![("path", J::S(path)), ("reason", J::S(reason.into()))]))
            .collect();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("changed", J::S(d.path.clone())),
            ("change", J::S(change.into())),
            ("reprove", J::N(n as u64)),
            ("defs", J::N(self.defs.len() as u64)),
        ]));
        rows
    }

    /// What changing a type re-proves (ADR-0026 §6): the definitions that
    /// name it or a type holding it (`types`), then, as for a change of
    /// their surface, the callers of an executable one that names it in its
    /// surface and what reveals or names a spec fn that does.
    fn type_impact(&self, ty: &TypeEntry, change: &str) -> Vec<J> {
        // The type and every type holding it.
        let mut affected: BTreeSet<String> = [ty.path.clone()].into();
        loop {
            let more: Vec<String> = self
                .types
                .values()
                .filter(|t| {
                    !affected.contains(&t.path) && t.names.iter().any(|n| affected.contains(n))
                })
                .map(|t| t.path.clone())
                .collect();
            if more.is_empty() {
                break;
            }
            affected.extend(more);
        }
        let mut hit: BTreeMap<String, &'static str> = BTreeMap::new();
        let mut direct = Vec::new();
        for a in &affected {
            for e in self.in_edges(a).filter(|e| e.kind == "types") {
                hit.insert(e.from.clone(), "types");
                direct.push(e.from.clone());
            }
        }
        for p in direct {
            let Some(d) = self.def(&p) else { continue };
            let surface = if d.f.kind == FnKind::Spec {
                crate::fmt::def_text(d.f)
            } else {
                crate::fmt::iface_text(d.f)
            };
            let names_it = affected.iter().any(|a| {
                surface.contains(a.as_str()) || surface.contains(a.rsplit("::").next().unwrap_or(a))
            });
            if !names_it {
                continue;
            }
            for row in self.impact(&d, "iface") {
                if let J::O(kv) = &row
                    && let (Some((_, J::S(path))), Some((_, J::S(reason)))) = (
                        kv.iter().find(|(k, _)| *k == "path"),
                        kv.iter().find(|(k, _)| *k == "reason"),
                    )
                    && reason != "changed"
                {
                    let r: &'static str = match reason.as_str() {
                        "calls" => "calls",
                        "uses" => "uses",
                        _ => "names",
                    };
                    hit.entry(path.clone()).or_insert(r);
                }
            }
        }
        let n = hit.len();
        let mut rows: Vec<J> = hit
            .into_iter()
            .map(|(path, reason)| J::O(vec![("path", J::S(path)), ("reason", J::S(reason.into()))]))
            .collect();
        rows.push(J::O(vec![
            ("summary", J::B(true)),
            ("changed", J::S(ty.path.clone())),
            ("change", J::S(change.into())),
            ("reprove", J::N(n as u64)),
            ("defs", J::N(self.defs.len() as u64)),
        ]));
        rows
    }

    fn resolves_to(&self, path: &[Ident], unit: &str, target: &str) -> bool {
        let plain = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let local = format!("{unit}::{plain}");
        match self.sigs.get(&local) {
            Some(sig)
                if self.defs.contains_key(&local)
                    && (path.len() == 1 || (path.len() == 2 && sig.in_impl)) =>
            {
                local == target
            }
            _ => plain == target,
        }
    }

    /// Edges reachable from `start` within `depth` steps.
    fn reach(&self, start: &str, dir: &str, depth: usize) -> BTreeSet<Edge> {
        let mut seen = BTreeSet::new();
        let mut frontier: BTreeSet<String> = [start.to_string()].into();
        let mut visited: BTreeSet<String> = frontier.clone();
        for _ in 0..depth {
            // Nothing new to expand: further steps add nothing (backlog 1D-14).
            if frontier.is_empty() {
                break;
            }
            // Only the edges at the frontier are visited (backlog 3A-03); an
            // edge with both ends in the frontier counts as outgoing, and its
            // source is visited already either way.
            let mut next = BTreeSet::new();
            for at in &frontier {
                let out = self.out_edges(at).filter(|_| dir != "in");
                let inn = self
                    .in_edges(at)
                    .filter(|e| dir != "out" && (dir == "in" || !frontier.contains(&e.from)));
                for (e, other) in out.map(|e| (e, &e.to)).chain(inn.map(|e| (e, &e.from))) {
                    seen.insert(e.clone());
                    if visited.insert(other.clone()) {
                        next.insert(other.clone());
                    }
                }
            }
            frontier = next;
        }
        seen
    }

    fn slice(&self, d: &Def) -> J {
        let code = crate::fmt::def_text(d.f);
        let mut deps: BTreeMap<String, String> = BTreeMap::new();
        for e in self
            .out_edges(&d.path)
            .filter(|e| e.to != d.path && e.kind != "writes")
        {
            if e.kind == "types" {
                if let Some(ty) = self.types.get(&e.to) {
                    deps.insert(e.to.clone(), ty.text.clone());
                }
                continue;
            }
            let Some(t) = self.def(&e.to) else {
                continue;
            };
            // A revealed spec fn contributes its definition; anything else its surface.
            let text = if e.kind == "uses" {
                crate::fmt::def_text(t.f)
            } else {
                crate::fmt::iface_text(t.f)
            };
            let entry = deps.entry(e.to.clone()).or_default();
            if text.len() > entry.len() {
                *entry = text;
            }
        }
        let bytes = code.len() + deps.values().map(String::len).sum::<usize>();
        J::O(vec![
            ("path", J::S(d.path.clone())),
            ("code", J::S(code)),
            (
                "deps",
                J::A(
                    deps.into_iter()
                        .map(|(p, text)| J::O(vec![("path", J::S(p)), ("text", J::S(text))]))
                        .collect(),
                ),
            ),
            ("bytes", J::N(bytes as u64)),
            ("approx_tokens", J::N(bytes.div_ceil(4) as u64)),
        ])
    }
}
