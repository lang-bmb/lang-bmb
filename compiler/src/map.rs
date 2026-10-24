//! Standard maps L0-L2 and read queries (ADR-0020, backlog 1D-05, 1D-06).

use std::borrow::Cow;
use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::json_str;
use crate::source::SourceMap;
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
}

/// Everything the maps need from a checked program except proof results.
/// It holds no reference into the program, so a long-lived MCP server keeps
/// it between calls while the program is unchanged (backlog 3A-03) and
/// answers each query through `Model::view`.
#[derive(Clone)]
pub struct Index {
    pub units: BTreeMap<String, Unit>,
    pub defs: BTreeMap<String, Entry>,
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

/// The definitions of a program in walk order: in each unit, its sub-units'
/// first, then its own (the order of `walk`).
fn fn_order<'p>(m: &'p Mod, out: &mut Vec<&'p FnDef>) {
    for s in &m.mods {
        fn_order(s, out);
    }
    out.extend(m.fns.iter());
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
    if path.len() == 1 {
        let local = format!("{unit}::{plain}");
        if t.fns.contains_key(&local) {
            return local;
        }
    }
    plain
}

/// Calls in an expression, with whether they occur in specification context.
fn calls_in_expr<'e>(e: &'e Expr, spec: bool, out: &mut Vec<(&'e [Ident], &'e [Expr], bool)>) {
    match &e.kind {
        ExprKind::Call { path, args } => {
            out.push((path, args, spec));
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
        ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Var(_)
        | ExprKind::Error => {}
    }
}

fn calls_in_block<'e>(b: &'e Block, spec: bool, out: &mut Vec<(&'e [Ident], &'e [Expr], bool)>) {
    for s in &b.stmts {
        match s {
            Stmt::Let { init, .. } => calls_in_expr(init, spec, out),
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

fn walk<'p>(
    m: &'p Mod,
    prefix: &str,
    units: &mut BTreeMap<String, Unit>,
    out: &mut Vec<(String, String, &'p FnDef)>,
) {
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
    let mut defs: Vec<String> = m
        .fns
        .iter()
        .map(|f| format!("{mp}::{}", f.name.name))
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
        walk(s, &format!("{mp}::"), units, out);
    }
    for f in &m.fns {
        out.push((format!("{mp}::{}", f.name.name), mp.clone(), f));
    }
}

impl Index {
    /// Builds the index of a checked program.
    pub fn build(p: &Program, t: &Tables, sm: &SourceMap) -> Index {
        let mut units = BTreeMap::new();
        let mut list = Vec::new();
        for m in &p.mods {
            walk(m, "", &mut units, &mut list);
        }
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
            for (cp, _, _) in &contract {
                edges.insert(Edge {
                    from: path.clone(),
                    to: resolve(cp, &unit, t),
                    kind: "contract",
                    arg: None,
                });
            }
            for (cp, args, spec) in &calls {
                let to = resolve(cp, &unit, t);
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
                            });
                        }
                    }
                }
                edges.insert(Edge {
                    from: path.clone(),
                    to,
                    kind,
                    arg: None,
                });
            }
            for a in f.attrs.iter().filter(|a| a.name.name == "uses") {
                for x in &a.args {
                    if let AttrArg::Ident(i) = x {
                        let to = if i.name.contains("::") {
                            i.name.clone()
                        } else {
                            format!("{unit}::{}", i.name)
                        };
                        edges.insert(Edge {
                            from: path.clone(),
                            to,
                            kind: "uses",
                            arg: None,
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
        let mut fns = Vec::new();
        for m in &p.mods {
            fn_order(m, &mut fns);
        }
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
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        rows.into_iter().map(|(_, j)| j).collect()
    }

    pub fn map(&self, level: &str) -> String {
        let mut out = format!("{HEADER}\n");
        if level == "L0" || level == "all" {
            out.push_str(&Self::lines(self.l0_rows()));
        }
        if level == "L1" || level == "all" {
            out.push_str(&Self::lines(self.all_defs().map(|d| self.l1(&d))));
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
        let body = match name {
            "units" => Self::lines(self.l0_rows()),
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
                Self::lines(
                    self.all_defs()
                        .filter(|d| d.path == *n || d.path.rsplit("::").next() == Some(n.as_str()))
                        .map(|d| self.l0_def(&d)),
                )
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
                            v.iter().any(|(cp, _, _)| self.resolves_to(cp, &d.unit, p))
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

    fn resolves_to(&self, path: &[Ident], unit: &str, target: &str) -> bool {
        let plain = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        if path.len() == 1 {
            let local = format!("{unit}::{plain}");
            if self.defs.contains_key(&local) {
                return local == target;
            }
        }
        plain == target
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
