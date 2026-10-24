//! Name resolution, type checking, second-class reference rules, specification
//! placement and effect checking for the milestone-1 subset.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::*;
use crate::diag::{Diag, Diags};
use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Pure,
    Io,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub path: String,
    pub kind: FnKind,
    pub params: Vec<TypeKind>,
    pub ret: TypeKind,
    pub effect: Effect,
    /// A spec fn whose body is the literal `true`.
    pub vacuous: bool,
    pub builtin: bool,
}

pub struct Checker<'a> {
    diags: &'a mut Diags,
    pub fns: BTreeMap<String, FnSig>,
    /// Exec-to-exec and spec-to-spec call edges, by path.
    calls: BTreeMap<String, BTreeSet<String>>,
    /// Type of every checked expression, keyed by its span `(file, lo, hi)`.
    pub types: BTreeMap<(u32, u32, u32), TypeKind>,
    /// Recursive definitions: path -> id of its strongly connected component.
    pub recursive: BTreeMap<String, usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Exec,
    /// Specification context. `old` is allowed only in `ensures`.
    Spec {
        allow_old: bool,
    },
}

#[derive(Clone)]
struct Local {
    ty: TypeKind,
    mutable: bool,
    is_param: bool,
}

struct FnCtx {
    path: String,
    mod_path: String,
    effect: Effect,
    scopes: Vec<BTreeMap<String, Local>>,
    all_names: BTreeSet<String>,
    forall_depth: u32,
}

pub fn ty_str(t: &TypeKind) -> String {
    match t {
        TypeKind::I64 => "i64".into(),
        TypeKind::U64 => "u64".into(),
        TypeKind::Usize => "usize".into(),
        TypeKind::Bool => "bool".into(),
        TypeKind::Unit => "()".into(),
        TypeKind::Array(e, n) => format!("[{}; {n}]", ty_str(&e.kind)),
        TypeKind::Slice(e) => format!("[{}]", ty_str(&e.kind)),
        TypeKind::Ref { mutable, inner } => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            ty_str(&inner.kind)
        ),
        TypeKind::Error => "{error}".into(),
    }
}

fn is_int(t: &TypeKind) -> bool {
    matches!(t, TypeKind::I64 | TypeKind::U64 | TypeKind::Usize)
}
fn is_scalar(t: &TypeKind) -> bool {
    is_int(t) || matches!(t, TypeKind::Bool)
}
fn elem_of(t: &TypeKind) -> Option<&TypeKind> {
    match t {
        TypeKind::Array(e, _) => Some(&e.kind),
        TypeKind::Ref { inner, .. } => match &inner.kind {
            TypeKind::Slice(e) => Some(&e.kind),
            _ => None,
        },
        _ => None,
    }
}
fn mk(kind: TypeKind) -> Box<Type> {
    Box::new(Type {
        kind,
        span: Span::default(),
    })
}

impl<'a> Checker<'a> {
    pub fn new(diags: &'a mut Diags) -> Self {
        let mut fns = BTreeMap::new();
        fns.insert(
            "print_i64".to_string(),
            FnSig {
                path: "print_i64".into(),
                kind: FnKind::Exec,
                params: vec![TypeKind::I64],
                ret: TypeKind::Unit,
                effect: Effect::Io,
                vacuous: false,
                builtin: true,
            },
        );
        Checker {
            diags,
            fns,
            calls: BTreeMap::new(),
            types: BTreeMap::new(),
            recursive: BTreeMap::new(),
        }
    }

    /// Hands the tables the proof gate needs to the verifier.
    pub fn into_tables(self) -> crate::verify::Tables {
        crate::verify::Tables {
            fns: self.fns,
            types: self.types,
            recursive: self.recursive,
        }
    }

    fn err(&mut self, code: &'static str, span: Span, msg: impl Into<String>) {
        self.diags.push(Diag::new(code, span, msg));
    }

    pub fn check_program(&mut self, p: &Program) {
        // Pass 1: collect signatures.
        let mut seen = BTreeMap::new();
        for m in &p.mods {
            if let Some(prev) = seen.insert(m.name.name.clone(), m.name.span) {
                let _ = prev;
                self.err(
                    "E0302",
                    m.name.span,
                    format!("unit `{}` is defined more than once", m.name.name),
                );
            }
            self.collect_mod(m, "");
        }
        // Pass 2: check bodies.
        for m in &p.mods {
            self.check_mod(m, "");
        }
        self.check_recursion(p);
        self.check_main(p);
    }

    fn collect_mod(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        let mut names = BTreeSet::new();
        for sub in &m.mods {
            if !names.insert(sub.name.name.clone()) {
                self.err(
                    "E0302",
                    sub.name.span,
                    format!("`{}` is defined more than once in `{mp}`", sub.name.name),
                );
            }
            self.collect_mod(sub, &format!("{mp}::"));
        }
        for f in &m.fns {
            if !names.insert(f.name.name.clone()) {
                self.err(
                    "E0302",
                    f.name.span,
                    format!("`{}` is defined more than once in `{mp}`", f.name.name),
                );
                continue;
            }
            let path = format!("{mp}::{}", f.name.name);
            let effect = self.fn_effect(f);
            let vacuous = f.kind == FnKind::Spec
                && f.body.stmts.is_empty()
                && matches!(
                    f.body.tail.as_deref(),
                    Some(Expr {
                        kind: ExprKind::Bool(true),
                        ..
                    })
                );
            self.fns.insert(
                path.clone(),
                FnSig {
                    path,
                    kind: f.kind,
                    params: f.params.iter().map(|p| p.ty.kind.clone()).collect(),
                    ret: f.ret.ty.kind.clone(),
                    effect,
                    vacuous,
                    builtin: false,
                },
            );
        }
    }

    /// Reads `#[effects(..)]` and reports attribute problems once, during collection.
    fn fn_effect(&mut self, f: &FnDef) -> Effect {
        let mut effect = None;
        for a in &f.attrs {
            match a.name.name.as_str() {
                "effects" => {
                    if f.kind == FnKind::Spec {
                        self.diags.push(
                            Diag::new("E0604", a.span, "spec functions are pure by definition")
                                .fix("remove the attribute", Some(String::new())),
                        );
                        continue;
                    }
                    if effect.is_some() {
                        self.err("E0605", a.span, "duplicate `#[effects(..)]` attribute");
                        continue;
                    }
                    let mut e = Effect::Pure;
                    if a.args.len() != 1 {
                        self.err(
                            "E0602",
                            a.span,
                            "`#[effects(..)]` takes exactly one effect: `pure` or `io`",
                        );
                    }
                    for arg in &a.args {
                        match arg {
                            AttrArg::Ident(i) if i.name == "pure" => e = Effect::Pure,
                            AttrArg::Ident(i) if i.name == "io" => e = Effect::Io,
                            AttrArg::Ident(i) => self.err(
                                "E0602",
                                i.span,
                                format!(
                                    "unknown effect `{}`; milestone 1 has `pure` and `io`",
                                    i.name
                                ),
                            ),
                            AttrArg::Str(_, sp) => {
                                self.err("E0602", *sp, "effect must be an identifier")
                            }
                        }
                    }
                    effect = Some(e);
                }
                "uses" => {}
                other => self.err(
                    "E0605",
                    a.span,
                    format!("unknown attribute `{other}` on a function"),
                ),
            }
        }
        if f.kind == FnKind::Exec && effect.is_none() {
            self.diags.push(
                Diag::new(
                    "E0601",
                    f.name.span,
                    format!("`{}` has no `#[effects(..)]` attribute", f.name.name),
                )
                .fix(
                    "add `#[effects(pure)]` above the function",
                    Some("#[effects(pure)]".into()),
                ),
            );
        }
        effect.unwrap_or(Effect::Pure)
    }

    fn check_mod(&mut self, m: &Mod, prefix: &str) {
        let mp = format!("{prefix}{}", m.name.name);
        let mut has_summary = false;
        for a in &m.attrs {
            if a.name.name == "summary"
                && a.args.len() == 1
                && matches!(a.args[0], AttrArg::Str(..))
            {
                has_summary = true;
            } else if a.name.name == "summary" {
                self.err("E0704", a.span, "`#[summary(..)]` takes exactly one string");
                has_summary = true;
            } else {
                self.err(
                    "E0605",
                    a.span,
                    format!("unknown attribute `{}` on a unit", a.name.name),
                );
            }
        }
        if !has_summary {
            self.diags.push(
                Diag::new(
                    "E0704",
                    m.name.span,
                    format!("unit `{}` has no summary", m.name.name),
                )
                .fix("add `#[summary(\"one line\")]` above `mod`", None),
            );
        }
        for sub in &m.mods {
            self.check_mod(sub, &format!("{mp}::"));
        }
        for f in &m.fns {
            self.check_fn(f, &mp);
        }
    }

    fn resolve(&self, path: &[Ident], mod_path: &str) -> Option<String> {
        if path.len() == 1 {
            let local = format!("{mod_path}::{}", path[0].name);
            if self.fns.contains_key(&local) {
                return Some(local);
            }
            if self.fns.get(&path[0].name).is_some_and(|s| s.builtin) {
                return Some(path[0].name.clone());
            }
            return None;
        }
        let p = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        self.fns.contains_key(&p).then_some(p)
    }

    // ---------- functions ----------

    fn check_fn(&mut self, f: &FnDef, mod_path: &str) {
        let path = format!("{mod_path}::{}", f.name.name);
        let Some(sig) = self.fns.get(&path).cloned() else {
            return;
        };
        let mut cx = FnCtx {
            path: path.clone(),
            mod_path: mod_path.to_string(),
            effect: sig.effect,
            scopes: vec![BTreeMap::new()],
            all_names: BTreeSet::new(),
            forall_depth: 0,
        };
        self.calls.entry(path.clone()).or_default();

        for a in &f.attrs {
            if a.name.name == "uses" {
                for arg in &a.args {
                    match arg {
                        AttrArg::Ident(i) => {
                            let target = if i.name.contains("::") {
                                Some(i.name.clone())
                            } else {
                                self.resolve(std::slice::from_ref(i), mod_path)
                            };
                            let ok = target
                                .and_then(|p| self.fns.get(&p))
                                .is_some_and(|s| s.kind == FnKind::Spec);
                            if !ok {
                                self.err(
                                    "E0509",
                                    i.span,
                                    format!("`{}` is not a spec function", i.name),
                                );
                            }
                        }
                        AttrArg::Str(_, sp) => {
                            self.err("E0509", *sp, "`#[uses(..)]` takes spec function names")
                        }
                    }
                }
            }
        }

        for p in &f.params {
            self.check_param_type(&p.ty, f.kind);
            self.declare(&mut cx, &p.name, p.ty.kind.clone(), false, true);
        }
        self.check_value_type(&f.ret.ty, "return type");

        if f.kind == FnKind::Spec && (!f.requires.is_empty() || !f.ensures.is_empty()) {
            let sp = f.requires.first().or(f.ensures.first()).unwrap().span;
            self.err(
                "E0207",
                sp,
                "spec functions take no `requires` / `ensures` in milestone 1",
            );
        }

        for r in &f.requires {
            self.check_contract(&mut cx, r, false);
        }
        if !f.ensures.is_empty() {
            cx.scopes.push(BTreeMap::new());
            if let Some(n) = &f.ret.name {
                self.declare(&mut cx, n, f.ret.ty.kind.clone(), false, false);
            }
            for e in &f.ensures {
                self.check_contract(&mut cx, e, true);
            }
            cx.scopes.pop();
        } else if let Some(n) = &f.ret.name {
            // The result name is still reserved so the body cannot rebind it.
            cx.all_names.insert(n.name.clone());
        }
        if let Some(d) = &f.decreases {
            let t = self.expr(&mut cx, d, None, Mode::Spec { allow_old: false });
            if !is_int(&t) && t != TypeKind::Error {
                self.err(
                    "E0401",
                    d.span,
                    format!("`decreases` must be an integer, found `{}`", ty_str(&t)),
                );
            }
        }

        let mode = if f.kind == FnKind::Spec {
            Mode::Spec { allow_old: false }
        } else {
            Mode::Exec
        };
        let t = self.block(&mut cx, &f.body, Some(&f.ret.ty.kind), mode);
        self.expect_ty(
            &t,
            &f.ret.ty.kind,
            f.body.tail.as_ref().map_or(f.body.span, |e| e.span),
        );
    }

    fn check_param_type(&mut self, t: &Type, kind: FnKind) {
        match &t.kind {
            TypeKind::Ref { mutable, inner } => match &inner.kind {
                TypeKind::Slice(e) if is_scalar(&e.kind) => {
                    if *mutable && kind == FnKind::Spec {
                        self.err("E0402", t.span, "spec function parameters cannot be `&mut`");
                    }
                }
                _ => self.err(
                    "E0402",
                    t.span,
                    "milestone 1 references are `&[T]` or `&mut [T]` with a scalar `T`",
                ),
            },
            _ => self.check_value_type(t, "parameter type"),
        }
    }

    /// Types for locals and results: no references, no bare slices.
    fn check_value_type(&mut self, t: &Type, what: &str) {
        match &t.kind {
            TypeKind::Ref { .. } => self.diags.push(
                Diag::new("E0403", t.span, format!("a reference cannot be a {what}")).fix(
                    "references are second-class: pass them as arguments only (ADR-0015)",
                    None,
                ),
            ),
            TypeKind::Slice(_) => self.err(
                "E0402",
                t.span,
                "a bare slice is unsized; use `&[T]` as a parameter or `[T; N]`",
            ),
            TypeKind::Array(e, _) if !is_scalar(&e.kind) && e.kind != TypeKind::Error => {
                self.err(
                    "E0402",
                    t.span,
                    "array elements must be scalars in milestone 1",
                );
            }
            _ => {}
        }
    }

    fn check_contract(&mut self, cx: &mut FnCtx, e: &Expr, ensures: bool) {
        let mode = Mode::Spec { allow_old: ensures };
        let t = self.expr(cx, e, Some(&TypeKind::Bool), mode);
        let named = match &e.kind {
            ExprKind::Call { path, .. } => self
                .resolve(path, &cx.mod_path)
                .and_then(|p| self.fns.get(&p).cloned()),
            _ => None,
        };
        match named {
            Some(sig) if sig.kind == FnKind::Spec && sig.ret == TypeKind::Bool => {
                if sig.vacuous {
                    self.err(
                        "E0502",
                        e.span,
                        format!("`{}` is the constant `true`; it states nothing", sig.path),
                    );
                }
            }
            _ => {
                if t != TypeKind::Error {
                    self.diags.push(
                        Diag::new("E0501", e.span, "contract clause is not a named predicate")
                            .fix("move the condition into `spec fn name(..) -> bool { .. }` and call it here", None),
                    );
                }
            }
        }
    }

    fn declare(
        &mut self,
        cx: &mut FnCtx,
        name: &Ident,
        ty: TypeKind,
        mutable: bool,
        is_param: bool,
    ) {
        if !cx.all_names.insert(name.name.clone()) {
            self.diags.push(
                Diag::new(
                    "E0303",
                    name.span,
                    format!("`{}` is already bound in this function", name.name),
                )
                .fix("choose a new name", None),
            );
        }
        cx.scopes.last_mut().unwrap().insert(
            name.name.clone(),
            Local {
                ty,
                mutable,
                is_param,
            },
        );
    }

    fn lookup(cx: &FnCtx, name: &str) -> Option<Local> {
        cx.scopes.iter().rev().find_map(|s| s.get(name).cloned())
    }

    fn expect_ty(&mut self, found: &TypeKind, want: &TypeKind, span: Span) {
        if found != want && *found != TypeKind::Error && *want != TypeKind::Error {
            self.err(
                "E0401",
                span,
                format!("expected `{}`, found `{}`", ty_str(want), ty_str(found)),
            );
        }
    }

    // ---------- blocks and statements ----------

    fn block(
        &mut self,
        cx: &mut FnCtx,
        b: &Block,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        cx.scopes.push(BTreeMap::new());
        for s in &b.stmts {
            self.stmt(cx, s, mode);
        }
        let t = match &b.tail {
            Some(e) => self.expr(cx, e, expected, mode),
            None => TypeKind::Unit,
        };
        cx.scopes.pop();
        t
    }

    fn stmt(&mut self, cx: &mut FnCtx, s: &Stmt, mode: Mode) {
        match s {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
                ..
            } => {
                self.check_value_type(ty, "local variable type");
                if *mutable && mode != Mode::Exec {
                    self.err(
                        "E0503",
                        name.span,
                        "`let mut` is not allowed in a specification",
                    );
                }
                let t = self.expr(cx, init, Some(&ty.kind), mode);
                self.expect_ty(&t, &ty.kind, init.span);
                self.declare(cx, name, ty.kind.clone(), *mutable, false);
            }
            Stmt::Assign {
                target,
                value,
                span,
            } => {
                if mode != Mode::Exec {
                    self.err(
                        "E0503",
                        *span,
                        "assignment is not allowed in a specification",
                    );
                }
                let tt = self.place(cx, target);
                let vt = self.expr(cx, value, Some(&tt), mode);
                self.expect_ty(&vt, &tt, value.span);
            }
            Stmt::While {
                cond,
                invariants,
                decreases,
                body,
                span,
            } => {
                if mode != Mode::Exec {
                    self.err("E0503", *span, "loops are not allowed in a specification");
                }
                let ct = self.expr(cx, cond, Some(&TypeKind::Bool), mode);
                self.expect_ty(&ct, &TypeKind::Bool, cond.span);
                // `old(a)` in an invariant or assertion is `a` on function entry.
                for inv in invariants {
                    let t = self.expr(
                        cx,
                        inv,
                        Some(&TypeKind::Bool),
                        Mode::Spec { allow_old: true },
                    );
                    self.expect_ty(&t, &TypeKind::Bool, inv.span);
                }
                match decreases {
                    Some(d) => {
                        let t = self.expr(cx, d, None, Mode::Spec { allow_old: false });
                        if !is_int(&t) && t != TypeKind::Error {
                            self.err(
                                "E0401",
                                d.span,
                                format!("`decreases` must be an integer, found `{}`", ty_str(&t)),
                            );
                        }
                    }
                    None => {
                        let sp = Span {
                            hi: cond.span.hi,
                            ..*span
                        };
                        self.diags.push(
                            Diag::new("E0506", sp, "`while` has no `decreases` clause")
                                .fix("add `decreases <expr>,` before the loop body", None),
                        );
                    }
                }
                let bt = self.block(cx, body, Some(&TypeKind::Unit), mode);
                self.expect_ty(&bt, &TypeKind::Unit, body.span);
            }
            Stmt::Assert { cond, .. } => {
                let t = self.expr(
                    cx,
                    cond,
                    Some(&TypeKind::Bool),
                    Mode::Spec { allow_old: true },
                );
                self.expect_ty(&t, &TypeKind::Bool, cond.span);
            }
            Stmt::Expr(e) => {
                self.expr(cx, e, None, mode);
            }
        }
    }

    /// Checks an assignment target and returns its type.
    fn place(&mut self, cx: &mut FnCtx, target: &Expr) -> TypeKind {
        match &target.kind {
            ExprKind::Var(id) => match Self::lookup(cx, &id.name) {
                Some(l) => {
                    if !l.mutable {
                        let fix = if l.is_param {
                            "copy the parameter into a `let mut` local"
                        } else {
                            "declare it with `let mut`"
                        };
                        self.diags.push(
                            Diag::new(
                                "E0408",
                                target.span,
                                format!("`{}` is not mutable", id.name),
                            )
                            .fix(fix, None),
                        );
                    }
                    l.ty
                }
                None => {
                    self.err("E0301", id.span, format!("unknown name `{}`", id.name));
                    TypeKind::Error
                }
            },
            ExprKind::Index { base, index } => {
                let ExprKind::Var(id) = &base.kind else {
                    self.err("E0408", target.span, "only `name[i]` can be assigned");
                    return TypeKind::Error;
                };
                let Some(l) = Self::lookup(cx, &id.name) else {
                    self.err("E0301", id.span, format!("unknown name `{}`", id.name));
                    return TypeKind::Error;
                };
                let writable = match &l.ty {
                    TypeKind::Array(..) => l.mutable,
                    TypeKind::Ref { mutable, .. } => *mutable,
                    _ => false,
                };
                let it = self.expr(cx, index, Some(&TypeKind::Usize), Mode::Exec);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                let Some(elem) = elem_of(&l.ty).cloned() else {
                    self.err(
                        "E0407",
                        base.span,
                        format!(
                            "`{}` has type `{}` and cannot be indexed",
                            id.name,
                            ty_str(&l.ty)
                        ),
                    );
                    return TypeKind::Error;
                };
                if !writable {
                    self.err(
                        "E0408",
                        target.span,
                        format!(
                            "elements of `{}` cannot be assigned (`{}`)",
                            id.name,
                            ty_str(&l.ty)
                        ),
                    );
                }
                elem
            }
            _ => {
                self.err("E0408", target.span, "invalid assignment target");
                TypeKind::Error
            }
        }
    }

    // ---------- expressions ----------

    fn expr(
        &mut self,
        cx: &mut FnCtx,
        e: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        let t = self.expr_inner(cx, e, expected, mode);
        self.types
            .insert((e.span.file, e.span.lo, e.span.hi), t.clone());
        t
    }

    fn expr_inner(
        &mut self,
        cx: &mut FnCtx,
        e: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
    ) -> TypeKind {
        match &e.kind {
            ExprKind::Error => TypeKind::Error,
            ExprKind::Unit => TypeKind::Unit,
            ExprKind::Bool(_) => TypeKind::Bool,
            ExprKind::Int(v) => self.int_lit(*v, false, expected, e.span, mode),
            ExprKind::Var(id) => match Self::lookup(cx, &id.name) {
                Some(l) => l.ty,
                None => {
                    let fix = if id.name == "result" {
                        Some("name the result: `-> (r: T)` and use `r`")
                    } else {
                        None
                    };
                    let mut d = Diag::new("E0301", id.span, format!("unknown name `{}`", id.name));
                    if let Some(f) = fix {
                        d = d.fix(f, None);
                    }
                    self.diags.push(d);
                    TypeKind::Error
                }
            },
            ExprKind::Unary(UnOp::Neg, inner) => {
                if let ExprKind::Int(v) = inner.kind {
                    return self.int_lit(v, true, expected, e.span, mode);
                }
                let t = self.expr(cx, inner, expected, mode);
                match t {
                    TypeKind::I64 | TypeKind::Error => t,
                    TypeKind::U64 | TypeKind::Usize => {
                        self.err(
                            "E0411",
                            e.span,
                            format!("cannot negate a value of type `{}`", ty_str(&t)),
                        );
                        TypeKind::Error
                    }
                    _ => {
                        self.err("E0401", e.span, format!("cannot negate `{}`", ty_str(&t)));
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Unary(UnOp::Not, inner) => {
                let t = self.expr(cx, inner, Some(&TypeKind::Bool), mode);
                self.expect_ty(&t, &TypeKind::Bool, inner.span);
                TypeKind::Bool
            }
            ExprKind::Binary(op, l, r) => self.binary(cx, *op, l, r, expected, mode, e.span),
            ExprKind::Index { base, index } => {
                let bt = self.expr(cx, base, None, mode);
                let it = self.expr(cx, index, Some(&TypeKind::Usize), mode);
                self.expect_ty(&it, &TypeKind::Usize, index.span);
                match elem_of(&bt) {
                    Some(t) => t.clone(),
                    None => {
                        if bt != TypeKind::Error {
                            self.err(
                                "E0407",
                                base.span,
                                format!("type `{}` cannot be indexed", ty_str(&bt)),
                            );
                        }
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Method { recv, name, args } => self.method(cx, recv, name, args, mode),
            ExprKind::Call { path, args } => self.call(cx, path, args, e.span, mode),
            ExprKind::Ref { .. } => {
                self.diags.push(Diag::new(
                    "E0409",
                    e.span,
                    "`&` / `&mut` may only appear directly as a call argument",
                ));
                TypeKind::Error
            }
            ExprKind::If { cond, then, els } => {
                let ct = self.expr(cx, cond, Some(&TypeKind::Bool), mode);
                self.expect_ty(&ct, &TypeKind::Bool, cond.span);
                let tt = self.block(cx, then, expected, mode);
                match els {
                    Some(el) => {
                        let et = self.expr(
                            cx,
                            el,
                            Some(&tt).filter(|t| **t != TypeKind::Error).or(expected),
                            mode,
                        );
                        self.expect_ty(&et, &tt, el.span);
                        tt
                    }
                    None => {
                        self.expect_ty(&tt, &TypeKind::Unit, then.span);
                        TypeKind::Unit
                    }
                }
            }
            ExprKind::Block(b) => self.block(cx, b, expected, mode),
            ExprKind::Array(_) | ExprKind::Repeat { .. } if mode != Mode::Exec => {
                self.err(
                    "E0503",
                    e.span,
                    "array literals are only allowed in executable code",
                );
                TypeKind::Error
            }
            ExprKind::Array(items) => {
                let want_elem = match expected {
                    Some(TypeKind::Array(el, _)) => Some(el.kind.clone()),
                    _ => None,
                };
                let mut elem = want_elem.clone();
                for it in items {
                    let t = self.expr(cx, it, elem.as_ref(), mode);
                    match &elem {
                        None => elem = Some(t),
                        Some(want) => self.expect_ty(&t, want, it.span),
                    }
                }
                match elem {
                    Some(el) => TypeKind::Array(mk(el), items.len() as u64),
                    None => {
                        self.err(
                            "E0401",
                            e.span,
                            "cannot determine the element type of an empty array",
                        );
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Repeat {
                value,
                count,
                count_span,
            } => {
                let want_elem = match expected {
                    Some(TypeKind::Array(el, _)) => Some(el.kind.clone()),
                    _ => None,
                };
                let t = self.expr(cx, value, want_elem.as_ref(), mode);
                if *count > u32::MAX as u128 {
                    self.err("E0406", *count_span, "array length is too large");
                }
                TypeKind::Array(mk(t), *count as u64)
            }
            ExprKind::Old(inner) => {
                let ok_mode = matches!(mode, Mode::Spec { allow_old: true });
                let var_ty = match &inner.kind {
                    ExprKind::Var(id) => Self::lookup(cx, &id.name),
                    _ => None,
                };
                match var_ty {
                    Some(Local {
                        ty: ty @ TypeKind::Ref { mutable: true, .. },
                        is_param: true,
                        ..
                    }) if ok_mode => ty,
                    _ => {
                        let code = if mode == Mode::Exec { "E0503" } else { "E0508" };
                        self.err(
                            code,
                            e.span,
                            "`old(x)` is only allowed in `ensures`, loop invariants and `assert`, on a `&mut` parameter `x`",
                        );
                        TypeKind::Error
                    }
                }
            }
            ExprKind::Forall { binders, body } => {
                if mode == Mode::Exec {
                    self.err(
                        "E0503",
                        e.span,
                        "`forall` is only allowed in specifications",
                    );
                    return TypeKind::Error;
                }
                cx.scopes.push(BTreeMap::new());
                for b in binders {
                    if !is_int(&b.ty.kind) && b.ty.kind != TypeKind::Error {
                        self.err("E0402", b.ty.span, "`forall` binders must be integers");
                    }
                    self.declare(cx, &b.name, b.ty.kind.clone(), false, false);
                }
                let mut triggers = Vec::new();
                collect_triggers(body, &mut triggers);
                if triggers.is_empty() {
                    self.diags.push(
                        Diag::new("E0511", e.span, "`forall` has no `#[trigger]`").fix(
                            "mark a term that mentions every bound variable, e.g. `#[trigger] a[i]`",
                            None,
                        ),
                    );
                }
                // All triggers of one `forall` form a single multi-pattern.
                if !triggers.is_empty() {
                    for b in binders {
                        if !triggers.iter().any(|t| mentions_strict(t, &b.name.name)) {
                            self.err(
                                "E0512",
                                b.name.span,
                                format!("no trigger mentions bound variable `{}`", b.name.name),
                            );
                        }
                    }
                }
                cx.forall_depth += 1;
                let t = self.expr(cx, body, Some(&TypeKind::Bool), mode);
                cx.forall_depth -= 1;
                cx.scopes.pop();
                self.expect_ty(&t, &TypeKind::Bool, body.span);
                TypeKind::Bool
            }
            ExprKind::Trigger(inner) => {
                if mode == Mode::Exec {
                    self.err(
                        "E0503",
                        e.span,
                        "`#[trigger]` is only allowed in specifications",
                    );
                } else if cx.forall_depth == 0 {
                    self.err("E0510", e.span, "`#[trigger]` outside `forall`");
                }
                self.expr(cx, inner, expected, mode)
            }
        }
    }

    fn int_lit(
        &mut self,
        v: u128,
        negative: bool,
        expected: Option<&TypeKind>,
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let t = match expected {
            // A negative literal in a specification is just an integer (ADR-0022).
            Some(TypeKind::U64 | TypeKind::Usize) if negative && mode != Mode::Exec => {
                TypeKind::I64
            }
            Some(t) if is_int(t) => t.clone(),
            _ => TypeKind::I64,
        };
        let fits = match t {
            TypeKind::I64 => {
                if negative {
                    v <= 1u128 << 63
                } else {
                    v <= i64::MAX as u128
                }
            }
            _ => {
                if negative && v != 0 {
                    self.err(
                        "E0411",
                        span,
                        format!("cannot negate a value of type `{}`", ty_str(&t)),
                    );
                    return TypeKind::Error;
                }
                v <= u64::MAX as u128
            }
        };
        if !fits {
            self.err(
                "E0406",
                span,
                format!("literal does not fit in `{}`", ty_str(&t)),
            );
        }
        t
    }

    #[allow(clippy::too_many_arguments)]
    fn binary(
        &mut self,
        cx: &mut FnCtx,
        op: BinOp,
        l: &Expr,
        r: &Expr,
        expected: Option<&TypeKind>,
        mode: Mode,
        span: Span,
    ) -> TypeKind {
        match op {
            BinOp::And | BinOp::Or | BinOp::Implies => {
                if op == BinOp::Implies && mode == Mode::Exec {
                    self.err("E0503", span, "`==>` is only allowed in specifications");
                }
                let lt = self.expr(cx, l, Some(&TypeKind::Bool), mode);
                let rt = self.expr(cx, r, Some(&TypeKind::Bool), mode);
                self.expect_ty(&lt, &TypeKind::Bool, l.span);
                self.expect_ty(&rt, &TypeKind::Bool, r.span);
                TypeKind::Bool
            }
            _ => {
                // Literals take their type from the other operand.
                let lit = |e: &Expr| {
                    matches!(e.kind, ExprKind::Int(_))
                        || matches!(&e.kind, ExprKind::Unary(UnOp::Neg, i) if matches!(i.kind, ExprKind::Int(_)))
                };
                let hint = if op.is_arith() {
                    expected.filter(|t| is_int(t))
                } else {
                    None
                };
                let (lt, rt) = if lit(l) && !lit(r) {
                    let rt = self.expr(cx, r, hint, mode);
                    let lt = self.expr(cx, l, Some(&rt), mode);
                    (lt, rt)
                } else {
                    let lt = self.expr(cx, l, hint, mode);
                    let rt = self.expr(cx, r, Some(&lt), mode);
                    (lt, rt)
                };
                if lt == TypeKind::Error || rt == TypeKind::Error {
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                // Specifications compute over unbounded integers (ADR-0017 §2),
                // so integer types may mix there (ADR-0022).
                if lt != rt && mode != Mode::Exec && is_int(&lt) && is_int(&rt) {
                    return if op.is_arith() {
                        TypeKind::I64
                    } else {
                        TypeKind::Bool
                    };
                }
                if lt != rt {
                    self.err(
                        "E0401",
                        span,
                        format!(
                            "`{}` needs operands of one type, found `{}` and `{}`",
                            op.symbol(),
                            ty_str(&lt),
                            ty_str(&rt)
                        ),
                    );
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                let ok = match op {
                    BinOp::Eq | BinOp::Ne => is_scalar(&lt),
                    _ => is_int(&lt),
                };
                if !ok {
                    self.err(
                        "E0401",
                        span,
                        format!("`{}` is not defined for `{}`", op.symbol(), ty_str(&lt)),
                    );
                    return if op.is_arith() {
                        TypeKind::Error
                    } else {
                        TypeKind::Bool
                    };
                }
                if op.is_arith() { lt } else { TypeKind::Bool }
            }
        }
    }

    fn method(
        &mut self,
        cx: &mut FnCtx,
        recv: &Expr,
        name: &Ident,
        args: &[Expr],
        mode: Mode,
    ) -> TypeKind {
        let rt = self.expr(cx, recv, None, mode);
        match name.name.as_str() {
            "len" => {
                if !args.is_empty() {
                    self.err("E0405", name.span, "`len` takes no arguments");
                }
                if elem_of(&rt).is_none() && rt != TypeKind::Error {
                    self.err(
                        "E0404",
                        name.span,
                        format!("`len` is not defined for `{}`", ty_str(&rt)),
                    );
                    return TypeKind::Error;
                }
                TypeKind::Usize
            }
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" => {
                if args.len() != 1 {
                    self.err(
                        "E0405",
                        name.span,
                        format!("`{}` takes one argument", name.name),
                    );
                    return TypeKind::Error;
                }
                let at = self.expr(cx, &args[0], Some(&rt), mode);
                if !is_int(&rt) && rt != TypeKind::Error {
                    self.err(
                        "E0404",
                        name.span,
                        format!("`{}` is not defined for `{}`", name.name, ty_str(&rt)),
                    );
                    return TypeKind::Error;
                }
                self.expect_ty(&at, &rt, args[0].span);
                rt
            }
            other => {
                let fix = match other {
                    "unwrap" | "expect" => {
                        Some("handle each case explicitly; panics are not allowed")
                    }
                    "iter" | "iter_mut" | "into_iter" | "map" | "for_each" => {
                        Some("use `while` with an index")
                    }
                    "clone" => Some("arrays and scalars are copied by value; remove `.clone()`"),
                    "checked_add" | "saturating_add" | "pow" | "abs" => {
                        Some("not available in milestone 1")
                    }
                    _ => None,
                };
                let mut d = Diag::new("E0404", name.span, format!("unknown method `{other}`"));
                if let Some(f) = fix {
                    d = d.fix(f, None);
                }
                self.diags.push(d);
                for a in args {
                    self.expr(cx, a, None, mode);
                }
                TypeKind::Error
            }
        }
    }

    fn call(
        &mut self,
        cx: &mut FnCtx,
        path: &[Ident],
        args: &[Expr],
        span: Span,
        mode: Mode,
    ) -> TypeKind {
        let name = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        let Some(target) = self.resolve(path, &cx.mod_path) else {
            let hint = if path.len() == 1 {
                format!(
                    "no `{name}` in unit `{}`; other units are reached by full path `unit::{name}`",
                    cx.mod_path
                )
            } else {
                format!("no function `{name}`")
            };
            self.diags.push(
                Diag::new("E0304", span, format!("unknown function `{name}`")).fix(hint, None),
            );
            for a in args {
                if !matches!(a.kind, ExprKind::Ref { .. }) {
                    self.expr(cx, a, None, mode);
                }
            }
            return TypeKind::Error;
        };
        let sig = self.fns[&target].clone();
        match (mode, sig.kind) {
            (Mode::Exec, FnKind::Spec) => {
                self.err(
                    "E0505",
                    span,
                    format!(
                        "`{name}` is a spec function and cannot be called from executable code"
                    ),
                );
            }
            (Mode::Spec { .. }, FnKind::Exec) => {
                self.err(
                    "E0504",
                    span,
                    format!("`{name}` is executable and cannot be called from a specification"),
                );
            }
            (Mode::Exec, FnKind::Exec) => {
                if cx.effect == Effect::Pure && sig.effect == Effect::Io {
                    self.err(
                        "E0603",
                        span,
                        format!("`{name}` has effect `io`, but `{}` is `pure`", cx.path),
                    );
                }
                self.calls
                    .entry(cx.path.clone())
                    .or_default()
                    .insert(target.clone());
            }
            (Mode::Spec { .. }, FnKind::Spec) => {
                self.calls
                    .entry(cx.path.clone())
                    .or_default()
                    .insert(target.clone());
            }
        }
        if args.len() != sig.params.len() {
            self.err(
                "E0405",
                span,
                format!(
                    "`{name}` takes {} argument(s), found {}",
                    sig.params.len(),
                    args.len()
                ),
            );
        }
        let mut mut_roots: Vec<(String, usize)> = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let Some(pt) = sig.params.get(i) else {
                self.expr(cx, a, None, mode);
                continue;
            };
            if let TypeKind::Ref { mutable: pm, inner } = pt {
                if let Some(root) = self.ref_arg(cx, a, *pm, inner, mode)
                    && *pm
                {
                    mut_roots.push((root, i));
                }
            } else {
                let t = self.expr(cx, a, Some(pt), mode);
                if !(mode != Mode::Exec && is_int(&t) && is_int(pt)) {
                    self.expect_ty(&t, pt, a.span);
                }
            }
        }
        for (root, i) in mut_roots {
            for (j, a) in args.iter().enumerate() {
                if j != i && mentions(a, &root) {
                    self.err(
                        "E0410",
                        a.span,
                        format!("`{root}` is passed as `&mut` and also used by another argument"),
                    );
                }
            }
        }
        sig.ret
    }

    /// Checks an argument for a reference parameter; returns the root variable name.
    fn ref_arg(
        &mut self,
        cx: &mut FnCtx,
        a: &Expr,
        pm: bool,
        inner: &Type,
        mode: Mode,
    ) -> Option<String> {
        let want = format!("&{}{}", if pm { "mut " } else { "" }, ty_str(&inner.kind));
        let TypeKind::Slice(pe) = &inner.kind else {
            return None;
        };
        let (var, explicit_ref, am) = match &a.kind {
            ExprKind::Ref { mutable, expr } => match &expr.kind {
                ExprKind::Var(id) => (id, true, *mutable),
                _ => {
                    self.err("E0409", a.span, format!("expected `{want}` of a variable"));
                    return None;
                }
            },
            ExprKind::Var(id) => (id, false, false),
            ExprKind::Old(inner_e) => {
                let t = self.expr(cx, a, None, mode);
                if let (ExprKind::Var(id), TypeKind::Ref { inner: ri, .. }) = (&inner_e.kind, &t)
                    && !pm
                    && ri.kind == inner.kind
                {
                    return Some(id.name.clone());
                }
                if t != TypeKind::Error {
                    self.err(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                }
                return None;
            }
            _ => {
                let t = self.expr(cx, a, None, mode);
                if t != TypeKind::Error {
                    self.err(
                        "E0401",
                        a.span,
                        format!("expected `{want}`, found `{}`", ty_str(&t)),
                    );
                }
                return None;
            }
        };
        let Some(l) = Self::lookup(cx, &var.name) else {
            self.err("E0301", var.span, format!("unknown name `{}`", var.name));
            return None;
        };
        let ok = match (&l.ty, explicit_ref) {
            // `&arr` / `&mut arr` for a local array.
            (TypeKind::Array(e, _), true) => {
                if e.kind != pe.kind {
                    false
                } else if am && !l.mutable {
                    self.diags.push(
                        Diag::new("E0408", a.span, format!("`{}` is not mutable", var.name))
                            .fix("declare it with `let mut`", None),
                    );
                    return None;
                } else {
                    am == pm
                }
            }
            // Passing a reference parameter on: `f(a)`.
            (
                TypeKind::Ref {
                    mutable: lm,
                    inner: li,
                },
                false,
            ) => li.kind == inner.kind && (*lm || !pm),
            _ => false,
        };
        if !ok {
            let found = if explicit_ref {
                format!("&{}{}", if am { "mut " } else { "" }, ty_str(&l.ty))
            } else {
                ty_str(&l.ty)
            };
            let mut d = Diag::new(
                "E0401",
                a.span,
                format!("expected `{want}`, found `{found}`"),
            );
            if !explicit_ref && matches!(l.ty, TypeKind::Array(..)) {
                let r = format!("&{}{}", if pm { "mut " } else { "" }, var.name);
                d = d.fix(format!("pass `{r}`"), Some(r));
            }
            self.diags.push(d);
            return None;
        }
        Some(var.name.clone())
    }

    // ---------- whole-program checks ----------

    fn check_recursion(&mut self, p: &Program) {
        let mut defs: BTreeMap<String, (Span, bool)> = BTreeMap::new();
        fn walk(m: &Mod, prefix: &str, out: &mut BTreeMap<String, (Span, bool)>) {
            let mp = format!("{prefix}{}", m.name.name);
            for s in &m.mods {
                walk(s, &format!("{mp}::"), out);
            }
            for f in &m.fns {
                out.entry(format!("{mp}::{}", f.name.name))
                    .or_insert((f.name.span, f.decreases.is_some()));
            }
        }
        for m in &p.mods {
            walk(m, "", &mut defs);
        }
        let comps = sccs_with_cycles(&self.calls);
        for (id, comp) in comps.iter().enumerate() {
            for path in comp {
                self.recursive.insert(path.clone(), id);
            }
        }
        for path in comps.into_iter().flatten() {
            if let Some((span, has_dec)) = defs.get(&path)
                && !has_dec
            {
                self.diags.push(
                    Diag::new(
                        "E0507",
                        *span,
                        format!("`{path}` is recursive and has no `decreases` clause"),
                    )
                    .fix("add `decreases <expr>,` after `ensures`", None),
                );
            }
        }
    }

    fn check_main(&mut self, p: &Program) {
        let mut mains = Vec::new();
        fn walk<'b>(m: &'b Mod, out: &mut Vec<&'b FnDef>) {
            for s in &m.mods {
                walk(s, out);
            }
            out.extend(
                m.fns
                    .iter()
                    .filter(|f| f.kind == FnKind::Exec && f.name.name == "main"),
            );
        }
        for m in &p.mods {
            walk(m, &mut mains);
        }
        for (i, f) in mains.iter().enumerate() {
            if i > 0 {
                self.err("E0702", f.name.span, "the program already has a `main`");
            }
            let io = f.attrs.iter().any(|a| {
                a.name.name == "effects"
                    && a.args
                        .iter()
                        .any(|x| matches!(x, AttrArg::Ident(i) if i.name == "io"))
            });
            if !f.params.is_empty() || f.ret.ty.kind != TypeKind::Unit || !io {
                self.err(
                    "E0701",
                    f.name.span,
                    "`main` must be `#[effects(io)] fn main() -> ()`",
                );
            }
        }
    }
}

/// True if `e` mentions variable `name` anywhere.
fn mentions(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::Var(id) => id.name == name,
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().any(|a| mentions(a, name))
        }
        ExprKind::Method { recv, args, .. } => {
            mentions(recv, name) || args.iter().any(|a| mentions(a, name))
        }
        ExprKind::Index { base, index } => mentions(base, name) || mentions(index, name),
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x) => mentions(x, name),
        ExprKind::Repeat { value, .. } => mentions(value, name),
        ExprKind::Binary(_, l, r) => mentions(l, name) || mentions(r, name),
        // Blocks and conditionals as call arguments are rare; be conservative.
        ExprKind::If { .. } | ExprKind::Block(_) | ExprKind::Forall { .. } => true,
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::Unit | ExprKind::Error => false,
    }
}

/// `#[trigger]` terms of a `forall` body, not descending into nested `forall`s.
fn collect_triggers<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &e.kind {
        ExprKind::Trigger(inner) => {
            out.push(inner);
            collect_triggers(inner, out);
        }
        ExprKind::Forall { .. } => {}
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().for_each(|a| collect_triggers(a, out))
        }
        ExprKind::Method { recv, args, .. } => {
            collect_triggers(recv, out);
            args.iter().for_each(|a| collect_triggers(a, out));
        }
        ExprKind::Index { base, index } => {
            collect_triggers(base, out);
            collect_triggers(index, out);
        }
        ExprKind::Unary(_, x) | ExprKind::Ref { expr: x, .. } | ExprKind::Old(x) => {
            collect_triggers(x, out)
        }
        ExprKind::Repeat { value, .. } => collect_triggers(value, out),
        ExprKind::Binary(_, l, r) => {
            collect_triggers(l, out);
            collect_triggers(r, out);
        }
        ExprKind::If { cond, then, els } => {
            collect_triggers(cond, out);
            if let Some(t) = &then.tail {
                collect_triggers(t, out);
            }
            if let Some(x) = els {
                collect_triggers(x, out);
            }
        }
        ExprKind::Block(b) => {
            if let Some(t) = &b.tail {
                collect_triggers(t, out);
            }
        }
        ExprKind::Var(_)
        | ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::Unit
        | ExprKind::Error => {}
    }
}

/// Like `mentions`, but exact for every expression kind (used for triggers).
fn mentions_strict(e: &Expr, name: &str) -> bool {
    match &e.kind {
        ExprKind::If { cond, then, els } => {
            mentions_strict(cond, name)
                || then.tail.as_ref().is_some_and(|t| mentions_strict(t, name))
                || els.as_ref().is_some_and(|x| mentions_strict(x, name))
        }
        ExprKind::Block(b) => b.tail.as_ref().is_some_and(|t| mentions_strict(t, name)),
        ExprKind::Forall { body, .. } => mentions_strict(body, name),
        ExprKind::Call { args, .. } | ExprKind::Array(args) => {
            args.iter().any(|a| mentions_strict(a, name))
        }
        ExprKind::Method { recv, args, .. } => {
            mentions_strict(recv, name) || args.iter().any(|a| mentions_strict(a, name))
        }
        ExprKind::Index { base, index } => {
            mentions_strict(base, name) || mentions_strict(index, name)
        }
        ExprKind::Unary(_, x)
        | ExprKind::Ref { expr: x, .. }
        | ExprKind::Old(x)
        | ExprKind::Trigger(x) => mentions_strict(x, name),
        ExprKind::Repeat { value, .. } => mentions_strict(value, name),
        ExprKind::Binary(_, l, r) => mentions_strict(l, name) || mentions_strict(r, name),
        _ => mentions(e, name),
    }
}

/// Strongly connected components that contain a cycle (Tarjan), each sorted,
/// in a deterministic order.
fn sccs_with_cycles(g: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    struct T<'g> {
        g: &'g BTreeMap<String, BTreeSet<String>>,
        index: BTreeMap<&'g str, usize>,
        low: BTreeMap<&'g str, usize>,
        on: BTreeSet<&'g str>,
        stack: Vec<&'g str>,
        next: usize,
        out: Vec<Vec<String>>,
    }
    impl<'g> T<'g> {
        fn visit(&mut self, v: &'g str) {
            self.index.insert(v, self.next);
            self.low.insert(v, self.next);
            self.next += 1;
            self.stack.push(v);
            self.on.insert(v);
            if let Some(ws) = self.g.get(v) {
                for w in ws {
                    let w = w.as_str();
                    if !self.index.contains_key(w) {
                        self.visit(w);
                        let lw = self.low[w];
                        let lv = self.low.get_mut(v).unwrap();
                        *lv = (*lv).min(lw);
                    } else if self.on.contains(w) {
                        let iw = self.index[w];
                        let lv = self.low.get_mut(v).unwrap();
                        *lv = (*lv).min(iw);
                    }
                }
            }
            if self.low[v] == self.index[v] {
                let mut comp = Vec::new();
                loop {
                    let w = self.stack.pop().unwrap();
                    self.on.remove(w);
                    comp.push(w);
                    if w == v {
                        break;
                    }
                }
                let self_loop = self.g.get(v).is_some_and(|s| s.contains(v));
                if comp.len() > 1 || self_loop {
                    let mut c: Vec<String> = comp.iter().map(|s| s.to_string()).collect();
                    c.sort();
                    self.out.push(c);
                }
            }
        }
    }
    let mut t = T {
        g,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        on: BTreeSet::new(),
        stack: Vec::new(),
        next: 0,
        out: Vec::new(),
    };
    for v in g.keys() {
        if !t.index.contains_key(v.as_str()) {
            t.visit(v);
        }
    }
    t.out.sort();
    t.out
}
