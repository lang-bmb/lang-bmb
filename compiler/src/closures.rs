//! Function values (ADR-0053, backlog 2V-20): lowered before checking to
//! the traits, structs and `impl`s Rust itself has underneath, so the
//! checker, prover and code generator see ADR-0046 generics only.
//!
//! - A bound `F: Fn(A, B) -> R` of function `g` is a trait of `g`'s unit,
//!   `__fn_g_F`, with the spec methods `pre(self, a, b)` and
//!   `post(self, a, b, r)` and the method `call(self, a, b) -> (r: R)
//!   requires self.pre(a, b) ensures self.post(a, b, r)`, whose effects are
//!   `g`'s (ADR-0053 §5: calling `f` is an effect of `g`). In `g`, `f(a, b)`
//!   is `f.call(a, b)`, `f.requires(a, b)` is `f.pre(a, b)` and
//!   `f.ensures(a, b, r)` is `f.post(a, b, r)`.
//! - A function item `h` passed where `g` takes an `F` (`g::<_>(h, ..)`,
//!   the `_` written for the nameless type, ADR-0053 §2) is a struct with no
//!   fields in `h`'s unit, `__fnitem_h_g_F`, whose `impl` of the trait is
//!   `h`: `pre` is `h`'s `requires`, `post` its `ensures`, `call` calls `h`
//!   (and has `h`'s effects, which the trait's must hold: E0608 through
//!   the `impl`'s conformance). The argument is that struct's value and the
//!   `_` its type.
//!
//! - A closure is a struct of its caller's unit and that trait's `impl`:
//!   `pre` and `post` are its `requires` and `ensures`, `call` its body
//!   with the `ensures` clauses themselves (so a failed one is reported
//!   where it is written, backlog 2V-20f); the impl's refinement (E0817)
//!   proves the trait's `post` from them.
//!
//! The generated items are text parsed under file ids of their own
//! (`source::GENERATED` down), so every node has its own span. The source
//! program the formatter prints is not lowered: `check::check_resolved`
//! lowers its copy.

use std::collections::BTreeMap;

use crate::ast::*;
use crate::diag::{Diag, Diags};

/// An `Fn`-bounded type parameter: its position among the type
/// parameters, its name and bound, and the value parameters of type `F` or
/// `&F` (position, by reference).
type Bounded = (usize, String, FnBound, Vec<(usize, bool)>);

/// A function with `Fn`-bounded type parameters.
struct Generic {
    unit: String,
    /// Its place among the generic functions in source order: the
    /// generated names are numbered, so renaming a user's item renames
    /// nothing generated (identities stay put, ADR-0059).
    order: usize,
    /// For each such parameter: its position among the type parameters,
    /// its name and bound, and the value parameters of type `F` or `&F`
    /// (position, by reference).
    bounded: Vec<Bounded>,
    effects: String,
    /// The span of each bounded parameter's name, by its position.
    spans: BTreeMap<usize, crate::source::Span>,
}

/// A function's text needed to make it a value.
struct Item {
    unit: String,
    span: crate::source::Span,
    params: Vec<(String, String)>,
    ret_name: String,
    ret: String,
    requires: Vec<String>,
    ensures: Vec<String>,
    effects: String,
}

fn effects_of(f: &FnDef) -> String {
    f.attrs
        .iter()
        .find(|a| a.name.name == "effects")
        .map(|a| {
            a.args
                .iter()
                .map(|x| match x {
                    AttrArg::Ident(i) => i.name.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "pure".into())
}

/// `call`'s receiver for a bound of `kind` on `ty` (ADR-0053 §2), and the
/// state its `post` reads: an `FnMut` call may change it, so `post` is of
/// the state before (`old(self)`; after the call it is a value of its own,
/// ADR-0053 open question 3).
fn receiver(kind: FnTrait, ty: &str) -> (String, &'static str) {
    match kind {
        FnTrait::Fn => (format!("&{ty}"), "self"),
        FnTrait::FnMut => (format!("&mut {ty}"), "old(self)"),
        FnTrait::FnOnce => (ty.to_string(), "self"),
    }
}

/// The trait of the `k`th `Fn`-bounded parameter of the generic function
/// numbered `order`.
fn trait_name(order: usize, k: usize) -> String {
    format!("__fn_{order}_{k}")
}

/// The value parameters of `f` whose type is type parameter `t` or a
/// reference to it.
fn takes(f: &FnDef, t: &str) -> Vec<(usize, bool)> {
    let is_t = |k: &TypeKind| matches!(k, TypeKind::Named { path, args } if args.is_empty() && path.len() == 1 && path[0] == t);
    f.params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| match &p.ty.kind {
            k if is_t(k) => Some((i, false)),
            TypeKind::Ref { inner, .. } if is_t(&inner.kind) => Some((i, true)),
            _ => None,
        })
        .collect()
}

/// Returns the generated files whose diagnostics repeat one already
/// reported (the caller drops them after checking).
/// The names a block binds anywhere inside it: `let`s, `for` variables,
/// `match` bindings, quantifier binders.
fn bound_names(b: &Block, out: &mut std::collections::BTreeSet<String>) {
    let mut b = b.clone();
    fn stmts(b: &Block, out: &mut std::collections::BTreeSet<String>) {
        for s in &b.stmts {
            match s {
                Stmt::Let { name, .. } => {
                    out.insert(name.name.clone());
                }
                Stmt::For { var, body, .. } => {
                    out.insert(var.name.clone());
                    stmts(body, out);
                }
                Stmt::While { body, .. } | Stmt::Proof { body, .. } => stmts(body, out),
                _ => {}
            }
        }
    }
    stmts(&b, out);
    let mut wrap = FnDef {
        kind: FnKind::Exec,
        name: Ident {
            name: String::new(),
            span: b.span,
        },
        generics: Vec::new(),
        attrs: Vec::new(),
        params: Vec::new(),
        ret: Ret {
            name: None,
            ty: Type {
                kind: TypeKind::Unit,
                span: b.span,
            },
        },
        requires: Vec::new(),
        ensures: Vec::new(),
        decreases: None,
        body: std::mem::replace(
            &mut b,
            Block {
                stmts: Vec::new(),
                tail: None,
                span: Default::default(),
            },
        ),
        span: Default::default(),
    };
    crate::ast::fn_exprs_mut(&mut wrap, &mut |e| match &e.kind {
        ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => stmts(b, out),
        ExprKind::If { then, .. } => stmts(then, out),
        ExprKind::Match { arms, .. } => {
            for a in arms {
                out.extend(a.pat.binds.iter().flatten().map(|i| i.name.clone()));
            }
        }
        ExprKind::Quant { binders, .. } => out.extend(binders.iter().map(|b| b.name.name.clone())),
        _ => {}
    });
}

/// The locals of `c`'s enclosing function that `c` names (its captures,
/// ADR-0053 §2), in the order first named.
fn captures(c: &Closure, outer: &std::collections::BTreeSet<String>) -> Vec<String> {
    let mut own: std::collections::BTreeSet<String> =
        c.params.iter().map(|p| p.name.name.clone()).collect();
    bound_names(&c.body, &mut own);
    if let Some(n) = &c.ret.name {
        own.insert(n.name.clone());
    }
    let mut used: Vec<String> = Vec::new();
    let mut probe = Expr {
        kind: ExprKind::Closure(Box::new(c.clone())),
        span: c.span,
    };
    crate::ast::expr_walk(&mut probe, &mut |e| {
        if let ExprKind::Var(v) = &e.kind
            && outer.contains(&v.name)
            && !own.contains(&v.name)
            && !used.contains(&v.name)
        {
            used.push(v.name.clone());
        }
    });
    used
}

/// The written type of each local of `f` (parameters and `let`s); a
/// local without one (a `for` variable, a `match` binding) maps to `None`.
fn local_types(f: &FnDef) -> BTreeMap<String, Option<Type>> {
    let mut out: BTreeMap<String, Option<Type>> = BTreeMap::new();
    let mut names = std::collections::BTreeSet::new();
    bound_names(&f.body, &mut names);
    for n in names {
        out.insert(n, None);
    }
    for t in [&f.body] {
        let mut g = f.clone();
        g.body = t.clone();
        let mut lets: Vec<(String, Type)> = Vec::new();
        fn stmts(b: &Block, out: &mut Vec<(String, Type)>) {
            for s in &b.stmts {
                match s {
                    Stmt::Let { name, ty, .. } => out.push((name.name.clone(), ty.clone())),
                    Stmt::While { body, .. }
                    | Stmt::For { body, .. }
                    | Stmt::Proof { body, .. } => stmts(body, out),
                    _ => {}
                }
            }
        }
        stmts(&g.body, &mut lets);
        crate::ast::fn_exprs_mut(&mut g, &mut |e| match &e.kind {
            ExprKind::Block(b) | ExprKind::Loop { body: b, .. } => stmts(b, &mut lets),
            ExprKind::If { then, .. } => stmts(then, &mut lets),
            _ => {}
        });
        for (n, t) in lets {
            out.insert(n, Some(t));
        }
    }
    for p in &f.params {
        out.insert(p.name.name.clone(), Some(p.ty.clone()));
    }
    out
}

/// `e` with each read of a captured local `c` as `self.c` (the closure's
/// struct holds it, ADR-0053 §2).
fn through_self(e: &mut Expr, caps: &[String]) {
    crate::ast::expr_walk(e, &mut |x| {
        if let ExprKind::Var(v) = &x.kind
            && caps.contains(&v.name)
        {
            let at = x.span;
            let me = Expr {
                kind: ExprKind::Var(Ident {
                    name: "self".into(),
                    span: crate::source::Span {
                        file: crate::source::CAPTURE,
                        lo: at.lo,
                        hi: at.lo,
                    },
                }),
                span: crate::source::Span { hi: at.lo, ..at },
            };
            x.kind = ExprKind::Field {
                base: Box::new(me),
                name: v.clone(),
            };
        }
    });
}

/// Writes a call's rewrites (ADR-0053 lowering): argument `j` becomes the
/// value of struct `sname` (with the captures `caps` as its fields, `&` of
/// it when the parameter takes `&F`) and type argument `k` its type; a
/// type argument of a reported closure (`rej`) becomes an error type, so
/// its `_` is not reported again (E0305).
fn apply_call(e: &mut Expr, todo: &[(usize, String, usize, bool, Vec<String>)], rej: &[usize]) {
    let ExprKind::Call {
        path, targs, args, ..
    } = &mut e.kind
    else {
        return;
    };
    let last = path.len() - 1;
    for k in rej {
        if let Some(t) = targs.iter_mut().find(|t| t.seg == last)
            && let Some(a) = t.args.get_mut(*k)
        {
            a.kind = TypeKind::Error;
        }
    }
    for (j, sname, k, by_ref, caps) in todo {
        let (j, k, by_ref) = (*j, *k, *by_ref);
        // Distinct spans: the types are kept by span.
        let at = args[j].span;
        let sp = if by_ref {
            crate::source::Span { lo: at.hi, ..at }
        } else {
            at
        };
        let fsp = crate::source::Span { hi: at.lo, ..at };
        let segs: Vec<Ident> = sname
            .split("::")
            .map(|s| Ident {
                name: s.to_string(),
                span: sp,
            })
            .collect();
        let lit = Expr {
            // A struct has a field (E0420): one that carries nothing.
            kind: ExprKind::StructLit {
                path: segs,
                fields: std::iter::once((
                    Ident {
                        name: "item".into(),
                        span: sp,
                    },
                    Expr {
                        kind: ExprKind::Bool(false),
                        span: fsp,
                    },
                ))
                .chain(caps.iter().enumerate().map(|(i, n)| {
                    // Each capture's value at a span of its own.
                    let p = at.lo + 1 + i as u32;
                    let vsp = crate::source::Span { lo: p, hi: p, ..at };
                    (
                        Ident {
                            name: n.clone(),
                            span: vsp,
                        },
                        Expr {
                            kind: ExprKind::Var(Ident {
                                name: n.clone(),
                                span: vsp,
                            }),
                            span: vsp,
                        },
                    )
                }))
                .collect(),
                rest: None,
            },
            span: sp,
        };
        args[j] = if by_ref {
            Expr {
                kind: ExprKind::Ref {
                    mutable: false,
                    expr: Box::new(lit),
                },
                span: at,
            }
        } else {
            lit
        };
        if let Some(t) = targs.iter_mut().find(|t| t.seg == last)
            && let Some(a) = t.args.get_mut(k)
        {
            a.kind = TypeKind::Named {
                path: sname.split("::").map(String::from).collect(),
                args: Vec::new(),
            };
        }
    }
}

/// E0264 for each closure the lowering left: one that is not the argument
/// of an `Fn`-bounded parameter (ADR-0053 §3: a closure does not leave the
/// function that makes it, and is passed down).
fn leftover_closures(p: &mut Program, diags: &mut Diags) {
    crate::ast::exprs_mut(p, &mut |e| {
        if let ExprKind::Closure(c) = &e.kind {
            diags.push(
                Diag::new(
                    "E0264",
                    c.span,
                    "a closure is the argument of a function's `Fn`-bounded parameter",
                )
                .fix(
                    "pass it directly where a function takes `f: F` with `F: Fn(..) -> R`, or name it as a `fn` of the unit",
                    None,
                ),
            );
        }
    });
}

/// What the lowering made: the generated files whose diagnostics repeat
/// one already reported, and the names the items made stand for.
#[derive(Default)]
pub struct Lowered {
    pub quiet: Vec<u32>,
    /// Each generated struct and trait by full path, and what a reader
    /// calls it (backlog 2V-20g): a closure `u::f::{closure#0}` (numbered
    /// in source order within the definition that writes it, as rustc's
    /// definition paths), a function item passed as a value by its own
    /// path, a bound's trait `u::g::F` (the function and its parameter).
    pub names: BTreeMap<String, String>,
    /// Each place a function value is passed (backlog 2V-20h): the
    /// argument's span and what it passes, the closure's `call` or the
    /// function item. The map draws a `passes` edge from the definition
    /// whose body holds the span.
    pub passes: Vec<(crate::source::Span, String)>,
    /// Where each generated definition is written: a closure's struct and
    /// methods at the closure, a function item's at that function, a
    /// bound's trait at the bound (the map's `file` and `line`).
    pub at: BTreeMap<String, crate::source::Span>,
}

pub fn lower(p: &mut Program, diags: &mut Diags) -> Lowered {
    // The generic functions with `Fn` bounds, and every free function.
    let mut generics: BTreeMap<String, Generic> = BTreeMap::new();
    let mut items: BTreeMap<String, Item> = BTreeMap::new();
    fn scan(
        m: &Mod,
        prefix: &str,
        generics: &mut BTreeMap<String, Generic>,
        items: &mut BTreeMap<String, Item>,
    ) {
        let unit = format!("{prefix}{}", m.name.name);
        for f in &m.fns {
            let path = format!("{unit}::{}", f.name.name);
            let bounded: Vec<_> = f
                .generics
                .iter()
                .enumerate()
                .filter_map(|(i, g)| {
                    let b = g.fn_bound.clone()?;
                    Some((i, g.name.name.clone(), b, takes(f, &g.name.name)))
                })
                .collect();
            if !bounded.is_empty() {
                let order = generics.len();
                generics.insert(
                    path.clone(),
                    Generic {
                        unit: unit.clone(),
                        order,
                        spans: bounded
                            .iter()
                            .map(|(i, ..)| (*i, f.generics[*i].name.span))
                            .collect(),
                        bounded,
                        effects: effects_of(f),
                    },
                );
            }
            if f.kind == FnKind::Exec && f.generics.is_empty() {
                let ret_name = f
                    .ret
                    .name
                    .as_ref()
                    .map_or("result".to_string(), |n| n.name.clone());
                items.insert(
                    path,
                    Item {
                        unit: unit.clone(),
                        span: f.name.span,
                        params: f
                            .params
                            .iter()
                            .map(|p| (p.name.name.clone(), crate::fmt::type_str(&p.ty)))
                            .collect(),
                        ret_name,
                        ret: crate::fmt::type_str(&f.ret.ty),
                        requires: f.requires.iter().map(crate::fmt::expr_text).collect(),
                        ensures: f.ensures.iter().map(crate::fmt::expr_text).collect(),
                        effects: effects_of(f),
                    },
                );
            }
        }
        for s in &m.mods {
            scan(s, &format!("{unit}::"), generics, items);
        }
    }
    for m in &p.mods {
        scan(m, "", &mut generics, &mut items);
    }
    if generics.is_empty() {
        leftover_closures(p, diags);
        return Lowered::default();
    }
    let mut pending: Vec<Diag> = Vec::new();
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    let mut passes: Vec<(crate::source::Span, String)> = Vec::new();
    let mut located: BTreeMap<String, crate::source::Span> = BTreeMap::new();
    // A generated item and its methods, at one span.
    let methods = ["call", "pre", "post"];
    for (gp, g) in &generics {
        for (k, fname, _, _) in &g.bounded {
            let tr = format!("{}::{}", g.unit, trait_name(g.order, *k));
            names.insert(tr.clone(), format!("{gp}::{fname}"));
            for m in methods {
                located.insert(format!("{tr}::{m}"), g.spans[k]);
            }
            located.insert(tr, g.spans[k]);
        }
    }
    // Closures written so far in each definition, for their numbers.
    let mut written: BTreeMap<String, usize> = BTreeMap::new();

    // Generated text by unit; a second file per unit holds the items made
    // for an argument already reported (E0608), whose own errors would
    // repeat it.
    let mut text: BTreeMap<(String, bool), String> = BTreeMap::new();
    for g in generics.values() {
        for (k, _, b, _) in &g.bounded {
            let args: Vec<String> = (0..b.params.len()).map(|i| format!("a{i}")).collect();
            let typed: Vec<String> = b
                .params
                .iter()
                .enumerate()
                .map(|(i, t)| format!("a{i}: {}", crate::fmt::type_str(t)))
                .collect();
            let ret = crate::fmt::type_str(&b.ret);
            let comma = |xs: &[String]| xs.iter().map(|x| format!(", {x}")).collect::<String>();
            let t = text.entry((g.unit.clone(), false)).or_default();
            t.push_str(&format!(
                "trait {tn} {{\n    spec fn pre(self: &Self{p}) -> bool;\n\n    spec fn post(self: &Self{p}, r: {ret}) -> bool;\n\n    #[effects({e})]\n    fn call(self: {rv}{p}) -> (r: {ret})\n        requires self.pre({a}),\n        ensures {st}.post({ar}r);\n}}\n\n",
                rv = receiver(b.kind, "Self").0,
                st = receiver(b.kind, "Self").1,
                tn = trait_name(g.order, *k),
                p = comma(&typed),
                e = g.effects,
                a = args.join(", "),
                ar = args.iter().map(|x| format!("{x}, ")).collect::<String>(),
            ));
        }
    }

    // Each function item passed as a function value: its struct and impl,
    // once per (item, trait).
    let mut made: BTreeMap<(String, String), String> = BTreeMap::new();
    let mut items_made = 0usize;
    let resolve = |path: &[Ident], unit: &str| -> Option<String> {
        let full = path
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>()
            .join("::");
        if path.len() == 1 {
            let local = format!("{unit}::{full}");
            if generics.contains_key(&local) || items.contains_key(&local) {
                return Some(local);
            }
        }
        (generics.contains_key(&full) || items.contains_key(&full)).then_some(full)
    };
    // Rounds (backlog 2V-20e): a closure spliced into the `call` of
    // another closure's impl is lowered in the next round, as a call in a
    // function written so.
    let mut closures_made = 0usize;
    let mut made_files = 0usize;
    let mut quiet = Vec::new();
    let mut first = true;
    loop {
        // Each closure's struct (full path) and the closure.
        let mut closure_bodies: BTreeMap<String, (Closure, Vec<String>)> = BTreeMap::new();
        let mut done: Vec<(String, Vec<FnDef>)> = Vec::new();
        let before = closures_made;
        {
            fn units<'m>(m: &'m Mod, prefix: &str, out: &mut Vec<(String, &'m Mod)>) {
                let unit = format!("{prefix}{}", m.name.name);
                out.push((unit.clone(), m));
                for s in &m.mods {
                    units(s, &format!("{unit}::"), out);
                }
            }
            let mut all = Vec::new();
            for m in &p.mods {
                units(m, "", &mut all);
            }
            for (unit, m) in &all {
                let mut changed: Vec<FnDef> = Vec::new();
                // Each function with the path a reader knows it by.
                let owned: Vec<(String, &FnDef)> = m
                    .fns
                    .iter()
                    .map(|f| (format!("{unit}::{}", f.name.name), f))
                    .chain(m.impls.iter().flat_map(|i| {
                        i.fns
                            .iter()
                            .map(move |f| (format!("{unit}::{}::{}", i.ty.name, f.name.name), f))
                    }))
                    .chain(m.trait_impls.iter().flat_map(|i| {
                        let ty = crate::fmt::type_str(&i.self_ty);
                        let tr = i
                            .trait_path
                            .iter()
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join("::");
                        let made = names.get(&format!("{unit}::{ty}")).cloned();
                        i.fns.iter().map(move |f| {
                            // A closure's body is the closure.
                            let path = match &made {
                                Some(c) if f.name.name == "call" => c.clone(),
                                _ => format!("<{unit}::{ty} as {tr}>::{}", f.name.name),
                            };
                            (path, f)
                        })
                    }))
                    .collect();
                for (owner, f) in owned {
                    let mut f = f.clone();
                    let decls = local_types(&f);
                    let outer: std::collections::BTreeSet<String> = decls.keys().cloned().collect();
                    // A call inside a closure waits for the round after that
                    // closure's body is a `call` of its own (its locals are in
                    // scope there). The walk below visits inner expressions
                    // first, so they are told apart by address, which nothing
                    // moves before the walk.
                    let mut nested: std::collections::BTreeSet<usize> =
                        std::collections::BTreeSet::new();
                    crate::ast::fn_exprs_mut(&mut f, &mut |e| {
                        if let ExprKind::Closure(c) = &mut e.kind {
                            let mut mark = |x: &mut Expr| {
                                nested.insert(x as *const Expr as usize);
                            };
                            c.requires
                                .iter_mut()
                                .for_each(|x| crate::ast::expr_mut(x, &mut mark));
                            c.ensures
                                .iter_mut()
                                .for_each(|x| crate::ast::expr_mut(x, &mut mark));
                            crate::ast::block_mut(&mut c.body, &mut mark);
                        }
                    });
                    crate::ast::fn_exprs_mut(&mut f, &mut |e| {
                        if nested.contains(&(e as *const Expr as usize)) {
                            return;
                        }
                        // What this call's arguments become, applied in place
                        // (not by span: a program read from JSON has none).
                        let mut todo: Vec<(usize, String, usize, bool, Vec<String>)> = Vec::new();
                        let mut rej: Vec<usize> = Vec::new();
                        let ExprKind::Call { path, args, .. } = &e.kind else {
                            return;
                        };
                        let Some(gp) = resolve(path, unit) else {
                            return;
                        };
                        let Some(g) = generics.get(&gp) else {
                            return;
                        };
                        for (k, fname, bnd, at) in &g.bounded {
                            for (j, by_ref) in at {
                                let Some(a) = args.get(*j) else { continue };
                                // A closure: a struct of its own and its impl
                                // (ADR-0053 §2, §4), the body and contract
                                // spliced in after parsing (`closure_bodies`).
                                if let ExprKind::Closure(c) = &a.kind {
                                    let caps = captures(c, &outer);
                                    if caps.iter().any(|n| n == "self") {
                                        pending.push(
                                        Diag::new(
                                            "E0262",
                                            c.span,
                                            "a closure inside a closure reads a value the outer one captured",
                                        )
                                        .fix("bind the captured value to a typed `let` in the outer closure's body and capture that", None),
                                    );
                                        rej.push(*k);
                                        continue;
                                    }
                                    let untyped: Vec<&String> =
                                        caps.iter().filter(|n| decls[*n].is_none()).collect();
                                    if !caps.is_empty() && (!c.is_move || !untyped.is_empty()) {
                                        let d = if c.is_move {
                                            let u: Vec<&str> =
                                                untyped.iter().map(|x| x.as_str()).collect();
                                            Diag::new(
                                            "E0262",
                                            c.span,
                                            format!("a closure captures `{}`, whose type is not written", u.join("`, `")),
                                        )
                                        .fix(format!("bind it to a typed `let` before the closure: `let {0}2: T = {0};`", u[0]), None)
                                        } else {
                                            Diag::new(
                                            "E0262",
                                            c.span,
                                            format!("a closure that captures (`{}`) is written `move`", caps.join("`, `")),
                                        )
                                        .fix("write `move` before the closure: captures move into it (ADR-0053 §2)", Some(format!("move {}", crate::fmt::expr_text(a))))
                                        };
                                        pending.push(d);
                                        // Its `_` is not reported again (E0305).
                                        rej.push(*k);
                                        continue;
                                    }
                                    let tn = trait_name(g.order, *k);
                                    let tr = format!("{}::{tn}", g.unit);
                                    let sname = format!("__closure_{closures_made}");
                                    closures_made += 1;
                                    let nth = written.entry(owner.clone()).or_default();
                                    names.insert(
                                        format!("{unit}::{sname}"),
                                        format!("{owner}::{{closure#{nth}}}"),
                                    );
                                    *nth += 1;
                                    let typed: String = c
                                        .params
                                        .iter()
                                        .map(|p| {
                                            format!(
                                                ", {}: {}",
                                                p.name.name,
                                                crate::fmt::type_str(&p.ty)
                                            )
                                        })
                                        .collect();
                                    let names: Vec<&str> =
                                        c.params.iter().map(|p| p.name.name.as_str()).collect();
                                    let rn = c
                                        .ret
                                        .name
                                        .as_ref()
                                        .map_or("result".to_string(), |n| n.name.clone());
                                    let slots = |n: usize| {
                                        if n == 0 {
                                            "true".to_string()
                                        } else {
                                            vec!["true"; n].join(" && ")
                                        }
                                    };
                                    let fields: String = caps
                                        .iter()
                                        .map(|n| {
                                            format!(
                                                ", {n}: {}",
                                                crate::fmt::type_str(
                                                    decls[n].as_ref().expect("typed")
                                                )
                                            )
                                        })
                                        .collect();
                                    let t = text.entry((unit.clone(), false)).or_default();
                                    t.push_str(&format!(
                                    "struct {sname} {{ item: bool{fields} }}\n\nimpl {tr} for {sname} {{\n    spec fn pre(self: &{sname}{typed}) -> bool {{ {pre} }}\n\n    spec fn post(self: &{sname}{typed}, {rn}: {ret}) -> bool {{ {post} }}\n\n    #[effects({e})]\n    fn call(self: {rv}{typed}) -> ({rn}: {ret})\n        requires self.pre({a}),\n        ensures {st}.post({a}{sep}{rn}),\n    {{\n        {rn}\n    }}\n}}\n\n",
                                    rv = receiver(bnd.kind, &sname).0,
                                    st = receiver(bnd.kind, &sname).1,
                                    pre = slots(c.requires.len()),
                                    post = slots(c.ensures.len()),
                                    ret = crate::fmt::type_str(&c.ret.ty),
                                    e = g.effects,
                                    a = names.join(", "),
                                    sep = if names.is_empty() { "" } else { ", " },
                                ));
                                    let full = format!("{unit}::{sname}");
                                    passes.push((a.span, format!("<{full} as {tr}>::call")));
                                    for m in methods {
                                        located.insert(format!("<{full} as {tr}>::{m}"), c.span);
                                    }
                                    located.insert(full.clone(), c.span);
                                    closure_bodies
                                        .insert(full.clone(), ((**c).clone(), caps.clone()));
                                    todo.push((*j, full, *k, *by_ref, caps.clone()));
                                    continue;
                                }
                                let target = match &a.kind {
                                    ExprKind::Var(id) => resolve(std::slice::from_ref(id), unit),
                                    ExprKind::Path(ps) => resolve(ps, unit),
                                    _ => None,
                                };
                                let Some(h) = target.filter(|h| items.contains_key(h)) else {
                                    continue;
                                };
                                let tn = trait_name(g.order, *k);
                                let key = (h.clone(), format!("{}::{tn}", g.unit));
                                // Calling `f` is an effect of `g` (ADR-0053 §5).
                                let set = |e: &str| -> Vec<String> {
                                    e.split(", ")
                                        .filter(|x| !x.is_empty() && *x != "pure")
                                        .map(String::from)
                                        .collect()
                                };
                                let (have, allowed) = (set(&items[&h].effects), set(&g.effects));
                                let extra: Vec<&String> =
                                    have.iter().filter(|x| !allowed.contains(x)).collect();
                                let reported = !extra.is_empty();
                                if reported {
                                    let extra: Vec<&str> =
                                        extra.iter().map(|x| x.as_str()).collect();
                                    pending.push(
                                    Diag::new(
                                        "E0608",
                                        a.span,
                                        format!(
                                            "`{}` has effect `{}`, which `{gp}` does not declare for its `{fname}`",
                                            h.rsplit("::").next().unwrap_or_default(),
                                            extra.join(", ")
                                        ),
                                    )
                                    .fix(
                                        format!("declare `{}` on `{gp}`, or pass a function without it", extra.join(", ")),
                                        None,
                                    ),
                                );
                                }
                                let sname = made.entry(key.clone()).or_insert_with(|| {
                                let hn = h.rsplit("::").next().unwrap_or_default();
                                let sname = format!("__fnitem_{}", items_made);
                                items_made += 1;
                                names.insert(format!("{}::{sname}", items[&h].unit), h.clone());
                                let it = &items[&h];
                                let typed: String = it
                                    .params
                                    .iter()
                                    .map(|(n, t)| format!(", {n}: {t}"))
                                    .collect();
                                let names: Vec<&str> =
                                    it.params.iter().map(|(n, _)| n.as_str()).collect();
                                let conj = |xs: &[String]| {
                                    if xs.is_empty() {
                                        "true".to_string()
                                    } else {
                                        xs.iter().map(|x| format!("({x})")).collect::<Vec<_>>().join(" && ")
                                    }
                                };
                                let t = text.entry((it.unit.clone(), reported)).or_default();
                                t.push_str(&format!(
                                    "struct {sname} {{ item: bool }}\n\nimpl {tr} for {sname} {{\n    spec fn pre(self: &{sname}{typed}) -> bool {{ {pre} }}\n\n    spec fn post(self: &{sname}{typed}, {rn}: {ret}) -> bool {{ {post} }}\n\n    #[effects({e})]\n    fn call(self: {rv}{typed}) -> ({rn}: {ret})\n        requires self.pre({a}),\n        ensures {st}.post({a}{sep}{rn}),\n    {{\n        {hn}({a})\n    }}\n}}\n\n",
                                    rv = receiver(bnd.kind, &sname).0,
                                    st = receiver(bnd.kind, &sname).1,
                                    tr = key.1,
                                    pre = conj(&it.requires),
                                    post = conj(&it.ensures),
                                    rn = it.ret_name,
                                    ret = it.ret,
                                    e = it.effects,
                                    a = names.join(", "),
                                    sep = if names.is_empty() { "" } else { ", " },
                                ));
                                let full = format!("{}::{sname}", it.unit);
                                for m in methods {
                                    located.insert(format!("<{full} as {}>::{m}", key.1), it.span);
                                }
                                located.insert(full.clone(), it.span);
                                full
                            });
                                passes.push((a.span, h.clone()));
                                todo.push((*j, sname.clone(), *k, *by_ref, Vec::new()));
                            }
                        }
                        apply_call(e, &todo, &rej);
                    });
                    changed.push(f);
                }
                done.push((unit.clone(), changed));
            }
        }
        // The functions with their calls rewritten, back in place (same order).
        for (unit, fs) in done {
            let segs: Vec<&str> = unit.split("::").collect();
            if let Some(m) = find(&mut p.mods, &segs) {
                let slots = m
                    .fns
                    .iter_mut()
                    .chain(m.impls.iter_mut().flat_map(|i| &mut i.fns))
                    .chain(m.trait_impls.iter_mut().flat_map(|i| &mut i.fns));
                for (slot, f) in slots.zip(fs) {
                    *slot = f;
                }
            }
        }

        // The generic functions themselves: the bound is the trait, the calls
        // are its methods.
        fn rewrite_generic(m: &mut Mod, prefix: &str, generics: &BTreeMap<String, Generic>) {
            let unit = format!("{prefix}{}", m.name.name);
            for f in &mut m.fns {
                let path = format!("{unit}::{}", f.name.name);
                let Some(g) = generics.get(&path) else {
                    continue;
                };
                let mut values: Vec<String> = Vec::new();
                for (k, _, _, at) in &g.bounded {
                    let tp = &mut f.generics[*k];
                    tp.fn_bound = None;
                    tp.bounds.push(vec![Ident {
                        name: trait_name(g.order, *k),
                        span: tp.name.span,
                    }]);
                    values.extend(at.iter().map(|(i, _)| f.params[*i].name.name.clone()));
                }
                crate::ast::fn_exprs_mut(f, &mut |e| match &mut e.kind {
                    ExprKind::Call { path, args, .. }
                        if path.len() == 1 && values.contains(&path[0].name) =>
                    {
                        let recv = Expr {
                            kind: ExprKind::Var(path[0].clone()),
                            span: path[0].span,
                        };
                        e.kind = ExprKind::Method {
                            recv: Box::new(recv),
                            name: Ident {
                                name: "call".into(),
                                span: path[0].span,
                            },
                            args: std::mem::take(args),
                        };
                    }
                    ExprKind::Method { recv, name, .. }
                        if matches!(&recv.kind, ExprKind::Var(v) if values.contains(&v.name))
                            && (name.name == "requires" || name.name == "ensures") =>
                    {
                        name.name = if name.name == "requires" {
                            "pre"
                        } else {
                            "post"
                        }
                        .into();
                    }
                    _ => {}
                });
                // An `FnMut` taken by value is called through a mutable local of
                // the body (`call` takes `&mut self`; parameters are not `mut`).
                let mutable: Vec<(String, Ident)> = g
                    .bounded
                    .iter()
                    .filter(|(_, _, b, _)| b.kind == FnTrait::FnMut)
                    .flat_map(|(k, _, _, at)| {
                        at.iter()
                            .filter(|(_, by_ref)| !by_ref)
                            .map(|(i, _)| {
                                (f.params[*i].name.name.clone(), f.generics[*k].name.clone())
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                if !mutable.is_empty() {
                    let (req, ens) = (
                        std::mem::take(&mut f.requires),
                        std::mem::take(&mut f.ensures),
                    );
                    let renamed: Vec<String> =
                        mutable.iter().map(|(n, _)| format!("__fm_{n}")).collect();
                    crate::ast::fn_exprs_mut(f, &mut |e| {
                        if let ExprKind::Var(v) = &mut e.kind
                            && let Some(k) = mutable.iter().position(|(n, _)| *n == v.name)
                        {
                            v.name = renamed[k].clone();
                        }
                    });
                    (f.requires, f.ensures) = (req, ens);
                    for ((n, t), r) in mutable.iter().zip(&renamed).rev() {
                        let at = f.body.span;
                        f.body.stmts.insert(
                            0,
                            Stmt::Let {
                                mutable: true,
                                name: Ident {
                                    name: r.clone(),
                                    span: crate::source::Span { hi: at.lo, ..at },
                                },
                                ty: Type {
                                    kind: TypeKind::Named {
                                        path: vec![t.name.clone()],
                                        args: Vec::new(),
                                    },
                                    span: t.span,
                                },
                                ty_missing: false,
                                init: Expr {
                                    kind: ExprKind::Var(Ident {
                                        name: n.clone(),
                                        span: crate::source::Span {
                                            lo: at.lo + 1,
                                            hi: at.lo + 1,
                                            ..at
                                        },
                                    }),
                                    span: crate::source::Span {
                                        lo: at.lo + 1,
                                        hi: at.lo + 1,
                                        ..at
                                    },
                                },
                                span: at,
                            },
                        );
                    }
                }
            }
            for s in &mut m.mods {
                rewrite_generic(s, &format!("{unit}::"), generics);
            }
        }
        if first {
            for m in &mut p.mods {
                rewrite_generic(m, "", &generics);
            }
            first = false;
        }

        for d in pending.drain(..) {
            diags.push(d);
        }
        // The generated items, parsed, into their units.
        for ((unit, reported), body) in std::mem::take(&mut text) {
            let last = unit.rsplit("::").next().unwrap_or_default().to_string();
            let src = format!("#[summary(\"\")]\nmod {last} {{\n{body}}}\n");
            // A file id of its own for each text, so every node has its own span.
            let id = crate::source::GENERATED - made_files as u32;
            made_files += 1;
            if reported {
                quiet.push(id);
            }
            let toks = crate::lexer::lex(id, &src, diags);
            let parsed = crate::parser::Parser::new(toks, diags, &src).parse_program();
            let Some(made_mod) = parsed.mods.into_iter().next() else {
                continue;
            };
            let segs: Vec<&str> = unit.split("::").collect();
            let mut made_mod = made_mod;
            for i in &mut made_mod.trait_impls {
                let TypeKind::Named { path, .. } = &i.self_ty.kind else {
                    continue;
                };
                let Some((c, caps)) = closure_bodies.get(&format!("{unit}::{}", path.join("::")))
                else {
                    continue;
                };
                // Captures are the struct's fields: `self.c` (ADR-0053 §2).
                let mut c = c.clone();
                let mut wrapped = Expr {
                    kind: ExprKind::Closure(Box::new(c.clone())),
                    span: c.span,
                };
                through_self(&mut wrapped, caps);
                if let ExprKind::Closure(x) = wrapped.kind {
                    c = *x;
                }
                let post_path = format!(
                    "<{unit}::{} as {}>::post",
                    path.join("::"),
                    i.trait_path
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join("::")
                );
                for f in &mut i.fns {
                    match f.name.name.as_str() {
                        "pre" => splice(&mut f.body, &c.requires),
                        "post" => splice(&mut f.body, &c.ensures),
                        "call" => {
                            f.body = c.body.clone();
                            // `call` ensures the closure's own clauses, so a
                            // failed one is reported at the clause written
                            // (backlog 2V-20f); the trait's `post` follows
                            // from them (the impl's refinement, E0817). An
                            // `FnMut`'s clauses are of the state before the
                            // call, as its `post` is (ADR-0053 open question 3).
                            if !c.ensures.is_empty() {
                                let before = matches!(
                                    f.params.first().map(|p| &p.ty.kind),
                                    Some(TypeKind::Ref { mutable: true, .. })
                                );
                                f.ensures = c
                                    .ensures
                                    .iter()
                                    .map(|e| {
                                        let mut e = e.clone();
                                        if before {
                                            of_before(&mut e);
                                        }
                                        e
                                    })
                                    .collect();
                                // The refinement reads `post`'s definition.
                                let at = f.name.span;
                                f.attrs.push(Attr {
                                    name: Ident {
                                        name: "uses".into(),
                                        span: at,
                                    },
                                    args: vec![AttrArg::Ident(Ident {
                                        name: post_path.clone(),
                                        span: at,
                                    })],
                                    span: at,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Some(m) = find(&mut p.mods, &segs) {
                m.traits.extend(made_mod.traits);
                m.types.extend(made_mod.types);
                m.trait_impls.extend(made_mod.trait_impls);
            }
        }
        // Another round while the last one lowered a closure and one is left.
        let mut left = false;
        crate::ast::exprs_mut(p, &mut |e| left |= matches!(e.kind, ExprKind::Closure(_)));
        if !left || closures_made == before {
            break;
        }
    }
    // A closure no round lowered is not an argument of a bounded parameter.
    leftover_closures(p, diags);
    Lowered {
        quiet,
        names,
        passes,
        at: located,
    }
}

/// Whether `base` of a field read is a closure's own struct as
/// `through_self` and `of_before` write it (`self` named in the file
/// `source::CAPTURE`, or `old` of that): the read is of a capture, which
/// the closure's text names alone (`count`, not `self.count`, backlog
/// 2V-20g). No parsed or JSON-read `self` is in that file.
pub fn is_capture_base(base: &Expr) -> bool {
    let me = |e: &Expr| matches!(&e.kind, ExprKind::Var(v) if v.name == "self" && v.span.file == crate::source::CAPTURE);
    match &base.kind {
        ExprKind::Old(x) => me(x),
        _ => me(base),
    }
}

/// `e` with each `self` as `old(self)`: an `FnMut` closure's `ensures`
/// read its captures as they were before the call.
fn of_before(e: &mut Expr) {
    crate::ast::expr_walk(e, &mut |x| {
        if matches!(&x.kind, ExprKind::Var(v) if v.name == "self") {
            let inner = x.clone();
            x.kind = ExprKind::Old(Box::new(inner));
        }
    });
}

/// The `true && true && ..` placeholders of `body`, left to right, as the
/// clauses `cs` (their own spans: a failed clause points at what was
/// written).
fn splice(body: &mut Block, cs: &[Expr]) {
    fn fill(e: &mut Expr, cs: &mut std::slice::Iter<'_, Expr>) {
        match &mut e.kind {
            ExprKind::Binary(BinOp::And, a, b) => {
                fill(a, cs);
                fill(b, cs);
            }
            ExprKind::Bool(true) => {
                if let Some(c) = cs.next() {
                    *e = c.clone();
                }
            }
            _ => {}
        }
    }
    if let Some(t) = body.tail.as_mut() {
        fill(t, &mut cs.iter());
    }
}

fn find<'m>(mods: &'m mut [Mod], segs: &[&str]) -> Option<&'m mut Mod> {
    let (first, rest) = segs.split_first()?;
    let m = mods.iter_mut().find(|m| m.name.name == *first)?;
    if rest.is_empty() {
        Some(m)
    } else {
        find(&mut m.mods, rest)
    }
}
