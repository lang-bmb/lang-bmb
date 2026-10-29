//! A walk over every place a program names another definition (calls,
//! types, struct literals, variants, patterns, trait bounds, `impl` heads,
//! `#[uses]` and `#[invariant]` targets) and every declaration name, with
//! a hook that may rewrite each one. The transaction `rename` (ADR-0059 §5,
//! `workspace.rs`) and the version alignment of `bmb weaken` (backlog
//! 2D-19, `weaken.rs`) are its two rewrites.

use crate::ast::*;

/// Where a path is written, which decides how it can name a definition.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Site {
    /// The callee of a call (`f(..)`, `T::new(..)`, `T::V(..)`,
    /// `Trait::m(..)`), or a `#[uses]` / `#[invariant]` target.
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

/// A declaration name the walk meets, in the unit being walked.
pub(crate) enum Decl<'a> {
    /// A function of the unit (`spec fn` and `proof fn` included).
    Fn(&'a mut Ident),
    /// A `struct` or `enum` declaration.
    Type(&'a mut Ident),
    /// The type an `impl T { .. }` block is for.
    ImplTy(&'a mut Ident),
    /// A function of the `impl` block for type `ty` (its name before any
    /// rewrite of the block's head).
    ImplFn { ty: String, name: &'a mut Ident },
    /// A trait declaration.
    Trait(&'a mut Ident),
}

/// What a walk does at each place.
pub(crate) trait Rewrite {
    /// The path `segs`, written at `site` in `unit` (its full path) with
    /// the type parameters `tparams` in scope. It may change the segments
    /// and their number.
    fn path(&mut self, segs: &mut Vec<String>, site: Site, unit: &[String], tparams: &[String]);
    /// A declaration name.
    fn decl(&mut self, _unit: &[String], _d: Decl<'_>) {}
    /// A variable read (`ExprKind::Var`).
    fn var(&mut self, _name: &mut String) {}
}

/// Walks one function of `unit` with `r`.
pub(crate) fn walk_fn(f: &mut FnDef, unit: &[String], r: &mut dyn Rewrite) {
    let mut w = Walker {
        r,
        unit: unit.to_vec(),
        tparams: Vec::new(),
    };
    w.function(f);
}

/// Walks every unit of `p` with `r`.
pub(crate) fn walk(p: &mut Program, r: &mut dyn Rewrite) {
    let mut w = Walker {
        r,
        unit: Vec::new(),
        tparams: Vec::new(),
    };
    for m in &mut p.mods {
        w.module(m, &[]);
    }
}

struct Walker<'r> {
    r: &'r mut dyn Rewrite,
    /// The unit being walked.
    unit: Vec<String>,
    /// Type parameters in scope (a one-segment type path that names one is
    /// no reference to a type).
    tparams: Vec<String>,
}

impl Walker<'_> {
    fn segs(&mut self, segs: &mut Vec<String>, site: Site) {
        self.r.path(segs, site, &self.unit, &self.tparams);
    }

    /// Rewrites a path of identifiers; segments added in front take the
    /// first segment's span.
    fn idents(&mut self, path: &mut Vec<Ident>, site: Site) -> isize {
        let mut s: Vec<String> = path.iter().map(|i| i.name.clone()).collect();
        self.segs(&mut s, site);
        let delta = s.len() as isize - path.len() as isize;
        if delta == 0 {
            for (i, n) in path.iter_mut().zip(s) {
                i.name = n;
            }
        } else {
            let span = path.first().map(|i| i.span).unwrap_or_default();
            *path = s.into_iter().map(|name| Ident { name, span }).collect();
        }
        delta
    }

    fn ty(&mut self, t: &mut Type) {
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

    /// The spec fns an attribute names (`#[uses(..)]`, `#[invariant(..)]`).
    fn attr_targets(&mut self, a: &mut Attr) {
        for x in &mut a.args {
            if let AttrArg::Ident(i) = x {
                let mut s: Vec<String> = i.name.split("::").map(String::from).collect();
                self.segs(&mut s, Site::Call);
                i.name = s.join("::");
            }
        }
    }

    fn function(&mut self, f: &mut FnDef) {
        let depth = self.tparams.len();
        self.generics(&mut f.generics);
        for a in f.attrs.iter_mut().filter(|a| a.name.name == "uses") {
            self.attr_targets(a);
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

    fn block(&mut self, b: &mut Block) {
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

    fn expr(&mut self, e: &mut Expr) {
        match &mut e.kind {
            ExprKind::Call { path, targs, args } => {
                let before = path.len() as isize;
                let delta = self.idents(path, Site::Call);
                // Type arguments stay on the segment they followed, counted
                // from the end (`f::<i64>`, `Stack::<i64>::new`).
                for t in targs.iter_mut() {
                    let from_end = before - 1 - t.seg as isize;
                    t.seg = (before + delta - 1 - from_end).max(0) as usize;
                }
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
            ExprKind::Path(path) => {
                self.idents(path, Site::Variant);
            }
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
            ExprKind::Var(name) => self.r.var(&mut name.name),
            ExprKind::Int(..)
            | ExprKind::ByteLit(_)
            | ExprKind::Float(_)
            | ExprKind::ByteStr(_)
            | ExprKind::Bool(_)
            | ExprKind::Unit
            | ExprKind::Continue { .. }
            | ExprKind::Error => {}
        }
    }

    /// Walks unit `m` (and its sub-units): its declarations and every
    /// reference.
    fn module(&mut self, m: &mut Mod, prefix: &[String]) {
        let mut unit = prefix.to_vec();
        unit.push(m.name.name.clone());
        for s in &mut m.mods {
            self.module(s, &unit);
        }
        self.unit = unit.clone();
        for f in &mut m.fns {
            self.r.decl(&unit, Decl::Fn(&mut f.name));
            self.function(f);
        }
        for td in &mut m.types {
            self.r.decl(&unit, Decl::Type(&mut td.name));
            // `#[invariant(p)]` names its predicate as `#[uses]` does.
            for a in td.attrs.iter_mut().filter(|a| a.name.name == "invariant") {
                self.attr_targets(a);
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
            let ty = i.ty.name.clone();
            self.r.decl(&unit, Decl::ImplTy(&mut i.ty));
            let depth = self.tparams.len();
            self.generics(&mut i.generics);
            i.ty_args.iter_mut().for_each(|t| self.ty(t));
            for f in &mut i.fns {
                self.r.decl(
                    &unit,
                    Decl::ImplFn {
                        ty: ty.clone(),
                        name: &mut f.name,
                    },
                );
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
            self.r.decl(&unit, Decl::Trait(&mut tr.name));
            for tf in &mut tr.fns {
                self.function(&mut tf.f);
            }
        }
    }
}
