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
        ExprKind::Unary(..) | ExprKind::Ref { .. } | ExprKind::Trigger(_) | ExprKind::Deref(_) => 7,
        // Always parenthesized when nested (ADR-0019 §1).
        ExprKind::If { .. }
        | ExprKind::Block(_)
        | ExprKind::Forall { .. }
        | ExprKind::Match { .. }
        | ExprKind::Loop { .. }
        | ExprKind::Return(_)
        | ExprKind::Break { .. } => 0,
        _ => 8,
    }
}

pub fn type_str(t: &Type) -> String {
    crate::check::ty_str(&t.kind)
}

fn path_str(p: &[Ident]) -> String {
    p.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

/// `<T: Copy, U: Copy + geo::Shape>`, or nothing (ADR-0046).
pub fn generics_str(gs: &[TypeParam]) -> String {
    if gs.is_empty() {
        return String::new();
    }
    let ps: Vec<String> = gs
        .iter()
        .map(|g| {
            if g.bounds.is_empty() {
                return g.name.name.clone();
            }
            let bs: Vec<String> = g
                .bounds
                .iter()
                .map(|b| {
                    b.iter()
                        .map(|i| i.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::")
                })
                .collect();
            format!("{}: {}", g.name.name, bs.join(" + "))
        })
        .collect();
    format!("<{}>", ps.join(", "))
}

/// `T<A, B>`, or `T`.
fn applied(name: &str, args: &[Type]) -> String {
    if args.is_empty() {
        return name.to_string();
    }
    let a: Vec<String> = args.iter().map(type_str).collect();
    format!("{name}<{}>", a.join(", "))
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
            // `f`, `T::f` (an `impl` function of the unit's type) and
            // `Trait::f` (a method of the unit's trait) are local, as the
            // checker resolves them.
            Some((fns, unit)) if path.len() <= 2 => {
                let local = format!("{unit}::{plain}");
                match fns.get(&local) {
                    Some(sig) if path.len() == 1 || sig.in_impl || sig.trait_of.is_some() => local,
                    _ => plain,
                }
            }
            _ => plain,
        }
    }

    fn uses_name(&self, name: &str) -> String {
        match &self.qualify {
            Some((fns, unit)) => {
                crate::check::uses_target(name, unit, fns).unwrap_or_else(|| name.to_string())
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
        // Types by name, each followed by its `impl` block (ADR-0026 §6).
        let mut types: Vec<&TypeDef> = m.types.iter().collect();
        types.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut impls: Vec<&Impl> = m.impls.iter().collect();
        impls.sort_by(|a, b| a.ty.name.cmp(&b.ty.name));
        // Traits by name, then the trait `impl`s for types of other units;
        // an `impl` for a type of this unit follows that type (ADR-0046).
        let mut traits: Vec<&TraitDef> = m.traits.iter().collect();
        traits.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let local_type = |i: &TraitImpl| match &i.self_ty.kind {
            TypeKind::Named { path, .. } if path.len() == 1 => {
                m.types.iter().any(|t| t.name.name == path[0])
            }
            _ => false,
        };
        let mut timpls: Vec<&TraitImpl> = m.trait_impls.iter().collect();
        timpls.sort_by_key(|i| (path_str(&i.trait_path), type_str(&i.self_ty)));
        let mut first = true;
        for s in subs {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.module(s, depth + 1);
        }
        for t in &traits {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.trait_def(t, depth + 1);
        }
        for i in timpls.iter().filter(|i| !local_type(i)) {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.trait_impl(i, depth + 1);
        }
        let mut printed = BTreeSet::new();
        for t in &types {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.type_def(t, depth + 1);
            for i in impls.iter().filter(|i| i.ty.name == t.name.name) {
                self.out.push('\n');
                self.impl_block(i, depth + 1);
                printed.insert(i.ty.name.clone());
            }
            for i in timpls.iter().filter(|i| {
                matches!(&i.self_ty.kind, TypeKind::Named { path, .. } if path.len() == 1 && path[0] == t.name.name)
            }) {
                self.out.push('\n');
                self.trait_impl(i, depth + 1);
            }
        }
        // An `impl` without its type in the unit (the checker reports it).
        for i in impls.iter().filter(|i| !printed.contains(&i.ty.name)) {
            if !first {
                self.out.push('\n');
            }
            first = false;
            self.impl_block(i, depth + 1);
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

    fn attr_line(&mut self, a: &Attr, depth: usize) {
        let args: Vec<String> = a
            .args
            .iter()
            .map(|x| match x {
                AttrArg::Ident(i) => i.name.clone(),
                AttrArg::Str(s, _) => format!("\"{}\"", escape(s)),
            })
            .collect();
        self.line(depth, &format!("#[{}({})]", a.name.name, args.join(", ")));
    }

    /// `struct T { f: U, .. }` / `enum T { A, B(U), .. }` on one line; fields and
    /// variants keep their declaration order (ADR-0026 §6).
    pub(crate) fn type_def(&mut self, t: &TypeDef, depth: usize) {
        self.type_def_as(t, depth, &t.name.name);
    }

    fn type_def_as(&mut self, t: &TypeDef, depth: usize, name: &str) {
        for a in &t.attrs {
            self.attr_line(a, depth);
        }
        let (kw, items): (&str, Vec<String>) = match &t.kind {
            TypeDefKind::Struct(fs) => (
                "struct",
                fs.iter()
                    .map(|f| format!("{}: {}", f.name.name, type_str(&f.ty)))
                    .collect(),
            ),
            TypeDefKind::Enum(vs) => (
                "enum",
                vs.iter()
                    .map(|v| {
                        if v.payload.is_empty() {
                            v.name.name.clone()
                        } else {
                            let ps: Vec<String> = v.payload.iter().map(type_str).collect();
                            format!("{}({})", v.name.name, ps.join(", "))
                        }
                    })
                    .collect(),
            ),
        };
        let name = format!("{name}{}", generics_str(&t.generics));
        if items.is_empty() {
            self.line(depth, &format!("{kw} {name} {{}}"));
        } else {
            self.line(depth, &format!("{kw} {name} {{ {} }}", items.join(", ")));
        }
    }

    fn impl_block(&mut self, i: &Impl, depth: usize) {
        let mut fns: Vec<&FnDef> = i.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let head = format!(
            "impl{} {}",
            generics_str(&i.generics),
            applied(&i.ty.name, &i.ty_args)
        );
        if fns.is_empty() {
            self.line(depth, &format!("{head} {{}}"));
            return;
        }
        self.line(depth, &format!("{head} {{"));
        for (k, f) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            self.function(f, depth + 1, None, true);
        }
        self.line(depth, "}");
    }

    /// `trait T { fn m(..) -> R; .. }`, methods by name (ADR-0046 §1).
    fn trait_def(&mut self, t: &TraitDef, depth: usize) {
        for a in &t.attrs {
            self.attr_line(a, depth);
        }
        let mut fns: Vec<&TraitFn> = t.fns.iter().collect();
        fns.sort_by(|a, b| a.f.name.name.cmp(&b.f.name.name));
        if fns.is_empty() {
            self.line(depth, &format!("trait {} {{}}", t.name.name));
            return;
        }
        self.line(depth, &format!("trait {} {{", t.name.name));
        for (k, tf) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
            if tf.has_body {
                self.function(&tf.f, depth + 1, None, true);
            } else {
                self.function(&tf.f, depth + 1, None, false);
                // The signature ends with `;` (after its last clause).
                if self.out.ends_with(",\n") {
                    self.out.truncate(self.out.len() - 2);
                } else {
                    self.out.pop();
                }
                self.out.push_str(";\n");
            }
        }
        self.line(depth, "}");
    }

    /// `impl<..> Trait for T { .. }`, functions by name (ADR-0046 §2).
    fn trait_impl(&mut self, i: &TraitImpl, depth: usize) {
        let mut fns: Vec<&FnDef> = i.fns.iter().collect();
        fns.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let head = format!(
            "impl{} {} for {}",
            generics_str(&i.generics),
            path_str(&i.trait_path),
            type_str(&i.self_ty)
        );
        if fns.is_empty() {
            self.line(depth, &format!("{head} {{}}"));
            return;
        }
        self.line(depth, &format!("{head} {{"));
        for (k, f) in fns.iter().enumerate() {
            if k > 0 {
                self.out.push('\n');
            }
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
        let sig = format!(
            "{kw} {name}{}({}) -> {ret}",
            generics_str(&f.generics),
            params.join(", ")
        );
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
            ExprKind::Match { .. } => self.match_lines(e, depth, "", suffix),
            ExprKind::Loop { .. } => self.loop_lines(e, depth, "", suffix),
            _ => {
                let s = self.expr(e, 0);
                self.line(depth, &format!("{s}{suffix}"));
            }
        }
    }

    /// `head['l: ]loop`, its clauses on their own lines as `while`'s, and
    /// the body.
    fn loop_lines(&mut self, e: &Expr, depth: usize, head: &str, suffix: &str) {
        let ExprKind::Loop {
            label,
            invariants,
            decreases,
            body,
        } = &e.kind
        else {
            unreachable!()
        };
        let l = label
            .as_ref()
            .map_or(String::new(), |l| format!("'{}: ", l.name));
        if invariants.is_empty() && decreases.is_none() {
            self.block_open(body, depth, &format!("{head}{l}loop "), suffix);
            return;
        }
        self.line(depth, &format!("{head}{l}loop"));
        if !invariants.is_empty() {
            let xs: Vec<String> = invariants.iter().map(|e| self.expr(e, 0)).collect();
            self.line(depth + 1, &format!("invariant {},", xs.join(", ")));
        }
        if let Some(d) = decreases {
            let d = self.expr(d, 0);
            self.line(depth + 1, &format!("decreases {d},"));
        }
        self.block_open(body, depth, "", suffix);
    }

    /// `match` with one arm per line (each arm's body on one line).
    fn match_lines(&mut self, e: &Expr, depth: usize, prefix: &str, suffix: &str) {
        let ExprKind::Match { scrut, arms } = &e.kind else {
            unreachable!()
        };
        let s = self.cond(scrut);
        self.line(depth, &format!("{prefix}match {s} {{"));
        for a in arms {
            let body = self.expr(&a.body, 0);
            let pat = pat_str(&a.pat);
            self.line(depth + 1, &format!("{pat} => {body},"));
        }
        self.line(depth, &format!("}}{suffix}"));
    }

    /// A condition or scrutinee: parenthesized when it holds a struct literal,
    /// which Rust does not accept there unparenthesized.
    fn cond(&mut self, e: &Expr) -> String {
        let s = self.expr(e, 0);
        if has_struct_lit(e) {
            format!("({s})")
        } else {
            s
        }
    }

    fn if_lines(&mut self, e: &Expr, depth: usize, prefix: &str, suffix: &str) {
        let ExprKind::If { cond, then, els } = &e.kind else {
            unreachable!()
        };
        let c = self.cond(cond);
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
                label,
                cond,
                invariants,
                decreases,
                body,
                ..
            } => {
                let c = self.cond(cond);
                let l = label
                    .as_ref()
                    .map_or(String::new(), |l| format!("'{}: ", l.name));
                self.line(depth, &format!("{l}while {c}"));
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
            Stmt::LetFields {
                path, fields, init, ..
            } => {
                let fs: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
                let head = format!("let {} {{ {} }} = ", self.path(path), fs.join(", "));
                self.multiline_value(init, depth, &head, ";");
            }
            Stmt::Expr(e) => match &e.kind {
                ExprKind::If { .. }
                | ExprKind::Block(_)
                | ExprKind::Match { .. }
                | ExprKind::Loop { .. } => self.expr_stmt(e, depth, ""),
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
            ExprKind::Match { .. } => self.match_lines(v, depth, head, suffix),
            ExprKind::Loop { .. } => self.loop_lines(v, depth, head, suffix),
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
            ExprKind::ByteLit(v) => crate::lexer::byte_text(&[*v], '\''),
            ExprKind::ByteStr(v) => crate::lexer::byte_text(v, '"'),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Unit => "()".into(),
            ExprKind::Return(None) => "return".into(),
            ExprKind::Return(Some(v)) => format!("return {}", self.expr(v, 0)),
            ExprKind::Break { label, value } => {
                let mut t = "break".to_string();
                if let Some(l) = label {
                    t.push_str(&format!(" '{}", l.name));
                }
                if let Some(v) = value {
                    t.push_str(&format!(" {}", self.expr(v, 0)));
                }
                t
            }
            ExprKind::Continue { label: None } => "continue".into(),
            ExprKind::Continue { label: Some(l) } => format!("continue '{}", l.name),
            ExprKind::Var(id) => id.name.clone(),
            ExprKind::Call { path, targs, args } => {
                let a: Vec<String> = args.iter().map(|x| self.expr(x, 0)).collect();
                let mut segs: Vec<String> = self.path(path).split("::").map(String::from).collect();
                // A qualified path gains its unit's segments in front.
                let off = segs.len() - path.len();
                for t in targs {
                    let ts: Vec<String> = t.args.iter().map(type_str).collect();
                    segs[t.seg + off] = format!("{}::<{}>", segs[t.seg + off], ts.join(", "));
                }
                format!("{}({})", segs.join("::"), a.join(", "))
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
            ExprKind::StructLit { path, fields, rest } => {
                let mut fs: Vec<String> = fields
                    .iter()
                    .map(|(n, x)| format!("{}: {}", n.name, self.expr(x, 0)))
                    .collect();
                if let Some(r) = rest {
                    fs.push(format!("..{}", self.expr(r, 0)));
                }
                format!("{} {{ {} }}", self.path(path), fs.join(", "))
            }
            ExprKind::Path(path) => self.path(path),
            ExprKind::Deref(x) => format!("*{}", self.expr(x, 7)),
            ExprKind::Field { base, name } => format!("{}.{}", self.expr(base, 8), name.name),
            // Nested multi-line forms are printed on one line inside expressions.
            ExprKind::If { .. }
            | ExprKind::Block(_)
            | ExprKind::Match { .. }
            | ExprKind::Loop { .. } => {
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
    // The `impl` a function is checked and proved in (backlog 2T-03): its
    // type parameters with their bounds, and for a trait's method or a
    // trait impl's function the trait and the type, which its contract and
    // refinement obligations depend on.
    if let Some(sig) = fns
        .values()
        .find(|s| s.span == f.span && !s.builtin && !s.path.ends_with('>'))
        && (sig.impl_generics > 0 || sig.path.starts_with('<'))
        && let Some((head, _)) = sig.path.rsplit_once("::")
    {
        let gs: Vec<String> = sig.generics[..sig.impl_generics]
            .iter()
            .map(|g| format!("{}: {}", g.name, g.bounds.join(" + ")))
            .collect();
        pr.out = format!("impl<{}> {head}\n", gs.join(", "));
    }
    pr.function(f, 0, Some("_"), with_body);
    pr.out
}

pub fn hashes(p: &Program, fns: &BTreeMap<String, FnSig>) -> Vec<DefHash> {
    let mut defs = crate::check::defs_of(p, fns);
    defs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out: Vec<DefHash> = defs
        .into_iter()
        .map(|(path, unit, f)| {
            let (body, iface) = def_hash(f, &unit, fns);
            DefHash { path, body, iface }
        })
        .collect();
    out.extend(type_hashes(p));
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// A trait declaration in normal form on one line (map L1 `signature`).
pub fn trait_text(t: &TraitDef) -> String {
    trait_text_as(t, &t.name.name)
}

/// `trait_text` with the trait's own name written as `name` (hashing
/// uses `_`).
pub fn trait_text_as(t: &TraitDef, name: &str) -> String {
    let mut q = t.clone();
    q.name.name = name.to_string();
    let mut pr = plain();
    pr.trait_def(&q, 0);
    pr.out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// A type declaration in normal form on one line (map L1 `signature`).
pub fn type_text(t: &TypeDef) -> String {
    let mut pr = plain();
    pr.type_def(t, 0);
    pr.out.trim_end().to_string()
}

/// Hashes of the type declarations. A type declaration is its own surface:
/// body hash = interface hash, its name written `_` (ADR-0026 §6, as for
/// definitions, ADR-0019 §2).
pub fn type_hashes(p: &Program) -> Vec<DefHash> {
    let mut out = Vec::new();
    fn types(m: &Mod, prefix: &str, out: &mut Vec<DefHash>) {
        let mp = format!("{prefix}{}", m.name.name);
        for s in &m.mods {
            types(s, &format!("{mp}::"), out);
        }
        let local: BTreeSet<&str> = m.types.iter().map(|t| t.name.name.as_str()).collect();
        // Unit-local type names written in full (the qualified normal form).
        fn qualify(t: &Type, mp: &str, local: &BTreeSet<&str>) -> Type {
            let q = |x: &Type| Box::new(qualify(x, mp, local));
            let kind = match &t.kind {
                TypeKind::Named { path, args }
                    if path.len() == 1 && local.contains(path[0].as_str()) =>
                {
                    TypeKind::Named {
                        path: vec![mp.to_string(), path[0].clone()],
                        args: args.iter().map(|a| qualify(a, mp, local)).collect(),
                    }
                }
                TypeKind::Named { path, args } => TypeKind::Named {
                    path: path.clone(),
                    args: args.iter().map(|a| qualify(a, mp, local)).collect(),
                },
                TypeKind::Array(e, n) => TypeKind::Array(q(e), *n),
                TypeKind::Slice(e) => TypeKind::Slice(q(e)),
                TypeKind::Ref { mutable, inner } => TypeKind::Ref {
                    mutable: *mutable,
                    inner: q(inner),
                },
                k => k.clone(),
            };
            Type { kind, span: t.span }
        }
        for t in &m.types {
            let mut q = t.clone();
            match &mut q.kind {
                TypeDefKind::Struct(fs) => fs
                    .iter_mut()
                    .for_each(|f| f.ty = qualify(&f.ty, &mp, &local)),
                TypeDefKind::Enum(vs) => vs
                    .iter_mut()
                    .flat_map(|v| v.payload.iter_mut())
                    .for_each(|ty| *ty = qualify(ty, &mp, &local)),
            }
            let mut pr = plain();
            pr.type_def_as(&q, 0, "_");
            let h = crate::sha256::hex(pr.out.as_bytes());
            out.push(DefHash {
                path: format!("{mp}::{}", t.name.name),
                body: h.clone(),
                iface: h,
            });
        }
    }
    for m in &p.mods {
        types(m, "", &mut out);
    }
    out
}

/// Body and interface hash of one definition in `unit` (ADR-0019 §2).
pub fn def_hash(f: &FnDef, unit: &str, fns: &BTreeMap<String, FnSig>) -> (String, String) {
    let body = crate::sha256::hex(qualified_text(f, unit, fns, true).as_bytes());
    let iface = if f.kind == FnKind::Spec {
        body.clone()
    } else {
        crate::sha256::hex(qualified_text(f, unit, fns, false).as_bytes())
    };
    (body, iface)
}

/// What a caller's query reads of an executable function (backlog 2D-22):
/// its qualified surface without `#[uses]`. A definition's `#[uses]` only
/// reveals spec fn bodies in its own queries (`verify::uses_of`), so the
/// incremental keys of its callers leave it out.
pub fn caller_text(f: &FnDef, unit: &str, fns: &BTreeMap<String, FnSig>) -> String {
    let mut g = f.clone();
    g.attrs.retain(|a| a.name.name != "uses");
    qualified_text(&g, unit, fns, false)
}

fn plain() -> Printer<'static> {
    Printer {
        qualify: None,
        out: String::new(),
    }
}

/// Normal form of one definition.
pub fn def_text(f: &FnDef) -> String {
    let mut pr = plain();
    pr.function(f, 0, None, true);
    pr.out
}

/// An expression in normal form on one line (the text of a fix).
pub fn expr_text(e: &Expr) -> String {
    plain().expr(e, 0)
}

/// A pattern in normal form.
pub fn pat_text(p: &Pat) -> String {
    pat_str(p)
}

/// Normal form of a definition without its body (its surface).
pub fn iface_text(f: &FnDef) -> String {
    let mut pr = plain();
    pr.function(f, 0, None, false);
    pr.out
}

/// One-line signature and contract clauses in normal form (map L1).
pub fn surface_parts(f: &FnDef) -> (String, Vec<String>, Vec<String>, Option<String>) {
    let mut pr = plain();
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
    let sig = format!(
        "{kw} {}{}({}) -> {ret}",
        f.name.name,
        generics_str(&f.generics),
        params.join(", ")
    );
    let req = f.requires.iter().map(|e| pr.expr(e, 0)).collect();
    let ens = f.ensures.iter().map(|e| pr.expr(e, 0)).collect();
    let dec = f.decreases.as_ref().map(|e| pr.expr(e, 0));
    (sig, req, ens, dec)
}

fn pat_str(p: &Pat) -> String {
    let path = p
        .path
        .iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::");
    if !p.parens {
        return path;
    }
    let bs: Vec<&str> = p
        .binds
        .iter()
        .map(|b| b.as_ref().map_or("_", |i| i.name.as_str()))
        .collect();
    format!("{path}({})", bs.join(", "))
}

/// Whether a struct literal occurs in `e` outside a delimited argument list.
fn has_struct_lit(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::StructLit { .. } => true,
        ExprKind::Binary(_, l, r) => has_struct_lit(l) || has_struct_lit(r),
        ExprKind::Unary(_, x) | ExprKind::Ref { expr: x, .. } | ExprKind::Deref(x) => {
            has_struct_lit(x)
        }
        ExprKind::Field { base, .. } => has_struct_lit(base),
        ExprKind::Method { recv, .. } => has_struct_lit(recv),
        ExprKind::Index { base, .. } => has_struct_lit(base),
        _ => false,
    }
}
