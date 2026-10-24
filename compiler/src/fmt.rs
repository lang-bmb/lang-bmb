//! Normal form printer (ADR-0019 §1) and definition hashes (§2).

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::check::FnSig;

/// How names are written: as in the source, or fully qualified for hashing.
struct Printer<'a> {
    /// `(known definitions, unit of the definition being printed)`.
    qualify: Option<(&'a BTreeMap<String, FnSig>, String)>,
    out: String,
}

const INDENT: &str = "    ";

fn prec(e: &Expr) -> u8 {
    match &e.kind {
        ExprKind::Binary(op, ..) => match op {
            BinOp::Implies => 1,
            BinOp::Or => 2,
            BinOp::And => 3,
            o if o.is_cmp() => 4,
            BinOp::Add | BinOp::Sub => 5,
            _ => 6,
        },
        ExprKind::Unary(..) | ExprKind::Ref { .. } | ExprKind::Trigger(_) => 7,
        // Always parenthesized when nested (ADR-0019 §1).
        ExprKind::If { .. } | ExprKind::Block(_) | ExprKind::Forall { .. } => 0,
        _ => 8,
    }
}

pub fn type_str(t: &Type) -> String {
    crate::check::ty_str(&t.kind)
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

impl<'a> Printer<'a> {
    fn path(&self, path: &[Ident]) -> String {
        let plain = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        match &self.qualify {
            Some((fns, unit)) if path.len() == 1 => {
                let local = format!("{unit}::{plain}");
                if fns.contains_key(&local) {
                    local
                } else {
                    plain
                }
            }
            _ => plain,
        }
    }

    fn uses_name(&self, name: &str) -> String {
        match &self.qualify {
            Some((fns, unit)) if !name.contains("::") => {
                let local = format!("{unit}::{name}");
                if fns.contains_key(&local) {
                    local
                } else {
                    name.to_string()
                }
            }
            _ => name.to_string(),
        }
    }

    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str(INDENT);
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    // ---------- items ----------

    fn module(&mut self, m: &Mod, depth: usize) {
        let summary = m.attrs.iter().find_map(|a| match a.args.first() {
            Some(AttrArg::Str(s, _)) if a.name.name == "summary" => Some(s.clone()),
            _ => None,
        });
        if let Some(s) = summary {
            self.line(depth, &format!("#[summary(\"{}\")]", escape(&s)));
        }
        self.line(depth, &format!("mod {} {{", m.name.name));
        let mut subs: Vec<&Mod> = m.mods.iter().collect();
        subs.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut fns: Vec<&FnDef> = m.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut first = true;
        for s in subs {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.module(s, depth + 1);
        }
        for f in fns {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.function(f, depth + 1, None, true);
        }
        self.line(depth, "}");
    }

    /// `rename`: print the definition's own name as this (hashing uses `_`).
    fn function(&mut self, f: &FnDef, depth: usize, rename: Option<&str>, with_body: bool) {
        let effects = f.attrs.iter().find(|a| a.name.name == "effects");
        if let Some(a) = effects {
            let args: Vec<String> = a
                .args
                .iter()
                .map(|x| match x {
                    AttrArg::Ident(i) => i.name.clone(),
                    AttrArg::Str(s, _) => format!("\"{}\"", escape(s)),
                })
                .collect();
            self.line(depth, &format!("#[effects({})]", args.join(", ")));
        }
        let uses: BTreeSet<String> = f
            .attrs
            .iter()
            .filter(|a| a.name.name == "uses")
            .flat_map(|a| a.args.iter())
            .filter_map(|x| match x {
                AttrArg::Ident(i) => Some(self.uses_name(&i.name)),
                AttrArg::Str(..) => None,
            })
            .collect();
        if !uses.is_empty() {
            self.line(
                depth,
                &format!(
                    "#[uses({})]",
                    uses.into_iter().collect::<Vec<_>>().join(", ")
                ),
            );
        }
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name.name, type_str(&p.ty)))
            .collect();
        let ret = match &f.ret.name {
            Some(n) => format!("({}: {})", n.name, type_str(&f.ret.ty)),
            None => type_str(&f.ret.ty),
        };
        let kw = if f.kind == FnKind::Spec {
            "spec fn"
        } else {
            "fn"
        };
        let name = rename.unwrap_or(&f.name.name);
        let sig = format!("{kw} {name}({}) -> {ret}", params.join(", "));
        let has_clauses = !f.requires.is_empty() || !f.ensures.is_empty() || f.decreases.is_some();
        if !with_body {
            self.line(depth, &sig);
            self.clauses(f, depth);
            return;
        }
        if has_clauses {
            self.line(depth, &sig);
            self.clauses(f, depth);
            self.block_lines(&f.body, depth, "");
        } else {
            self.block_open(&f.body, depth, &format!("{sig} "), "");
        }
    }

    fn clauses(&mut self, f: &FnDef, depth: usize) {
        for (kw, items) in [("requires", &f.requires), ("ensures", &f.ensures)] {
            if !items.is_empty() {
                let xs: Vec<String> = items.iter().map(|e| self.expr(e, 0)).collect();
                self.line(depth + 1, &format!("{kw} {},", xs.join(", ")));
            }
        }
        if let Some(d) = &f.decreases {
            let d = self.expr(d, 0);
            self.line(depth + 1, &format!("decreases {d},"));
        }
    }

    // ---------- blocks and statements ----------

    /// Prints `{` on its own line at `depth`, then the block.
    fn block_lines(&mut self, b: &Block, depth: usize, suffix: &str) {
        self.block_open(b, depth, "", suffix);
    }

    /// Prints `prefix{` ... `}suffix` with the block's contents indented.
    fn block_open(&mut self, b: &Block, depth: usize, prefix: &str, suffix: &str) {
        if b.stmts.is_empty() && b.tail.is_none() {
            self.line(depth, &format!("{prefix}{{}}{suffix}"));
            return;
        }
        self.line(depth, &format!("{prefix}{{"));
        for s in &b.stmts {
            self.stmt(s, depth + 1);
        }
        if let Some(t) = &b.tail {
            self.expr_stmt(t, depth + 1, "");
        }
        self.line(depth, &format!("}}{suffix}"));
    }

    /// Prints an expression that may span lines (`if`, blocks) as a statement.
    fn expr_stmt(&mut self, e: &Expr, depth: usize, suffix: &str) {
        match &e.kind {
            ExprKind::If { .. } => self.if_lines(e, depth, "", suffix),
            ExprKind::Block(b) => self.block_open(b, depth, "", suffix),
            _ => {
                let s = self.expr(e, 0);
                self.line(depth, &format!("{s}{suffix}"));
            }
        }
    }

    fn if_lines(&mut self, e: &Expr, depth: usize, prefix: &str, suffix: &str) {
        let ExprKind::If { cond, then, els } = &e.kind else {
            unreachable!()
        };
        let c = self.expr(cond, 0);
        match els {
            None => self.block_open(then, depth, &format!("{prefix}if {c} "), suffix),
            Some(x) => {
                self.block_open(then, depth, &format!("{prefix}if {c} "), "");
                // Join `}` and `else` on one line.
                self.out.pop();
                match &x.kind {
                    ExprKind::If { .. } => {
                        let rest = self.capture(|p| p.if_lines(x, depth, "", suffix));
                        self.out.push_str(" else ");
                        self.out.push_str(rest.trim_start());
                    }
                    ExprKind::Block(b) => {
                        let rest = self.capture(|p| p.block_open(b, depth, "", suffix));
                        self.out.push_str(" else ");
                        self.out.push_str(rest.trim_start());
                    }
                    _ => unreachable!("else branch is a block or if"),
                }
            }
        }
    }

    fn capture(&mut self, f: impl FnOnce(&mut Self)) -> String {
        let saved = std::mem::take(&mut self.out);
        f(self);
        std::mem::replace(&mut self.out, saved)
    }

    fn stmt(&mut self, s: &Stmt, depth: usize) {
        match s {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
                ..
            } => {
                let m = if *mutable { "mut " } else { "" };
                let head = format!("let {m}{}: {} = ", name.name, type_str(ty));
                self.multiline_value(init, depth, &head, ";");
            }
            Stmt::Assign { target, value, .. } => {
                let head = format!("{} = ", self.expr(target, 0));
                self.multiline_value(value, depth, &head, ";");
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                let c = self.expr(cond, 0);
                self.line(depth, &format!("while {c}"));
                if !invariants.is_empty() {
                    let xs: Vec<String> = invariants.iter().map(|e| self.expr(e, 0)).collect();
                    self.line(depth + 1, &format!("invariant {},", xs.join(", ")));
                }
                if let Some(d) = decreases {
                    let d = self.expr(d, 0);
                    self.line(depth + 1, &format!("decreases {d},"));
                }
                self.block_lines(body, depth, "");
            }
            Stmt::Assert { cond, .. } => {
                let c = self.expr(cond, 0);
                self.line(depth, &format!("assert({c});"));
            }
            Stmt::Expr(e) => match &e.kind {
                ExprKind::If { .. } | ExprKind::Block(_) => self.expr_stmt(e, depth, ""),
                _ => {
                    let x = self.expr(e, 0);
                    self.line(depth, &format!("{x};"));
                }
            },
        }
    }

    /// `head value;` where the value may be a multi-line `if` or block.
    fn multiline_value(&mut self, v: &Expr, depth: usize, head: &str, suffix: &str) {
        match &v.kind {
            ExprKind::If { .. } => self.if_lines(v, depth, head, suffix),
            ExprKind::Block(b) => self.block_open(b, depth, head, suffix),
            _ => {
                let x = self.expr(v, 0);
                self.line(depth, &format!("{head}{x}{suffix}"));
            }
        }
    }

    // ---------- expressions (single line) ----------

    /// Prints `e`, parenthesized if its precedence is below `min`.
    fn expr(&mut self, e: &Expr, min: u8) -> String {
        let s = self.expr_raw(e);
        if prec(e) < min || (prec(e) == 0 && min > 0) {
            format!("({s})")
        } else {
            s
        }
    }

    fn expr_raw(&mut self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(v) => v.to_string(),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unit => "()".into(),
            ExprKind::Var(id) => id.name.clone(),
            ExprKind::Call { path, args } => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                format!("{}({})", self.path(path), a.join(", "))
            }
            ExprKind::Method { recv, name, args } => {
                let r = self.expr(recv, 8);
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                format!("{r}.{}({})", name.name, a.join(", "))
            }
            ExprKind::Index { base, index } => {
                format!("{}[{}]", self.expr(base, 8), self.expr(index, 0))
            }
            ExprKind::Unary(op, x) => {
                let o = if *op == UnOp::Neg { "-" } else { "!" };
                format!("{o}{}", self.expr(x, 7))
            }
            ExprKind::Binary(op, l, r) => {
                let p = prec(e);
                let (lm, rm) = match op {
                    BinOp::Implies => (p + 1, p),
                    o if o.is_cmp() => (p + 1, p + 1),
                    _ => (p, p + 1),
                };
                format!("{} {} {}", self.expr(l, lm), op.symbol(), self.expr(r, rm))
            }
            ExprKind::Ref { mutable, expr } => {
                format!(
                    "&{}{}",
                    if *mutable { "mut " } else { "" },
                    self.expr(expr, 7)
                )
            }
            ExprKind::Array(items) => {
                let a: Vec<String> = items.iter().map(|x| self.expr(x, 0)).collect();
                format!("[{}]", a.join(", "))
            }
            ExprKind::Repeat { value, count, .. } => format!("[{}; {count}]", self.expr(value, 0)),
            ExprKind::Old(x) => format!("old({})", self.expr(x, 0)),
            ExprKind::Forall { binders, body } => {
                let bs: Vec<String> = binders
                    .iter()
                    .map(|b| format!("{}: {}", b.name.name, type_str(&b.ty)))
                    .collect();
                format!("forall|{}| {}", bs.join(", "), self.expr(body, 0))
            }
            ExprKind::Trigger(x) => format!("#[trigger] {}", self.expr(x, 7)),
            // Nested multi-line forms are printed on one line inside expressions.
            ExprKind::If { .. } | ExprKind::Block(_) => {
                let text = self.capture(|p| p.expr_stmt(e, 0, ""));
                text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
            }
            ExprKind::Error => "<error>".into(),
        }
    }
}

/// Normal form of a whole program, one string per top-level unit, keyed by
/// unit name (ADR-0019 §3).
pub fn format_units(p: &Program) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for m in &p.mods {
        let mut pr = Printer {
            qualify: None,
            out: String::new(),
        };
        pr.module(m, 0);
        out.insert(m.name.name.clone(), pr.out);
    }
    out
}

/// Body and interface hashes of one definition (ADR-0019 §2).
pub struct DefHash {
    pub path: String,
    pub body: String,
    pub iface: String,
}

fn collect<'p>(m: &'p Mod, prefix: &str, out: &mut Vec<(String, String, &'p FnDef)>) {
    let mp = format!("{prefix}{}", m.name.name);
    for s in &m.mods {
        collect(s, &format!("{mp}::"), out);
    }
    for f in &m.fns {
        out.push((format!("{mp}::{}", f.name.name), mp.clone(), f));
    }
}

/// Qualified normal form of one definition, as hashed.
pub fn qualified_text(
    f: &FnDef,
    unit: &str,
    fns: &BTreeMap<String, FnSig>,
    with_body: bool,
) -> String {
    let mut pr = Printer {
        qualify: Some((fns, unit.to_string())),
        out: String::new(),
    };
    pr.function(f, 0, Some("_"), with_body);
    pr.out
}

pub fn hashes(p: &Program, fns: &BTreeMap<String, FnSig>) -> Vec<DefHash> {
    let mut defs = Vec::new();
    for m in &p.mods {
        collect(m, "", &mut defs);
    }
    defs.sort_by(|a, b| a.0.cmp(&b.0));
    defs.into_iter()
        .map(|(path, unit, f)| {
            let body = crate::sha256::hex(qualified_text(f, &unit, fns, true).as_bytes());
            let iface = if f.kind == FnKind::Spec {
                body.clone()
            } else {
                crate::sha256::hex(qualified_text(f, &unit, fns, false).as_bytes())
            };
            DefHash { path, body, iface }
        })
        .collect()
}
