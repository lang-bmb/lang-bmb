//! Standard maps L0-L2 and read queries (ADR-0020, backlog 1D-05, 1D-06).

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

pub struct Def<'p> {
    pub path: String,
    pub unit: String,
    pub f: &'p FnDef,
    pub file: String,
    pub line: u32,
    pub status: &'static str,
    pub obligations: usize,
    pub failed: usize,
    pub body: String,
    pub iface: String,
}

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

pub struct Model<'p> {
    pub units: BTreeMap<String, Unit>,
    pub defs: BTreeMap<String, Def<'p>>,
    pub edges: BTreeSet<Edge>,
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

impl<'p> Model<'p> {
    pub fn build(p: &'p Program, t: &Tables, sm: &SourceMap, outcome: &Outcome) -> Model<'p> {
        let mut units = BTreeMap::new();
        let mut list = Vec::new();
        for m in &p.mods {
            walk(m, "", &mut units, &mut list);
        }
        let hashes: BTreeMap<String, (String, String)> = crate::fmt::hashes(p, &t.fns)
            .into_iter()
            .map(|h| (h.path, (h.body, h.iface)))
            .collect();
        let status: BTreeMap<&str, (usize, usize)> = outcome
            .statuses
            .iter()
            .map(|s| (s.path.as_str(), (s.obligations, s.failed)))
            .collect();
        let mut defs = BTreeMap::new();
        let mut edges = BTreeSet::new();
        for (path, unit, f) in list {
            let (line, _) = sm.line_col(f.name.span.file, f.name.span.lo);
            let (obligations, failed) = status.get(path.as_str()).copied().unwrap_or((0, 0));
            let st = if failed > 0 {
                "failed"
            } else if obligations > 0 {
                "verified"
            } else {
                "none"
            };
            let (body, iface) = hashes.get(&path).cloned().unwrap_or_default();

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
                Def {
                    path,
                    unit,
                    f,
                    file: sm.name(f.name.span.file).to_string(),
                    line,
                    status: st,
                    obligations,
                    failed,
                    body,
                    iface,
                },
            );
        }
        Model { units, defs, edges }
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
            ("hash", J::S(d.body.chars().take(7 + 12).collect())),
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
            .edges
            .iter()
            .filter(|e| e.from == d.path && e.kind == "uses")
            .map(|e| e.to.clone())
            .collect();
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
            ("body", J::S(d.body.clone())),
            ("iface", J::S(d.iface.clone())),
            ("file", J::S(d.file.clone())),
            ("line", J::N(d.line as u64)),
        ])
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
        for d in self.defs.values() {
            rows.push((d.path.clone(), self.l0_def(d)));
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
            out.push_str(&Self::lines(self.defs.values().map(|d| self.l1(d))));
        }
        if level == "L2" || level == "all" {
            out.push_str(&Self::lines(self.edges.iter().map(Self::l2)));
        }
        out
    }

    // ---------- queries ----------

    /// Runs one ADR-0020 query. `Err` is a usage error message.
    pub fn query(&self, name: &str, args: &[String]) -> Result<String, String> {
        let def = |i: usize| -> Result<&Def, String> {
            let p = args.get(i).ok_or("missing definition path")?;
            self.defs.get(p).ok_or(format!("unknown definition `{p}`"))
        };
        let body = match name {
            "units" => Self::lines(self.l0_rows()),
            "surface" => Self::lines([self.l1(def(0)?)]),
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
            "slice" => Self::lines([self.slice(def(0)?)]),
            "find" => {
                let n = args.first().ok_or("missing name")?;
                Self::lines(
                    self.defs
                        .values()
                        .filter(|d| d.path == *n || d.path.rsplit("::").next() == Some(n.as_str()))
                        .map(|d| self.l0_def(d)),
                )
            }
            "contracts" => {
                let p = args.first().ok_or("missing predicate path")?;
                let by = |clause: fn(&FnDef) -> &Vec<Expr>| -> Vec<String> {
                    self.defs
                        .values()
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
                    .edges
                    .iter()
                    .filter(|e| e.kind == "uses" && e.to == *p)
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
                    .defs
                    .values()
                    .map(|d| {
                        J::O(vec![
                            ("path", J::S(d.path.clone())),
                            ("status", J::S(d.status.into())),
                            ("obligations", J::N(d.obligations as u64)),
                            ("failed", J::N(d.failed as u64)),
                            ("effects", J::S(Self::effects(d))),
                        ])
                    })
                    .collect();
                let verified = self
                    .defs
                    .values()
                    .filter(|d| d.status == "verified")
                    .count();
                let failed = self.defs.values().filter(|d| d.status == "failed").count();
                let io = self
                    .defs
                    .values()
                    .filter(|d| Self::effects(d) == "io")
                    .count();
                rows.push(J::O(vec![
                    ("summary", J::B(true)),
                    ("defs", J::N(self.defs.len() as u64)),
                    ("verified", J::N(verified as u64)),
                    ("failed", J::N(failed as u64)),
                    ("io_defs", J::N(io as u64)),
                ]));
                Self::lines(rows)
            }
            other => return Err(format!("unknown query `{other}`")),
        };
        Ok(body)
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
            let mut next = BTreeSet::new();
            for e in &self.edges {
                let out = (dir != "in") && frontier.contains(&e.from);
                let inn = (dir != "out") && frontier.contains(&e.to);
                if out || inn {
                    seen.insert(e.clone());
                    let other = if out { &e.to } else { &e.from };
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
            .edges
            .iter()
            .filter(|e| e.from == d.path && e.to != d.path && e.kind != "writes")
        {
            let Some(t) = self.defs.get(&e.to) else {
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
