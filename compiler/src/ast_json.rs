//! The syntax tree as a JSON document (backlog T1-06, ADR-0073): what the
//! JSON AST edit surface reads (`read_ast`) and patches (`edit_ast`).
//!
//! The document is an object keyed by top-level unit name. A unit is
//! `{"summary", "units", "traits", "types", "impls", "trait_impls", "fns"}`;
//! units, traits, types, impls (by type name) and functions are objects keyed
//! by name, so a JSON Pointer names a definition as a path does
//! (`/demo/fns/get/body/tail`). Every other node is an object whose `kind`
//! says what it is, with the fields of `ast.rs` under their names there.
//! Names, paths (`"a::b"`) and operators (`"+"`) are strings. Empty lists,
//! `false` flags and absent parts are left out. Spans are not part of the
//! document: positions are pointers.
//!
//! What the normal form drops is not in the document either (function
//! attributes other than `effects` and `uses`, unit attributes other than
//! `summary`), and the parser's recovery forms (`use` items, `let` without a
//! type, destructuring `let`, `..base`, wildcard arms) have no JSON: they never
//! occur in a workspace that checks.

use std::collections::BTreeSet;

use crate::ast::*;
use crate::json::Value;
use crate::source::Span;

/// `(span, JSON Pointer)` of each node of an exported program, parents
/// before children.
pub type Spans = Vec<(Span, String)>;

type R = Result<Value, String>;

fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

fn obj(kv: Vec<(&str, Value)>) -> Value {
    Value::Object(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// One segment of a JSON Pointer (RFC 6901 §3).
pub fn escape(seg: &str) -> String {
    seg.replace('~', "~0").replace('/', "~1")
}

fn path_text(p: &[Ident]) -> String {
    p.iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

fn bin_name(op: BinOp) -> &'static str {
    op.symbol()
}

fn bin_op(s: &str) -> Option<BinOp> {
    use BinOp::*;
    [
        Add, Sub, Mul, Div, Rem, Eq, Ne, Lt, Le, Gt, Ge, And, Or, Implies, BitAnd, BitOr, BitXor,
        Shl, Shr,
    ]
    .into_iter()
    .find(|o| o.symbol() == s)
}

// ---------- export ----------

/// The JSON document of `p`.
pub fn export(p: &Program) -> R {
    Ex {
        at: Vec::new(),
        rec: None,
    }
    .program(p)
}

/// The JSON document of `p` and, in `spans`, the pointer of every node
/// with a span (to name the node a diagnostic points at).
pub fn export_spans(p: &Program, spans: &mut Spans) -> R {
    Ex {
        at: Vec::new(),
        rec: Some(spans),
    }
    .program(p)
}

struct Ex<'a> {
    at: Vec<String>,
    rec: Option<&'a mut Spans>,
}

impl Ex<'_> {
    fn ptr(&self) -> String {
        self.at.iter().map(|s| format!("/{}", escape(s))).collect()
    }

    fn mark(&mut self, sp: Span) {
        if self.rec.is_some() {
            let p = self.ptr();
            if let Some(r) = &mut self.rec {
                r.push((sp, p));
            }
        }
    }

    fn fail<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("{}: {what}", self.ptr()))
    }

    fn under<T>(
        &mut self,
        seg: &str,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        self.at.push(seg.to_string());
        let r = f(self);
        self.at.pop();
        r
    }

    /// `items` as an object keyed by `key`, sorted by key; a repeated key
    /// is an error (the program does not check).
    fn keyed<T>(
        &mut self,
        field: &str,
        items: &[T],
        key: impl Fn(&T) -> String,
        f: impl Fn(&mut Self, &T) -> R,
    ) -> Result<Value, String> {
        let mut xs: Vec<&T> = items.iter().collect();
        xs.sort_by_key(|x| key(x));
        let mut kv: Vec<(String, Value)> = Vec::new();
        self.under(field, |e| {
            for x in xs {
                let k = key(x);
                if kv.iter().any(|(j, _)| *j == k) {
                    return e.fail(&format!("two members named `{k}`"));
                }
                let v = e.under(&k, |e| f(e, x))?;
                kv.push((k, v));
            }
            Ok(())
        })?;
        Ok(Value::Object(kv))
    }

    fn list<T>(&mut self, field: &str, items: &[T], f: impl Fn(&mut Self, &T) -> R) -> R {
        self.under(field, |e| {
            let mut out = Vec::new();
            for (k, x) in items.iter().enumerate() {
                out.push(e.under(&k.to_string(), |e| f(e, x))?);
            }
            Ok(Value::Array(out))
        })
    }

    fn program(&mut self, p: &Program) -> R {
        self.keyed_root(&p.mods)
    }

    fn keyed_root(&mut self, mods: &[Mod]) -> R {
        let mut xs: Vec<&Mod> = mods.iter().collect();
        xs.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        let mut kv: Vec<(String, Value)> = Vec::new();
        for m in xs {
            let k = m.name.name.clone();
            if kv.iter().any(|(j, _)| *j == k) {
                return self.fail(&format!("two units named `{k}`"));
            }
            let v = self.under(&k, |e| e.unit(m))?;
            kv.push((k, v));
        }
        Ok(Value::Object(kv))
    }

    fn unit(&mut self, m: &Mod) -> R {
        self.mark(m.span);
        if !m.uses.is_empty() {
            return self.fail("`use` items are not part of the language");
        }
        let mut kv: Vec<(&str, Value)> = Vec::new();
        let summary = m.attrs.iter().find_map(|a| match a.args.first() {
            Some(AttrArg::Str(s, _)) if a.name.name == "summary" => Some(s.clone()),
            _ => None,
        });
        if let Some(sm) = summary {
            kv.push(("summary", Value::Str(sm)));
        }
        if !m.mods.is_empty() {
            kv.push(("units", self.under("units", |e| e.keyed_root(&m.mods))?));
        }
        if !m.traits.is_empty() {
            let v = self.keyed("traits", &m.traits, |t| t.name.name.clone(), Ex::trait_def)?;
            kv.push(("traits", v));
        }
        if !m.types.is_empty() {
            let v = self.keyed("types", &m.types, |t| t.name.name.clone(), Ex::type_def)?;
            kv.push(("types", v));
        }
        if !m.impls.is_empty() {
            let v = self.keyed("impls", &m.impls, |i| i.ty.name.clone(), Ex::impl_block)?;
            kv.push(("impls", v));
        }
        if !m.trait_impls.is_empty() {
            let mut ti: Vec<TraitImpl> = m.trait_impls.clone();
            ti.sort_by_key(|i| (path_text(&i.trait_path), crate::fmt::type_str(&i.self_ty)));
            let v = self.list("trait_impls", &ti, Ex::trait_impl)?;
            kv.push(("trait_impls", v));
        }
        if !m.fns.is_empty() {
            let v = self.keyed(
                "fns",
                &m.fns,
                |f| f.name.name.clone(),
                |e, f| e.function(f, true),
            )?;
            kv.push(("fns", v));
        }
        Ok(obj(kv))
    }

    fn attrs(&mut self, attrs: &[Attr]) -> R {
        self.list("attrs", attrs, |e, a| {
            e.mark(a.span);
            let args: Vec<Value> = a
                .args
                .iter()
                .map(|x| match x {
                    AttrArg::Ident(i) => s(&i.name),
                    AttrArg::Str(t, _) => obj(vec![("str", s(t))]),
                })
                .collect();
            let mut kv = vec![("name", s(&a.name.name))];
            if !args.is_empty() {
                kv.push(("args", Value::Array(args)));
            }
            Ok(obj(kv))
        })
    }

    fn generics(&mut self, gs: &[TypeParam]) -> R {
        self.list("generics", gs, |e, g| {
            e.mark(g.span);
            let mut kv = vec![("name", s(&g.name.name))];
            if !g.bounds.is_empty() {
                kv.push((
                    "bounds",
                    Value::Array(g.bounds.iter().map(|b| s(&path_text(b))).collect()),
                ));
            }
            Ok(obj(kv))
        })
    }

    fn type_def(&mut self, t: &TypeDef) -> R {
        self.mark(t.span);
        let mut kv: Vec<(&str, Value)> = Vec::new();
        match &t.kind {
            TypeDefKind::Struct(fs) => {
                kv.push(("kind", s("struct")));
                if !t.generics.is_empty() {
                    kv.push(("generics", self.generics(&t.generics)?));
                }
                if !t.attrs.is_empty() {
                    kv.push(("attrs", self.attrs(&t.attrs)?));
                }
                // Fields keep their declaration order (ADR-0026 §6).
                let mut out: Vec<(String, Value)> = Vec::new();
                self.under("fields", |e| {
                    for f in fs {
                        if out.iter().any(|(k, _)| *k == f.name.name) {
                            return e.fail(&format!("two fields named `{}`", f.name.name));
                        }
                        let v = e.under(&f.name.name, |e| e.ty(&f.ty))?;
                        out.push((f.name.name.clone(), v));
                    }
                    Ok(())
                })?;
                kv.push(("fields", Value::Object(out)));
            }
            TypeDefKind::Enum(vs) => {
                kv.push(("kind", s("enum")));
                if !t.generics.is_empty() {
                    kv.push(("generics", self.generics(&t.generics)?));
                }
                if !t.attrs.is_empty() {
                    kv.push(("attrs", self.attrs(&t.attrs)?));
                }
                let mut out: Vec<(String, Value)> = Vec::new();
                self.under("variants", |e| {
                    for v in vs {
                        if out.iter().any(|(k, _)| *k == v.name.name) {
                            return e.fail(&format!("two variants named `{}`", v.name.name));
                        }
                        let x = e.under(&v.name.name, |e| {
                            e.mark(v.span);
                            let mut ps = Vec::new();
                            for (k, p) in v.payload.iter().enumerate() {
                                ps.push(e.under(&k.to_string(), |e| e.ty(p))?);
                            }
                            Ok(Value::Array(ps))
                        })?;
                        out.push((v.name.name.clone(), x));
                    }
                    Ok(())
                })?;
                kv.push(("variants", Value::Object(out)));
            }
        }
        Ok(obj(kv))
    }

    fn impl_block(&mut self, i: &Impl) -> R {
        self.mark(i.span);
        let mut kv: Vec<(&str, Value)> = Vec::new();
        if !i.generics.is_empty() {
            kv.push(("generics", self.generics(&i.generics)?));
        }
        if !i.ty_args.is_empty() {
            kv.push(("ty_args", self.list("ty_args", &i.ty_args, Ex::ty)?));
        }
        let fns = self.keyed(
            "fns",
            &i.fns,
            |f| f.name.name.clone(),
            |e, f| e.function(f, true),
        )?;
        kv.push(("fns", fns));
        Ok(obj(kv))
    }

    fn trait_def(&mut self, t: &TraitDef) -> R {
        self.mark(t.span);
        let mut kv: Vec<(&str, Value)> = Vec::new();
        if !t.attrs.is_empty() {
            kv.push(("attrs", self.attrs(&t.attrs)?));
        }
        let fns = self.keyed(
            "fns",
            &t.fns,
            |f| f.f.name.name.clone(),
            |e, f| e.function(&f.f, f.has_body),
        )?;
        kv.push(("fns", fns));
        Ok(obj(kv))
    }

    fn trait_impl(&mut self, i: &TraitImpl) -> R {
        self.mark(i.span);
        let mut kv: Vec<(&str, Value)> = Vec::new();
        if !i.generics.is_empty() {
            kv.push(("generics", self.generics(&i.generics)?));
        }
        kv.push(("trait", s(&path_text(&i.trait_path))));
        kv.push(("self_ty", self.under("self_ty", |e| e.ty(&i.self_ty))?));
        let fns = self.keyed(
            "fns",
            &i.fns,
            |f| f.name.name.clone(),
            |e, f| e.function(f, true),
        )?;
        kv.push(("fns", fns));
        Ok(obj(kv))
    }

    fn function(&mut self, f: &FnDef, body: bool) -> R {
        self.mark(f.span);
        let kind = match f.kind {
            FnKind::Exec => "fn",
            FnKind::Spec => "spec fn",
            FnKind::Proof => "proof fn",
        };
        let mut kv: Vec<(&str, Value)> = vec![("kind", s(kind))];
        // The normal form keeps `effects` and `uses` only (fmt.rs `function`).
        if let Some(a) = f.attrs.iter().find(|a| a.name.name == "effects") {
            let mut xs = Vec::new();
            for x in &a.args {
                match x {
                    AttrArg::Ident(i) => xs.push(s(&i.name)),
                    AttrArg::Str(..) => return self.fail("`effects` takes names, not strings"),
                }
            }
            kv.push(("effects", Value::Array(xs)));
        }
        let uses: BTreeSet<&str> = f
            .attrs
            .iter()
            .filter(|a| a.name.name == "uses")
            .flat_map(|a| a.args.iter())
            .filter_map(|x| match x {
                AttrArg::Ident(i) => Some(i.name.as_str()),
                AttrArg::Str(..) => None,
            })
            .collect();
        if !uses.is_empty() {
            kv.push(("uses", Value::Array(uses.into_iter().map(s).collect())));
        }
        if !f.generics.is_empty() {
            kv.push(("generics", self.generics(&f.generics)?));
        }
        if !f.params.is_empty() {
            let ps = self.list("params", &f.params, Ex::param)?;
            kv.push(("params", ps));
        }
        let ret = self.under("ret", |e| {
            let mut r = Vec::new();
            if let Some(n) = &f.ret.name {
                r.push(("name", s(&n.name)));
            }
            r.push(("ty", e.under("ty", |e| e.ty(&f.ret.ty))?));
            Ok(obj(r))
        })?;
        kv.push(("ret", ret));
        if !f.requires.is_empty() {
            kv.push(("requires", self.list("requires", &f.requires, Ex::expr)?));
        }
        if !f.ensures.is_empty() {
            kv.push(("ensures", self.list("ensures", &f.ensures, Ex::expr)?));
        }
        if let Some(d) = &f.decreases {
            kv.push(("decreases", self.under("decreases", |e| e.expr(d))?));
        }
        if body {
            kv.push(("body", self.under("body", |e| e.block(&f.body))?));
        }
        Ok(obj(kv))
    }

    fn param(&mut self, p: &Param) -> R {
        self.mark(p.name.span.to(p.ty.span));
        Ok(obj(vec![
            ("name", s(&p.name.name)),
            ("ty", self.under("ty", |e| e.ty(&p.ty))?),
        ]))
    }

    fn ty(&mut self, t: &Type) -> R {
        self.mark(t.span);
        Ok(match &t.kind {
            TypeKind::Int(i) => obj(vec![("kind", s(i.name()))]),
            TypeKind::F64 => obj(vec![("kind", s("f64"))]),
            TypeKind::Bool => obj(vec![("kind", s("bool"))]),
            TypeKind::Unit => obj(vec![("kind", s("unit"))]),
            TypeKind::Array(e, n) => obj(vec![
                ("kind", s("array")),
                ("elem", self.under("elem", |x| x.ty(e))?),
                ("len", Value::Num(n.to_string())),
            ]),
            TypeKind::Slice(e) => obj(vec![
                ("kind", s("slice")),
                ("elem", self.under("elem", |x| x.ty(e))?),
            ]),
            TypeKind::Ref { mutable, inner } => {
                let mut kv = vec![("kind", s("ref"))];
                if *mutable {
                    kv.push(("mutable", Value::Bool(true)));
                }
                kv.push(("inner", self.under("inner", |x| x.ty(inner))?));
                obj(kv)
            }
            TypeKind::Named { path, args } => {
                let mut kv = vec![("kind", s("named")), ("path", s(&path.join("::")))];
                if !args.is_empty() {
                    kv.push(("args", self.list("args", args, Ex::ty)?));
                }
                obj(kv)
            }
            TypeKind::Vec(e) => obj(vec![
                ("kind", s("Vec")),
                ("elem", self.under("elem", |x| x.ty(e))?),
            ]),
            TypeKind::Box(e) => obj(vec![
                ("kind", s("Box")),
                ("elem", self.under("elem", |x| x.ty(e))?),
            ]),
            TypeKind::Param(n) => obj(vec![("kind", s("named")), ("path", s(n))]),
            TypeKind::Never | TypeKind::Error => return self.fail("not a type of the language"),
        })
    }

    fn block(&mut self, b: &Block) -> R {
        self.mark(b.span);
        Ok(obj(self.block_fields(b)?))
    }

    fn block_fields(&mut self, b: &Block) -> Result<Vec<(&'static str, Value)>, String> {
        let mut kv = Vec::new();
        if !b.stmts.is_empty() {
            kv.push(("stmts", self.list("stmts", &b.stmts, Ex::stmt)?));
        }
        if let Some(t) = &b.tail {
            kv.push(("tail", self.under("tail", |e| e.expr(t))?));
        }
        Ok(kv)
    }

    fn label(kv: &mut Vec<(&str, Value)>, l: &Option<Ident>) {
        if let Some(l) = l {
            kv.push(("label", s(&l.name)));
        }
    }

    fn exprs(
        &mut self,
        field: &'static str,
        xs: &[Expr],
        kv: &mut Vec<(&str, Value)>,
    ) -> Result<(), String> {
        if !xs.is_empty() {
            kv.push((field, self.list(field, xs, Ex::expr)?));
        }
        Ok(())
    }

    fn opt_expr(
        &mut self,
        field: &'static str,
        x: Option<&Expr>,
        kv: &mut Vec<(&str, Value)>,
    ) -> Result<(), String> {
        if let Some(x) = x {
            kv.push((field, self.under(field, |e| e.expr(x))?));
        }
        Ok(())
    }

    fn stmt(&mut self, st: &Stmt) -> R {
        let mut kv: Vec<(&str, Value)> = Vec::new();
        match st {
            Stmt::Let {
                mutable,
                name,
                ty,
                ty_missing,
                init,
                span,
            } => {
                self.mark(*span);
                if *ty_missing {
                    return self.fail("`let` without a type is not part of the language");
                }
                kv.push(("kind", s("let")));
                if *mutable {
                    kv.push(("mutable", Value::Bool(true)));
                }
                kv.push(("name", s(&name.name)));
                kv.push(("ty", self.under("ty", |e| e.ty(ty))?));
                kv.push(("init", self.under("init", |e| e.expr(init))?));
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                self.mark(*span);
                kv.push(("kind", s("assign")));
                kv.push(("target", self.under("target", |e| e.expr(target))?));
                kv.push(("value", self.under("value", |e| e.expr(value))?));
            }
            Stmt::While {
                label,
                cond,
                invariants,
                decreases,
                body,
                span,
            } => {
                self.mark(*span);
                kv.push(("kind", s("while")));
                Ex::label(&mut kv, label);
                kv.push(("cond", self.under("cond", |e| e.expr(cond))?));
                self.exprs("invariants", invariants, &mut kv)?;
                self.opt_expr("decreases", decreases.as_ref(), &mut kv)?;
                kv.push(("body", self.under("body", |e| e.block(body))?));
            }
            Stmt::For {
                label,
                var,
                lo,
                hi,
                inclusive,
                invariants,
                body,
                span,
            } => {
                self.mark(*span);
                kv.push(("kind", s("for")));
                Ex::label(&mut kv, label);
                kv.push(("var", s(&var.name)));
                kv.push(("lo", self.under("lo", |e| e.expr(lo))?));
                kv.push(("hi", self.under("hi", |e| e.expr(hi))?));
                if *inclusive {
                    kv.push(("inclusive", Value::Bool(true)));
                }
                self.exprs("invariants", invariants, &mut kv)?;
                kv.push(("body", self.under("body", |e| e.block(body))?));
            }
            Stmt::Assert { cond, by, span } => {
                self.mark(*span);
                kv.push(("kind", s("assert")));
                kv.push(("cond", self.under("cond", |e| e.expr(cond))?));
                if let Some(p) = by {
                    kv.push(("by", s(p.name())));
                }
            }
            Stmt::Proof { body, span } => {
                self.mark(*span);
                kv.push(("kind", s("proof")));
                kv.push(("body", self.under("body", |e| e.block(body))?));
            }
            Stmt::LetFields { span, .. } => {
                self.mark(*span);
                return self.fail("destructuring `let` is not part of the language");
            }
            Stmt::Expr(x) => {
                self.mark(x.span);
                kv.push(("kind", s("expr")));
                kv.push(("expr", self.under("expr", |e| e.expr(x))?));
            }
        }
        Ok(obj(kv))
    }

    fn expr(&mut self, x: &Expr) -> R {
        self.mark(x.span);
        let mut kv: Vec<(&str, Value)> = Vec::new();
        let mut k = |name: &str| kv.push(("kind", s(name)));
        match &x.kind {
            ExprKind::Int(v, r) => {
                k("int");
                kv.push(("value", Value::Num(v.to_string())));
                let radix = match r {
                    Radix::Dec => None,
                    Radix::Hex => Some("hex"),
                    Radix::Oct => Some("oct"),
                    Radix::Bin => Some("bin"),
                };
                if let Some(r) = radix {
                    kv.push(("radix", s(r)));
                }
            }
            ExprKind::Float(b) => {
                k("float");
                kv.push(("value", Value::Num(crate::lexer::float_text(*b))));
            }
            ExprKind::ByteLit(b) => {
                k("byte");
                kv.push(("value", Value::Num(b.to_string())));
            }
            ExprKind::ByteStr(bs) => {
                k("bytes");
                kv.push((
                    "value",
                    Value::Array(bs.iter().map(|b| Value::Num(b.to_string())).collect()),
                ));
            }
            ExprKind::Bool(b) => {
                k("bool");
                kv.push(("value", Value::Bool(*b)));
            }
            ExprKind::Unit => k("unit"),
            ExprKind::Var(i) => {
                k("var");
                kv.push(("name", s(&i.name)));
            }
            ExprKind::Call { path, targs, args } => {
                k("call");
                kv.push(("path", s(&path_text(path))));
                if !targs.is_empty() {
                    let v = self.list("targs", targs, |e, t| {
                        Ok(obj(vec![
                            ("seg", Value::Num(t.seg.to_string())),
                            ("args", e.list("args", &t.args, Ex::ty)?),
                        ]))
                    })?;
                    kv.push(("targs", v));
                }
                self.exprs("args", args, &mut kv)?;
            }
            ExprKind::Method { recv, name, args } => {
                k("method");
                kv.push(("recv", self.under("recv", |e| e.expr(recv))?));
                kv.push(("name", s(&name.name)));
                self.exprs("args", args, &mut kv)?;
            }
            ExprKind::Index { base, index } => {
                k("index");
                kv.push(("base", self.under("base", |e| e.expr(base))?));
                kv.push(("index", self.under("index", |e| e.expr(index))?));
            }
            ExprKind::Unary(op, e) => {
                k("unary");
                kv.push(("op", s(if *op == UnOp::Neg { "-" } else { "!" })));
                kv.push(("expr", self.under("expr", |x| x.expr(e))?));
            }
            ExprKind::Binary(op, l, r) => {
                k("binary");
                kv.push(("op", s(bin_name(*op))));
                kv.push(("lhs", self.under("lhs", |e| e.expr(l))?));
                kv.push(("rhs", self.under("rhs", |e| e.expr(r))?));
            }
            ExprKind::Ref { mutable, expr } => {
                k("ref");
                if *mutable {
                    kv.push(("mutable", Value::Bool(true)));
                }
                kv.push(("expr", self.under("expr", |e| e.expr(expr))?));
            }
            ExprKind::If { cond, then, els } => {
                k("if");
                kv.push(("cond", self.under("cond", |e| e.expr(cond))?));
                kv.push(("then", self.under("then", |e| e.block(then))?));
                self.opt_expr("else", els.as_deref(), &mut kv)?;
            }
            ExprKind::Block(b) => {
                k("block");
                kv.extend(self.block_fields(b)?);
            }
            ExprKind::Array(items) => {
                k("array");
                self.exprs("elems", items, &mut kv)?;
            }
            ExprKind::Repeat { value, count, .. } => {
                k("repeat");
                kv.push(("value", self.under("value", |e| e.expr(value))?));
                kv.push(("count", Value::Num(count.to_string())));
            }
            ExprKind::Old(e) => {
                k("old");
                kv.push(("expr", self.under("expr", |x| x.expr(e))?));
            }
            ExprKind::Forall { binders, body } => {
                k("forall");
                kv.push(("binders", self.list("binders", binders, Ex::param)?));
                kv.push(("body", self.under("body", |e| e.expr(body))?));
            }
            ExprKind::Trigger(e) => {
                k("trigger");
                kv.push(("expr", self.under("expr", |x| x.expr(e))?));
            }
            ExprKind::StructLit { path, fields, rest } => {
                if rest.is_some() {
                    return self.fail("`..base` is not part of the language");
                }
                k("struct");
                kv.push(("path", s(&path_text(path))));
                let mut out: Vec<(String, Value)> = Vec::new();
                self.under("fields", |e| {
                    for (n, v) in fields {
                        if out.iter().any(|(j, _)| *j == n.name) {
                            return e.fail(&format!("field `{}` given twice", n.name));
                        }
                        let x = e.under(&n.name, |e| e.expr(v))?;
                        out.push((n.name.clone(), x));
                    }
                    Ok(())
                })?;
                kv.push(("fields", Value::Object(out)));
            }
            ExprKind::Path(p) => {
                k("path");
                kv.push(("path", s(&path_text(p))));
            }
            ExprKind::Deref(e) => {
                k("deref");
                kv.push(("expr", self.under("expr", |x| x.expr(e))?));
            }
            ExprKind::Field { base, name } => {
                k("field");
                kv.push(("base", self.under("base", |e| e.expr(base))?));
                kv.push(("name", s(&name.name)));
            }
            ExprKind::Match { scrut, arms } => {
                k("match");
                kv.push(("scrut", self.under("scrut", |e| e.expr(scrut))?));
                let v = self.list("arms", arms, |e, a| {
                    e.mark(a.span);
                    if a.pat.origin != PatOrigin::Written {
                        return e
                            .fail("this arm is not part of the language (one arm per variant)");
                    }
                    let pat = e.under("pat", |e| {
                        e.mark(a.pat.span);
                        let mut kv = vec![("path", s(&path_text(&a.pat.path)))];
                        if !a.pat.binds.is_empty() {
                            let bs = a
                                .pat
                                .binds
                                .iter()
                                .map(|b| s(b.as_ref().map_or("_", |i| i.name.as_str())))
                                .collect();
                            kv.push(("binds", Value::Array(bs)));
                        }
                        Ok(obj(kv))
                    })?;
                    Ok(obj(vec![
                        ("pat", pat),
                        ("body", e.under("body", |e| e.expr(&a.body))?),
                    ]))
                })?;
                kv.push(("arms", v));
            }
            ExprKind::Return(v) => {
                k("return");
                self.opt_expr("value", v.as_deref(), &mut kv)?;
            }
            ExprKind::Try(e) => {
                k("try");
                kv.push(("expr", self.under("expr", |x| x.expr(e))?));
            }
            ExprKind::Loop {
                label,
                invariants,
                decreases,
                body,
            } => {
                k("loop");
                Ex::label(&mut kv, label);
                self.exprs("invariants", invariants, &mut kv)?;
                self.opt_expr("decreases", decreases.as_deref(), &mut kv)?;
                kv.push(("body", self.under("body", |e| e.block(body))?));
            }
            ExprKind::Break { label, value } => {
                k("break");
                Ex::label(&mut kv, label);
                self.opt_expr("value", value.as_deref(), &mut kv)?;
            }
            ExprKind::Continue { label } => {
                k("continue");
                Ex::label(&mut kv, label);
            }
            ExprKind::Error => return self.fail("not an expression of the language"),
        }
        Ok(obj(kv))
    }
}

// ---------- import ----------

/// The program a JSON document describes. Spans are empty: the caller
/// prints the program in normal form and parses it back (ADR-0073), so
/// positions and diagnostics come from that text.
pub fn import(doc: &Value) -> Result<Program, String> {
    let mut im = Im { at: Vec::new() };
    Ok(Program {
        mods: im.units(doc)?,
    })
}

struct Im {
    at: Vec<String>,
}

/// An object's fields, checked against the names a node may have.
struct Obj<'v> {
    kv: &'v [(String, Value)],
}

impl<'v> Obj<'v> {
    fn get(&self, k: &str) -> Option<&'v Value> {
        self.kv.iter().find(|(j, _)| j == k).map(|(_, v)| v)
    }
}

fn sp() -> Span {
    Span::default()
}

fn id(name: &str) -> Ident {
    Ident {
        name: name.to_string(),
        span: sp(),
    }
}

/// A name as the parser takes it: ASCII letters, digits and `_`, not
/// starting with a digit. (Keywords are caught when the printed program is
/// parsed back.)
fn is_name(x: &str) -> bool {
    let mut cs = x.chars();
    cs.next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && cs.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Im {
    fn ptr(&self) -> String {
        self.at.iter().map(|s| format!("/{}", escape(s))).collect()
    }

    fn fail<T>(&self, what: &str) -> Result<T, String> {
        let p = self.ptr();
        Err(format!("{}: {what}", if p.is_empty() { "/" } else { &p }))
    }

    fn under<T>(
        &mut self,
        seg: &str,
        f: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        self.at.push(seg.to_string());
        let r = f(self);
        self.at.pop();
        r
    }

    /// `v` as an object with only the fields in `allowed`.
    fn obj<'v>(&self, v: &'v Value, what: &str, allowed: &[&str]) -> Result<Obj<'v>, String> {
        let Value::Object(kv) = v else {
            return self.fail(&format!("expected {what} (an object)"));
        };
        for (k, _) in kv {
            if !allowed.contains(&k.as_str()) {
                return self.fail(&format!(
                    "{what} has no field `{k}` (fields: {})",
                    allowed.join(", ")
                ));
            }
        }
        let mut seen = BTreeSet::new();
        for (k, _) in kv {
            if !seen.insert(k.as_str()) {
                return self.fail(&format!("field `{k}` given twice"));
            }
        }
        Ok(Obj { kv })
    }

    fn req<'v>(&self, o: &Obj<'v>, k: &str) -> Result<&'v Value, String> {
        o.get(k)
            .map_or_else(|| self.fail(&format!("missing field `{k}`")), Ok)
    }

    fn string(&mut self, o: &Obj, k: &str) -> Result<String, String> {
        let v = self.req(o, k)?;
        self.under(k, |m| match v {
            Value::Str(x) => Ok(x.clone()),
            _ => m.fail("expected a string"),
        })
    }

    fn name(&mut self, o: &Obj, k: &str) -> Result<Ident, String> {
        let n = self.string(o, k)?;
        self.check_name(k, &n)?;
        Ok(id(&n))
    }

    fn check_name(&mut self, k: &str, n: &str) -> Result<(), String> {
        if is_name(n) {
            Ok(())
        } else {
            self.under(k, |m| m.fail(&format!("`{n}` is not a name")))
        }
    }

    fn path(&mut self, o: &Obj, k: &str) -> Result<Vec<Ident>, String> {
        let p = self.string(o, k)?;
        self.path_of(k, &p)
    }

    fn path_of(&mut self, k: &str, p: &str) -> Result<Vec<Ident>, String> {
        let segs: Vec<&str> = p.split("::").collect();
        if segs.iter().all(|x| is_name(x)) {
            Ok(segs.into_iter().map(id).collect())
        } else {
            self.under(k, |m| {
                m.fail(&format!("`{p}` is not a path (names joined by `::`)"))
            })
        }
    }

    fn flag(&mut self, o: &Obj, k: &str) -> Result<bool, String> {
        match o.get(k) {
            None => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => self.under(k, |m| m.fail("expected true or false")),
        }
    }

    fn number(&mut self, o: &Obj, k: &str) -> Result<u128, String> {
        let v = self.req(o, k)?;
        self.under(k, |m| match v {
            Value::Num(n) if !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => n
                .parse::<u128>()
                .map_or_else(|_| m.fail("number out of range"), Ok),
            _ => m.fail("expected a non-negative integer"),
        })
    }

    fn array<'v>(&mut self, o: &Obj<'v>, k: &str) -> Result<&'v [Value], String> {
        match o.get(k) {
            None => Ok(&[]),
            Some(Value::Array(xs)) => Ok(xs),
            Some(_) => self.under(k, |m| m.fail("expected an array")),
        }
    }

    fn keyed<'v>(&mut self, o: &Obj<'v>, k: &str) -> Result<&'v [(String, Value)], String> {
        match o.get(k) {
            None => Ok(&[]),
            Some(Value::Object(kv)) => {
                let mut seen = BTreeSet::new();
                for (n, _) in kv {
                    if !seen.insert(n.as_str()) {
                        return self.under(k, |m| m.fail(&format!("`{n}` given twice")));
                    }
                    if !is_name(n) {
                        return self.under(k, |m| m.fail(&format!("`{n}` is not a name")));
                    }
                }
                Ok(kv)
            }
            Some(_) => self.under(k, |m| m.fail("expected an object keyed by name")),
        }
    }

    fn list<T>(
        &mut self,
        o: &Obj,
        k: &str,
        f: impl Fn(&mut Self, &Value) -> Result<T, String>,
    ) -> Result<Vec<T>, String> {
        let xs = self.array(o, k)?;
        self.under(k, |m| {
            let mut out = Vec::new();
            for (i, x) in xs.iter().enumerate() {
                out.push(m.under(&i.to_string(), |m| f(m, x))?);
            }
            Ok(out)
        })
    }

    fn opt<T>(
        &mut self,
        o: &Obj,
        k: &str,
        f: impl FnOnce(&mut Self, &Value) -> Result<T, String>,
    ) -> Result<Option<T>, String> {
        match o.get(k) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => self.under(k, |m| f(m, v)).map(Some),
        }
    }

    fn field<T>(
        &mut self,
        o: &Obj,
        k: &str,
        f: impl FnOnce(&mut Self, &Value) -> Result<T, String>,
    ) -> Result<T, String> {
        let v = self.req(o, k)?;
        self.under(k, |m| f(m, v))
    }

    fn kind<'v>(&self, v: &'v Value, what: &str) -> Result<&'v str, String> {
        match v.get("kind") {
            Some(Value::Str(k)) => Ok(k),
            _ => self.fail(&format!("expected {what} (an object with a `kind`)")),
        }
    }

    // ---------- items ----------

    fn units(&mut self, v: &Value) -> Result<Vec<Mod>, String> {
        let Value::Object(kv) = v else {
            return self.fail("expected an object keyed by unit name");
        };
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for (name, u) in kv {
            if !seen.insert(name.as_str()) {
                return self.fail(&format!("unit `{name}` given twice"));
            }
            if !is_name(name) {
                return self.fail(&format!("`{name}` is not a unit name"));
            }
            out.push(self.under(name, |m| m.unit(name, u))?);
        }
        Ok(out)
    }

    fn unit(&mut self, name: &str, v: &Value) -> Result<Mod, String> {
        let o = self.obj(
            v,
            "a unit",
            &[
                "summary",
                "units",
                "traits",
                "types",
                "impls",
                "trait_impls",
                "fns",
            ],
        )?;
        let mut attrs = Vec::new();
        if o.get("summary").is_some() {
            let summary = self.string(&o, "summary")?;
            attrs.push(Attr {
                name: id("summary"),
                args: vec![AttrArg::Str(summary, sp())],
                span: sp(),
            });
        }
        let mods = match o.get("units") {
            None => Vec::new(),
            Some(u) => self.under("units", |m| m.units(u))?,
        };
        let mut traits = Vec::new();
        for (n, t) in self.keyed(&o, "traits")? {
            traits.push(self.under("traits", |m| m.under(n, |m| m.trait_def(n, t)))?);
        }
        let mut types = Vec::new();
        for (n, t) in self.keyed(&o, "types")? {
            types.push(self.under("types", |m| m.under(n, |m| m.type_def(n, t)))?);
        }
        let mut impls = Vec::new();
        for (n, i) in self.keyed(&o, "impls")? {
            impls.push(self.under("impls", |m| m.under(n, |m| m.impl_block(n, i)))?);
        }
        let trait_impls = self.list(&o, "trait_impls", Im::trait_impl)?;
        let fns = self
            .fns(&o, "fns", false)?
            .into_iter()
            .map(|(f, _)| f)
            .collect();
        Ok(Mod {
            name: id(name),
            attrs,
            fns,
            mods,
            types,
            impls,
            traits,
            trait_impls,
            uses: Vec::new(),
            span: sp(),
        })
    }

    /// Functions keyed by name; `bodiless` allows a trait method without a
    /// body (the flag says which have one).
    fn fns(&mut self, o: &Obj, k: &str, bodiless: bool) -> Result<Vec<(FnDef, bool)>, String> {
        let mut out = Vec::new();
        for (n, f) in self.keyed(o, k)? {
            out.push(self.under(k, |m| m.under(n, |m| m.function(n, f, bodiless)))?);
        }
        Ok(out)
    }

    fn attrs(&mut self, o: &Obj) -> Result<Vec<Attr>, String> {
        self.list(o, "attrs", |m, v| {
            let a = m.obj(v, "an attribute", &["name", "args"])?;
            let name = m.name(&a, "name")?;
            let args = m.list(&a, "args", |m, x| match x {
                Value::Str(t) => {
                    m.path_of("", t)?;
                    Ok(AttrArg::Ident(id(t)))
                }
                Value::Object(_) => {
                    let s = m.obj(x, "a string argument", &["str"])?;
                    Ok(AttrArg::Str(m.string(&s, "str")?, sp()))
                }
                _ => m.fail("expected a name or {\"str\": ..}"),
            })?;
            Ok(Attr {
                name,
                args,
                span: sp(),
            })
        })
    }

    fn generics(&mut self, o: &Obj) -> Result<Vec<TypeParam>, String> {
        self.list(o, "generics", |m, v| {
            let g = m.obj(v, "a type parameter", &["name", "bounds"])?;
            let name = m.name(&g, "name")?;
            let bounds = m.list(&g, "bounds", |m, b| match b {
                Value::Str(p) => m.path_of("", p),
                _ => m.fail("expected a trait path"),
            })?;
            Ok(TypeParam {
                name,
                bounds,
                span: sp(),
            })
        })
    }

    fn type_def(&mut self, name: &str, v: &Value) -> Result<TypeDef, String> {
        let kind = self.kind(v, "a struct or enum")?;
        let (o, kind) = match kind {
            "struct" => {
                let o = self.obj(v, "a struct", &["kind", "generics", "attrs", "fields"])?;
                let mut fs = Vec::new();
                for (n, t) in self.keyed(&o, "fields")? {
                    fs.push(Field {
                        name: id(n),
                        ty: self.under("fields", |m| m.under(n, |m| m.ty(t)))?,
                    });
                }
                (o, TypeDefKind::Struct(fs))
            }
            "enum" => {
                let o = self.obj(v, "an enum", &["kind", "generics", "attrs", "variants"])?;
                let mut vs = Vec::new();
                for (n, p) in self.keyed(&o, "variants")? {
                    let payload = self.under("variants", |m| {
                        m.under(n, |m| match p {
                            Value::Array(xs) => {
                                let mut out = Vec::new();
                                for (i, x) in xs.iter().enumerate() {
                                    out.push(m.under(&i.to_string(), |m| m.ty(x))?);
                                }
                                Ok(out)
                            }
                            _ => m.fail(
                                "expected the payload types (an array, empty for a unit variant)",
                            ),
                        })
                    })?;
                    vs.push(Variant {
                        name: id(n),
                        payload,
                        span: sp(),
                    });
                }
                (o, TypeDefKind::Enum(vs))
            }
            other => return self.fail(&format!("unknown type kind `{other}` (struct, enum)")),
        };
        Ok(TypeDef {
            name: id(name),
            generics: self.generics(&o)?,
            attrs: self.attrs(&o)?,
            kind,
            span: sp(),
        })
    }

    fn impl_block(&mut self, ty: &str, v: &Value) -> Result<Impl, String> {
        let o = self.obj(v, "an impl", &["generics", "ty_args", "fns"])?;
        Ok(Impl {
            generics: self.generics(&o)?,
            ty: id(ty),
            ty_args: self.list(&o, "ty_args", Im::ty)?,
            fns: self
                .fns(&o, "fns", false)?
                .into_iter()
                .map(|(f, _)| f)
                .collect(),
            span: sp(),
        })
    }

    fn trait_def(&mut self, name: &str, v: &Value) -> Result<TraitDef, String> {
        let o = self.obj(v, "a trait", &["attrs", "fns"])?;
        Ok(TraitDef {
            name: id(name),
            attrs: self.attrs(&o)?,
            fns: self
                .fns(&o, "fns", true)?
                .into_iter()
                .map(|(f, has_body)| TraitFn { f, has_body })
                .collect(),
            span: sp(),
        })
    }

    fn trait_impl(&mut self, v: &Value) -> Result<TraitImpl, String> {
        let o = self.obj(v, "a trait impl", &["generics", "trait", "self_ty", "fns"])?;
        Ok(TraitImpl {
            generics: self.generics(&o)?,
            trait_path: self.path(&o, "trait")?,
            self_ty: self.field(&o, "self_ty", Im::ty)?,
            fns: self
                .fns(&o, "fns", false)?
                .into_iter()
                .map(|(f, _)| f)
                .collect(),
            span: sp(),
        })
    }

    fn function(&mut self, name: &str, v: &Value, bodiless: bool) -> Result<(FnDef, bool), String> {
        let o = self.obj(
            v,
            "a function",
            &[
                "kind",
                "effects",
                "uses",
                "generics",
                "params",
                "ret",
                "requires",
                "ensures",
                "decreases",
                "body",
            ],
        )?;
        let kind = match self.string(&o, "kind")?.as_str() {
            "fn" => FnKind::Exec,
            "spec fn" => FnKind::Spec,
            "proof fn" => FnKind::Proof,
            other => {
                return self.under("kind", |m| {
                    m.fail(&format!(
                        "unknown function kind `{other}` (fn, spec fn, proof fn)"
                    ))
                });
            }
        };
        let mut attrs = Vec::new();
        for (attr, field) in [("effects", "effects"), ("uses", "uses")] {
            if o.get(field).is_none() {
                continue;
            }
            let args = self.list(&o, field, |m, x| match x {
                Value::Str(t) => {
                    m.path_of("", t)?;
                    Ok(AttrArg::Ident(id(t)))
                }
                _ => m.fail("expected a name"),
            })?;
            attrs.push(Attr {
                name: id(attr),
                args,
                span: sp(),
            });
        }
        let generics = self.generics(&o)?;
        let params = self.list(&o, "params", Im::param)?;
        let ret = self.field(&o, "ret", |m, r| {
            let r = m.obj(r, "a return type", &["name", "ty"])?;
            let name = match r.get("name") {
                None => None,
                Some(_) => Some(m.name(&r, "name")?),
            };
            Ok(Ret {
                name,
                ty: m.field(&r, "ty", Im::ty)?,
            })
        })?;
        let requires = self.list(&o, "requires", Im::expr)?;
        let ensures = self.list(&o, "ensures", Im::expr)?;
        let decreases = self.opt(&o, "decreases", Im::expr)?;
        let (body, has_body) = match o.get("body") {
            Some(_) => (self.field(&o, "body", Im::block)?, true),
            None if bodiless => (
                Block {
                    stmts: Vec::new(),
                    tail: None,
                    span: sp(),
                },
                false,
            ),
            None => return self.fail("missing field `body`"),
        };
        Ok((
            FnDef {
                kind,
                name: id(name),
                generics,
                attrs,
                params,
                ret,
                requires,
                ensures,
                decreases,
                body,
                span: sp(),
            },
            has_body,
        ))
    }

    fn param(&mut self, v: &Value) -> Result<Param, String> {
        let o = self.obj(v, "a parameter", &["name", "ty"])?;
        Ok(Param {
            name: self.name(&o, "name")?,
            ty: self.field(&o, "ty", Im::ty)?,
        })
    }

    fn ty(&mut self, v: &Value) -> Result<Type, String> {
        let k = self.kind(v, "a type")?;
        let kind = if let Some(i) = IntTy::from_name(k) {
            self.obj(v, "a type", &["kind"])?;
            TypeKind::Int(i)
        } else {
            match k {
                "f64" | "bool" | "unit" => {
                    self.obj(v, "a type", &["kind"])?;
                    match k {
                        "f64" => TypeKind::F64,
                        "bool" => TypeKind::Bool,
                        _ => TypeKind::Unit,
                    }
                }
                "array" => {
                    let o = self.obj(v, "an array type", &["kind", "elem", "len"])?;
                    let elem = self.field(&o, "elem", Im::ty)?;
                    let len = self.number(&o, "len")?;
                    let len = u64::try_from(len)
                        .map_or_else(|_| self.under("len", |m| m.fail("too large")), Ok)?;
                    TypeKind::Array(Box::new(elem), len)
                }
                "slice" | "Vec" | "Box" => {
                    let o = self.obj(v, "a type", &["kind", "elem"])?;
                    let elem = Box::new(self.field(&o, "elem", Im::ty)?);
                    match k {
                        "slice" => TypeKind::Slice(elem),
                        "Vec" => TypeKind::Vec(elem),
                        _ => TypeKind::Box(elem),
                    }
                }
                "ref" => {
                    let o = self.obj(v, "a reference type", &["kind", "mutable", "inner"])?;
                    TypeKind::Ref {
                        mutable: self.flag(&o, "mutable")?,
                        inner: Box::new(self.field(&o, "inner", Im::ty)?),
                    }
                }
                "named" => {
                    let o = self.obj(v, "a named type", &["kind", "path", "args"])?;
                    TypeKind::Named {
                        path: self.path(&o, "path")?.into_iter().map(|i| i.name).collect(),
                        args: self.list(&o, "args", Im::ty)?,
                    }
                }
                other => {
                    return self.fail(&format!(
                        "unknown type kind `{other}` (i8 .. usize, f64, bool, unit, array, slice, ref, named, Vec, Box)"
                    ));
                }
            }
        };
        Ok(Type { kind, span: sp() })
    }

    // ---------- blocks, statements, expressions ----------

    fn block(&mut self, v: &Value) -> Result<Block, String> {
        // A block where one is expected may also say `"kind": "block"`.
        let o = self.obj(v, "a block", &["kind", "stmts", "tail"])?;
        if let Some(k) = o.get("kind")
            && k.as_str() != Some("block")
        {
            return self.fail("expected a block");
        }
        self.block_of(&o)
    }

    fn block_of(&mut self, o: &Obj) -> Result<Block, String> {
        Ok(Block {
            stmts: self.list(o, "stmts", Im::stmt)?,
            tail: self.opt(o, "tail", Im::expr)?.map(Box::new),
            span: sp(),
        })
    }

    fn label(&mut self, o: &Obj) -> Result<Option<Ident>, String> {
        match o.get("label") {
            None => Ok(None),
            Some(_) => Ok(Some(self.name(o, "label")?)),
        }
    }

    fn stmt(&mut self, v: &Value) -> Result<Stmt, String> {
        let k = self.kind(v, "a statement")?;
        Ok(match k {
            "let" => {
                let o = self.obj(v, "a `let`", &["kind", "mutable", "name", "ty", "init"])?;
                Stmt::Let {
                    mutable: self.flag(&o, "mutable")?,
                    name: self.name(&o, "name")?,
                    ty: self.field(&o, "ty", Im::ty)?,
                    ty_missing: false,
                    init: self.field(&o, "init", Im::expr)?,
                    span: sp(),
                }
            }
            "assign" => {
                let o = self.obj(v, "an assignment", &["kind", "target", "value"])?;
                Stmt::Assign {
                    target: self.field(&o, "target", Im::expr)?,
                    value: self.field(&o, "value", Im::expr)?,
                    span: sp(),
                }
            }
            "while" => {
                let o = self.obj(
                    v,
                    "a `while`",
                    &["kind", "label", "cond", "invariants", "decreases", "body"],
                )?;
                Stmt::While {
                    label: self.label(&o)?,
                    cond: self.field(&o, "cond", Im::expr)?,
                    invariants: self.list(&o, "invariants", Im::expr)?,
                    decreases: self.opt(&o, "decreases", Im::expr)?,
                    body: self.field(&o, "body", Im::block)?,
                    span: sp(),
                }
            }
            "for" => {
                let o = self.obj(
                    v,
                    "a `for`",
                    &[
                        "kind",
                        "label",
                        "var",
                        "lo",
                        "hi",
                        "inclusive",
                        "invariants",
                        "body",
                    ],
                )?;
                Stmt::For {
                    label: self.label(&o)?,
                    var: self.name(&o, "var")?,
                    lo: self.field(&o, "lo", Im::expr)?,
                    hi: self.field(&o, "hi", Im::expr)?,
                    inclusive: self.flag(&o, "inclusive")?,
                    invariants: self.list(&o, "invariants", Im::expr)?,
                    body: self.field(&o, "body", Im::block)?,
                    span: sp(),
                }
            }
            "assert" => {
                let o = self.obj(v, "an `assert`", &["kind", "cond", "by"])?;
                let by = match o.get("by") {
                    None => None,
                    Some(_) => {
                        let b = self.string(&o, "by")?;
                        match Prover::from_name(&b) {
                            Some(p) => Some(p),
                            None => {
                                return self.under("by", |m| {
                                    m.fail(&format!("unknown prover `{b}` (bit_vector)"))
                                });
                            }
                        }
                    }
                };
                Stmt::Assert {
                    cond: self.field(&o, "cond", Im::expr)?,
                    by,
                    span: sp(),
                }
            }
            "proof" => {
                let o = self.obj(v, "a `proof` block", &["kind", "body"])?;
                Stmt::Proof {
                    body: self.field(&o, "body", Im::block)?,
                    span: sp(),
                }
            }
            "expr" => {
                let o = self.obj(v, "an expression statement", &["kind", "expr"])?;
                Stmt::Expr(self.field(&o, "expr", Im::expr)?)
            }
            other => {
                return self.fail(&format!(
                    "unknown statement kind `{other}` (let, assign, while, for, assert, proof, expr)"
                ));
            }
        })
    }

    fn exprs(&mut self, o: &Obj, k: &str) -> Result<Vec<Expr>, String> {
        self.list(o, k, Im::expr)
    }

    fn sub(&mut self, o: &Obj, k: &str) -> Result<Box<Expr>, String> {
        self.field(o, k, Im::expr).map(Box::new)
    }

    fn expr(&mut self, v: &Value) -> Result<Expr, String> {
        let k = self.kind(v, "an expression")?;
        let fields = |m: &Im, what: &str, extra: &[&str]| {
            let mut all = vec!["kind"];
            all.extend_from_slice(extra);
            m.obj(v, what, &all)
        };
        let kind = match k {
            "int" => {
                let o = fields(self, "an integer", &["value", "radix"])?;
                let radix = match o.get("radix").map(|r| r.as_str()) {
                    None => Radix::Dec,
                    Some(Some("dec")) => Radix::Dec,
                    Some(Some("hex")) => Radix::Hex,
                    Some(Some("oct")) => Radix::Oct,
                    Some(Some("bin")) => Radix::Bin,
                    Some(_) => {
                        return self.under("radix", |m| m.fail("expected dec, hex, oct or bin"));
                    }
                };
                ExprKind::Int(self.number(&o, "value")?, radix)
            }
            "float" => {
                let o = fields(self, "a float", &["value"])?;
                let bits = self.field(&o, "value", |m, x| match x {
                    Value::Num(n) => match n.parse::<f64>() {
                        Ok(f) if f.is_finite() && f >= 0.0 && !n.starts_with('-') => {
                            Ok(f.to_bits())
                        }
                        _ => m.fail("expected a finite non-negative number (negate with `unary`)"),
                    },
                    _ => m.fail("expected a number"),
                })?;
                ExprKind::Float(bits)
            }
            "byte" => {
                let o = fields(self, "a byte", &["value"])?;
                let b = self.number(&o, "value")?;
                ExprKind::ByteLit(
                    u8::try_from(b).map_or_else(|_| self.fail("a byte is 0..=255"), Ok)?,
                )
            }
            "bytes" => {
                let o = fields(self, "a byte string", &["value"])?;
                let bs = self.list(&o, "value", |m, x| match x {
                    Value::Num(n) => n
                        .parse::<u8>()
                        .map_or_else(|_| m.fail("a byte is 0..=255"), Ok),
                    _ => m.fail("expected a byte (0..=255)"),
                })?;
                ExprKind::ByteStr(bs)
            }
            "bool" => {
                let o = fields(self, "a bool", &["value"])?;
                match self.req(&o, "value")? {
                    Value::Bool(b) => ExprKind::Bool(*b),
                    _ => return self.under("value", |m| m.fail("expected true or false")),
                }
            }
            "unit" => {
                fields(self, "`()`", &[])?;
                ExprKind::Unit
            }
            "var" => {
                let o = fields(self, "a variable", &["name"])?;
                ExprKind::Var(self.name(&o, "name")?)
            }
            "call" => {
                let o = fields(self, "a call", &["path", "targs", "args"])?;
                let path = self.path(&o, "path")?;
                let n = path.len();
                let targs = self.list(&o, "targs", |m, t| {
                    let t = m.obj(t, "type arguments", &["seg", "args"])?;
                    let seg = m.number(&t, "seg")?;
                    if seg >= n as u128 {
                        return m.under("seg", |m| m.fail("no such segment in the path"));
                    }
                    Ok(TArgs {
                        seg: seg as usize,
                        args: m.list(&t, "args", Im::ty)?,
                        span: sp(),
                    })
                })?;
                ExprKind::Call {
                    path,
                    targs,
                    args: self.exprs(&o, "args")?,
                }
            }
            "method" => {
                let o = fields(self, "a method call", &["recv", "name", "args"])?;
                ExprKind::Method {
                    recv: self.sub(&o, "recv")?,
                    name: self.name(&o, "name")?,
                    args: self.exprs(&o, "args")?,
                }
            }
            "index" => {
                let o = fields(self, "an index", &["base", "index"])?;
                ExprKind::Index {
                    base: self.sub(&o, "base")?,
                    index: self.sub(&o, "index")?,
                }
            }
            "unary" => {
                let o = fields(self, "a unary operation", &["op", "expr"])?;
                let op = match self.string(&o, "op")?.as_str() {
                    "-" => UnOp::Neg,
                    "!" => UnOp::Not,
                    other => {
                        return self.under("op", |m| {
                            m.fail(&format!("unknown operator `{other}` (-, !)"))
                        });
                    }
                };
                ExprKind::Unary(op, self.sub(&o, "expr")?)
            }
            "binary" => {
                let o = fields(self, "a binary operation", &["op", "lhs", "rhs"])?;
                let s = self.string(&o, "op")?;
                let Some(op) = bin_op(&s) else {
                    return self.under("op", |m| m.fail(&format!("unknown operator `{s}`")));
                };
                ExprKind::Binary(op, self.sub(&o, "lhs")?, self.sub(&o, "rhs")?)
            }
            "ref" => {
                let o = fields(self, "a reference", &["mutable", "expr"])?;
                ExprKind::Ref {
                    mutable: self.flag(&o, "mutable")?,
                    expr: self.sub(&o, "expr")?,
                }
            }
            "if" => {
                let o = fields(self, "an `if`", &["cond", "then", "else"])?;
                let els = self.opt(&o, "else", |m, x| {
                    let e = m.expr(x)?;
                    match e.kind {
                        ExprKind::If { .. } | ExprKind::Block(_) => Ok(Box::new(e)),
                        _ => m.fail("`else` is an `if` or a `block`"),
                    }
                })?;
                ExprKind::If {
                    cond: self.sub(&o, "cond")?,
                    then: self.field(&o, "then", Im::block)?,
                    els,
                }
            }
            "block" => {
                let o = fields(self, "a block", &["stmts", "tail"])?;
                ExprKind::Block(self.block_of(&o)?)
            }
            "array" => {
                let o = fields(self, "an array", &["elems"])?;
                ExprKind::Array(self.exprs(&o, "elems")?)
            }
            "repeat" => {
                let o = fields(self, "an array repeat", &["value", "count"])?;
                ExprKind::Repeat {
                    value: self.sub(&o, "value")?,
                    count: self.number(&o, "count")?,
                    count_span: sp(),
                }
            }
            "old" | "trigger" | "deref" | "try" => {
                let o = fields(self, "an expression", &["expr"])?;
                let e = self.sub(&o, "expr")?;
                match k {
                    "old" => ExprKind::Old(e),
                    "trigger" => ExprKind::Trigger(e),
                    "deref" => ExprKind::Deref(e),
                    _ => ExprKind::Try(e),
                }
            }
            "forall" => {
                let o = fields(self, "a `forall`", &["binders", "body"])?;
                ExprKind::Forall {
                    binders: self.list(&o, "binders", Im::param)?,
                    body: self.sub(&o, "body")?,
                }
            }
            "struct" => {
                let o = fields(self, "a struct literal", &["path", "fields"])?;
                let path = self.path(&o, "path")?;
                let mut fs = Vec::new();
                for (n, x) in self.keyed(&o, "fields")? {
                    fs.push((id(n), self.under("fields", |m| m.under(n, |m| m.expr(x)))?));
                }
                ExprKind::StructLit {
                    path,
                    fields: fs,
                    rest: None,
                }
            }
            "path" => {
                let o = fields(self, "a path", &["path"])?;
                ExprKind::Path(self.path(&o, "path")?)
            }
            "field" => {
                let o = fields(self, "a field access", &["base", "name"])?;
                ExprKind::Field {
                    base: self.sub(&o, "base")?,
                    name: self.name(&o, "name")?,
                }
            }
            "match" => {
                let o = fields(self, "a `match`", &["scrut", "arms"])?;
                let arms = self.list(&o, "arms", |m, a| {
                    let a = m.obj(a, "an arm", &["pat", "body"])?;
                    let pat = m.field(&a, "pat", |m, p| {
                        let p = m.obj(p, "a pattern", &["path", "binds"])?;
                        let path = m.path(&p, "path")?;
                        let binds = m.list(&p, "binds", |m, b| match b {
                            Value::Str(x) if x == "_" => Ok(None),
                            Value::Str(x) if is_name(x) => Ok(Some(id(x))),
                            _ => m.fail("expected a name or \"_\""),
                        })?;
                        Ok(Pat {
                            parens: !binds.is_empty(),
                            path,
                            binds,
                            origin: PatOrigin::Written,
                            span: sp(),
                        })
                    })?;
                    Ok(Arm {
                        pat,
                        body: m.field(&a, "body", Im::expr)?,
                        span: sp(),
                    })
                })?;
                ExprKind::Match {
                    scrut: self.sub(&o, "scrut")?,
                    arms,
                }
            }
            "return" => {
                let o = fields(self, "a `return`", &["value"])?;
                ExprKind::Return(self.opt(&o, "value", Im::expr)?.map(Box::new))
            }
            "loop" => {
                let o = fields(
                    self,
                    "a `loop`",
                    &["label", "invariants", "decreases", "body"],
                )?;
                ExprKind::Loop {
                    label: self.label(&o)?,
                    invariants: self.exprs(&o, "invariants")?,
                    decreases: self.opt(&o, "decreases", Im::expr)?.map(Box::new),
                    body: self.field(&o, "body", Im::block)?,
                }
            }
            "break" => {
                let o = fields(self, "a `break`", &["label", "value"])?;
                ExprKind::Break {
                    label: self.label(&o)?,
                    value: self.opt(&o, "value", Im::expr)?.map(Box::new),
                }
            }
            "continue" => {
                let o = fields(self, "a `continue`", &["label"])?;
                ExprKind::Continue {
                    label: self.label(&o)?,
                }
            }
            other => {
                return self.fail(&format!(
                    "unknown expression kind `{other}` (int, float, byte, bytes, bool, unit, var, call, method, index, unary, binary, ref, if, block, array, repeat, old, forall, trigger, struct, path, deref, field, match, return, try, loop, break, continue)"
                ));
            }
        };
        Ok(Expr { kind, span: sp() })
    }
}

/// The innermost node in `spans` whose range holds `[lo, hi)` in `file`:
/// the pointer a diagnostic at that range refers to.
pub fn node_at(spans: &Spans, file: u32, lo: u32, hi: u32) -> Option<&str> {
    spans
        .iter()
        .filter(|(s, _)| s.file == file && s.lo <= lo && hi <= s.hi && s.hi > s.lo)
        // The smallest range; among equal ranges the deepest node.
        .min_by_key(|(s, p)| (s.hi - s.lo, std::cmp::Reverse(p.len())))
        .map(|(_, p)| p.as_str())
}

// ---------- JSON Pointer and JSON Patch ----------

/// The segments of a JSON Pointer (RFC 6901): `""` is the whole document.
pub fn pointer(p: &str) -> Result<Vec<String>, String> {
    if p.is_empty() {
        return Ok(Vec::new());
    }
    let Some(rest) = p.strip_prefix('/') else {
        return Err(format!("`{p}` is not a JSON Pointer (it starts with `/`)"));
    };
    rest.split('/')
        .map(|seg| {
            let mut out = String::new();
            let mut cs = seg.chars();
            while let Some(c) = cs.next() {
                if c != '~' {
                    out.push(c);
                    continue;
                }
                match cs.next() {
                    Some('0') => out.push('~'),
                    Some('1') => out.push('/'),
                    _ => return Err(format!("`{p}`: `~` is written `~0` and `/` `~1`")),
                }
            }
            Ok(out)
        })
        .collect()
}

fn index(seg: &str, len: usize, allow_end: bool) -> Result<usize, String> {
    if allow_end && seg == "-" {
        return Ok(len);
    }
    let ok = !seg.is_empty()
        && seg.bytes().all(|b| b.is_ascii_digit())
        && (seg == "0" || !seg.starts_with('0'));
    match seg.parse::<usize>() {
        Ok(i) if ok && (i < len || (allow_end && i == len)) => Ok(i),
        _ => Err(format!(
            "array index `{seg}` is out of range (length {len})"
        )),
    }
}

/// The value at `segs`.
pub fn resolve<'v>(doc: &'v Value, segs: &[String]) -> Result<&'v Value, String> {
    let mut cur = doc;
    for (k, seg) in segs.iter().enumerate() {
        let here = || {
            segs[..=k]
                .iter()
                .map(|s| format!("/{}", escape(s)))
                .collect::<String>()
        };
        cur = match cur {
            Value::Object(kv) => match kv.iter().find(|(j, _)| j == seg) {
                Some((_, v)) => v,
                None => return Err(format!("{}: no such member", here())),
            },
            Value::Array(xs) => {
                &xs[index(seg, xs.len(), false).map_err(|e| format!("{}: {e}", here()))?]
            }
            _ => return Err(format!("{}: not inside an object or array", here())),
        };
    }
    Ok(cur)
}

fn resolve_mut<'v>(doc: &'v mut Value, segs: &[String]) -> Result<&'v mut Value, String> {
    let mut cur = doc;
    for (k, seg) in segs.iter().enumerate() {
        let here = segs[..=k]
            .iter()
            .map(|s| format!("/{}", escape(s)))
            .collect::<String>();
        cur = match cur {
            Value::Object(kv) => match kv.iter_mut().find(|(j, _)| j == seg) {
                Some((_, v)) => v,
                None => return Err(format!("{here}: no such member")),
            },
            Value::Array(xs) => {
                let i = index(seg, xs.len(), false).map_err(|e| format!("{here}: {e}"))?;
                &mut xs[i]
            }
            _ => return Err(format!("{here}: not inside an object or array")),
        };
    }
    Ok(cur)
}

fn add(doc: &mut Value, segs: &[String], v: Value) -> Result<(), String> {
    let Some((last, parent)) = segs.split_last() else {
        *doc = v;
        return Ok(());
    };
    match resolve_mut(doc, parent)? {
        Value::Object(kv) => {
            match kv.iter_mut().find(|(j, _)| j == last) {
                Some((_, x)) => *x = v,
                None => kv.push((last.clone(), v)),
            }
            Ok(())
        }
        Value::Array(xs) => {
            let i = index(last, xs.len(), true)?;
            xs.insert(i, v);
            Ok(())
        }
        _ => Err("the parent is not an object or array".into()),
    }
}

fn remove(doc: &mut Value, segs: &[String]) -> Result<Value, String> {
    let Some((last, parent)) = segs.split_last() else {
        return Err("cannot remove the whole document".into());
    };
    match resolve_mut(doc, parent)? {
        Value::Object(kv) => match kv.iter().position(|(j, _)| j == last) {
            Some(i) => Ok(kv.remove(i).1),
            None => Err("no such member".into()),
        },
        Value::Array(xs) => {
            let i = index(last, xs.len(), false)?;
            Ok(xs.remove(i))
        }
        _ => Err("the parent is not an object or array".into()),
    }
}

/// Applies a JSON Patch (RFC 6902: `add`, `remove`, `replace`, `move`,
/// `copy`, `test`) to `doc`, all or nothing: on an error `doc` is as it was
/// and the message names the operation.
pub fn apply_patch(doc: &mut Value, patch: &Value) -> Result<(), String> {
    let Value::Array(ops) = patch else {
        return Err("a JSON Patch is an array of operations".into());
    };
    let mut work = doc.clone();
    for (k, op) in ops.iter().enumerate() {
        apply_op(&mut work, op).map_err(|e| format!("operation {k}: {e}"))?;
    }
    *doc = work;
    Ok(())
}

fn apply_op(doc: &mut Value, op: &Value) -> Result<(), String> {
    let Value::Object(kv) = op else {
        return Err("expected an object {\"op\", \"path\", ..}".into());
    };
    let allowed: &[&str] = match op.get("op").and_then(Value::as_str) {
        Some("add" | "replace" | "test") => &["op", "path", "value"],
        Some("remove") => &["op", "path"],
        Some("move" | "copy") => &["op", "path", "from"],
        Some(other) => {
            return Err(format!(
                "unknown op `{other}` (add, remove, replace, move, copy, test)"
            ));
        }
        None => return Err("missing `op`".into()),
    };
    if let Some((k, _)) = kv.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        return Err(format!("unexpected field `{k}`"));
    }
    let name = op.get("op").and_then(Value::as_str).unwrap_or_default();
    let ptr = |k: &str| -> Result<Vec<String>, String> {
        match op.get(k) {
            Some(Value::Str(p)) => pointer(p),
            _ => Err(format!("missing `{k}` (a JSON Pointer string)")),
        }
    };
    let value = || {
        op.get("value")
            .cloned()
            .ok_or("missing `value`".to_string())
    };
    let path = ptr("path")?;
    let shown = op.get("path").and_then(Value::as_str).unwrap_or_default();
    let at = |e: String| format!("{name} `{shown}`: {e}");
    match name {
        "add" => add(doc, &path, value()?).map_err(at),
        "remove" => remove(doc, &path).map(|_| ()).map_err(at),
        "replace" => {
            let v = value()?;
            let slot = resolve_mut(doc, &path).map_err(at)?;
            *slot = v;
            Ok(())
        }
        "move" => {
            let from = ptr("from")?;
            if path.len() > from.len() && path[..from.len()] == from[..] {
                return Err(at("cannot move a value into itself".into()));
            }
            let v = remove(doc, &from).map_err(at)?;
            add(doc, &path, v).map_err(at)
        }
        "copy" => {
            let from = ptr("from")?;
            let v = resolve(doc, &from).map_err(at)?.clone();
            add(doc, &path, v).map_err(at)
        }
        _ => {
            let v = value()?;
            if *resolve(doc, &path).map_err(at)? == v {
                Ok(())
            } else {
                Err(at("the value differs".into()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn spans_name_the_innermost_node() {
        let text = "#[summary(\"d\")]\nmod d {\n    #[effects(pure)]\n    fn f(x: i64) -> i64 {\n        let y: i64 = x + 1;\n        y\n    }\n}\n";
        let (program, _, d) =
            crate::workspace::parse_files(&[("src/d.bmb".to_string(), text.to_string())]);
        assert!(!d.has_errors());
        let mut spans = Vec::new();
        super::export_spans(&program, &mut spans).unwrap();
        let at = |needle: &str| {
            let lo = text.find(needle).unwrap() as u32;
            super::node_at(&spans, 0, lo, lo + needle.len() as u32).map(String::from)
        };
        assert_eq!(at("x + 1").as_deref(), Some("/d/fns/f/body/stmts/0/init"));
        assert_eq!(at("1").as_deref(), Some("/d/fns/f/body/stmts/0/init/rhs"));
        let y = text.find("        y\n").unwrap() as u32 + 8;
        assert_eq!(
            super::node_at(&spans, 0, y, y + 1),
            Some("/d/fns/f/body/tail")
        );
    }

    #[test]
    fn import_says_where_and_what_is_wrong() {
        let doc = |s: &str| crate::json::parse(s).unwrap();
        let err = |s: &str| super::import(&doc(s)).unwrap_err();
        let e = err(r#"{"d":{"fns":{"f":{"kind":"fn","ret":{"ty":{"kind":"i65"}},"body":{}}}}}"#);
        assert!(
            e.starts_with("/d/fns/f/ret/ty: unknown type kind `i65`"),
            "{e}"
        );
        let e = err(
            r#"{"d":{"fns":{"f":{"kind":"fn","ret":{"ty":{"kind":"i64"}},"body":{"tail":{"kind":"var","nme":"x"}}}}}}"#,
        );
        assert!(
            e.contains("/d/fns/f/body/tail: a variable has no field `nme`"),
            "{e}"
        );
        let e = err(r#"{"d":{"fns":{"f":{"kind":"fn","ret":{"ty":{"kind":"unit"}}}}}}"#);
        assert!(e.contains("/d/fns/f: missing field `body`"), "{e}");
        let e = err(r#"{"d e":{}}"#);
        assert!(e.contains("not a unit name"), "{e}");
        let e = err(
            r#"{"d":{"fns":{"f":{"kind":"fn","ret":{"ty":{"kind":"unit"}},"body":{"tail":{"kind":"if","cond":{"kind":"bool","value":true},"then":{},"else":{"kind":"unit"}}}}}}}"#,
        );
        assert!(e.contains("`else` is an `if` or a `block`"), "{e}");
    }

    #[test]
    fn json_patch_follows_rfc_6902() {
        let mut doc = crate::json::parse(r#"{"a":{"b":[1,2]},"c":3}"#).unwrap();
        let patch = crate::json::parse(
            r#"[{"op":"add","path":"/a/b/-","value":4},{"op":"add","path":"/a/b/0","value":0},
                {"op":"replace","path":"/c","value":{"x":1}},{"op":"copy","from":"/c","path":"/d"},
                {"op":"move","from":"/d/x","path":"/e"},{"op":"remove","path":"/a/b/1"},
                {"op":"test","path":"/a/b","value":[0,2,4]},{"op":"add","path":"/f~1g","value":true}]"#,
        )
        .unwrap();
        super::apply_patch(&mut doc, &patch).unwrap();
        assert_eq!(
            doc.to_json(),
            r#"{"a":{"b":[0,2,4]},"c":{"x":1},"d":{},"e":1,"f/g":true}"#
        );
        // All or nothing: a failing operation leaves the document as it was.
        let before = doc.clone();
        let bad = crate::json::parse(
            r#"[{"op":"remove","path":"/c"},{"op":"test","path":"/e","value":2}]"#,
        )
        .unwrap();
        let e = super::apply_patch(&mut doc, &bad).unwrap_err();
        assert!(
            e.starts_with("operation 1: test `/e`: the value differs"),
            "{e}"
        );
        assert_eq!(doc, before);
        for (p, why) in [
            (
                r#"[{"op":"replace","path":"/zz","value":1}]"#,
                "no such member",
            ),
            (
                r#"[{"op":"add","path":"/a/b/9","value":1}]"#,
                "out of range",
            ),
            (
                r#"[{"op":"move","from":"/a","path":"/a/b/x"}]"#,
                "into itself",
            ),
            (r#"[{"op":"frob","path":"/a"}]"#, "unknown op"),
            (
                r#"[{"op":"add","path":"a","value":1}]"#,
                "not a JSON Pointer",
            ),
        ] {
            let e = super::apply_patch(&mut doc, &crate::json::parse(p).unwrap()).unwrap_err();
            assert!(e.contains(why), "{p}: {e}");
        }
    }
}
